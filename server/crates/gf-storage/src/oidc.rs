use chrono::{DateTime, Duration, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{StorageError, StorageResult, User};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct OidcAuthState {
    pub id: Uuid,
    pub provider: String,
    pub tenant_slug: String,
    pub state_hash: String,
    pub nonce: String,
    pub pkce_verifier: String,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

pub fn oidc_state_hash(state: &str) -> String {
    format!("{:x}", Sha256::digest(state.as_bytes()))
}

pub async fn create_oidc_auth_state(
    pool: &PgPool,
    tenant_slug: &str,
    state: &str,
    nonce: &str,
    pkce_verifier: &str,
    ttl_minutes: i64,
) -> StorageResult<()> {
    sqlx::query(
        r#"
        INSERT INTO oidc_auth_states (tenant_slug,state_hash,nonce,pkce_verifier,expires_at)
        VALUES ($1,$2,$3,$4,$5)
        "#,
    )
    .bind(tenant_slug)
    .bind(oidc_state_hash(state))
    .bind(nonce)
    .bind(pkce_verifier)
    .bind(Utc::now() + Duration::minutes(ttl_minutes))
    .execute(pool)
    .await?;
    sqlx::query("DELETE FROM oidc_auth_states WHERE expires_at < now() - interval '1 hour'")
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn consume_oidc_auth_state(
    pool: &PgPool,
    state: &str,
) -> StorageResult<Option<OidcAuthState>> {
    Ok(sqlx::query_as::<_, OidcAuthState>(
        r#"
        UPDATE oidc_auth_states
        SET consumed_at=now()
        WHERE state_hash=$1
          AND consumed_at IS NULL
          AND expires_at > now()
        RETURNING *
        "#,
    )
    .bind(oidc_state_hash(state))
    .fetch_optional(pool)
    .await?)
}

pub struct OidcUserIdentity<'a> {
    pub tenant_slug: &'a str,
    pub issuer: &'a str,
    pub subject: &'a str,
    pub email: &'a str,
    pub display_name: &'a str,
    pub mapped_role: Option<&'a str>,
    pub default_role: &'a str,
}

pub async fn upsert_oidc_user_identity(
    pool: &PgPool,
    identity: OidcUserIdentity<'_>,
) -> StorageResult<User> {
    let mut tx = pool.begin().await?;
    let tenant_id = sqlx::query_scalar::<_, Uuid>("SELECT id FROM tenants WHERE slug=$1")
        .bind(identity.tenant_slug)
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| StorageError::Auth("tenant not found".into()))?;

    let existing_user_id = sqlx::query_scalar::<_, Uuid>(
        "SELECT user_id FROM oidc_identities WHERE tenant_id=$1 AND issuer=$2 AND subject=$3",
    )
    .bind(tenant_id)
    .bind(identity.issuer)
    .bind(identity.subject)
    .fetch_optional(&mut *tx)
    .await?;

    let user = if let Some(user_id) = existing_user_id {
        sqlx::query_as::<_, User>(
            r#"
            UPDATE users SET display_name=$2,updated_at=now()
            WHERE id=$1
            RETURNING id,tenant_id,email,password_hash,display_name,status
            "#,
        )
        .bind(user_id)
        .bind(identity.display_name)
        .fetch_one(&mut *tx)
        .await?
    } else {
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (tenant_id,email,display_name,password_hash)
            VALUES ($1,$2,$3,NULL)
            ON CONFLICT (tenant_id,email) DO UPDATE
            SET display_name=EXCLUDED.display_name,updated_at=now()
            RETURNING id,tenant_id,email,password_hash,display_name,status
            "#,
        )
        .bind(tenant_id)
        .bind(identity.email)
        .bind(identity.display_name)
        .fetch_one(&mut *tx)
        .await?;
        sqlx::query(
            r#"
            INSERT INTO oidc_identities (tenant_id,user_id,issuer,subject,email)
            VALUES ($1,$2,$3,$4,$5)
            ON CONFLICT (tenant_id,issuer,subject) DO UPDATE
            SET user_id=EXCLUDED.user_id,email=EXCLUDED.email,updated_at=now()
            "#,
        )
        .bind(tenant_id)
        .bind(user.id)
        .bind(identity.issuer)
        .bind(identity.subject)
        .bind(identity.email)
        .execute(&mut *tx)
        .await?;
        user
    };

    sqlx::query(
        r#"
        INSERT INTO tenant_memberships (tenant_id,user_id,role)
        VALUES ($1,$2,$3)
        ON CONFLICT (tenant_id,user_id) DO NOTHING
        "#,
    )
    .bind(tenant_id)
    .bind(user.id)
    .bind(identity.default_role)
    .execute(&mut *tx)
    .await?;
    if let Some(role) = identity.mapped_role {
        sqlx::query("UPDATE tenant_memberships SET role=$3 WHERE tenant_id=$1 AND user_id=$2")
            .bind(tenant_id)
            .bind(user.id)
            .bind(role)
            .execute(&mut *tx)
            .await?;
    }

    tx.commit().await?;
    Ok(user)
}
