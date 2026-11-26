use sha2::{Digest, Sha256};
use sqlx::PgPool;

use crate::StorageResult;

fn bucket_hash(bucket: &str) -> String {
    format!("{:x}", Sha256::digest(bucket.as_bytes()))
}

/// Atomically increments a fixed-window bucket and returns whether the request is allowed.
/// PostgreSQL is the coordination point, so the limit is shared by all API replicas.
pub async fn check_rate_limit(
    pool: &PgPool,
    bucket: &str,
    limit: i32,
    window_seconds: i32,
) -> StorageResult<bool> {
    let allowed = sqlx::query_scalar::<_, bool>(
        r#"
        INSERT INTO rate_limit_buckets (bucket_hash,window_start,request_count,expires_at)
        VALUES ($1,now(),1,now() + make_interval(secs => $3::double precision))
        ON CONFLICT (bucket_hash) DO UPDATE SET
            window_start = CASE
                WHEN rate_limit_buckets.window_start + make_interval(secs => $3::double precision) <= now()
                THEN now()
                ELSE rate_limit_buckets.window_start
            END,
            request_count = CASE
                WHEN rate_limit_buckets.window_start + make_interval(secs => $3::double precision) <= now()
                THEN 1
                ELSE rate_limit_buckets.request_count + 1
            END,
            expires_at = CASE
                WHEN rate_limit_buckets.window_start + make_interval(secs => $3::double precision) <= now()
                THEN now() + make_interval(secs => $3::double precision)
                ELSE rate_limit_buckets.expires_at
            END
        RETURNING request_count <= $2
        "#,
    )
    .bind(bucket_hash(bucket))
    .bind(limit)
    .bind(window_seconds)
    .fetch_one(pool)
    .await?;

    sqlx::query("DELETE FROM rate_limit_buckets WHERE expires_at < now() - interval '1 hour'")
        .execute(pool)
        .await?;
    Ok(allowed)
}
