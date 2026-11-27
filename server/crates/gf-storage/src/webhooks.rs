use sqlx::PgPool;
use uuid::Uuid;

use crate::StorageResult;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SourceWebhook {
    pub id: Uuid,
    pub source_id: Uuid,
    pub project_id: Uuid,
    pub provider: String,
    pub secret: String,
    pub enabled: bool,
    pub last_delivery_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_status: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn upsert_source_webhook(
    pool: &PgPool,
    source_id: Uuid,
    project_id: Uuid,
    provider: &str,
    secret: &str,
    enabled: bool,
) -> StorageResult<SourceWebhook> {
    Ok(sqlx::query_as::<_, SourceWebhook>(
        r#"
        INSERT INTO source_webhooks (source_id,project_id,provider,secret,enabled)
        VALUES ($1,$2,$3,$4,$5)
        ON CONFLICT (source_id) DO UPDATE
        SET project_id=excluded.project_id,
            provider=excluded.provider,
            secret=excluded.secret,
            enabled=excluded.enabled,
            updated_at=now()
        RETURNING *
        "#,
    )
    .bind(source_id)
    .bind(project_id)
    .bind(provider)
    .bind(secret)
    .bind(enabled)
    .fetch_one(pool)
    .await?)
}

pub async fn get_source_webhook(
    pool: &PgPool,
    source_id: Uuid,
) -> StorageResult<Option<SourceWebhook>> {
    Ok(
        sqlx::query_as::<_, SourceWebhook>("SELECT * FROM source_webhooks WHERE source_id=$1")
            .bind(source_id)
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn delete_source_webhook(pool: &PgPool, source_id: Uuid) -> StorageResult<bool> {
    let rows = sqlx::query("DELETE FROM source_webhooks WHERE source_id=$1")
        .bind(source_id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(rows > 0)
}

pub async fn record_source_webhook_delivery(
    pool: &PgPool,
    source_id: Uuid,
    status: &str,
) -> StorageResult<()> {
    sqlx::query(
        "UPDATE source_webhooks SET last_delivery_at=now(),last_status=$2,updated_at=now() WHERE source_id=$1",
    )
    .bind(source_id)
    .bind(status)
    .execute(pool)
    .await?;
    Ok(())
}
