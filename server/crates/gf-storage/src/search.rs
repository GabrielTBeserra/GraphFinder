use std::collections::HashMap;

use sqlx::FromRow;
use sqlx::PgPool;
use uuid::Uuid;

use crate::embeddings::E5_SMALL_DIMENSIONS;
use crate::{StorageError, StorageResult};

#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    pub language: Option<String>,
    pub document_type: Option<String>,
}

#[derive(Debug, Clone, FromRow)]
pub struct SearchHit {
    pub chunk_id: Uuid,
    pub document_id: Uuid,
    pub path: String,
    pub heading_path: Vec<String>,
    pub content: String,
    pub score: f32,
}

pub async fn lexical_search(
    pool: &PgPool,
    project_id: Uuid,
    query: &str,
    filters: &SearchFilters,
    limit: i64,
) -> StorageResult<Vec<SearchHit>> {
    Ok(sqlx::query_as::<_, SearchHit>(
        r#"
        SELECT
            c.id AS chunk_id,
            c.document_id,
            d.path,
            c.heading_path,
            c.content,
            ts_rank(c.search_vector, plainto_tsquery('simple', $2))::real AS score
        FROM chunks c
        JOIN documents d ON d.id = c.document_id
        JOIN projects p ON p.id = c.project_id AND p.active_generation = c.generation
        WHERE c.project_id = $1
          AND c.search_vector @@ plainto_tsquery('simple', $2)
          AND ($3::text IS NULL OR d.language = $3)
          AND ($4::text IS NULL OR d.document_type = $4)
        ORDER BY score DESC
        LIMIT $5
        "#,
    )
    .bind(project_id)
    .bind(query)
    .bind(filters.language.as_deref())
    .bind(filters.document_type.as_deref())
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn vector_search(
    pool: &PgPool,
    project_id: Uuid,
    embedding: &[f32],
    filters: &SearchFilters,
    limit: i64,
) -> StorageResult<Vec<SearchHit>> {
    if embedding.len() != E5_SMALL_DIMENSIONS {
        return Err(StorageError::InvalidEmbeddingDimensions {
            expected: E5_SMALL_DIMENSIONS,
            actual: embedding.len(),
        });
    }

    let vector = format!(
        "[{}]",
        embedding
            .iter()
            .map(|value| value.to_string())
            .collect::<Vec<_>>()
            .join(",")
    );

    Ok(sqlx::query_as::<_, SearchHit>(
        r#"
        SELECT
            c.id AS chunk_id,
            c.document_id,
            d.path,
            c.heading_path,
            c.content,
            (1 - (ce.embedding <=> $2::vector))::real AS score
        FROM chunk_embeddings ce
        JOIN chunks c ON c.id = ce.chunk_id
        JOIN documents d ON d.id = c.document_id
        JOIN projects p ON p.id = c.project_id AND p.active_generation = c.generation
        WHERE ce.project_id = $1
          AND ($3::text IS NULL OR d.language = $3)
          AND ($4::text IS NULL OR d.document_type = $4)
        ORDER BY ce.embedding <=> $2::vector
        LIMIT $5
        "#,
    )
    .bind(project_id)
    .bind(vector)
    .bind(filters.language.as_deref())
    .bind(filters.document_type.as_deref())
    .bind(limit)
    .fetch_all(pool)
    .await?)
}

pub async fn hybrid_search(
    pool: &PgPool,
    project_id: Uuid,
    query: &str,
    embedding: &[f32],
    filters: &SearchFilters,
    limit: i64,
) -> StorageResult<Vec<SearchHit>> {
    const RRF_K: f32 = 60.0;

    let lexical = lexical_search(pool, project_id, query, filters, limit).await?;
    let vector = vector_search(pool, project_id, embedding, filters, limit).await?;
    let mut combined: HashMap<Uuid, (SearchHit, f32)> = HashMap::new();

    for (rank, hit) in lexical.into_iter().enumerate() {
        let contribution = 1.0 / (RRF_K + rank as f32 + 1.0);
        let key = hit.chunk_id;
        combined
            .entry(key)
            .and_modify(|(_, score)| *score += contribution)
            .or_insert((hit, contribution));
    }

    for (rank, hit) in vector.into_iter().enumerate() {
        let contribution = 1.0 / (RRF_K + rank as f32 + 1.0);
        let key = hit.chunk_id;
        combined
            .entry(key)
            .and_modify(|(_, score)| *score += contribution)
            .or_insert((hit, contribution));
    }

    let mut hits = combined
        .into_values()
        .map(|(mut hit, score)| {
            hit.score = score;
            hit
        })
        .collect::<Vec<_>>();
    hits.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    hits.truncate(limit.max(0) as usize);
    Ok(hits)
}
