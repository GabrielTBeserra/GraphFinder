use std::path::PathBuf;
use std::time::Duration;

use gf_core::EmbeddingProvider;
use gf_storage::models::Source;
use gf_storage::{
    acquire_index_job, complete_index_job, fail_index_job, get_source, update_index_job_progress,
    IndexJob,
};
use sqlx::PgPool;
use thiserror::Error;
use uuid::Uuid;

use crate::{
    clone_or_fetch_public_repo, index_checkout, validate_git_source_url, GitAuth, GitCloneError,
    GitCloneOptions, GitSourcePolicy, GitSourceValidationError, IndexingError,
};

#[derive(Debug, Clone)]
pub struct WorkerConfig {
    pub repo_cache_root: PathBuf,
    pub allow_private_git_hosts: bool,
    pub allowed_git_hosts: Vec<String>,
    pub git_timeout: Duration,
    pub max_repo_bytes: u64,
}

#[derive(Debug, Error)]
pub enum WorkerError {
    #[error(transparent)]
    Storage(#[from] gf_storage::StorageError),
    #[error(transparent)]
    Validation(#[from] GitSourceValidationError),
    #[error(transparent)]
    Clone(#[from] GitCloneError),
    #[error(transparent)]
    Indexing(#[from] IndexingError),
    #[error("job {0} has no source")]
    MissingSource(Uuid),
    #[error("source {0} not found")]
    SourceNotFound(Uuid),
    #[error("git provider credential error: {0}")]
    Credential(String),
}

pub async fn process_next_index_job(
    pool: &PgPool,
    config: &WorkerConfig,
    embedder: &dyn EmbeddingProvider,
) -> Result<bool, WorkerError> {
    let Some(job) = acquire_index_job(pool).await? else {
        return Ok(false);
    };

    let generation = match process_job(pool, &job, config, embedder).await {
        Ok(generation) => generation,
        Err(error) => {
            let _ = fail_index_job(pool, job.id, &error.to_string()).await;
            return Err(error);
        }
    };
    complete_index_job(pool, job.id, generation).await?;
    Ok(true)
}

async fn process_job(
    pool: &PgPool,
    job: &IndexJob,
    config: &WorkerConfig,
    embedder: &dyn EmbeddingProvider,
) -> Result<i64, WorkerError> {
    let source_id = job.source_id.ok_or(WorkerError::MissingSource(job.id))?;
    let source = get_source(pool, source_id)
        .await?
        .ok_or(WorkerError::SourceNotFound(source_id))?;
    process_source(pool, job, &source, config, embedder).await
}

async fn git_auth_for_source(
    pool: &PgPool,
    source: &Source,
    validated_url: &crate::ValidatedGitUrl,
) -> Result<Option<GitAuth>, WorkerError> {
    let Some(reference) = source.credential_ref.as_deref() else {
        return Ok(None);
    };
    let provider_id = Uuid::parse_str(reference)
        .map_err(|_| WorkerError::Credential("credential_ref is not a git provider id".into()))?;
    let project = gf_storage::get_project(pool, source.project_id)
        .await?
        .ok_or_else(|| WorkerError::Credential("source project not found".into()))?;
    let provider = gf_storage::get_git_provider(pool, project.tenant_id, provider_id)
        .await?
        .ok_or_else(|| WorkerError::Credential("git provider not found".into()))?;
    if !provider.host.eq_ignore_ascii_case(&validated_url.host) {
        return Err(WorkerError::Credential(format!(
            "git provider host {} does not match source host {}",
            provider.host, validated_url.host
        )));
    }
    let key = gf_storage::credential_key_from_env().map_err(WorkerError::Credential)?;
    let secret =
        gf_storage::decrypt_secret(&key, &provider.secret_ciphertext, &provider.secret_nonce)
            .map_err(|error| WorkerError::Credential(error.to_string()))?;
    let _ = gf_storage::touch_git_provider(pool, provider.id).await;
    Ok(Some(GitAuth {
        username: provider.username,
        secret,
    }))
}

async fn process_source(
    pool: &PgPool,
    job: &IndexJob,
    source: &Source,
    config: &WorkerConfig,
    embedder: &dyn EmbeddingProvider,
) -> Result<i64, WorkerError> {
    let policy = GitSourcePolicy {
        allow_private_hosts: config.allow_private_git_hosts,
        allowed_hosts: config.allowed_git_hosts.clone(),
    };
    update_index_job_progress(pool, job.id, 10, "validating_git_source").await?;
    let url = validate_git_source_url(&source.uri, &policy)?;
    let branch = job
        .payload
        .get("branch")
        .and_then(|v| v.as_str())
        .map(str::to_string)
        .or_else(|| source.branch.clone());
    let auth = git_auth_for_source(pool, source, &url).await?;
    update_index_job_progress(pool, job.id, 20, "cloning_repository").await?;
    let checkout = clone_or_fetch_public_repo(
        &url,
        &GitCloneOptions {
            cache_root: config.repo_cache_root.clone(),
            source_id: source.id,
            branch,
            timeout: config.git_timeout,
            max_bytes: config.max_repo_bytes,
            auth,
        },
    )
    .await?;
    update_index_job_progress(pool, job.id, 40, "indexing_repository").await?;
    let report = index_checkout(
        pool,
        &checkout.path,
        job.project_id,
        Some(source.id),
        &source.include_patterns,
        &source.exclude_patterns,
        embedder,
    )
    .await?;
    gf_storage::record_generation_source_revision(
        pool,
        job.project_id,
        report.generation,
        source.id,
        &checkout.commit_sha,
    )
    .await?;
    update_index_job_progress(pool, job.id, 95, "published_generation").await?;
    Ok(report.generation)
}
