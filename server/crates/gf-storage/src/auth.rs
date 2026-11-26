use argon2::password_hash::{
    rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString,
};
use argon2::Argon2;
use chrono::{DateTime, Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{StorageError, StorageResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct User {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub email: String,
    pub password_hash: Option<String>,
    pub display_name: String,
    pub status: String,
}

pub struct AuthSession {
    pub id: Uuid,
    pub user_id: Uuid,
    pub tenant_id: Uuid,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

pub fn hash_password(password: &str) -> StorageResult<String> {
    let salt = SaltString::generate(&mut OsRng);
    Ok(Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|e| StorageError::Auth(e.to_string()))?
        .to_string())
}

pub fn verify_password(password: &str, hash: &str) -> StorageResult<bool> {
    let parsed = PasswordHash::new(hash).map_err(|e| StorageError::Auth(e.to_string()))?;
    Ok(Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .is_ok())
}

pub fn session_token() -> String {
    format!("gfs_{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

fn token_hash(token: &str) -> String {
    format!("{:x}", Sha256::digest(token.as_bytes()))
}

pub async fn ensure_tenant(pool: &PgPool, slug: &str, name: &str) -> StorageResult<Uuid> {
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO tenants (slug, name) VALUES ($1,$2)
        ON CONFLICT (slug) DO UPDATE SET name=EXCLUDED.name
        RETURNING id
        "#,
    )
    .bind(slug)
    .bind(name)
    .fetch_one(pool)
    .await?)
}

pub async fn upsert_user(
    pool: &PgPool,
    tenant_id: Uuid,
    email: &str,
    display_name: &str,
    password_hash: &str,
) -> StorageResult<Uuid> {
    Ok(sqlx::query_scalar(
        r#"
        INSERT INTO users (tenant_id,email,display_name,password_hash)
        VALUES ($1,$2,$3,$4)
        ON CONFLICT (tenant_id,email) DO UPDATE
        SET display_name=EXCLUDED.display_name,password_hash=EXCLUDED.password_hash,updated_at=now()
        RETURNING id
        "#,
    )
    .bind(tenant_id)
    .bind(email)
    .bind(display_name)
    .bind(password_hash)
    .fetch_one(pool)
    .await?)
}

pub async fn find_user_by_email(
    pool: &PgPool,
    tenant_slug: &str,
    email: &str,
) -> StorageResult<Option<User>> {
    Ok(sqlx::query_as::<_, User>(
        r#"
        SELECT u.id,u.tenant_id,u.email,u.password_hash,u.display_name,u.status
        FROM users u JOIN tenants t ON t.id=u.tenant_id
        WHERE t.slug=$1 AND u.email=$2
        "#,
    )
    .bind(tenant_slug)
    .bind(email)
    .fetch_optional(pool)
    .await?)
}

pub async fn create_session(
    pool: &PgPool,
    user: &User,
    ttl_hours: i64,
) -> StorageResult<AuthSession> {
    let token = session_token();
    let expires_at = Utc::now() + Duration::hours(ttl_hours);
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO auth_sessions (tenant_id,user_id,token_hash,expires_at)
        VALUES ($1,$2,$3,$4) RETURNING id
        "#,
    )
    .bind(user.tenant_id)
    .bind(user.id)
    .bind(token_hash(&token))
    .bind(expires_at)
    .fetch_one(pool)
    .await?;
    Ok(AuthSession {
        id,
        user_id: user.id,
        tenant_id: user.tenant_id,
        token,
        expires_at,
    })
}

pub async fn revoke_session(pool: &PgPool, token: &str) -> StorageResult<bool> {
    let rows = sqlx::query(
        "UPDATE auth_sessions SET revoked_at=now() WHERE token_hash=$1 AND revoked_at IS NULL",
    )
    .bind(token_hash(token))
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows > 0)
}

#[derive(Debug, Clone)]
pub struct Principal {
    pub tenant_id: Uuid,
    pub user_id: Option<Uuid>,
    pub token_id: Option<Uuid>,
    pub scopes: Vec<String>,
    pub project_ids: Option<Vec<Uuid>>,
    pub kind: String,
}

pub struct CreatedApiToken {
    pub id: Uuid,
    pub token: String,
    pub expires_at: Option<DateTime<Utc>>,
}

pub fn api_token_secret() -> String {
    format!("gfp_{}{}", Uuid::new_v4().simple(), Uuid::new_v4().simple())
}

pub async fn create_api_token(
    pool: &PgPool,
    tenant_id: Uuid,
    user_id: Option<Uuid>,
    name: &str,
    scopes: Vec<String>,
    project_ids: Option<Vec<Uuid>>,
    expires_in_days: Option<i64>,
) -> StorageResult<CreatedApiToken> {
    let token = api_token_secret();
    let expires_at = expires_in_days.map(|days| Utc::now() + Duration::days(days));
    let id = sqlx::query_scalar::<_, Uuid>(
        r#"
        INSERT INTO api_tokens (tenant_id,user_id,name,token_hash,scopes,project_ids,expires_at)
        VALUES ($1,$2,$3,$4,$5,$6,$7) RETURNING id
        "#,
    )
    .bind(tenant_id)
    .bind(user_id)
    .bind(name)
    .bind(token_hash(&token))
    .bind(scopes)
    .bind(project_ids)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;
    Ok(CreatedApiToken {
        id,
        token,
        expires_at,
    })
}

pub async fn revoke_api_token(pool: &PgPool, tenant_id: Uuid, id: Uuid) -> StorageResult<bool> {
    let rows = sqlx::query(
        "UPDATE api_tokens SET revoked_at=now() WHERE tenant_id=$1 AND id=$2 AND revoked_at IS NULL",
    )
    .bind(tenant_id)
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();
    Ok(rows > 0)
}

pub async fn authenticate_bearer_token(
    pool: &PgPool,
    token: &str,
) -> StorageResult<Option<Principal>> {
    let hash = token_hash(token);
    if let Some(row) = sqlx::query_as::<_, (Uuid, Uuid)>(
        r#"
        SELECT tenant_id,user_id FROM auth_sessions
        WHERE token_hash=$1 AND revoked_at IS NULL AND expires_at > now()
        "#,
    )
    .bind(&hash)
    .fetch_optional(pool)
    .await?
    {
        sqlx::query("UPDATE auth_sessions SET last_used_at=now() WHERE token_hash=$1")
            .bind(&hash)
            .execute(pool)
            .await?;
        return Ok(Some(Principal {
            tenant_id: row.0,
            user_id: Some(row.1),
            token_id: None,
            scopes: vec!["session".into()],
            project_ids: None,
            kind: "session".into(),
        }));
    }

    if let Some(row) =
        sqlx::query_as::<_, (Uuid, Uuid, Option<Uuid>, Vec<String>, Option<Vec<Uuid>>)>(
            r#"
        SELECT tenant_id,id,user_id,scopes,project_ids FROM api_tokens
        WHERE token_hash=$1 AND revoked_at IS NULL
          AND (expires_at IS NULL OR expires_at > now())
        "#,
        )
        .bind(&hash)
        .fetch_optional(pool)
        .await?
    {
        sqlx::query("UPDATE api_tokens SET last_used_at=now() WHERE id=$1")
            .bind(row.1)
            .execute(pool)
            .await?;
        return Ok(Some(Principal {
            tenant_id: row.0,
            token_id: Some(row.1),
            user_id: row.2,
            scopes: row.3,
            project_ids: row.4,
            kind: "api_token".into(),
        }));
    }
    Ok(None)
}
