use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use thiserror::Error;
use tokio::process::Command;
use uuid::Uuid;

use crate::ValidatedGitUrl;

#[derive(Debug, Clone)]
pub struct GitCloneOptions {
    pub cache_root: PathBuf,
    pub source_id: Uuid,
    pub branch: Option<String>,
    pub timeout: Duration,
    pub max_bytes: u64,
}

#[derive(Debug, Clone)]
pub struct GitCheckout {
    pub path: PathBuf,
    pub commit_sha: String,
    pub size_bytes: u64,
}

#[derive(Debug, Error)]
pub enum GitCloneError {
    #[error("git command `{command}` failed with status {status}: {stderr}")]
    Git {
        command: String,
        status: String,
        stderr: String,
    },
    #[error("git command timed out after {0:?}")]
    Timeout(Duration),
    #[error("repository cache is invalid or corrupted")]
    InvalidCache,
    #[error("repository size {actual} exceeds limit {maximum}")]
    SizeLimit { actual: u64, maximum: u64 },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

pub async fn clone_or_fetch_public_repo(
    url: &ValidatedGitUrl,
    options: &GitCloneOptions,
) -> Result<GitCheckout, GitCloneError> {
    let checkout_path = options.cache_root.join(options.source_id.to_string());
    if checkout_path.exists() && !checkout_path.join(".git").exists() {
        return Err(GitCloneError::InvalidCache);
    }

    if checkout_path.exists() {
        fetch_existing(&checkout_path, url, options).await?;
    } else {
        clone_new(&checkout_path, url, options).await?;
    }

    let commit_sha = git_output(
        &["rev-parse", "HEAD"],
        Some(&checkout_path),
        options.timeout,
    )
    .await?;
    let size_bytes = directory_size(&checkout_path)?;
    if size_bytes > options.max_bytes {
        return Err(GitCloneError::SizeLimit {
            actual: size_bytes,
            maximum: options.max_bytes,
        });
    }

    Ok(GitCheckout {
        path: checkout_path,
        commit_sha: commit_sha.trim().to_string(),
        size_bytes,
    })
}

async fn clone_new(
    path: &Path,
    url: &ValidatedGitUrl,
    options: &GitCloneOptions,
) -> Result<(), GitCloneError> {
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let mut args = vec![
        "clone",
        "--depth",
        "1",
        "--single-branch",
        "--filter=blob:none",
    ];
    if let Some(branch) = options.branch.as_deref() {
        args.push("--branch");
        args.push(branch);
    }
    args.push(url.normalized_url.as_str());
    let path_arg = path.to_string_lossy().into_owned();
    args.push(path_arg.as_str());
    run_git(&args, None, options.timeout).await.map(drop)
}

async fn fetch_existing(
    path: &Path,
    url: &ValidatedGitUrl,
    options: &GitCloneOptions,
) -> Result<(), GitCloneError> {
    let remote_url = git_output(
        &["remote", "get-url", "origin"],
        Some(path),
        options.timeout,
    )
    .await?;
    if remote_url.trim() != url.normalized_url {
        return Err(GitCloneError::InvalidCache);
    }

    let branch = options.branch.as_deref().unwrap_or("HEAD");
    run_git(
        &["fetch", "--depth", "1", "origin", branch],
        Some(path),
        options.timeout,
    )
    .await?;
    run_git(
        &["checkout", "--force", "--detach", "FETCH_HEAD"],
        Some(path),
        options.timeout,
    )
    .await?;
    run_git(
        &["reset", "--hard", "FETCH_HEAD"],
        Some(path),
        options.timeout,
    )
    .await?;
    run_git(&["clean", "-ffdx"], Some(path), options.timeout).await?;
    Ok(())
}

async fn git_output(
    args: &[&str],
    cwd: Option<&Path>,
    timeout: Duration,
) -> Result<String, GitCloneError> {
    let output = run_git(args, cwd, timeout).await?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

async fn run_git(
    args: &[&str],
    cwd: Option<&Path>,
    timeout: Duration,
) -> Result<std::process::Output, GitCloneError> {
    let mut command = Command::new("git");
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .env("GIT_TERMINAL_PROMPT", "0")
        .env("GIT_ASKPASS", "true")
        .env("SSH_ASKPASS", "true");
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    let output = tokio::time::timeout(timeout, command.output())
        .await
        .map_err(|_| GitCloneError::Timeout(timeout))??;
    if !output.status.success() {
        return Err(GitCloneError::Git {
            command: format!("git {}", args.join(" ")),
            status: output.status.to_string(),
            stderr: redact_url_credentials(&String::from_utf8_lossy(&output.stderr)),
        });
    }
    Ok(output)
}

fn redact_url_credentials(message: &str) -> String {
    let mut redacted = message.to_string();
    for part in message.split_whitespace() {
        if let Some(scheme_end) = part.find("://") {
            let after_scheme = &part[scheme_end + 3..];
            if let Some(at) = after_scheme.find('@') {
                if after_scheme[..at].contains(':') {
                    let replacement = format!(
                        "{}://***:***@{}",
                        &part[..scheme_end],
                        &after_scheme[at + 1..]
                    );
                    redacted = redacted.replace(part, &replacement);
                }
            }
        }
    }
    redacted
}

fn directory_size(path: &Path) -> std::io::Result<u64> {
    let mut total = 0;
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let metadata = entry.metadata()?;
        if metadata.is_file() {
            total += metadata.len();
        } else if metadata.is_dir() {
            total += directory_size(&entry.path())?;
        }
    }
    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command as StdCommand;

    #[tokio::test]
    async fn clones_repository_and_resolves_commit() {
        let temp = tempfile::tempdir().unwrap();
        let origin = temp.path().join("origin");
        std::fs::create_dir(&origin).unwrap();
        for args in [
            vec!["init"],
            vec!["config", "user.email", "test@example.com"],
            vec!["config", "user.name", "Test"],
        ] {
            assert!(StdCommand::new("git")
                .args(&args)
                .current_dir(&origin)
                .status()
                .unwrap()
                .success());
        }
        std::fs::write(origin.join("README.md"), "# test\n").unwrap();
        assert!(StdCommand::new("git")
            .args(["add", "."])
            .current_dir(&origin)
            .status()
            .unwrap()
            .success());
        assert!(StdCommand::new("git")
            .args(["commit", "-m", "init"])
            .current_dir(&origin)
            .status()
            .unwrap()
            .success());

        let url = ValidatedGitUrl {
            normalized_url: origin.to_string_lossy().into_owned(),
            host: "local-test".into(),
            port: 443,
        };
        let options = GitCloneOptions {
            cache_root: temp.path().join("cache"),
            source_id: Uuid::new_v4(),
            branch: None,
            timeout: Duration::from_secs(10),
            max_bytes: 10_000_000,
        };
        let checkout = clone_or_fetch_public_repo(&url, &options).await.unwrap();
        assert_eq!(checkout.commit_sha.len(), 40);
        assert!(checkout.path.join("README.md").exists());
        let second = clone_or_fetch_public_repo(&url, &options).await.unwrap();
        assert_eq!(checkout.commit_sha, second.commit_sha);
    }
}
