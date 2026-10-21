use super::parser::ParsedChunk;
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteRecord {
    pub id: String,
    pub path: String,
    pub title: String,
    pub summary: String,
    pub content: String,
    #[serde(default)]
    pub content_hash: String,
    pub updated_at: i64,
    pub tags: Vec<String>,
    pub outgoing_links: Vec<String>,
    pub backlinks: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoteSummary {
    pub id: String,
    pub path: String,
    pub title: String,
    pub summary: String,
    pub updated_at: i64,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub title: String,
    pub tags: Vec<String>,
    pub degree: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphData {
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchHit {
    pub id: String,
    pub title: String,
    pub snippet: String,
    pub score: f32,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkHit {
    pub chunk_id: String,
    pub note_id: String,
    pub note_title: String,
    pub heading: Option<String>,
    pub deep_link: String,
    pub content: String,
    pub token_estimate: usize,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkSuggestion {
    pub target_note_id: String,
    pub target_title: String,
    pub heading: Option<String>,
    pub deep_link: String,
    pub similarity_score: f32,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionSummary {
    pub heading: String,
    pub token_estimate: usize,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressedContext {
    pub note_id: String,
    pub title: String,
    pub summary: String,
    pub tags: Vec<String>,
    pub token_estimate: usize,
    pub connected_notes: Vec<String>,
    pub sections: Vec<SectionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neighborhood {
    pub center: Option<NoteRecord>,
    pub nodes: Vec<GraphNode>,
    pub links: Vec<GraphLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultStats {
    pub total_notes: usize,
    pub total_chunks: usize,
    pub total_links: usize,
    pub total_tags: usize,
    pub total_tokens: usize,
}

pub struct DbManager {
    conn: Connection,
}

impl DbManager {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            r#"
            PRAGMA journal_mode = WAL;
            PRAGMA synchronous = NORMAL;
            PRAGMA temp_store = MEMORY;
            PRAGMA cache_size = -64000;
            PRAGMA mmap_size = 268435456;
            PRAGMA foreign_keys = ON;
            "#,
        )?;
        let manager = Self { conn };
        manager.init_schema()?;
        Ok(manager)
    }

    #[cfg(test)]
    pub fn memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch(
            r#"
            PRAGMA foreign_keys = ON;
            "#,
        )?;
        let manager = Self { conn };
        manager.init_schema()?;
        Ok(manager)
    }

    pub fn get_all_notes_mtimes(&self) -> Result<HashMap<String, i64>> {
        let mut stmt = self.conn.prepare("SELECT id, updated_at FROM notes")?;
        let rows = stmt.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, i64>(1)?)))?;
        let mut map = HashMap::new();
        for item in rows {
            let (id, mtime) = item?;
            map.insert(id, mtime);
        }
        Ok(map)
    }

    pub fn get_all_notes_meta(&self) -> Result<HashMap<String, (i64, String)>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, updated_at, content_hash FROM notes")?;
        let rows = stmt.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, i64>(1)?,
                r.get::<_, String>(2).unwrap_or_default(),
            ))
        })?;
        let mut map = HashMap::new();
        for item in rows {
            let (id, mtime, hash) = item?;
            map.insert(id, (mtime, hash));
        }
        Ok(map)
    }

    pub fn get_note_mtime(&self, id: &str) -> Result<Option<i64>> {
        let mut stmt = self.conn.prepare("SELECT updated_at FROM notes WHERE id = ?1")?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS meta (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                summary TEXT NOT NULL DEFAULT '',
                content TEXT NOT NULL,
                content_hash TEXT NOT NULL DEFAULT '',
                updated_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS note_chunks (
                id TEXT PRIMARY KEY,
                note_id TEXT NOT NULL,
                chunk_index INTEGER NOT NULL,
                heading TEXT,
                content TEXT NOT NULL,
                token_estimate INTEGER NOT NULL,
                vector BLOB NOT NULL
            );

            CREATE TABLE IF NOT EXISTS links (
                source_id TEXT NOT NULL,
                target_title TEXT NOT NULL,
                label TEXT,
                PRIMARY KEY (source_id, target_title)
            );

            CREATE TABLE IF NOT EXISTS tags (
                note_id TEXT NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (note_id, tag)
            );

            CREATE TABLE IF NOT EXISTS embeddings (
                note_id TEXT PRIMARY KEY,
                vector BLOB NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS notes_fts USING fts5(
                id UNINDEXED,
                title,
                content,
                tokenize='unicode61 remove_diacritics 2'
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS chunks_fts USING fts5(
                chunk_id UNINDEXED,
                note_id UNINDEXED,
                note_title,
                heading,
                content,
                tokenize='unicode61 remove_diacritics 2'
            );

            CREATE INDEX IF NOT EXISTS idx_notes_title ON notes(title);
            CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_title);
            CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);
            CREATE INDEX IF NOT EXISTS idx_chunks_note ON note_chunks(note_id);
            "#,
        )?;

        // Safe migrations for legacy databases
        let _ = self.conn.execute(
            "ALTER TABLE notes ADD COLUMN summary TEXT NOT NULL DEFAULT ''",
            [],
        );
        let _ = self.conn.execute(
            "ALTER TABLE notes ADD COLUMN content_hash TEXT NOT NULL DEFAULT ''",
            [],
        );

        Ok(())
    }

    #[allow(clippy::too_many_arguments)]
    pub fn upsert_note(
        &mut self,
        id: &str,
        path: &str,
        title: &str,
        summary: &str,
        content: &str,
        content_hash: &str,
        updated_at: i64,
        links: &[String],
        tags: &[String],
        note_embedding: Option<&[f32]>,
        chunks: &[(ParsedChunk, Vec<f32>)],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        // 1. Upsert note
        tx.execute(
            r#"
            INSERT INTO notes (id, path, title, summary, content, content_hash, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                path = excluded.path,
                title = excluded.title,
                summary = excluded.summary,
                content = excluded.content,
                content_hash = excluded.content_hash,
                updated_at = excluded.updated_at
            "#,
            params![id, path, title, summary, content, content_hash, updated_at],
        )?;

        // 2. Refresh links
        tx.execute("DELETE FROM links WHERE source_id = ?1", params![id])?;
        for link in links {
            tx.execute(
                "INSERT OR IGNORE INTO links (source_id, target_title) VALUES (?1, ?2)",
                params![id, link],
            )?;
        }

        // 3. Refresh tags
        tx.execute("DELETE FROM tags WHERE note_id = ?1", params![id])?;
        for tag in tags {
            tx.execute(
                "INSERT OR IGNORE INTO tags (note_id, tag) VALUES (?1, ?2)",
                params![id, tag],
            )?;
        }

        // 4. Update note embedding
        if let Some(emb) = note_embedding {
            let bytes = f32_slice_to_bytes(emb);
            tx.execute(
                r#"
                INSERT INTO embeddings (note_id, vector)
                VALUES (?1, ?2)
                ON CONFLICT(note_id) DO UPDATE SET vector = excluded.vector
                "#,
                params![id, bytes],
            )?;
        }

        // 5. Replace semantic chunks
        tx.execute("DELETE FROM note_chunks WHERE note_id = ?1", params![id])?;
        tx.execute("DELETE FROM chunks_fts WHERE note_id = ?1", params![id])?;

        for (chunk, vec) in chunks {
            let chunk_id = format!("{}#chunk-{}", id, chunk.chunk_index);
            let bytes = f32_slice_to_bytes(vec);
            tx.execute(
                r#"
                INSERT INTO note_chunks (id, note_id, chunk_index, heading, content, token_estimate, vector)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                "#,
                params![
                    chunk_id,
                    id,
                    chunk.chunk_index as i64,
                    chunk.heading.as_deref(),
                    chunk.content,
                    chunk.token_estimate as i64,
                    bytes
                ],
            )?;

            // Index in FTS5 for fast hybrid search
            tx.execute(
                r#"
                INSERT INTO chunks_fts (chunk_id, note_id, note_title, heading, content)
                VALUES (?1, ?2, ?3, ?4, ?5)
                "#,
                params![
                    chunk_id,
                    id,
                    title,
                    chunk.heading.as_deref().unwrap_or(""),
                    chunk.content
                ],
            )?;
        }

        // 6. Update document-level FTS
        tx.execute("DELETE FROM notes_fts WHERE id = ?1", params![id])?;
        tx.execute(
            "INSERT INTO notes_fts (id, title, content) VALUES (?1, ?2, ?3)",
            params![id, title, content],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn delete_note(&mut self, id: &str) -> Result<()> {
        let tx = self.conn.transaction()?;
        tx.execute("DELETE FROM notes WHERE id = ?1", params![id])?;
        tx.execute("DELETE FROM note_chunks WHERE note_id = ?1", params![id])?;
        tx.execute("DELETE FROM chunks_fts WHERE note_id = ?1", params![id])?;
        tx.execute("DELETE FROM links WHERE source_id = ?1", params![id])?;
        tx.execute("DELETE FROM tags WHERE note_id = ?1", params![id])?;
        tx.execute("DELETE FROM embeddings WHERE note_id = ?1", params![id])?;
        tx.execute("DELETE FROM notes_fts WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(())
    }

    pub fn rename_note(
        &mut self,
        old_id: &str,
        new_id: &str,
        new_path: &str,
        new_title: &str,
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        tx.execute(
            r#"
            UPDATE notes 
            SET id = ?1, path = ?2, title = ?3
            WHERE id = ?4
            "#,
            params![new_id, new_path, new_title, old_id],
        )?;

        tx.execute(
            "UPDATE note_chunks SET note_id = ?1 WHERE note_id = ?2",
            params![new_id, old_id],
        )?;
        tx.execute(
            "UPDATE chunks_fts SET note_id = ?1, note_title = ?2 WHERE note_id = ?3",
            params![new_id, new_title, old_id],
        )?;
        tx.execute(
            "UPDATE links SET source_id = ?1 WHERE source_id = ?2",
            params![new_id, old_id],
        )?;
        tx.execute(
            "UPDATE tags SET note_id = ?1 WHERE note_id = ?2",
            params![new_id, old_id],
        )?;
        tx.execute(
            "UPDATE embeddings SET note_id = ?1 WHERE note_id = ?2",
            params![new_id, old_id],
        )?;
        tx.execute(
            "UPDATE notes_fts SET id = ?1, title = ?2 WHERE id = ?3",
            params![new_id, new_title, old_id],
        )?;

        tx.commit()?;
        Ok(())
    }

    pub fn get_note_by_id(&self, id: &str) -> Result<Option<NoteRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, title, summary, content, content_hash, updated_at FROM notes WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let path: String = row.get(1)?;
            let title: String = row.get(2)?;
            let summary: String = row.get(3)?;
            let content: String = row.get(4)?;
            let content_hash: String = row.get(5).unwrap_or_default();
            let updated_at: i64 = row.get(6)?;

            let tags = self.get_tags_for_note(&id)?;
            let outgoing_links = self.get_outgoing_links_for_note(&id)?;
            let backlinks = self.get_backlinks_for_title(&title)?;

            Ok(Some(NoteRecord {
                id,
                path,
                title,
                summary,
                content,
                content_hash,
                updated_at,
                tags,
                outgoing_links,
                backlinks,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn get_note_by_title(&self, title: &str) -> Result<Option<NoteRecord>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id FROM notes WHERE LOWER(title) = LOWER(?1) LIMIT 1")?;
        let mut rows = stmt.query(params![title])?;
        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            self.get_note_by_id(&id)
        } else {
            Ok(None)
        }
    }

    pub fn get_all_notes(&self) -> Result<Vec<NoteSummary>> {
        let mut tag_stmt = self.conn.prepare("SELECT note_id, tag FROM tags ORDER BY tag ASC")?;
        let tag_rows = tag_stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut tags_by_note: HashMap<String, Vec<String>> = HashMap::new();
        for r in tag_rows {
            let (note_id, tag) = r?;
            tags_by_note.entry(note_id).or_default().push(tag);
        }

        let mut stmt = self.conn.prepare(
            "SELECT id, path, title, summary, updated_at FROM notes ORDER BY updated_at DESC",
        )?;
        let rows = stmt.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
                row.get::<_, i64>(4)?,
            ))
        })?;

        let mut results = Vec::new();
        for item in rows {
            let (id, path, title, summary, updated_at) = item?;
            let tags = tags_by_note.remove(&id).unwrap_or_default();
            results.push(NoteSummary {
                id,
                path,
                title,
                summary,
                updated_at,
                tags,
            });
        }
        Ok(results)
    }

    pub fn get_vault_stats(&self) -> Result<VaultStats> {
        let total_notes: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM notes", [], |r| r.get(0))?;
        let total_chunks: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM note_chunks", [], |r| r.get(0))?;
        let total_links: usize = self
            .conn
            .query_row("SELECT COUNT(*) FROM links", [], |r| r.get(0))?;
        let total_tags: usize = self
            .conn
            .query_row("SELECT COUNT(DISTINCT tag) FROM tags", [], |r| r.get(0))?;
        let total_tokens: usize = self
            .conn
            .query_row(
                "SELECT COALESCE(SUM(token_estimate), 0) FROM note_chunks",
                [],
                |r| r.get(0),
            )?;

        Ok(VaultStats {
            total_notes,
            total_chunks,
            total_links,
            total_tags,
            total_tokens,
        })
    }

    pub fn get_graph_data(&self) -> Result<GraphData> {
        let mut stmt_titles = self.conn.prepare("SELECT id, title FROM notes")?;
        let mut title_to_id: HashMap<String, String> = HashMap::new();
        let mut id_to_title: HashMap<String, String> = HashMap::new();

        let title_rows = stmt_titles.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;

        for r in title_rows {
            let (id, title) = r?;
            title_to_id.insert(title.to_lowercase(), id.clone());
            id_to_title.insert(id, title);
        }

        let mut stmt_links = self
            .conn
            .prepare("SELECT source_id, target_title, label FROM links")?;
        let mut links = Vec::new();
        let mut degree_map: HashMap<String, usize> = HashMap::new();

        let link_rows = stmt_links.query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;

        for r in link_rows {
            let (source_id, target_title, label) = r?;
            let base_title = target_title
                .split('#')
                .next()
                .unwrap_or(&target_title)
                .trim();
            let target_key = base_title.to_lowercase();
            let target_id = if let Some(id) = title_to_id.get(&target_key) {
                id.clone()
            } else {
                format!("unresolved:{}", target_title)
            };

            *degree_map.entry(source_id.clone()).or_insert(0) += 1;
            *degree_map.entry(target_id.clone()).or_insert(0) += 1;

            links.push(GraphLink {
                source: source_id,
                target: target_id,
                label,
            });
        }

        let mut tag_stmt = self.conn.prepare("SELECT note_id, tag FROM tags ORDER BY tag ASC")?;
        let tag_rows = tag_stmt.query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut tags_by_note: HashMap<String, Vec<String>> = HashMap::new();
        for r in tag_rows {
            let (note_id, tag) = r?;
            tags_by_note.entry(note_id).or_default().push(tag);
        }

        let mut nodes = Vec::new();
        for (id, title) in &id_to_title {
            let tags = tags_by_note.remove(id).unwrap_or_default();
            let degree = *degree_map.get(id).unwrap_or(&0);
            nodes.push(GraphNode {
                id: id.clone(),
                title: title.clone(),
                tags,
                degree,
            });
        }

        let mut seen_unresolved = HashSet::new();
        for link in &links {
            if link.target.starts_with("unresolved:") && !seen_unresolved.contains(&link.target) {
                seen_unresolved.insert(link.target.clone());
                let raw_title = link.target.trim_start_matches("unresolved:");
                let degree = *degree_map.get(&link.target).unwrap_or(&1);
                nodes.push(GraphNode {
                    id: link.target.clone(),
                    title: raw_title.to_string(),
                    tags: vec!["unresolved".to_string()],
                    degree,
                });
            }
        }

        Ok(GraphData { nodes, links })
    }

    pub fn get_neighborhood(&self, note_id_or_title: &str, depth: usize) -> Result<Neighborhood> {
        let center = self
            .get_note_by_id(note_id_or_title)?
            .or_else(|| self.get_note_by_title(note_id_or_title).ok().flatten());

        let graph = self.get_graph_data()?;
        if center.is_none() {
            return Ok(Neighborhood {
                center: None,
                nodes: Vec::new(),
                links: Vec::new(),
            });
        }

        let center_note = center.unwrap();
        let center_id = center_note.id.clone();

        let mut visited = HashSet::new();
        let mut current_level = HashSet::new();
        current_level.insert(center_id.clone());
        visited.insert(center_id.clone());

        for _ in 0..depth {
            let mut next_level = HashSet::new();
            for link in &graph.links {
                if current_level.contains(&link.source) && !visited.contains(&link.target) {
                    visited.insert(link.target.clone());
                    next_level.insert(link.target.clone());
                }
                if current_level.contains(&link.target) && !visited.contains(&link.source) {
                    visited.insert(link.source.clone());
                    next_level.insert(link.source.clone());
                }
            }
            current_level = next_level;
        }

        let sub_nodes: Vec<GraphNode> = graph
            .nodes
            .into_iter()
            .filter(|n| visited.contains(&n.id))
            .collect();

        let sub_links: Vec<GraphLink> = graph
            .links
            .into_iter()
            .filter(|l| visited.contains(&l.source) && visited.contains(&l.target))
            .collect();

        Ok(Neighborhood {
            center: Some(center_note),
            nodes: sub_nodes,
            links: sub_links,
        })
    }

    /// Hybrid Search: combines dense vector cosine similarity with BM25 FTS5 using Reciprocal Rank Fusion (RRF)
    pub fn search_chunks_hybrid(
        &self,
        query_text: &str,
        query_vector: &[f32],
        limit: usize,
    ) -> Result<Vec<ChunkHit>> {
        let fts_query = sanitize_fts5_query(query_text);
        let has_fts = !fts_query.is_empty();
        let has_vec = !query_vector.is_empty() && query_vector.iter().any(|&v| v.abs() > 1e-6);

        // 1. Vector Candidate Collection
        let mut vec_ranks: HashMap<String, usize> = HashMap::new();
        let mut vec_scores: HashMap<String, f32> = HashMap::new();
        if has_vec {
            let mut q_norm_sq = 0.0f32;
            for &q in query_vector {
                q_norm_sq += q * q;
            }
            let query_is_unit = (q_norm_sq - 1.0).abs() < 1e-3;

            let mut stmt = self.conn.prepare("SELECT c.id, c.vector FROM note_chunks c")?;
            let rows = stmt.query_map([], |row| {
                let id: String = row.get(0)?;
                let blob_ref = row.get_ref(1)?;
                let blob = blob_ref.as_blob()?;
                let score = dot_product_bytes(query_vector, blob, query_is_unit);
                Ok((id, score))
            })?;

            let mut scored: Vec<(String, f32)> = Vec::new();
            for r in rows {
                scored.push(r?);
            }
            scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
            scored.truncate(50);

            for (rank, (id, score)) in scored.into_iter().enumerate() {
                vec_scores.insert(id.clone(), score);
                vec_ranks.insert(id, rank + 1);
            }
        }

        // 2. FTS5 Candidate Collection
        let mut fts_ranks: HashMap<String, usize> = HashMap::new();
        if has_fts {
            let fts_stmt = self.conn.prepare(
                "SELECT chunk_id, rank FROM chunks_fts WHERE chunks_fts MATCH ?1 ORDER BY rank LIMIT 50",
            );
            if let Ok(mut stmt) = fts_stmt {
                if let Ok(rows) = stmt.query_map(params![fts_query], |row| {
                    Ok(row.get::<_, String>(0)?)
                }) {
                    for (rank, r) in rows.flatten().enumerate() {
                        fts_ranks.insert(r, rank + 1);
                    }
                }
            }
        }

        // 3. Reciprocal Rank Fusion (RRF)
        let mut all_chunk_ids: HashSet<String> = HashSet::new();
        all_chunk_ids.extend(vec_ranks.keys().cloned());
        all_chunk_ids.extend(fts_ranks.keys().cloned());

        if all_chunk_ids.is_empty() {
            return Ok(Vec::new());
        }

        let k = 60.0f32;
        let mut rrf_scored: Vec<(String, f32)> = all_chunk_ids
            .into_iter()
            .map(|id| {
                let mut rrf = 0.0f32;
                if let Some(&vr) = vec_ranks.get(&id) {
                    rrf += 1.0 / (k + vr as f32);
                }
                if let Some(&fr) = fts_ranks.get(&id) {
                    rrf += 1.0 / (k + fr as f32);
                }
                (id, rrf)
            })
            .collect();

        rrf_scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        rrf_scored.truncate(limit);

        let max_rrf = (1.0 / 61.0) + (1.0 / 61.0);

        // 4. Fetch full content and metadata for winning chunks
        let mut chunk_stmt = self.conn.prepare(
            r#"
            SELECT c.id, c.note_id, n.title, c.heading, c.content, c.token_estimate
            FROM note_chunks c
            JOIN notes n ON n.id = c.note_id
            WHERE c.id = ?1
            "#,
        )?;

        let mut hits = Vec::with_capacity(rrf_scored.len());
        for (chunk_id, rrf_score) in rrf_scored {
            if let Ok(hit) = chunk_stmt.query_row(params![chunk_id], |row| {
                let id: String = row.get(0)?;
                let note_id: String = row.get(1)?;
                let note_title: String = row.get(2)?;
                let heading: Option<String> = row.get(3)?;
                let content: String = row.get(4)?;
                let token_estimate = row.get::<_, i64>(5)? as usize;

                let deep_link = match &heading {
                    Some(h) if !h.trim().is_empty() => format!("[[{}#{}]]", note_title, h),
                    _ => format!("[[{}]]", note_title),
                };

                let raw_vec_score = vec_scores.get(&id).copied().unwrap_or(0.0);
                let normalized_rrf = (rrf_score / max_rrf).clamp(0.0, 1.0);
                let display_score = if raw_vec_score > 0.0 {
                    (raw_vec_score * 0.6 + normalized_rrf * 0.4).clamp(0.0, 1.0)
                } else {
                    normalized_rrf
                };

                Ok(ChunkHit {
                    chunk_id: id,
                    note_id,
                    note_title,
                    heading,
                    deep_link,
                    content,
                    token_estimate,
                    score: display_score,
                })
            }) {
                hits.push(hit);
            }
        }

        Ok(hits)
    }

    /// Dense Section Search
    pub fn search_chunks(&self, query_vector: &[f32], limit: usize) -> Result<Vec<ChunkHit>> {
        self.search_chunks_hybrid("", query_vector, limit)
    }

    /// Auto-Linker: Suggests notes and sections that are semantically relevant to a draft text
    pub fn suggest_links(
        &self,
        text_vector: &[f32],
        exclude_note_id: &str,
        threshold: f32,
        limit: usize,
    ) -> Result<Vec<LinkSuggestion>> {
        let hits = self.search_chunks(text_vector, limit * 3)?;
        let mut suggestions = Vec::new();
        let mut seen_notes = HashSet::new();

        for hit in hits {
            if hit.note_id == exclude_note_id || hit.score < threshold {
                continue;
            }
            if seen_notes.contains(&hit.note_id) {
                continue;
            }
            seen_notes.insert(hit.note_id.clone());

            let preview = make_snippet(&hit.content, 120);
            suggestions.push(LinkSuggestion {
                target_note_id: hit.note_id,
                target_title: hit.note_title,
                heading: hit.heading,
                deep_link: hit.deep_link,
                similarity_score: hit.score,
                preview,
            });

            if suggestions.len() >= limit {
                break;
            }
        }

        Ok(suggestions)
    }

    /// Token-efficient compressed digest for AI agents (consumes ~50-100 tokens max)
    pub fn get_compressed_context(
        &self,
        note_id_or_title: &str,
    ) -> Result<Option<CompressedContext>> {
        let note = match self.get_note_by_id(note_id_or_title)? {
            Some(n) => n,
            None => match self.get_note_by_title(note_id_or_title)? {
                Some(n) => n,
                None => return Ok(None),
            },
        };

        let mut stmt_chunks = self.conn.prepare(
            "SELECT heading, content, token_estimate FROM note_chunks WHERE note_id = ?1 ORDER BY chunk_index ASC",
        )?;
        let rows = stmt_chunks.query_map(params![note.id], |row| {
            let heading: Option<String> = row.get(0)?;
            let content: String = row.get(1)?;
            let token_estimate: usize = row.get::<_, i64>(2)? as usize;
            Ok((heading, content, token_estimate))
        })?;

        let mut sections = Vec::new();
        let mut total_tokens = 0;
        for r in rows {
            let (heading, content, token_estimate) = r?;
            total_tokens += token_estimate;
            let snippet = make_snippet(&content, 120);
            sections.push(SectionSummary {
                heading: heading.unwrap_or_else(|| "Geral".to_string()),
                token_estimate,
                snippet,
            });
        }

        let mut connected = note.outgoing_links.clone();
        connected.extend(note.backlinks.clone());
        connected.sort();
        connected.dedup();

        Ok(Some(CompressedContext {
            note_id: note.id,
            title: note.title,
            summary: note.summary,
            tags: note.tags,
            token_estimate: total_tokens,
            connected_notes: connected,
            sections,
        }))
    }

    /// Retrieves an exact atomic section chunk from a note by heading or deep link fragment
    pub fn get_section(
        &self,
        note_id_or_title: &str,
        section_heading: &str,
    ) -> Result<Option<ChunkHit>> {
        let note = match self.get_note_by_id(note_id_or_title)? {
            Some(n) => n,
            None => match self.get_note_by_title(note_id_or_title)? {
                Some(n) => n,
                None => return Ok(None),
            },
        };

        let mut stmt = self.conn.prepare(
            "SELECT id, heading, content, token_estimate FROM note_chunks WHERE note_id = ?1 ORDER BY chunk_index ASC",
        )?;
        let rows = stmt.query_map(params![note.id], |row| {
            let id: String = row.get(0)?;
            let heading: Option<String> = row.get(1)?;
            let content: String = row.get(2)?;
            let token_estimate: usize = row.get::<_, i64>(3)? as usize;
            Ok((id, heading, content, token_estimate))
        })?;

        let clean_section = section_heading.trim().trim_start_matches('#').trim();
        let target_heading_lower = clean_section.to_lowercase();

        for r in rows {
            let (chunk_id, heading_opt, content, token_estimate) = r?;
            let matches = if let Some(ref h) = heading_opt {
                let h_lower = h.to_lowercase();
                h_lower == target_heading_lower
                    || h_lower.contains(&target_heading_lower)
                    || target_heading_lower.contains(&h_lower)
            } else {
                clean_section.eq_ignore_ascii_case("geral") || clean_section.is_empty()
            };

            if matches {
                let deep_link = if let Some(ref h) = heading_opt {
                    format!("[[{}#{}]]", note.title, h)
                } else {
                    format!("[[{}]]", note.title)
                };
                return Ok(Some(ChunkHit {
                    chunk_id,
                    note_id: note.id,
                    note_title: note.title,
                    heading: heading_opt,
                    deep_link,
                    content,
                    token_estimate,
                    score: 1.0,
                }));
            }
        }
        Ok(None)
    }


    pub fn semantic_search(&self, query_vector: &[f32], limit: usize) -> Result<Vec<SearchHit>> {
        let mut q_norm_sq = 0.0f32;
        for &q in query_vector {
            q_norm_sq += q * q;
        }
        let query_is_unit = (q_norm_sq - 1.0).abs() < 1e-3;

        let mut stmt = self
            .conn
            .prepare("SELECT note_id, vector FROM embeddings")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let blob_ref = row.get_ref(1)?;
            let blob = blob_ref.as_blob()?;
            let sim = dot_product_bytes(query_vector, blob, query_is_unit);
            Ok((id, sim))
        })?;

        let mut scored: Vec<(String, f32)> = Vec::new();
        for r in rows {
            scored.push(r?);
        }

        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        scored.truncate(limit);

        let mut hits = Vec::new();
        for (id, score) in scored {
            if let Some(note) = self.get_note_by_id(&id)? {
                let snippet = if !note.summary.is_empty() {
                    note.summary.clone()
                } else {
                    make_snippet(&note.content, 160)
                };
                hits.push(SearchHit {
                    id: note.id,
                    title: note.title,
                    snippet,
                    score,
                    tags: note.tags,
                });
            }
        }

        Ok(hits)
    }

    fn get_tags_for_note(&self, note_id: &str) -> Result<Vec<String>> {
        let mut stmt = self
            .conn
            .prepare("SELECT tag FROM tags WHERE note_id = ?1 ORDER BY tag ASC")?;
        let rows = stmt.query_map(params![note_id], |row| row.get(0))?;
        let mut tags = Vec::new();
        for t in rows {
            tags.push(t?);
        }
        Ok(tags)
    }

    fn get_outgoing_links_for_note(&self, note_id: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            "SELECT target_title FROM links WHERE source_id = ?1 ORDER BY target_title ASC",
        )?;
        let rows = stmt.query_map(params![note_id], |row| row.get(0))?;
        let mut links = Vec::new();
        for l in rows {
            links.push(l?);
        }
        Ok(links)
    }

    fn get_backlinks_for_title(&self, title: &str) -> Result<Vec<String>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT DISTINCT n.title 
            FROM links l
            JOIN notes n ON n.id = l.source_id
            WHERE LOWER(l.target_title) = LOWER(?1) OR LOWER(l.target_title) LIKE LOWER(?1 || '#%')
            ORDER BY n.title ASC
            "#,
        )?;
        let rows = stmt.query_map(params![title], |row| row.get(0))?;
        let mut backlinks = Vec::new();
        for b in rows {
            backlinks.push(b?);
        }
        Ok(backlinks)
    }
}

