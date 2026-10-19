use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use walkdir::WalkDir;

use super::db::{
    ChunkHit, CompressedContext, DbManager, GraphData, LinkSuggestion, Neighborhood, NoteRecord,
    NoteSummary, SearchHit,
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
    if Path::new("../package.json").exists() && Path::new("../vault").exists() {
        return PathBuf::from("../vault");
    }
    PathBuf::from("./vault")
}

pub fn resolve_db_path() -> PathBuf {
    if let Ok(d) = std::env::var("GRAPHFINDER_DB") {
        return PathBuf::from(d);
    }
    if Path::new("../package.json").exists() && Path::new("../vault").exists() {
        return PathBuf::from("../index.db");
    }
    PathBuf::from("./index.db")
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

    #[allow(dead_code)]
    pub fn vault_path(&self) -> &Path {
        &self.vault_path
    }

    #[allow(dead_code)]
    pub fn db_path(&self) -> &Path {
        &self.db_path
    }

    /// Scans the vault directory and indexes all markdown files and their atomic chunks into SQLite
    pub async fn sync_vault(&self) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
        let mut count = 0;
        let mut found_ids = Vec::new();

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
                let stem = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("untitled");

                let content = match fs::read_to_string(path) {
                    Ok(c) => c,
                    Err(_) => continue,
                };

                let metadata = fs::metadata(path)?;
                let updated_at = metadata
                    .modified()
                    .map(|t| {
                        t.duration_since(std::time::UNIX_EPOCH)
                            .unwrap_or_default()
                            .as_secs() as i64
                    })
                    .unwrap_or(0);

                let parsed = parse_markdown(stem, &content);
                let id = rel_path.clone();
                found_ids.push(id.clone());

                // Embed the note overview
                let note_embed_text = format!(
                    "{} {}\n{}",
                    parsed.title,
                    parsed.tags.join(" "),
                    parsed.summary
                );
                let note_embedding = self.embeddings.embed_text(&note_embed_text).await?;

                // Embed each section chunk individually for granular deep-linking
                let mut chunk_embeddings = Vec::new();
                for chunk in parsed.chunks {
                    let chunk_text = format!(
                        "{} {} {}\n{}",
                        parsed.title,
                        chunk.heading.as_deref().unwrap_or(""),
                        chunk.tags.join(" "),
                        chunk.content
                    );
                    let chunk_vec = self.embeddings.embed_text(&chunk_text).await?;
                    chunk_embeddings.push((chunk, chunk_vec));
                }

                let mut db = self.db.lock().await;
                db.upsert_note(
                    &id,
                    &rel_path,
                    &parsed.title,
                    &parsed.summary,
                    &content,
                    updated_at,
                    &parsed.links,
                    &parsed.tags,
                    Some(&note_embedding),
                    &chunk_embeddings,
                )?;
                count += 1;
            }
        }

        // Clean up notes in db that no longer exist on disk
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

        // 1. Initial parse of the note to compute semantic embeddings
        let parsed_initial = parse_markdown(stem, content);
        let note_embed_text = format!(
            "{} {}\n{}",
            parsed_initial.title,
            parsed_initial.tags.join(" "),
            parsed_initial.summary
        );
        let note_embedding = self.embeddings.embed_text(&note_embed_text).await?;

        // 2. Semantic Auto-Link (threshold: 0.65 = 65% affinity)
        // Find existing notes and sections in the vault that have >= 65% affinity
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
        }

        // Also check individual chunks (if note has multiple sections)
        for chunk in &parsed_initial.chunks {
            if chunk.content.trim().len() > 30 {
                let chunk_text = format!(
                    "{} {}\n{}",
                    parsed_initial.title,
                    chunk.heading.as_deref().unwrap_or(""),
                    chunk.content
                );
                if let Ok(chunk_vec) = self.embeddings.embed_text(&chunk_text).await {
                    let db = self.db.lock().await;
                    if let Ok(suggestions) = db.suggest_links(&chunk_vec, &id, 0.65, 5) {
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

        // 3. Filter out targets that are already linked in the content
        let content_lower = content.to_lowercase();
        let mut links_to_add = Vec::new();

        for s in auto_suggestions {
            let target_lower = s.target_title.to_lowercase();
            // Check if content already contains [[TargetTitle or [[TargetTitle#...
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

        // 4. Format and append auto-links if any match >= 65%
        let mut final_content = content.to_string();
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

        // 5. Write final_content (with auto-links included) to disk
        fs::write(&file_path, &final_content)?;

        // 6. Re-parse final_content so parsed.links includes the newly created wikilinks
        let parsed = parse_markdown(stem, &final_content);

        // Generate chunk-level embeddings
        let mut chunk_embeddings = Vec::new();
        for chunk in parsed.chunks {
            let chunk_text = format!(
                "{} {} {}\n{}",
                parsed.title,
                chunk.heading.as_deref().unwrap_or(""),
                chunk.tags.join(" "),
                chunk.content
            );
            let chunk_vec = self.embeddings.embed_text(&chunk_text).await?;
            chunk_embeddings.push((chunk, chunk_vec));
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let mut db = self.db.lock().await;
        db.upsert_note(
            &id,
            &rel_path,
            &parsed.title,
            &parsed.summary,
            &final_content,
            now,
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

    /// Cirurgical search returning exact token-efficient sections with deep links
    pub async fn search_chunks(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<ChunkHit>, Box<dyn std::error::Error + Send + Sync>> {
        let query_emb = self.embeddings.embed_text(query).await?;
        let db = self.db.lock().await;
        Ok(db.search_chunks(&query_emb, limit)?)
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
