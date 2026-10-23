use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::Mutex;
use walkdir::WalkDir;

use super::code::{is_supported_code_file, parse_code_file, resolve_import_path, ParsedCodeFile};
use super::db::{
    ChunkHit, CodeChunkHit, CodeDependencies, CodeFileRecord, CompressedContext, DbManager,
    FileContext, GraphData, LinkSuggestion, Neighborhood, NoteRecord, NoteSummary, SearchHit,
    VaultStats,
};
use super::embeddings::EmbeddingEngine;
use super::parser::{parse_markdown, ParsedNote};

#[derive(Clone)]
pub struct Engine {
    vault_path: PathBuf,
    db_path: PathBuf,
    db: Arc<Mutex<DbManager>>,
    embeddings: Arc<EmbeddingEngine>,
    sync_mutex: Arc<Mutex<()>>,
    cached_count: Arc<AtomicUsize>,
}

pub fn resolve_vault_path() -> PathBuf {
    if let Ok(v) = std::env::var("GRAPHFINDER_VAULT") {
        return PathBuf::from(v);
    }
    let base = super::project::resolve_base_dir();
    let p = base.join("vault");
    if !p.exists() {
        let _ = fs::create_dir_all(&p);
    }
    p
}

pub fn resolve_db_path() -> PathBuf {
    if let Ok(d) = std::env::var("GRAPHFINDER_DB") {
        return PathBuf::from(d);
    }
    let base = super::project::resolve_base_dir();
    base.join("index.db")
}

impl Engine {
    pub fn new(
        vault_path: impl AsRef<Path>,
        db_path: impl AsRef<Path>,
        embeddings: Arc<EmbeddingEngine>,
    ) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let vault_path = vault_path.as_ref().to_path_buf();
        let db_path = db_path.as_ref().to_path_buf();
        if !vault_path.exists() {
            fs::create_dir_all(&vault_path)?;
        }

        let db = DbManager::new(db_path.to_str().unwrap_or("index.db"))?;
        let initial_count = db.total_indexed_count();