pub fn sanitize_fts5_query(raw: &str) -> String {
    let tokens: Vec<String> = raw
        .split(|c: char| !c.is_alphanumeric() && c != '_')
        .filter(|t| !t.trim().is_empty())
        .map(|t| format!("\"{}\"*", t.trim()))
        .collect();
    if tokens.is_empty() {
        return String::new();
    }
    tokens.join(" ")
}

fn f32_slice_to_bytes(slice: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(slice.len() * 4);
    for &f in slice {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

#[inline]
pub fn dot_product_bytes(query: &[f32], bytes: &[u8], query_is_unit: bool) -> f32 {
    let chunks = bytes.chunks_exact(4);
    if chunks.len() != query.len() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_b = 0.0f32;

    for (i, chunk) in chunks.enumerate() {
        let val = f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
        dot += query[i] * val;
        norm_b += val * val;
    }

    if query_is_unit && (norm_b - 1.0).abs() < 1e-3 {
        return dot;
    }

    let denom = norm_b.sqrt();
    if denom <= f32::EPSILON {
        0.0
    } else {
        dot / denom
    }
}

fn make_snippet(content: &str, max_len: usize) -> String {
    let plain = content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .collect::<Vec<_>>()
        .join(" ");
    if plain.chars().count() <= max_len {
        plain
    } else {
        let mut s: String = plain.chars().take(max_len).collect();
        s.push_str("...");
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_db_memory_crud_and_fts() {
        let mut db = DbManager::memory().expect("in-memory db");

        let chunk1 = ParsedChunk {
            chunk_index: 0,
            heading: Some("Arquitetura SQLite".to_string()),
            content: "O SQLite em modo WAL garante concorrência sem bloqueio.".to_string(),
            token_estimate: 15,
            links: vec!["Armazenamento".to_string()],
            tags: vec!["sqlite".to_string()],
        };

        let vec1 = vec![1.0f32, 0.0, 0.0];
        let chunks = vec![(chunk1, vec1.clone())];

        db.upsert_note(
            "arq",
            "arq.md",
            "Arquitetura",
            "Resumo de arquitetura",
            "Conteúdo completo sobre SQLite WAL.",
            "hash123",
            1000,
            &["Armazenamento".to_string()],
            &["sqlite".to_string()],
            Some(&vec1),
            &chunks,
        )
        .expect("upsert ok");

        let note = db.get_note_by_id("arq").expect("get note").expect("found");
        assert_eq!(note.title, "Arquitetura");
        assert_eq!(note.content_hash, "hash123");
        assert_eq!(note.tags, vec!["sqlite"]);

        // Test Hybrid FTS Search
        let hits = db
            .search_chunks_hybrid("SQLite WAL", &vec1, 5)
            .expect("search hybrid");
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].heading.as_deref(), Some("Arquitetura SQLite"));

        // Test Stats
        let stats = db.get_vault_stats().expect("stats");
        assert_eq!(stats.total_notes, 1);
        assert_eq!(stats.total_chunks, 1);

        // Test Rename
        db.rename_note("arq", "arq-nova", "arq-nova.md", "Arquitetura Nova")
            .expect("rename ok");
        assert!(db.get_note_by_id("arq").expect("query").is_none());
        assert!(db.get_note_by_id("arq-nova").expect("query").is_some());

        // Test Delete
        db.delete_note("arq-nova").expect("delete ok");
        assert!(db.get_note_by_id("arq-nova").expect("query").is_none());
    }

    #[test]
    fn test_sanitize_fts5_query() {
        assert_eq!(
            sanitize_fts5_query("sqlite, wal! test-term"),
            "\"sqlite\"* \"wal\"* \"test\"* \"term\"*"
        );
        assert_eq!(sanitize_fts5_query("   "), "");
    }
}
