use super::embeddings::cosine_similarity;
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

pub struct DbManager {
    conn: Connection,
}

impl DbManager {
    pub fn new(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        let manager = Self { conn };
        manager.init_schema()?;
        Ok(manager)
    }

    #[cfg(test)]
    pub fn memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let manager = Self { conn };
        manager.init_schema()?;
        Ok(manager)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS notes (
                id TEXT PRIMARY KEY,
                path TEXT UNIQUE NOT NULL,
                title TEXT NOT NULL,
                summary TEXT NOT NULL DEFAULT '',
                content TEXT NOT NULL,
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
                content
            );

            CREATE INDEX IF NOT EXISTS idx_notes_title ON notes(title);
            CREATE INDEX IF NOT EXISTS idx_links_target ON links(target_title);
            CREATE INDEX IF NOT EXISTS idx_tags_tag ON tags(tag);
            CREATE INDEX IF NOT EXISTS idx_chunks_note ON note_chunks(note_id);
            "#,
        )?;

        // Safe migration for summary column
        let _ = self.conn.execute(
            "ALTER TABLE notes ADD COLUMN summary TEXT NOT NULL DEFAULT ''",
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
        updated_at: i64,
        links: &[String],
        tags: &[String],
        note_embedding: Option<&[f32]>,
        chunks: &[(ParsedChunk, Vec<f32>)],
    ) -> Result<()> {
        let tx = self.conn.transaction()?;

        // 1. Upsert note with summary
        tx.execute(
            r#"
            INSERT INTO notes (id, path, title, summary, content, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6)
            ON CONFLICT(id) DO UPDATE SET
                path = excluded.path,
                title = excluded.title,
                summary = excluded.summary,
                content = excluded.content,
                updated_at = excluded.updated_at
            "#,
            params![id, path, title, summary, content, updated_at],
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
        }

        // 6. Update FTS
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
        tx.execute("DELETE FROM links WHERE source_id = ?1", params![id])?;
        tx.execute("DELETE FROM tags WHERE note_id = ?1", params![id])?;
        tx.execute("DELETE FROM embeddings WHERE note_id = ?1", params![id])?;
        tx.execute("DELETE FROM notes_fts WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(())
    }

    pub fn get_note_by_id(&self, id: &str) -> Result<Option<NoteRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, path, title, summary, content, updated_at FROM notes WHERE id = ?1",
        )?;
        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let id: String = row.get(0)?;
            let path: String = row.get(1)?;
            let title: String = row.get(2)?;
            let summary: String = row.get(3)?;
            let content: String = row.get(4)?;
            let updated_at: i64 = row.get(5)?;

            let tags = self.get_tags_for_note(&id)?;
            let outgoing_links = self.get_outgoing_links_for_note(&id)?;
            let backlinks = self.get_backlinks_for_title(&title)?;

            Ok(Some(NoteRecord {
                id,
                path,
                title,
                summary,
                content,
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
            let tags = self.get_tags_for_note(&id)?;
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

        let mut nodes = Vec::new();
        for (id, title) in &id_to_title {
            let tags = self.get_tags_for_note(id)?;
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
                    tags: vec!["inexistente".to_string()],
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

    /// Dense Section Search: returns exact chunks/sections matching the query with token counts
    pub fn search_chunks(&self, query_vector: &[f32], limit: usize) -> Result<Vec<ChunkHit>> {
        let mut stmt = self.conn.prepare(
            r#"
            SELECT c.id, c.note_id, n.title, c.heading, c.content, c.token_estimate, c.vector
            FROM note_chunks c
            JOIN notes n ON n.id = c.note_id
            "#,
        )?;

        let rows = stmt.query_map([], |row| {
            let chunk_id: String = row.get(0)?;
            let note_id: String = row.get(1)?;
            let note_title: String = row.get(2)?;
            let heading: Option<String> = row.get(3)?;
            let content: String = row.get(4)?;
            let token_estimate: usize = row.get::<_, i64>(5)? as usize;
            let bytes: Vec<u8> = row.get(6)?;
            Ok((
                chunk_id,
                note_id,
                note_title,
                heading,
                content,
                token_estimate,
                bytes,
            ))
        })?;

        let mut scored: Vec<ChunkHit> = Vec::new();
        for r in rows {
            let (chunk_id, note_id, note_title, heading, content, token_estimate, bytes) = r?;
            let chunk_vec = bytes_to_f32_vec(&bytes);
            let score = cosine_similarity(query_vector, &chunk_vec);

            let deep_link = match &heading {
                Some(h) if !h.trim().is_empty() => format!("[[{}#{}]]", note_title, h),
                _ => format!("[[{}]]", note_title),
            };

            scored.push(ChunkHit {
                chunk_id,
                note_id,
                note_title,
                heading,
                deep_link,
                content,
                token_estimate,
                score,
            });
        }

        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored.truncate(limit);
        Ok(scored)
    }

    /// Auto-Linker: Suggests notes and sections that are semantically relevant to a given draft text
    pub fn suggest_links(
        &self,
        text_vector: &[f32],
        exclude_note_id: &str,
        threshold: f32,
        limit: usize,
    ) -> Result<Vec<LinkSuggestion>> {
        let hits = self.search_chunks(text_vector, limit * 2)?;
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

    pub fn semantic_search(&self, query_vector: &[f32], limit: usize) -> Result<Vec<SearchHit>> {
        let mut stmt = self
            .conn
            .prepare("SELECT note_id, vector FROM embeddings")?;
        let rows = stmt.query_map([], |row| {
            let id: String = row.get(0)?;
            let bytes: Vec<u8> = row.get(1)?;
            Ok((id, bytes))
        })?;

        let mut scored: Vec<(String, f32)> = Vec::new();
        for r in rows {
            let (id, bytes) = r?;
            let note_vec = bytes_to_f32_vec(&bytes);
            let sim = cosine_similarity(query_vector, &note_vec);
            scored.push((id, sim));
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

fn f32_slice_to_bytes(slice: &[f32]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(slice.len() * 4);
    for &f in slice {
        bytes.extend_from_slice(&f.to_le_bytes());
    }
    bytes
}

fn bytes_to_f32_vec(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
        .collect()
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
