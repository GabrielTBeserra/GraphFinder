use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use crate::StorageResult;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct IndexJob {
    pub id: Uuid,
    pub project_id: Uuid,
    pub source_id: Option<Uuid>,
    pub generation: Option<i64>,
    pub status: String,
    pub priority: i32,
    pub attempts: i32,
    pub max_attempts: i32,
    pub payload: Value,
    pub error: Option<String>,
    pub run_at: DateTime<Utc>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub created_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct NewIndexJob {
    pub project_id: Uuid,
    pub source_id: Option<Uuid>,
    pub priority: i32,
    pub max_attempts: i32,
    pub payload: Value,
    pub created_by: Option<Uuid>,
}

pub async fn enqueue_index_job(pool: &PgPool, job: NewIndexJob) -> StorageResult<IndexJob> {
    Ok(sqlx::query_as::<_, IndexJob>(
        r#"
        INSERT INTO index_jobs (project_id, source_id, priority, max_attempts, payload, created_by)
        VALUES ($1,$2,$3,$4,$5,$6)
        RETURNING *
        "#,
    )
    .bind(job.project_id)
    .bind(job.source_id)
    .bind(job.priority)
    .bind(job.max_attempts)
    .bind(job.payload)
    .bind(job.created_by)
    .fetch_one(pool)
    .await?)
}

pub async fn acquire_index_job(pool: &PgPool) -> StorageResult<Option<IndexJob>> {
    let mut tx = pool.begin().await?;
    let job = sqlx::query_as::<_, IndexJob>(
        r#"
        WITH candidate AS (
            SELECT j.id
            FROM index_jobs j
            WHERE j.status = 'queued'
              AND j.run_at <= now()
              AND NOT EXISTS (
                  SELECT 1 FROM index_jobs r
                  WHERE r.status = 'running'
                    AND r.project_id = j.project_id
                    AND r.source_id IS NOT DISTINCT FROM j.source_id
              )
            ORDER BY j.priority DESC, j.run_at, j.created_at
            LIMIT 1
            FOR UPDATE OF j SKIP LOCKED
        )
        UPDATE index_jobs j
        SET status = 'running', started_at = now(), attempts = j.attempts + 1, updated_at = now()
        FROM candidate
        WHERE j.id = candidate.id
        RETURNING j.*
        "#,
    )
    .fetch_optional(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(job)
}

pub async fn heartbeat_index_job(pool: &PgPool, id: Uuid) -> StorageResult<()> {
    sqlx::query("UPDATE index_jobs SET updated_at = now() WHERE id = $1 AND status = 'running'")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn complete_index_job(pool: &PgPool, id: Uuid, generation: i64) -> StorageResult<bool> {
    let rows = sqlx::query(
        r#"
        UPDATE index_jobs
        SET status='succeeded', generation=$2, finished_at=now(), updated_at=now(), error=NULL
        WHERE id=$1 AND status='running'
        "#,
    )
    .bind(id)
    .bind(generation)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows > 0)
}

pub async fn fail_index_job(pool: &PgPool, id: Uuid, error: &str) -> StorageResult<bool> {
    let rows = sqlx::query(
        r#"
        UPDATE index_jobs
        SET status = CASE WHEN attempts < max_attempts THEN 'queued' ELSE 'failed' END,
            error = $2,
            run_at = CASE WHEN attempts < max_attempts
                THEN now() + make_interval(secs => attempts * attempts * 30)
                ELSE run_at END,
            finished_at = CASE WHEN attempts < max_attempts THEN NULL ELSE now() END,
            updated_at = now()
        WHERE id = $1 AND status = 'running'
        "#,
    )
    .bind(id)
    .bind(error)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows > 0)
}

pub async fn cancel_index_job(pool: &PgPool, id: Uuid) -> StorageResult<bool> {
    let rows = sqlx::query(
        "UPDATE index_jobs SET status='cancelled', finished_at=now(), updated_at=now() WHERE id=$1 AND status IN ('queued','running')",
    )
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows > 0)
}

pub async fn requeue_stale_index_jobs(pool: &PgPool, stale_seconds: i32) -> StorageResult<u64> {
    let rows = sqlx::query(
        r#"
        UPDATE index_jobs
        SET status='queued', run_at=now(), updated_at=now(),
            error='worker heartbeat timed out'
        WHERE status='running'
          AND updated_at < now() - make_interval(secs => $1)
        "#,
    )
    .bind(stale_seconds)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows)
}
