use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;

use base64::engine::general_purpose::STANDARD as BASE64;
use base64::Engine;
use thiserror::Error;
use tokio::process::Command;
use uuid::Uuid;

use crate::ValidatedGitUrl;

#[derive(Debug, Clone)]
pub struct GitAuth {
    pub username: String,
    pub secret: String,
}

#[derive(Debug, Clone)]
pub struct GitCloneOptions {
    pub cache_root: PathBuf,
    pub source_id: Uuid,
    pub branch: Option<String>,
    pub timeout: Duration,
    pub max_bytes: u64,
    pub auth: Option<GitAuth>,
}

#[derive(Debug, Clone)]
pub struct GitCheckout {
    pub path: PathBuf,
    pub commit_sha: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitFailureDiagnostic {
    pub category: String,
    pub summary: String,
    pub auth_configured: bool,
    pub auth_username: Option<String>,
    pub token_type: Option<String>,
    pub token_had_surrounding_whitespace: bool,
    pub target_host: Option<String>,
    pub target_repository: Option<String>,
    pub likely_causes: Vec<String>,
    pub recommended_actions: Vec<String>,
}

impl fmt::Display for GitFailureDiagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Access diagnostic: {}", self.summary)?;
        if self.auth_configured {
            write!(f, "Credential context: authentication was configured")?;
            if let Some(username) = &self.auth_username {
                write!(f, " with username `{username}`")?;
            }
            if let Some(token_type) = &self.token_type {
                write!(f, " and token type `{token_type}`")?;
            }
            if self.token_had_surrounding_whitespace {
                write!(f, "; the stored token has leading/trailing whitespace")?;
            }
            writeln!(f, ".")?;
        } else {
            writeln!(
                f,
                "Credential context: no git provider credential was configured for this source."
            )?;
        }
        if !self.likely_causes.is_empty() {
            writeln!(f, "Most likely causes:")?;
            for cause in &self.likely_causes {
                writeln!(f, "- {cause}")?;
            }
        }
        if !self.recommended_actions.is_empty() {
            writeln!(f, "How to fix:")?;
            for (index, action) in self.recommended_actions.iter().enumerate() {
                writeln!(f, "{}. {action}", index + 1)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum GitCloneError {
    #[error("{diagnostic}\nTechnical details: git command `{command}` failed with status {status}: {stderr}")]
    Git {
        command: String,
        status: String,
        stderr: String,
        diagnostic: GitFailureDiagnostic,
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
        options.auth.as_ref(),
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
    run_git(&args, None, options.timeout, options.auth.as_ref())
        .await
        .map(drop)
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
        options.auth.as_ref(),
    )
    .await?;
    if remote_url.trim() != url.normalized_url {
        return Err(GitCloneError::InvalidCache);
    }

    let branch = options.branch.as_deref().unwrap_or("HEAD");
    run_git(
        &["fetch", "--depth", "1", url.normalized_url.as_str(), branch],
        Some(path),
        options.timeout,
        options.auth.as_ref(),
    )
    .await?;
    run_git(
        &["checkout", "--force", "--detach", "FETCH_HEAD"],
        Some(path),
        options.timeout,
        options.auth.as_ref(),
    )
    .await?;
    run_git(
        &["reset", "--hard", "FETCH_HEAD"],
        Some(path),
        options.timeout,
        options.auth.as_ref(),
    )
    .await?;
    run_git(
        &["clean", "-ffdx"],
        Some(path),
        options.timeout,
        options.auth.as_ref(),
    )
    .await?;
    Ok(())
}

async fn git_output(
    args: &[&str],
    cwd: Option<&Path>,
    timeout: Duration,
    auth: Option<&GitAuth>,
) -> Result<String, GitCloneError> {
    let output = run_git(args, cwd, timeout, auth).await?;
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

async fn run_git(
    args: &[&str],
    cwd: Option<&Path>,
    timeout: Duration,
    auth: Option<&GitAuth>,
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
    if let Some(auth) = auth {
        let basic = BASE64.encode(format!("{}:{}", auth.username, auth.secret));
        command
            .env("GIT_CONFIG_COUNT", "2")
            .env("GIT_CONFIG_KEY_0", "credential.helper")
            .env("GIT_CONFIG_VALUE_0", "")
            .env("GIT_CONFIG_KEY_1", "http.extraHeader")
            .env(
                "GIT_CONFIG_VALUE_1",
                format!("Authorization: Basic {basic}"),
            );
    }
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }

    let output = tokio::time::timeout(timeout, command.output())
        .await
        .map_err(|_| GitCloneError::Timeout(timeout))??;
    if !output.status.success() {
        let stderr = redact_url_credentials(&String::from_utf8_lossy(&output.stderr));
        return Err(GitCloneError::Git {
            command: format!("git {}", args.join(" ")),
            status: output.status.to_string(),
            diagnostic: diagnose_git_failure(&stderr, auth, args),
            stderr,
        });
    }
    Ok(output)
}

fn diagnose_git_failure(
    stderr: &str,
    auth: Option<&GitAuth>,
    args: &[&str],
) -> GitFailureDiagnostic {
    let lower = stderr.to_ascii_lowercase();
    let target = args.iter().find_map(|arg| url::Url::parse(arg).ok());
    let target_host = target
        .as_ref()
        .and_then(|url| url.host_str().map(str::to_string));
    let target_repository = target.as_ref().map(|url| {
        url.path_segments()
            .map(|segments| segments.collect::<Vec<_>>().join("/"))
            .unwrap_or_default()
            .trim_end_matches(".git")
            .to_string()
    });
    let is_github = target_host
        .as_deref()
        .map(|host| host.eq_ignore_ascii_case("github.com") || host.ends_with(".github.com"))
        .unwrap_or(false);
    let token_type = auth.map(|auth| classify_token(&auth.secret).to_string());
    let token_had_surrounding_whitespace = auth
        .map(|auth| auth.secret != auth.secret.trim())
        .unwrap_or(false);
    let auth_username = auth.map(|auth| auth.username.clone());

    let (category, summary, mut likely_causes, mut recommended_actions) = if lower
        .contains("write access to repository not granted")
        || lower.contains("the requested url returned error: 403")
        || lower.contains("requested url returned error: 403")
    {
        let target_name = target_repository.as_deref().unwrap_or("the repository");
        (
            "git_authorization_denied".to_string(),
            format!(
                "{} returned HTTP 403 while cloning `{target_name}`. Despite the `Write access` wording, indexing only needs read access; this usually means the configured token cannot see or read this repository.",
                target_host.as_deref().unwrap_or("The git host")
            ),
            vec![
                "the token does not grant read access to this exact repository".to_string(),
                "the repository owner/resource owner selected for the token does not match this repository".to_string(),
                "organization SAML SSO or token restrictions were not authorized for the token".to_string(),
                "the repository was renamed, deleted, made private, or the token owner lost access".to_string(),
            ],
            vec![
                "confirm that the repository URL is correct and still exists".to_string(),
                "for a fine-grained GitHub PAT, ensure Repository access includes this repository and Repository permissions → Contents is at least Read-only".to_string(),
                "confirm the token resource owner is the user or organization that owns the repository".to_string(),
                "if the repository belongs to an organization with SAML SSO, authorize the token for that organization or use a supported GitHub App/classic token".to_string(),
                "update the stored provider credential in GraphFinder and retry the index job".to_string(),
            ],
        )
    } else if lower.contains("authentication failed")
        || lower.contains("invalid username or password")
        || lower.contains("the requested url returned error: 401")
        || lower.contains("requested url returned error: 401")
    {
        (
            "git_authentication_failed".to_string(),
            format!(
                "{} rejected the supplied credentials while cloning. This indicates an invalid, expired, revoked, or malformed token rather than a repository permission problem.",
                target_host.as_deref().unwrap_or("The git host")
            ),
            vec![
                "the token expired or was revoked".to_string(),
                "the token was copied incompletely or with surrounding whitespace".to_string(),
                "the wrong credential was attached to this source".to_string(),
            ],
            vec![
                "create or copy the token again and save it in GraphFinder without whitespace".to_string(),
                "confirm this git source references the intended provider credential".to_string(),
                "retry the job after updating the provider credential".to_string(),
            ],
        )
    } else if lower.contains("remote branch") && lower.contains("not found in upstream origin") {
        let branch = args
            .windows(2)
            .find(|pair| pair[0] == "--branch")
            .map(|pair| pair[1].to_string())
            .unwrap_or_else(|| "the configured branch".to_string());
        (
            "git_branch_not_found".to_string(),
            format!(
                "The repository was reached, but `{branch}` does not exist as a remote branch. This is a branch configuration problem, not a token permission problem."
            ),
            vec![
                "the source branch in GraphFinder does not match a branch in the repository".to_string(),
                "the repository may use another default branch name, such as `master` instead of `main`".to_string(),
            ],
            vec![
                "check the repository branches in the git host".to_string(),
                "update the GraphFinder source branch or leave it empty to use the repository default branch".to_string(),
                "retry the index job after correcting the branch".to_string(),
            ],
        )
    } else if lower.contains("repository not found")
        || lower.contains("the requested url returned error: 404")
        || lower.contains("requested url returned error: 404")
    {
        (
            "git_repository_not_found_or_invisible".to_string(),
            format!(
                "{} returned repository-not-found. For private repositories this often means the token cannot see the repository, not necessarily that it does not exist.",
                target_host.as_deref().unwrap_or("The git host")
            ),
            vec![
                "the repository URL is incorrect or the repository was renamed".to_string(),
                "the token owner cannot access the repository".to_string(),
                "a fine-grained token does not include this repository in Repository access".to_string(),
            ],
            vec![
                "verify the clone URL in the git host UI".to_string(),
                "verify the token owner can open the repository in a browser".to_string(),
                "verify repository selection and Contents read permission on the token".to_string(),
            ],
        )
    } else if lower.contains("could not resolve host")
        || lower.contains("failed to connect")
        || lower.contains("operation timed out")
        || lower.contains("connection timed out")
    {
        (
            "git_network_error".to_string(),
            format!(
                "GraphFinder could not reach {}. This is a network/DNS/connectivity failure before authentication was evaluated.",
                target_host.as_deref().unwrap_or("the git host")
            ),
            vec![
                "DNS or outbound network access is unavailable from the GraphFinder worker".to_string(),
                "a proxy or firewall is blocking HTTPS access to the git host".to_string(),
            ],
            vec![
                "verify outbound HTTPS connectivity from the GraphFinder server/container".to_string(),
                "check proxy and firewall configuration, then retry".to_string(),
            ],
        )
    } else {
        (
            "git_command_failed".to_string(),
            "The git command failed, but the stderr did not match a known authentication, authorization, repository visibility, or network pattern.".to_string(),
            vec!["the repository state, branch, host configuration, or git output may require manual inspection".to_string()],
            vec!["inspect the technical details below and retry after correcting the underlying git/host issue".to_string()],
        )
    };

    if token_had_surrounding_whitespace {
        likely_causes.insert(
            0,
            "the stored token contains leading or trailing whitespace, which makes the Authorization header invalid".to_string(),
        );
        recommended_actions.insert(
            0,
            "save the token again without spaces or a trailing newline".to_string(),
        );
    }

    if is_github {
        if let (Some(token_type), Some(username)) =
            (token_type.as_deref(), auth_username.as_deref())
        {
            if token_type == "fine-grained GitHub PAT"
                && username.eq_ignore_ascii_case("x-access-token")
            {
                likely_causes.insert(
                    0,
                    "the credential uses username `x-access-token` with a fine-grained PAT; fine-grained PATs are commonly cloned with username `oauth2`".to_string(),
                );
                recommended_actions.insert(
                    0,
                    "update the GitHub provider username to `oauth2` for fine-grained PATs and retry".to_string(),
                );
            }
        }
    }

    GitFailureDiagnostic {
        category,
        summary,
        auth_configured: auth.is_some(),
        auth_username,
        token_type,
        token_had_surrounding_whitespace,
        target_host,
        target_repository,
        likely_causes,
        recommended_actions,
    }
}

fn classify_token(secret: &str) -> &'static str {
    if secret.starts_with("github_pat_") {
        "fine-grained GitHub PAT"
    } else if secret.starts_with("ghp_") {
        "classic GitHub PAT"
    } else if secret.starts_with("gho_") {
        "GitHub OAuth token"
    } else if secret.starts_with("ghs_") {
        "GitHub App server/installation token"
    } else if secret.starts_with("ghu_") {
        "GitHub App user token"
    } else {
        "unknown/custom token"
    }
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
            auth: None,
        };
        let checkout = clone_or_fetch_public_repo(&url, &options).await.unwrap();
        assert_eq!(checkout.commit_sha.len(), 40);
        assert!(checkout.path.join("README.md").exists());
        let second = clone_or_fetch_public_repo(&url, &options).await.unwrap();
        assert_eq!(checkout.commit_sha, second.commit_sha);
    }