        Ok(Self {
            vault_path,
            db_path,
            db: Arc::new(Mutex::new(db)),
            embeddings,
            sync_mutex: Arc::new(Mutex::new(())),
            cached_count: Arc::new(AtomicUsize::new(initial_count)),
        })
    }

    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    pub fn cached_count(&self) -> usize {
        self.cached_count.load(Ordering::Relaxed)
    }

    /// Scans the vault directory and indexes all markdown files AND source code files into SQLite
    pub async fn sync_vault(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        self.sync_vault_with_handle(None).await
    }

    pub async fn sync_vault_with_handle(
        &self,
        app_handle: Option<&tauri::AppHandle>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let _sync_guard = self.sync_mutex.lock().await;

        // 1. Fetch existing timestamps and content hashes for instant incremental sync
        let (existing_note_meta, existing_code_meta) = {
            let db = self.db.lock().await;
            (db.get_all_notes_meta()?, db.get_all_code_files_meta()?)
        };

        let vault_path = self.vault_path.clone();

        // Perform filesystem traversal, mtime filtering, and AST/Markdown parsing off the async executor
        let scan_res = tokio::task::spawn_blocking(move || {
            let ignored_dirs: HashSet<&str> = [
                ".git",
                "node_modules",
                "target",
                "dist",
                "build",
                ".next",
                "vendor",
                ".fastembed_cache",
                "__pycache__",
                ".venv",
                "venv",
                "env",
                ".idea",
                ".vscode",
                "coverage",
                "out",
                ".turbo",
                ".gradle",
                "bin",
                "obj",
                ".gemini",
                "public",
                "graphfinder-out",
                "mcp-tools",
                "vaults",
                "bower_components",
                "Pods",
                "DerivedData",
                "zig-cache",
                "zig-out",
            ]
            .into_iter()
            .collect();

            let ignored_files: HashSet<&str> = [
                "package-lock.json",
                "yarn.lock",
                "pnpm-lock.yaml",
                "cargo.lock",
                "poetry.lock",
                "composer.lock",
                "gemfile.lock",
            ]
            .into_iter()
            .collect();

            let gitignore_patterns: Vec<String> = {
                let gi_path = vault_path.join(".gitignore");
                if gi_path.exists() {
                    if let Ok(content) = fs::read_to_string(&gi_path) {
                        content
                            .lines()
                            .map(|l| l.trim())
                            .filter(|l| !l.is_empty() && !l.starts_with('#') && !l.starts_with('!'))
                            .map(|l| l.trim_start_matches('/').trim_end_matches('/').to_string())
                            .filter(|l| !l.is_empty())
                            .collect()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                }
            };

            let mut md_entries: Vec<(PathBuf, String, i64)> = Vec::new();
            let mut code_entries: Vec<(PathBuf, String, i64)> = Vec::new();
            let mut project_files_set = HashSet::new();
            let mut project_files_by_name: HashMap<String, Vec<String>> = HashMap::new();
            let mut found_note_ids = HashSet::new();
            let mut found_code_ids = HashSet::new();

            for entry in WalkDir::new(&vault_path)
                .into_iter()
                .filter_entry(|e| {
                    if e.depth() == 0 {
                        return true;
                    }
                    let file_name = e.file_name().to_string_lossy();
                    if e.file_type().is_dir() {
                        if file_name.starts_with('.') || ignored_dirs.contains(file_name.as_ref()) {
                            return false;
                        }
                        if gitignore_patterns.iter().any(|p| file_name == p.as_str()) {
                            return false;
                        }
                    }
                    true
                })
                .filter_map(|e| e.ok())
            {
                if !entry.file_type().is_file() {
                    continue;
                }
                let path = entry.path();
                let file_name_lower = entry.file_name().to_string_lossy().to_lowercase();
                if ignored_files.contains(file_name_lower.as_str()) {
                    continue;
                }

                let rel_path = match path.strip_prefix(&vault_path) {
                    Ok(p) => p.to_string_lossy().replace('\\', "/"),
                    Err(_) => continue,
                };

                if gitignore_patterns.iter().any(|p| {
                    if p.starts_with('*') {
                        rel_path.ends_with(p.trim_start_matches('*'))
                    } else {
                        rel_path == *p || rel_path.starts_with(&format!("{}/", p))
                    }
                }) {
                    continue;
                }

                let meta = match entry.metadata() {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let file_len = meta.len();
                let updated_at = meta
                    .modified()
                    .map(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64
                    })
                    .unwrap_or(0);

                if path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if file_len > 2_097_152 {
                        continue;
                    }
                    found_note_ids.insert(rel_path.clone());
                    md_entries.push((path.to_path_buf(), rel_path, updated_at));
                } else if is_supported_code_file(path) {
                    if file_len > 262_144 {
                        continue;
                    }
                    let file_name = path
                        .file_name()
                        .and_then(|s| s.to_str())
                        .unwrap_or("")
                        .to_string();
                    found_code_ids.insert(rel_path.clone());
                    project_files_set.insert(rel_path.clone());
                    project_files_by_name
                        .entry(file_name)
                        .or_default()
                        .push(rel_path.clone());
                    code_entries.push((path.to_path_buf(), rel_path, updated_at));
                }
            }

            // Parse only changed Markdown notes
            let mut parsed_notes: Vec<(String, String, i64, String, String, ParsedNote)> =
                Vec::new();
            for (path, rel_path, updated_at) in md_entries {
                if let Some((existing_mtime, existing_hash)) = existing_note_meta.get(&rel_path) {
                    if *existing_mtime == updated_at && !existing_hash.is_empty() {
                        continue;
                    }
                }
                let content = match fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let hash = crate::engine::parser::content_hash(&content);
                if let Some((_, existing_hash)) = existing_note_meta.get(&rel_path) {
                    if existing_hash == &hash && !existing_hash.is_empty() {
                        continue;
                    }
                }
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("untitled");
                let parsed = parse_markdown(stem, &content);
                parsed_notes.push((rel_path.clone(), rel_path, updated_at, content, hash, parsed));
            }

            // Parse only changed Code files
            let mut parsed_codes: Vec<(
                ParsedCodeFile,
                String,
                i64,
                Vec<(String, String, bool)>,
            )> = Vec::new();
            for (path, rel_path, updated_at) in code_entries {
                if let Some((existing_mtime, existing_hash)) = existing_code_meta.get(&rel_path) {
                    if *existing_mtime == updated_at && !existing_hash.is_empty() {
                        continue;
                    }
                }
                let content = match fs::read_to_string(&path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };
                let hash = crate::engine::parser::content_hash(&content);
                if let Some((_, existing_hash)) = existing_code_meta.get(&rel_path) {
                    if existing_hash == &hash && !existing_hash.is_empty() {
                        continue;
                    }
                }

                let parsed = parse_code_file(&rel_path, &content);
                let mut resolved_links = Vec::with_capacity(parsed.imports.len());
                for imp in &parsed.imports {
                    if let Some(target_file) = resolve_import_path(
                        &rel_path,
                        &imp.target,
                        &project_files_set,
                        &project_files_by_name,
                    ) {
                        resolved_links.push((rel_path.clone(), target_file, false));
                    } else {
                        resolved_links.push((rel_path.clone(), imp.target.clone(), true));
                    }
                }
                parsed_codes.push((parsed, hash, updated_at, resolved_links));
            }

            (
                found_note_ids,
                found_code_ids,
                parsed_notes,
                parsed_codes,
            )
        })
        .await?;

        let (found_note_ids, found_code_ids, parsed_notes, parsed_codes) = scan_res;
        let mut count = 0;

        // 2. Index Changed Markdown Notes
        for (id, rel_path, updated_at, content, hash, parsed) in parsed_notes {
            let note_embed_text = format!(
                "{} {}\n{}",
                parsed.title,
                parsed.tags.join(" "),
                parsed.summary
            );

            let mut batch_texts = Vec::with_capacity(1 + parsed.chunks.len());
            batch_texts.push(note_embed_text);

            for chunk in &parsed.chunks {
                let chunk_text = format!(
                    "{} {} {}\n{}",
                    parsed.title,
                    chunk.heading.as_deref().unwrap_or(""),
                    chunk.tags.join(" "),
                    chunk.content
                );
                batch_texts.push(chunk_text);
            }

            let mut all_vectors = self.embeddings.embed_batch(batch_texts).await?;
            if all_vectors.is_empty() {
                continue;
            }

            let note_embedding = all_vectors.remove(0);
            let mut chunk_embeddings = Vec::with_capacity(parsed.chunks.len());
            for (chunk, vec) in parsed.chunks.into_iter().zip(all_vectors.into_iter()) {
                chunk_embeddings.push((chunk, vec));
            }

            {
                let mut db = self.db.lock().await;
                db.upsert_note(
                    &id,
                    &rel_path,
                    &parsed.title,
                    &parsed.summary,
                    &content,
                    &hash,
                    updated_at,
                    &parsed.links,
                    &parsed.tags,
                    Some(&note_embedding),
                    &chunk_embeddings,
                )?;
            }
            count += 1;
        }

        // Clean up notes in db that no longer exist on disk
        let mut notes_deleted = false;
        {
            let mut db = self.db.lock().await;
            let existing = db.get_all_notes()?;
            for n in existing {
                if !found_note_ids.contains(&n.id) {
                    db.delete_note(&n.id)?;
                    notes_deleted = true;
                }
            }
        }

        // 3. Index Changed Code Files in Multi-File Batches
        let is_large_sync = parsed_codes.len() > 80;
        let max_embedded_chunks_per_file = if is_large_sync { 4 } else { 16 };

        // If this is a large initial sync, immediately persist the structural AST & FTS5 records first
        // so the graph canvas and file lists are populated in <200ms while embeddings compute in batches!
        if parsed_codes.len() > 25 {
            let empty_vecs: Vec<Vec<f32>> = Vec::new();
            for chunk_group in parsed_codes.chunks(100) {
                let bulk_items: Vec<(
                    &ParsedCodeFile,
                    &str,
                    i64,
                    &[Vec<f32>],
                    &[(String, String, bool)],
                )> = chunk_group
                    .iter()
                    .map(|(p, h, mtime, links)| {
                        (p, h.as_str(), *mtime, empty_vecs.as_slice(), links.as_slice())
                    })
                    .collect();
                let mut db = self.db.lock().await;
                db.save_code_files_bulk(&bulk_items)?;
            }
            self.cached_count
                .store(found_note_ids.len() + found_code_ids.len(), Ordering::Relaxed);
            if let Some(handle) = app_handle {
                use tauri::Emitter;
                let _ = handle.emit("vault-sync-complete", 0usize);
            }
            tokio::task::yield_now().await;
        }

        for file_batch in parsed_codes.chunks(24) {
            let mut flat_texts = Vec::new();
            let mut counts_per_file = Vec::with_capacity(file_batch.len());

            for (parsed, _, _, _) in file_batch {
                let embed_count = parsed.chunks.len().min(max_embedded_chunks_per_file);
                counts_per_file.push(embed_count);
                for chunk in parsed.chunks.iter().take(embed_count) {
                    let preview: &str = if chunk.content.len() > 500 {
                        let mut end = 500;
                        while end > 0 && !chunk.content.is_char_boundary(end) {
                            end -= 1;
                        }
                        &chunk.content[..end]
                    } else {
                        &chunk.content
                    };
                    flat_texts.push(format!(
                        "{}: {} (lines {}-{})\n{}",
                        parsed.name,
                        chunk.symbol_name.as_deref().unwrap_or("code"),
                        chunk.start_line,
                        chunk.end_line,
                        preview
                    ));
                }
            }

            let flat_vectors = if !flat_texts.is_empty() {
                self.embeddings.embed_batch(flat_texts).await?
            } else {
                Vec::new()
            };

            let mut vec_iter = flat_vectors.into_iter();
            let mut per_file_vectors: Vec<Vec<Vec<f32>>> = Vec::with_capacity(file_batch.len());
            for &c_len in &counts_per_file {
                let mut file_vecs = Vec::with_capacity(c_len);
                for _ in 0..c_len {
                    if let Some(v) = vec_iter.next() {
                        file_vecs.push(v);
                    }
                }
                per_file_vectors.push(file_vecs);
            }

            let bulk_refs: Vec<(
                &ParsedCodeFile,
                &str,
                i64,
                &[Vec<f32>],
                &[(String, String, bool)],
            )> = file_batch
                .iter()
                .zip(per_file_vectors.iter())
                .map(|((parsed, hash, mtime, links), vecs)| {
                    (
                        parsed,
                        hash.as_str(),
                        *mtime,
                        vecs.as_slice(),
                        links.as_slice(),
                    )
                })
                .collect();

            {
                let mut db = self.db.lock().await;
                db.save_code_files_bulk(&bulk_refs)?;
            }
            count += file_batch.len();
            tokio::task::yield_now().await;
        }

        // Clean up deleted code files and resolve function/method call graph only if changes occurred
        {
            let mut db = self.db.lock().await;
            let deleted_code = db.delete_missing_code_files(&found_code_ids)?;
            if count > 0 || deleted_code > 0 {
                let _ = db.resolve_call_graph();
            }
            if notes_deleted || deleted_code > 0 || count > 0 {
                let total = db.total_indexed_count();
                self.cached_count.store(total, Ordering::Relaxed);
            }
        }

        Ok(count)
    }

    /// Saves a note to disk and updates the SQLite index, sections and embeddings
    pub async fn save_note(
        &self,
        title: &str,
        content: &str,
        custom_filename: Option<&str>,
    ) -> Result<NoteRecord, Box<dyn std::error::Error + Send + Sync>> {
        let filename = match custom_filename {
            Some(f) if !f.trim().is_empty() => {
                if f.ends_with(".md") {
                    f.to_string()
                } else {
                    format!("{}.md", f)
                }
            }
            _ => {
                let safe_title: String = title
                    .chars()
                    .map(|c| {
                        if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                            c
                        } else {
                            '_'
                        }
                    })
                    .collect();
                format!("{}.md", safe_title.trim())
            }
        };

        let file_path = self.vault_path.join(&filename);
        let rel_path = filename.replace('\\', "/");
        let stem = file_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("untitled");

        let id = rel_path.clone();

        // 1. Initial parse of the note and batch vectorization
        let parsed_initial = parse_markdown(stem, content);
        let note_embed_text = format!(
            "{} {}\n{}",
            parsed_initial.title,
            parsed_initial.tags.join(" "),
            parsed_initial.summary
        );

        let mut batch_texts = Vec::with_capacity(1 + parsed_initial.chunks.len());
        batch_texts.push(note_embed_text);
        for chunk in &parsed_initial.chunks {
            let chunk_text = format!(
                "{} {} {}\n{}",
                parsed_initial.title,
                chunk.heading.as_deref().unwrap_or(""),
                chunk.tags.join(" "),
                chunk.content
            );
            batch_texts.push(chunk_text);
        }

        let mut all_vectors = self.embeddings.embed_batch(batch_texts).await?;
        let note_embedding = if !all_vectors.is_empty() {
            all_vectors.remove(0)
        } else {
            vec![0.0f32; 384]
        };
        let chunk_vectors = all_vectors;

        // 2. Optional Auto-Link (only mutates markdown file if explicitly enabled via env var)
        let auto_link_enabled = std::env::var("GRAPHFINDER_AUTO_APPEND_LINKS")
            .map(|v| v == "1" || v == "true")
            .unwrap_or(false);

        let mut final_content = content.to_string();

        if auto_link_enabled {
            let mut auto_suggestions = Vec::new();
            let mut seen_targets = std::collections::HashSet::new();

            {
                let db = self.db.lock().await;
                if let Ok(suggestions) = db.suggest_links(&note_embedding, &id, 0.65, 10) {
                    for s in suggestions {
                        let key = s.target_title.to_lowercase();
                        if !seen_targets.contains(&key) {
                            seen_targets.insert(key);
                            auto_suggestions.push(s);
                        }
                    }
                }

                for (chunk, chunk_vec) in parsed_initial.chunks.iter().zip(chunk_vectors.iter()) {
                    if chunk.content.trim().len() > 30 {
                        if let Ok(suggestions) = db.suggest_links(chunk_vec, &id, 0.65, 5) {
                            for s in suggestions {
                                let key = s.target_title.to_lowercase();
                                if !seen_targets.contains(&key) {
                                    seen_targets.insert(key);
                                    auto_suggestions.push(s);
                                }
                            }
                        }
                    }
                }
            }

            let content_lower = content.to_lowercase();
            let mut links_to_add = Vec::new();
            for s in auto_suggestions {
                let target_lower = s.target_title.to_lowercase();
                let already_linked = content_lower.contains(&format!("[[{}", target_lower));
                if !already_linked {
                    links_to_add.push(s);
                }
            }

            links_to_add.sort_by(|a, b| {
                b.similarity_score
                    .partial_cmp(&a.similarity_score)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });

            if !links_to_add.is_empty() {
                if !final_content.ends_with('\n') {
                    final_content.push('\n');
                }
                if !final_content.contains("### Conexões Semânticas")
                    && !final_content.contains("## Conexões Semânticas")
                {
                    final_content.push_str("\n### Conexões Semânticas (Auto-Link 65%+)\n");
                }
                for s in &links_to_add {
                    let score_pct = (s.similarity_score * 100.0).round() as u32;
                    final_content.push_str(&format!(
                        "- Conexão: {} (Afinidade: {}%)\n",
                        s.deep_link, score_pct
                    ));
                }
            }
        }

        // 3. Write final content to disk
        fs::write(&file_path, &final_content)?;

        // 4. Read exact mtime from filesystem metadata
        let metadata = fs::metadata(&file_path)?;
        let exact_updated_at = metadata
            .modified()
            .map(|t| {
                t.duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64
            })
            .unwrap_or_else(|_| {
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs() as i64
            });

        let hash = crate::engine::parser::content_hash(&final_content);
        let parsed = parse_markdown(stem, &final_content);

        let mut chunk_embeddings = Vec::new();
        for (i, chunk) in parsed.chunks.into_iter().enumerate() {
            let vec = if i < chunk_vectors.len() {
                chunk_vectors[i].clone()
            } else {
                let chunk_text = format!(
                    "{} {} {}\n{}",
                    parsed.title,
                    chunk.heading.as_deref().unwrap_or(""),
                    chunk.tags.join(" "),
                    chunk.content
                );
                self.embeddings.embed_text(&chunk_text).await.unwrap_or_default()
            };
            chunk_embeddings.push((chunk, vec));
        }

        let mut db = self.db.lock().await;
        db.upsert_note(
            &id,
            &rel_path,
            &parsed.title,
            &parsed.summary,
            &final_content,
            &hash,
            exact_updated_at,
            &parsed.links,
            &parsed.tags,
            Some(&note_embedding),
            &chunk_embeddings,
        )?;
        self.cached_count.store(db.total_indexed_count(), Ordering::Relaxed);

        let note = db
            .get_note_by_id(&id)?
            .ok_or("Failed to retrieve note after save")?;
        Ok(note)
    }

    /// Renames a note on disk, refactoring all wikilinks across other notes in the vault
    pub async fn rename_note(
        &self,
        old_id_or_title: &str,
        new_title: &str,
    ) -> Result<NoteRecord, Box<dyn std::error::Error + Send + Sync>> {
        let note = self
            .get_note(old_id_or_title)
            .await?
            .ok_or_else(|| format!("Note '{}' not found", old_id_or_title))?;

        let old_title = note.title.clone();
        let old_path = self.vault_path.join(&note.path);

        let safe_title: String = new_title
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' || c == ' ' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        let new_filename = format!("{}.md", safe_title.trim());
        let new_path = self.vault_path.join(&new_filename);
        let new_id = new_filename.replace('\\', "/");

        // 1. Refactor wikilinks across all other vault markdown notes
        for entry in WalkDir::new(&self.vault_path).into_iter().filter_map(|e| e.ok()) {
            let p = entry.path();
            if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("md") && p != old_path {
                if let Ok(c) = fs::read_to_string(p) {
                    let (rewritten, count) = crate::engine::parser::rewrite_wikilinks(&c, &old_title, new_title);
                    if count > 0 {
                        let _ = fs::write(p, rewritten);
                    }
                }
            }
        }

        // 2. Update heading/title in the note itself
        let updated_content = crate::engine::parser::replace_title(&note.content, new_title);

        // 3. Rename file on disk
        if old_path != new_path {
            if old_path.exists() {
                let _ = fs::remove_file(&old_path);
            }
        }
        fs::write(&new_path, &updated_content)?;

        // 4. Update SQLite database record
        {
            let mut db = self.db.lock().await;
            db.rename_note(&note.id, &new_id, &new_filename, new_title)?;
        }

        // 5. Re-index
        self.sync_vault().await?;
        self.get_note(&new_id)
            .await?
            .ok_or_else(|| "Failed to load renamed note".into())
    }

    /// Deletes a note from disk and cleans up SQLite database
    pub async fn delete_note(
        &self,
        id_or_title: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let note = self
            .get_note(id_or_title)
            .await?
            .ok_or_else(|| format!("Note '{}' not found", id_or_title))?;

        let file_path = self.vault_path.join(&note.path);
        if file_path.exists() {
            let _ = fs::remove_file(&file_path);
        }

        let mut db = self.db.lock().await;
        db.delete_note(&note.id)?;
        self.cached_count.store(db.total_indexed_count(), Ordering::Relaxed);
        Ok(())
    }

    pub async fn get_note(
        &self,
        id_or_title: &str,
    ) -> Result<Option<NoteRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().await;
        if let Some(n) = db.get_note_by_id(id_or_title)? {
            return Ok(Some(n));
        }
        Ok(db.get_note_by_title(id_or_title)?)
    }

    pub async fn list_notes(
        &self,
    ) -> Result<Vec<NoteSummary>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().await;
        Ok(db.get_all_notes()?)
    }

    pub async fn get_vault_stats(
        &self,
    ) -> Result<VaultStats, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().await;
        Ok(db.get_vault_stats()?)
    }

    pub async fn get_graph(&self) -> Result<GraphData, Box<dyn std::error::Error + Send + Sync>> {
        let mut graph = {
            let db = self.db.lock().await;
            db.get_graph_data()?
        };
        let analytics = super::graph_analytics::GraphAnalyticsEngine::new(&graph, HashMap::new());
        let report = analytics.analyze();
        let god_ids: HashSet<String> = report.god_nodes.into_iter().map(|g| g.id).collect();

        for node in &mut graph.nodes {
            if let Some(idx) = analytics.find_node_index(&node.id) {
                node.community = Some(analytics.node_community(idx));
                node.is_god_node = god_ids.contains(&node.id);
            }
        }
        Ok(graph)
    }

    pub async fn get_analytics_engine(
        &self,
    ) -> Result<super::graph_analytics::GraphAnalyticsEngine, Box<dyn std::error::Error + Send + Sync>> {
        let (graph, summaries) = {
            let db = self.db.lock().await;
            (db.get_graph_data()?, db.get_all_summaries()?)
        };
        Ok(super::graph_analytics::GraphAnalyticsEngine::new(&graph, summaries))
    }

    pub async fn analyze_graph(
        &self,
    ) -> Result<super::graph_analytics::GraphAnalysisReport, Box<dyn std::error::Error + Send + Sync>> {
        let engine = self.get_analytics_engine().await?;
        Ok(engine.analyze())
    }

    pub async fn find_path(
        &self,
        source: &str,
        target: &str,
    ) -> Result<Option<super::graph_analytics::PathResult>, Box<dyn std::error::Error + Send + Sync>> {
        let engine = self.get_analytics_engine().await?;
        Ok(engine.find_path(source, target))
    }

    pub async fn explain_node(
        &self,
        target: &str,
    ) -> Result<Option<super::graph_analytics::NodeExplanation>, Box<dyn std::error::Error + Send + Sync>> {
        let engine = self.get_analytics_engine().await?;
        Ok(engine.explain_node(target))
    }

    pub async fn export_artifacts(
        &self,
        out_dir: Option<&Path>,
    ) -> Result<super::graph_analytics::ExportSummary, Box<dyn std::error::Error + Send + Sync>> {
        let engine = self.get_analytics_engine().await?;
        let target_dir = match out_dir {
            Some(d) => d.to_path_buf(),
            None => self.vault_path.join("graphfinder-out"),
        };
        let project_name = self.vault_path.file_name().and_then(|s| s.to_str()).unwrap_or("GraphFinder Project");
        Ok(engine.export_artifacts(&target_dir, project_name)?)
    }

    pub async fn get_neighborhood(
        &self,
        target: &str,
        depth: usize,
    ) -> Result<Neighborhood, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().await;
        Ok(db.get_neighborhood(target, depth)?)
    }

    pub async fn search_semantic(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchHit>, Box<dyn std::error::Error + Send + Sync>> {
        let query_emb = self.embeddings.embed_text(query).await?;
        let db = self.db.lock().await;
        Ok(db.semantic_search(&query_emb, limit)?)
    }

    /// Hybrid search: combines dense vector similarity and BM25 FTS5 using Reciprocal Rank Fusion, with optional neural Reranker
    pub async fn search_chunks(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ChunkHit>, Box<dyn std::error::Error + Send + Sync>> {
        let query_emb = self.embeddings.embed_text(query).await.unwrap_or_default();
        let candidate_limit = if self.embeddings.is_reranker_ready() {
            (limit * 2).max(20)
        } else {
            limit
        };
        let mut hits = {
            let db = self.db.lock().await;
            db.search_chunks_hybrid(query, &query_emb, candidate_limit)?
        };
        self.embeddings.rerank_chunks(query, &mut hits, limit);
        Ok(hits)
    }

    /// Recommends existing notes/sections to link to while writing
    pub async fn suggest_links(
        &self,
        text: &str,
        exclude_note_id: &str,
        threshold: f32,
        limit: usize,
    ) -> Result<Vec<LinkSuggestion>, Box<dyn std::error::Error + Send + Sync>> {
        let text_emb = self.embeddings.embed_text(text).await?;
        let db = self.db.lock().await;
        Ok(db.suggest_links(&text_emb, exclude_note_id, threshold, limit)?)
    }

    /// Compressed context digest for AI prompt token savings
    pub async fn get_compressed_context(
        &self,
        target: &str,
    ) -> Result<Option<CompressedContext>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().await;
        Ok(db.get_compressed_context(target)?)
    }

    /// Retrieves an exact atomic section by target note and heading/fragment
    pub async fn read_section(
        &self,
        target: &str,
        section: &str,
    ) -> Result<Option<ChunkHit>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().await;
        Ok(db.get_section(target, section)?)
    }


    pub async fn link_notes(
        &self,
        from_title_or_id: &str,
        to_title: &str,
    ) -> Result<NoteRecord, Box<dyn std::error::Error + Send + Sync>> {
        let note = self
            .get_note(from_title_or_id)
            .await?
            .ok_or("Source note not found")?;

        let link_text = format!("\n- Relacionado: [[{}]]", to_title);
        let mut new_content = note.content.clone();
        if !new_content.contains(&format!("[[{}]]", to_title)) {
            new_content.push_str(&link_text);
        }

        self.save_note(&note.title, &new_content, Some(&note.path))
            .await
    }

    // ==========================================
    // Codebase Query & Search Methods
    // ==========================================

    pub async fn search_code(
        &self,
        query: &str,
        limit: usize,
        lang_filter: Option<&str>,
    ) -> Result<Vec<CodeChunkHit>, Box<dyn std::error::Error + Send + Sync>> {
        let query_emb = self.embeddings.embed_text(query).await.unwrap_or_default();
        let candidate_limit = if self.embeddings.is_reranker_ready() {
            (limit * 2).max(20)
        } else {
            limit
        };
        let mut hits = {
            let db = self.db.lock().await;
            db.search_code_hybrid(query, &query_emb, candidate_limit, lang_filter)?
        };
        self.embeddings.rerank_code_chunks(query, &mut hits, limit);
        Ok(hits)
    }

    pub async fn find_files(
        &self,
        pattern: Option<&str>,
        language: Option<&str>,
        limit: usize,
    ) -> Result<Vec<CodeFileRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().await;
        Ok(db.find_files(pattern, language, limit)?)
    }

    pub async fn get_file_context(
        &self,
        file_path: &str,
    ) -> Result<Option<FileContext>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().await;
        Ok(db.get_file_context(file_path)?)
    }

    pub async fn read_code_chunk(
        &self,
        file_path: &str,
        symbol_or_lines: Option<&str>,
    ) -> Result<Option<CodeChunkHit>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().await;
        Ok(db.read_code_chunk(file_path, symbol_or_lines)?)
    }

    pub async fn get_code_dependencies(
        &self,
        file_path: &str,
        direction: &str,
    ) -> Result<CodeDependencies, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().await;
        Ok(db.get_code_dependencies(file_path, direction)?)
    }

    pub async fn list_code_files(
        &self,
        limit: usize,
    ) -> Result<Vec<CodeFileRecord>, Box<dyn std::error::Error + Send + Sync>> {
        let db = self.db.lock().await;
        Ok(db.list_code_files(limit)?)
    }
}

