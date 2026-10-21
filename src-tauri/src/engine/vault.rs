use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use walkdir::WalkDir;

use super::db::{
    ChunkHit, CompressedContext, DbManager, GraphData, LinkSuggestion, Neighborhood, NoteRecord,
    NoteSummary, SearchHit, VaultStats,
};
use super::embeddings::EmbeddingEngine;
use super::parser::parse_markdown;

#[derive(Clone)]
pub struct Engine {
    vault_path: PathBuf,
    db_path: PathBuf,
    db: Arc<Mutex<DbManager>>,
    embeddings: Arc<EmbeddingEngine>,
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

        Ok(Self {
            vault_path,
            db_path,
            db: Arc::new(Mutex::new(db)),
            embeddings,
        })
    }

    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Scans the vault directory and indexes all markdown files and their atomic chunks into SQLite
    pub async fn sync_vault(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut count = 0;
        let mut found_ids = HashSet::new();

        // 1. Fetch existing timestamps and content hashes for instant incremental sync
        let existing_meta = {
            let db = self.db.lock().await;
            db.get_all_notes_meta()?
        };

        for entry in WalkDir::new(&self.vault_path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                let rel_path = path
                    .strip_prefix(&self.vault_path)?
                    .to_string_lossy()
                    .replace('\\', "/");
                let id = rel_path.clone();
                found_ids.insert(id.clone());

                let metadata = match fs::metadata(path) {
                    Ok(m) => m,
                    Err(_) => continue,
                };
                let updated_at = metadata
                    .modified()
                    .map(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64
                    })
                    .unwrap_or(0);

                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let hash = crate::engine::parser::content_hash(&content);

                // Incremental check: if mtime AND content hash match, skip re-embedding
                if let Some((existing_mtime, existing_hash)) = existing_meta.get(&id) {
                    if *existing_mtime == updated_at && existing_hash == &hash {
                        continue;
                    }
                }

                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("untitled");

                let parsed = parse_markdown(stem, &content);

                // Batch all texts (overview + sections) into a single vectorized ONNX inference
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
                count += 1;
            }
        }

        // Clean up notes in db that no longer exist on disk (O(1) HashSet check)
        let mut db = self.db.lock().await;
        let existing = db.get_all_notes()?;
        for n in existing {
            if !found_ids.contains(&n.id) {
                db.delete_note(&n.id)?;
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
            .ok_or_else(|| format!("Nota '{}' não encontrada", old_id_or_title))?;

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
            .ok_or_else(|| "Erro ao carregar nota renomeada".into())
    }

    /// Deletes a note from disk and cleans up SQLite database
    pub async fn delete_note(
        &self,
        id_or_title: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let note = self
            .get_note(id_or_title)
            .await?
            .ok_or_else(|| format!("Nota '{}' não encontrada", id_or_title))?;

        let file_path = self.vault_path.join(&note.path);
        if file_path.exists() {
            let _ = fs::remove_file(&file_path);
        }

        let mut db = self.db.lock().await;
        db.delete_note(&note.id)?;
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
        let db = self.db.lock().await;
        Ok(db.get_graph_data()?)
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

    /// Hybrid search: combines dense vector similarity and BM25 FTS5 using Reciprocal Rank Fusion
    pub async fn search_chunks(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ChunkHit>, Box<dyn std::error::Error + Send + Sync>> {
        let query_emb = self.embeddings.embed_text(query).await.unwrap_or_default();
        let db = self.db.lock().await;
        Ok(db.search_chunks_hybrid(query, &query_emb, limit)?)
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
}
