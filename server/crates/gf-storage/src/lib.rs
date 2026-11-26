//! PostgreSQL storage layer for GraphFinder Server.

pub mod audit;
pub mod auth;
pub mod embeddings;
pub mod generations;
pub mod jobs;
pub mod models;
pub mod oidc;
pub mod rbac;
pub mod repositories;
pub mod search;
pub mod vault;
pub mod webhooks;

use sqlx::PgPool;
use thiserror::Error;

pub use audit::*;
pub use auth::*;
pub use embeddings::*;
pub use generations::*;
pub use jobs::*;
pub use oidc::*;
pub use rbac::*;
pub use repositories::*;
pub use search::*;
pub use vault::*;
pub use webhooks::*;

pub type StorageResult<T> = Result<T, StorageError>;

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("migration error: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("invalid embedding dimensions: expected {expected}, got {actual}")]
    InvalidEmbeddingDimensions { expected: usize, actual: usize },
    #[error("authentication error: {0}")]
    Auth(String),
    #[error("conflict: {0}")]
    Conflict(String),
}

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

pub async fn connect(database_url: &str) -> StorageResult<PgPool> {
    Ok(PgPool::connect(database_url).await?)
}

pub async fn migrate(database_url: &str) -> StorageResult<()> {
    let pool = connect(database_url).await?;
    migrate_with_pool(&pool).await
}

pub async fn migrate_with_pool(pool: &PgPool) -> StorageResult<()> {
    MIGRATOR.run(pool).await?;
    Ok(())
}

pub fn migration_count() -> usize {
    MIGRATOR.migrations.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn migrations_are_embedded() {
        assert!(migration_count() >= 1);
    }
}
