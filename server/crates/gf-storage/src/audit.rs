use sqlx::PgPool;
use uuid::Uuid;

use crate::{Principal, StorageResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct AuditEvent {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub project_id: Option<Uuid>,
    pub actor_user_id: Option<Uuid>,
    pub api_token_id: Option<Uuid>,
    pub action: String,
    pub resource_type: String,
    pub resource_id: Option<Uuid>,
    pub summary: serde_json::Value,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn record_audit_event(
    pool: &PgPool,
    principal: &Principal,
    project_id: Option<Uuid>,
    action: &str,
    resource_type: &str,
    resource_id: Option<Uuid>,
    summary: serde_json::Value,
) -> StorageResult<()> {
    sqlx::query(
        r#"
        INSERT INTO audit_events (tenant_id,project_id,actor_user_id,api_token_id,action,resource_type,resource_id,summary)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        "#,
    )
    .bind(principal.tenant_id)
    .bind(project_id)
    .bind(principal.user_id)
    .bind(principal.token_id)
    .bind(action)
    .bind(resource_type)
    .bind(resource_id)
    .bind(summary)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_audit_events(
    pool: &PgPool,
    tenant_id: Uuid,
    limit: i64,
) -> StorageResult<Vec<AuditEvent>> {
    Ok(sqlx::query_as::<_, AuditEvent>(
        "SELECT * FROM audit_events WHERE tenant_id=$1 ORDER BY created_at DESC LIMIT $2",
    )
    .bind(tenant_id)
    .bind(limit.clamp(1, 200))
    .fetch_all(pool)
    .await?)
}
