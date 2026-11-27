use std::path::PathBuf;
use std::time::Duration;

use gf_core::EmbeddingProvider;
use gf_storage::models::Source;
use gf_storage::{
    acquire_index_job, complete_index_job, fail_index_job, get_source, update_index_job_progress,
    IndexJob,
};
use serde_json::{json, Value};
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

fn worker_error_details(error: &WorkerError) -> Value {
    match error {
        WorkerError::Clone(GitCloneError::Git {
            command,
            status,
            stderr,
            diagnostic,
        }) => json!({
            "category": &diagnostic.category,
            "summary": &diagnostic.summary,
            "auth": {
                "configured": diagnostic.auth_configured,
                "username": &diagnostic.auth_username,
                "token_type": &diagnostic.token_type,
                "token_had_surrounding_whitespace": diagnostic.token_had_surrounding_whitespace
            },
            "target": {
                "host": &diagnostic.target_host,
                "repository": &diagnostic.target_repository
            },
            "likely_causes": &diagnostic.likely_causes,
            "recommended_actions": &diagnostic.recommended_actions,
            "technical": {
                "command": command,
                "status": status,
                "stderr": stderr
            }
        }),
        WorkerError::Credential(message) => json!({
            "category": "git_credential_configuration_error",
            "summary": "GraphFinder could not load or validate the configured git provider credential before cloning.",
            "likely_causes": [message],
            "recommended_actions": [
                "check that the source references the correct provider credential",
                "check that the provider host matches the repository host",
                "check that GF_CREDENTIALS_ENCRYPTION_KEY is unchanged and correctly configured"
            ]
        }),
        _ => json!({
            "category": "indexing_error",
            "summary": error.to_string()
        }),
    }
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
            let details = worker_error_details(&error);
            let _ = fail_index_job(pool, job.id, &error.to_string(), &details).await;
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

fn effective_git_username(provider_type: &str, username: &str, secret: &str) -> String {
    if provider_type == "github"
        && secret.starts_with("github_pat_")
        && username.eq_ignore_ascii_case("x-access-token")
    {
        "oauth2".to_string()
    } else {
        username.to_string()
    }
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
            .map_err(|error| WorkerError::Credential(error.to_string()))?
            .trim()
            .to_string();
    let _ = gf_storage::touch_git_provider(pool, provider.id).await;
    let username = effective_git_username(&provider.provider, &provider.username, &secret);
    Ok(Some(GitAuth { username, secret }))
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

#[cfg(test)]
mod tests {
    use super::effective_git_username;

    #[test]
    fn fine_grained_github_pat_uses_oauth2_username_when_default_was_stored() {
        assert_eq!(
            effective_git_username("github", "x-access-token", "github_pat_test"),
            "oauth2"
        );
    }

    #[test]
    fn custom_usernames_and_other_token_types_are_preserved() {
        assert_eq!(
            effective_git_username("github", "custom-user", "github_pat_test"),
            "custom-user"
        );
        assert_eq!(
            effective_git_username("github", "x-access-token", "ghs_app_token"),
            "x-access-token"
        );
        assert_eq!(
            effective_git_username("gitlab", "oauth2", "token"),
            "oauth2"
        );
    }
}