    #[test]
    fn explains_github_403_as_read_authorization_problem() {
        let auth = GitAuth {
            username: "x-access-token".to_string(),
            secret: "github_pat_test".to_string(),
        };
        let diagnostic = diagnose_git_failure(
            "remote: Write access to repository not granted.\nfatal: unable to access 'https://github.com/acme/private.git/': The requested URL returned error: 403",
            Some(&auth),
            &[
                "clone",
                "--depth",
                "1",
                "https://github.com/acme/private.git",
                "target",
            ],
        );

        assert_eq!(diagnostic.category, "git_authorization_denied");
        assert_eq!(diagnostic.target_host.as_deref(), Some("github.com"));
        assert_eq!(
            diagnostic.target_repository.as_deref(),
            Some("acme/private")
        );
        assert!(diagnostic
            .summary
            .contains("indexing only needs read access"));
        assert!(diagnostic
            .recommended_actions
            .iter()
            .any(|action| action.contains("Contents is at least Read-only")));
        assert!(diagnostic
            .likely_causes
            .iter()
            .any(|cause| cause.contains("x-access-token")));
    }

    #[test]
    fn detects_token_surrounding_whitespace() {
        let auth = GitAuth {
            username: "oauth2".to_string(),
            secret: "github_pat_test\n".to_string(),
        };
        let diagnostic = diagnose_git_failure(
            "fatal: Authentication failed for 'https://github.com/acme/private.git/'",
            Some(&auth),
            &["clone", "https://github.com/acme/private.git", "target"],
        );

        assert_eq!(diagnostic.category, "git_authentication_failed");
        assert!(diagnostic.token_had_surrounding_whitespace);
        assert!(diagnostic
            .likely_causes
            .iter()
            .any(|cause| cause.contains("whitespace")));
    }

    #[test]
    fn explains_missing_remote_branch_separately_from_auth() {
        let diagnostic = diagnose_git_failure(
            "fatal: Remote branch main not found in upstream origin",
            None,
            &[
                "clone",
                "--branch",
                "main",
                "https://github.com/acme/private.git",
                "target",
            ],
        );

        assert_eq!(diagnostic.category, "git_branch_not_found");
        assert!(diagnostic.summary.contains("`main` does not exist"));
        assert!(diagnostic
            .recommended_actions
            .iter()
            .any(|action| action.contains("leave it empty")));
    }

    #[test]
    fn explains_404_as_possible_private_repository_invisibility() {
        let diagnostic = diagnose_git_failure(
            "remote: Repository not found.\nfatal: repository 'https://github.com/acme/private.git/' not found",
            None,
            &["clone", "https://github.com/acme/private.git", "target"],
        );

        assert_eq!(diagnostic.category, "git_repository_not_found_or_invisible");
        assert!(!diagnostic.auth_configured);
        assert!(diagnostic
            .summary
            .contains("token cannot see the repository"));
    }
}
