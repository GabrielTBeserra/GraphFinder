use sqlx::PgPool;
use uuid::Uuid;

use crate::{StorageError, StorageResult};

#[derive(Debug, Clone)]
pub struct VaultChunkWrite {
    pub chunk_index: i32,
    pub heading_path: Vec<String>,
    pub content: String,
    pub token_count: i32,
    pub content_hash: String,
    pub metadata: serde_json::Value,
    pub embedding: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct VaultDocumentWrite {
    pub project_id: Uuid,
    pub generation: i64,
    pub source_id: Uuid,
    pub path: String,
    pub title: String,
    pub content_hash: String,
    pub size_bytes: i64,
    pub metadata: serde_json::Value,
    pub chunks: Vec<VaultChunkWrite>,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VaultDocumentRecord {
    pub id: Uuid,
    pub project_id: Uuid,
    pub generation: i64,
    pub source_id: Option<Uuid>,
    pub path: String,
    pub title: Option<String>,
    pub revision: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn get_vault_document(
    pool: &PgPool,
    project_id: Uuid,
    path: &str,
) -> StorageResult<Option<VaultDocumentRecord>> {
    Ok(sqlx::query_as::<_, VaultDocumentRecord>(
        r#"
        SELECT d.id,d.project_id,d.generation,d.source_id,d.path,d.title,d.revision,d.updated_at
        FROM documents d
        JOIN projects p ON p.id=d.project_id
        JOIN sources s ON s.id=d.source_id AND s.project_id=d.project_id
        WHERE d.project_id=$1 AND d.generation=p.active_generation AND d.path=$2 AND s.kind='managed_vault'
        "#,
    )
    .bind(project_id)
    .bind(path)
    .fetch_optional(pool)
    .await?)
}

pub async fn upsert_vault_document(
    pool: &PgPool,
    write: VaultDocumentWrite,
    expected_revision: Option<i64>,
) -> StorageResult<VaultDocumentRecord> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        r#"
        INSERT INTO index_generations (project_id, generation, status, published_at)
        VALUES ($1,$2,'published',now())
        ON CONFLICT (project_id, generation) DO NOTHING
        "#,
    )
    .bind(write.project_id)
    .bind(write.generation)
    .execute(&mut *tx)
    .await?;

    let source_kind = sqlx::query_scalar::<_, String>(
        "SELECT kind FROM sources WHERE id=$1 AND project_id=$2 FOR UPDATE",
    )
    .bind(write.source_id)
    .bind(write.project_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| StorageError::Conflict("managed vault source not found".into()))?;
    if source_kind != "managed_vault" {
        return Err(StorageError::Conflict(
            "source is readonly or is not a managed vault".into(),
        ));
    }

    let existing = sqlx::query_as::<_, VaultDocumentRecord>(
        r#"
        SELECT id,project_id,generation,source_id,path,title,revision,updated_at
        FROM documents
        WHERE project_id=$1 AND generation=$2 AND path=$3
        FOR UPDATE
        "#,
    )
    .bind(write.project_id)
    .bind(write.generation)
    .bind(&write.path)
    .fetch_optional(&mut *tx)
    .await?;

    let document_id = if let Some(existing) = existing {
        if let Some(expected) = expected_revision {
            if expected != existing.revision {
                return Err(StorageError::Conflict(format!(
                    "document revision conflict: expected {expected}, current {}",
                    existing.revision
                )));
            }
        }
        sqlx::query(
            r#"
            UPDATE documents
            SET source_id=$4,title=$5,content_hash=$6,size_bytes=$7,metadata=$8,
                revision=revision+1,updated_at=now()
            WHERE id=$1 AND project_id=$2 AND generation=$3
            "#,
        )
        .bind(existing.id)
        .bind(write.project_id)
        .bind(write.generation)
        .bind(write.source_id)
        .bind(&write.title)
        .bind(&write.content_hash)
        .bind(write.size_bytes)
        .bind(&write.metadata)
        .execute(&mut *tx)
        .await?;
        sqlx::query("DELETE FROM chunks WHERE document_id=$1")
            .bind(existing.id)
            .execute(&mut *tx)
            .await?;
        existing.id
    } else {
        if expected_revision.is_some() {
            return Err(StorageError::Conflict(
                "document does not exist for revisioned update".into(),
            ));
        }
        sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO documents (
              project_id,generation,source_id,path,document_type,language,title,
              content_hash,size_bytes,metadata
            ) VALUES ($1,$2,$3,$4,'markdown','markdown',$5,$6,$7,$8)
            RETURNING id
            "#,
        )
        .bind(write.project_id)
        .bind(write.generation)
        .bind(write.source_id)
        .bind(&write.path)
        .bind(&write.title)
        .bind(&write.content_hash)
        .bind(write.size_bytes)
        .bind(&write.metadata)
        .fetch_one(&mut *tx)
        .await?
    };

    for chunk in &write.chunks {
        let chunk_id = sqlx::query_scalar::<_, Uuid>(
            r#"
            INSERT INTO chunks (
              document_id,project_id,generation,chunk_index,heading_path,content,
              token_count,content_hash,metadata
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            RETURNING id
            "#,
        )
        .bind(document_id)
        .bind(write.project_id)
        .bind(write.generation)
        .bind(chunk.chunk_index)
        .bind(&chunk.heading_path)
        .bind(&chunk.content)
        .bind(chunk.token_count)
        .bind(&chunk.content_hash)
        .bind(&chunk.metadata)
        .fetch_one(&mut *tx)
        .await?;
        let vector = format!(
            "[{}]",
            chunk
                .embedding
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>()
                .join(",")
        );
        sqlx::query(
            r#"
            INSERT INTO chunk_embeddings (chunk_id,project_id,generation,model,embedding)
            VALUES ($1,$2,$3,'multilingual-e5-small',$4::vector)
            "#,
        )
        .bind(chunk_id)
        .bind(write.project_id)
        .bind(write.generation)
        .bind(vector)
        .execute(&mut *tx)
        .await?;
    }

    let record = sqlx::query_as::<_, VaultDocumentRecord>(
        r#"
        SELECT id,project_id,generation,source_id,path,title,revision,updated_at
        FROM documents WHERE id=$1
        "#,
    )
    .bind(document_id)
    .fetch_one(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(record)
}
