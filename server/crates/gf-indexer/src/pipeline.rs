use std::path::Path;

use gf_core::{parse_code, parse_markdown, EmbeddingProvider};
use gf_storage::models::{NewChunk, NewDocument};
use gf_storage::{
    create_chunk, create_document, create_next_index_generation, fail_index_generation,
    publish_index_generation, upsert_chunk_embedding,
};
use serde_json::json;
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{scan_repository, ScanError};

#[derive(Debug, Clone, Default)]
pub struct IndexingReport {
    pub generation: i64,
    pub files_scanned: usize,
    pub documents_indexed: usize,
    pub chunks_indexed: usize,
    pub embeddings_written: usize,
    pub skipped_files: usize,
    pub published: bool,
}

#[derive(Debug, Error)]
pub enum IndexingError {
    #[error(transparent)]
    Scan(#[from] ScanError),
    #[error(transparent)]
    Storage(#[from] gf_storage::StorageError),
    #[error("embedding error: {0}")]
    Embedding(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub type IndexingResult<T> = Result<T, IndexingError>;

pub async fn index_checkout(
    pool: &PgPool,
    checkout_path: &Path,
    project_id: Uuid,
    source_id: Option<Uuid>,
    include_patterns: &[String],
    exclude_patterns: &[String],
    embedder: &dyn EmbeddingProvider,
) -> IndexingResult<IndexingReport> {
    let generation = create_next_index_generation(pool, project_id).await?;
    let mut report = IndexingReport {
        generation: generation.generation,
        ..IndexingReport::default()
    };

    let result = index_generation_contents(
        pool,
        checkout_path,
        project_id,
        generation.generation,
        source_id,
        include_patterns,
        exclude_patterns,
        embedder,
        &mut report,
    )
    .await;

    match result {
        Ok(()) => {
            report.published =
                publish_index_generation(pool, project_id, generation.generation).await?;
            Ok(report)
        }
        Err(error) => {
            let _ =
                fail_index_generation(pool, project_id, generation.generation, &error.to_string())
                    .await;
            Err(error)
        }
    }
}

pub fn dry_run_checkout(
    checkout_path: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> IndexingResult<IndexingReport> {
    let files = scan_repository(checkout_path, include_patterns, exclude_patterns)?;
    Ok(IndexingReport {
        files_scanned: files.len(),
        ..IndexingReport::default()
    })
}

async fn index_generation_contents(
    pool: &PgPool,
    checkout_path: &Path,
    project_id: Uuid,
    generation: i64,
    source_id: Option<Uuid>,
    include_patterns: &[String],
    exclude_patterns: &[String],
    embedder: &dyn EmbeddingProvider,
    report: &mut IndexingReport,
) -> IndexingResult<()> {
    let files = scan_repository(checkout_path, include_patterns, exclude_patterns)?;
    report.files_scanned = files.len();
    let mut pending_embeddings = Vec::new();

    for file in files {
        if file.size_bytes > 1024 * 1024 {
            report.skipped_files += 1;
            continue;
        }
        let absolute_path = checkout_path.join(&file.relative_path);
        let Ok(content) = std::fs::read_to_string(&absolute_path) else {
            report.skipped_files += 1;
            continue;
        };
        let is_markdown = file.relative_path.ends_with(".md");
        let parsed_code = parse_code(&file.relative_path, &content);
        if !is_markdown && parsed_code.is_none() {
            report.skipped_files += 1;
            continue;
        }

        let (document_type, language, title, metadata) = if is_markdown {
            let note = parse_markdown(
                Path::new(&file.relative_path)
                    .file_stem()
                    .and_then(|v| v.to_str())
                    .unwrap_or("note"),
                &content,
            );
            (
                "markdown".to_string(),
                None,
                Some(note.title),
                json!({"links": note.links, "tags": note.tags, "aliases": note.aliases}),
            )
        } else {
            let parsed = parsed_code.clone().expect("checked above");
            (
                "code".to_string(),
                Some(parsed.language),
                None,
                json!({"imports": parsed.imports, "symbols": parsed.symbols.iter().map(|s| json!({"name": s.name, "kind": s.kind, "start_line": s.start_line, "end_line": s.end_line})).collect::<Vec<_>>()}),
            )
        };

        let document = create_document(
            pool,
            NewDocument {
                project_id,
                generation,
                source_id,
                path: file.relative_path.clone(),
                document_type,
                language,
                title,
                content_hash: file.content_hash,
                size_bytes: file.size_bytes as i64,
                metadata,
            },
        )
        .await?;
        report.documents_indexed += 1;

        if is_markdown {
            let note = parse_markdown(
                Path::new(&file.relative_path)
                    .file_stem()
                    .and_then(|v| v.to_str())
                    .unwrap_or("note"),
                &content,
            );
            for chunk in note.chunks {
                let row = create_chunk(
                    pool,
                    NewChunk {
                        document_id: document.id,
                        project_id,
                        generation,
                        chunk_index: chunk.chunk_index as i32,
                        heading_path: chunk.heading.iter().cloned().collect(),
                        content: chunk.content.clone(),
                        token_count: chunk.token_estimate as i32,
                        content_hash: gf_core::content_hash(&chunk.content),
                        start_line: None,
                        end_line: None,
                        metadata: json!({"links": chunk.links, "tags": chunk.tags}),
                    },
                )
                .await?;
                report.chunks_indexed += 1;
                pending_embeddings.push((row.id, chunk.content));
            }
        } else if let Some(parsed) = parsed_code {
            let lines = content.lines().collect::<Vec<_>>();
            for (index, symbol) in parsed.symbols.iter().enumerate() {
                let start = symbol.start_line.max(1);
                let end = symbol.end_line.min(lines.len()).max(start);
                let chunk_content = lines[start - 1..end].join("\n");
                let row = create_chunk(
                    pool,
                    NewChunk {
                        document_id: document.id,
                        project_id,
                        generation,
                        chunk_index: index as i32,
                        heading_path: vec![symbol.name.clone()],
                        content: chunk_content.clone(),
                        token_count: gf_core::estimate_tokens(&chunk_content) as i32,
                        content_hash: gf_core::content_hash(&chunk_content),
                        start_line: Some(start as i32),
                        end_line: Some(end as i32),
                        metadata: json!({"symbol": symbol.name, "kind": symbol.kind}),
                    },
                )
                .await?;
                report.chunks_indexed += 1;
                pending_embeddings.push((row.id, chunk_content));
            }
        }
    }

    for batch in pending_embeddings.chunks(32) {
        let texts = batch
            .iter()
            .map(|(_, text)| text.clone())
            .collect::<Vec<_>>();
        let embeddings = embedder
            .embed_passages(&texts)
            .map_err(|e| IndexingError::Embedding(e.to_string()))?;
        for ((chunk_id, _), embedding) in batch.iter().zip(embeddings.iter()) {
            upsert_chunk_embedding(
                pool,
                *chunk_id,
                project_id,
                generation,
                embedder.model_id(),
                embedding,
            )
            .await?;
            report.embeddings_written += 1;
        }
    }
    Ok(())
}
