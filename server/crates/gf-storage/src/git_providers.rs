use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use sqlx::PgPool;
use uuid::Uuid;

use crate::{StorageError, StorageResult};

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct GitProvider {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub name: String,
    pub provider: String,
    pub host: String,
    pub username: String,
    pub secret_ciphertext: Vec<u8>,
    pub secret_nonce: Vec<u8>,
    pub secret_hint: String,
    pub last_used_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub struct NewGitProvider {
    pub tenant_id: Uuid,
    pub name: String,
    pub provider: String,
    pub host: String,
    pub username: String,
    pub secret: String,
}

pub struct GitProviderUpdate {
    pub name: Option<String>,
    pub username: Option<String>,
    pub secret: Option<String>,
}

pub type CredentialKey = [u8; 32];

pub fn credential_key_from_env() -> Result<CredentialKey, String> {
    let value = std::env::var("GF_CREDENTIALS_ENCRYPTION_KEY").map_err(|_| {
        "GF_CREDENTIALS_ENCRYPTION_KEY is required for git provider credentials".to_string()
    })?;
    parse_credential_key(&value)
}

pub fn parse_credential_key(value: &str) -> Result<CredentialKey, String> {
    let trimmed = value.trim();
    if let Ok(bytes) = BASE64.decode(trimmed) {
        if bytes.len() == 32 {
            return bytes
                .try_into()
                .map_err(|_| "credential key must be 32 bytes".to_string());
        }
    }
    if trimmed.len() == 64 && trimmed.chars().all(|ch| ch.is_ascii_hexdigit()) {
        let bytes = (0..trimmed.len())
            .step_by(2)
            .map(|index| u8::from_str_radix(&trimmed[index..index + 2], 16))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "invalid hex credential key".to_string())?;
        if bytes.len() == 32 {
            return bytes
                .try_into()
                .map_err(|_| "credential key must be 32 bytes".to_string());
        }
    }
    if trimmed.len() == 32 {
        return trimmed
            .as_bytes()
            .try_into()
            .map_err(|_| "credential key must be 32 bytes".to_string());
    }
    Err("credential key must be 32 bytes as raw text, hex or base64".to_string())
}

fn cipher(key: &CredentialKey) -> Aes256Gcm {
    Aes256Gcm::new_from_slice(key).expect("32-byte AES-256 key")
}

pub fn encrypt_secret(key: &CredentialKey, secret: &str) -> StorageResult<(Vec<u8>, Vec<u8>)> {
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let ciphertext = cipher(key)
        .encrypt(&nonce, secret.as_bytes())
        .map_err(|_| StorageError::Crypto("git provider secret encryption failed".into()))?;
    Ok((ciphertext, nonce.to_vec()))
}

pub fn decrypt_secret(
    key: &CredentialKey,
    ciphertext: &[u8],
    nonce: &[u8],
) -> StorageResult<String> {
    let nonce = Nonce::from_slice(nonce);
    let plaintext = cipher(key)
        .decrypt(nonce, ciphertext)
        .map_err(|_| StorageError::Crypto("git provider secret decryption failed".into()))?;
    String::from_utf8(plaintext)
        .map_err(|_| StorageError::Crypto("git provider secret is not utf-8".into()))
}

fn hint(secret: &str) -> String {
    let suffix = secret
        .chars()
        .rev()
        .take(4)
        .collect::<String>()
        .chars()
        .rev()
        .collect::<String>();
    format!("…{suffix}")
}

fn encrypted_parts(key: &CredentialKey, secret: &str) -> StorageResult<(Vec<u8>, Vec<u8>, String)> {
    let (ciphertext, nonce) = encrypt_secret(key, secret)?;
    Ok((ciphertext, nonce, hint(secret)))
}

pub async fn create_git_provider(
    pool: &PgPool,
    key: &CredentialKey,
    input: NewGitProvider,
) -> StorageResult<GitProvider> {
    let (ciphertext, nonce, hint) = encrypted_parts(key, &input.secret)?;
    Ok(sqlx::query_as::<_, GitProvider>(
        r#"
        INSERT INTO git_providers (tenant_id,name,provider,host,username,secret_ciphertext,secret_nonce,secret_hint)
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8)
        RETURNING *
        "#,
    )
    .bind(input.tenant_id)
    .bind(input.name)
    .bind(input.provider)
    .bind(input.host)
    .bind(input.username)
    .bind(ciphertext)
    .bind(nonce)
    .bind(hint)
    .fetch_one(pool)
    .await?)
}

