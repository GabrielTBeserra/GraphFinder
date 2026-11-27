use sqlx::PgPool;
use uuid::Uuid;

use crate::{StorageError, StorageResult};

pub const E5_SMALL_DIMENSIONS: usize = 384;

pub async fn upsert_chunk_embedding(
    pool: &PgPool,
    chunk_id: Uuid,
    project_id: Uuid,
    generation: i64,
    model: &str,
    embedding: &[f32],
) -> StorageResult<()> {
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

    sqlx::query(
        r#"
        INSERT INTO chunk_embeddings (chunk_id, project_id, generation, model, embedding)
        VALUES ($1, $2, $3, $4, $5::vector)
        ON CONFLICT (chunk_id) DO UPDATE
        SET model = EXCLUDED.model,
            embedding = EXCLUDED.embedding
        "#,
    )
    .bind(chunk_id)
    .bind(project_id)
    .bind(generation)
    .bind(model)
    .bind(vector)
    .execute(pool)
    .await?;

    Ok(())
}