pub async fn list_git_providers(pool: &PgPool, tenant_id: Uuid) -> StorageResult<Vec<GitProvider>> {
    Ok(sqlx::query_as::<_, GitProvider>(
        "SELECT * FROM git_providers WHERE tenant_id=$1 ORDER BY created_at DESC",
    )
    .bind(tenant_id)
    .fetch_all(pool)
    .await?)
}

pub async fn get_git_provider(
    pool: &PgPool,
    tenant_id: Uuid,
    id: Uuid,
) -> StorageResult<Option<GitProvider>> {
    Ok(
        sqlx::query_as::<_, GitProvider>(
            "SELECT * FROM git_providers WHERE tenant_id=$1 AND id=$2",
        )
        .bind(tenant_id)
        .bind(id)
        .fetch_optional(pool)
        .await?,
    )
}

pub async fn update_git_provider(
    pool: &PgPool,
    key: &CredentialKey,
    tenant_id: Uuid,
    id: Uuid,
    update: GitProviderUpdate,
) -> StorageResult<Option<GitProvider>> {
    let existing = get_git_provider(pool, tenant_id, id).await?;
    let Some(existing) = existing else {
        return Ok(None);
    };
    let (ciphertext, nonce, hint) = if let Some(secret) = update.secret {
        encrypted_parts(key, &secret)?
    } else {
        (
            existing.secret_ciphertext,
            existing.secret_nonce,
            existing.secret_hint,
        )
    };
    Ok(sqlx::query_as::<_, GitProvider>(
        r#"
        UPDATE git_providers
        SET name=COALESCE($3,name),
            username=COALESCE($4,username),
            secret_ciphertext=$5,
            secret_nonce=$6,
            secret_hint=$7,
            updated_at=now()
        WHERE tenant_id=$1 AND id=$2
        RETURNING *
        "#,
    )
    .bind(tenant_id)
    .bind(id)
    .bind(update.name)
    .bind(update.username)
    .bind(ciphertext)
    .bind(nonce)
    .bind(hint)
    .fetch_optional(pool)
    .await?)
}

pub async fn delete_git_provider(pool: &PgPool, tenant_id: Uuid, id: Uuid) -> StorageResult<bool> {
    let rows = sqlx::query("DELETE FROM git_providers WHERE tenant_id=$1 AND id=$2")
        .bind(tenant_id)
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    Ok(rows > 0)
}

pub async fn touch_git_provider(pool: &PgPool, id: Uuid) -> StorageResult<()> {
    sqlx::query("UPDATE git_providers SET last_used_at=now() WHERE id=$1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn count_git_provider_sources(pool: &PgPool, id: Uuid) -> StorageResult<i64> {
    Ok(
        sqlx::query_scalar("SELECT count(*) FROM sources WHERE credential_ref=$1")
            .bind(id.to_string())
            .fetch_one(pool)
            .await?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypted_provider_secret_round_trips() {
        let raw_key = "k".repeat(32);
        let key = parse_credential_key(&raw_key).unwrap();
        let (ciphertext, nonce) = encrypt_secret(&key, "secret-token").unwrap();
        assert_ne!(ciphertext, b"secret-token");
        assert_eq!(
            decrypt_secret(&key, &ciphertext, &nonce).unwrap(),
            "secret-token"
        );
    }

    #[test]
    fn parses_base64_hex_and_raw_keys() {
        let raw_key = "k".repeat(32);
        let base64_key = BASE64.encode([7u8; 32]);
        let hex_key = (0..32)
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        assert!(parse_credential_key(&raw_key).is_ok());
        assert!(parse_credential_key(&base64_key).is_ok());
        assert!(parse_credential_key(&hex_key).is_ok());
        assert!(parse_credential_key("short").is_err());
    }
}
