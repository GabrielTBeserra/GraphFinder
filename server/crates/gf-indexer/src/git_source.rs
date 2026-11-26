use std::net::{IpAddr, ToSocketAddrs};

use thiserror::Error;
use url::Url;

#[derive(Debug, Clone, Default)]
pub struct GitSourcePolicy {
    pub allow_private_hosts: bool,
    pub allowed_hosts: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedGitUrl {
    pub normalized_url: String,
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Error)]
pub enum GitSourceValidationError {
    #[error("invalid git URL: {0}")]
    InvalidUrl(#[from] url::ParseError),
    #[error("only https git URLs are supported")]
    UnsupportedScheme,
    #[error("git URL must include a host")]
    MissingHost,
    #[error("credentials embedded in git URLs are not allowed")]
    EmbeddedCredentials,
    #[error("query strings and fragments are not allowed in git URLs")]
    UnexpectedUrlParts,
    #[error("host {0} is private or internal; enable explicit internal access to use it")]
    PrivateHost(String),
    #[error("host {0} could not be resolved safely")]
    UnresolvableHost(String),
}

pub fn validate_git_source_url(
    input: &str,
    policy: &GitSourcePolicy,
) -> Result<ValidatedGitUrl, GitSourceValidationError> {
    let url = Url::parse(input)?;

    if url.scheme() != "https" {
        return Err(GitSourceValidationError::UnsupportedScheme);
    }
    if !url.username().is_empty() || url.password().is_some() {
        return Err(GitSourceValidationError::EmbeddedCredentials);
    }
    if url.query().is_some() || url.fragment().is_some() {
        return Err(GitSourceValidationError::UnexpectedUrlParts);
    }

    let host = url
        .host_str()
        .ok_or(GitSourceValidationError::MissingHost)?
        .trim_end_matches('.')
        .to_ascii_lowercase();
    let port = url.port_or_known_default().unwrap_or(443);
    let explicitly_allowed = policy
        .allowed_hosts
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(&host));

    if !explicitly_allowed {
        if let Ok(ip) = host.parse::<IpAddr>() {
            if !policy.allow_private_hosts && !is_public_ip(ip) {
                return Err(GitSourceValidationError::PrivateHost(host));
            }
        } else {
            let addresses = (host.as_str(), port)
                .to_socket_addrs()
                .map_err(|_| GitSourceValidationError::UnresolvableHost(host.clone()))?
                .collect::<Vec<_>>();
            if addresses.is_empty() {
                return Err(GitSourceValidationError::UnresolvableHost(host));
            }
            if !policy.allow_private_hosts
                && addresses.iter().any(|address| !is_public_ip(address.ip()))
            {
                return Err(GitSourceValidationError::PrivateHost(host));
            }
        }
    }

    Ok(ValidatedGitUrl {
        normalized_url: url.to_string(),
        host,
        port,
    })
}

fn is_public_ip(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(ip) => {
            let octets = ip.octets();
            !(octets[0] == 0
                || octets[0] == 10
                || octets[0] == 127
                || (octets[0] == 100 && (64..=127).contains(&octets[1]))
                || octets[0] == 169 && octets[1] == 254
                || octets[0] == 172 && (16..=31).contains(&octets[1])
                || octets[0] == 192 && octets[1] == 168
                || octets[0] == 192 && octets[1] == 0
                || octets[0] == 192 && octets[1] == 0 && octets[2] == 2
                || octets[0] == 198 && (18..=19).contains(&octets[1])
                || octets[0] == 198 && octets[1] == 51 && octets[2] == 100
                || octets[0] == 203 && octets[1] == 0 && octets[2] == 113
                || octets[0] >= 224)
        }
        IpAddr::V6(ip) => {
            let segments = ip.segments();
            !(ip.is_loopback()
                || ip.is_unspecified()
                || (segments[0] & 0xffc0) == 0xfe80
                || (segments[0] & 0xfe00) == 0xfc00
                || segments[0] == 0x2001 && segments[1] == 0xdb8)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_https_and_embedded_credentials() {
        assert!(matches!(
            validate_git_source_url("ssh://git@example.com/repo.git", &Default::default()),
            Err(GitSourceValidationError::UnsupportedScheme)
        ));
        assert!(matches!(
            validate_git_source_url(
                "https://user:pass@example.com/repo.git",
                &Default::default()
            ),
            Err(GitSourceValidationError::EmbeddedCredentials)
        ));
    }

    #[test]
    fn rejects_private_literals_and_localhost() {
        for url in [
            "https://127.0.0.1/repo.git",
            "https://10.0.0.2/repo.git",
            "https://192.168.1.2/repo.git",
            "https://localhost/repo.git",
        ] {
            assert!(matches!(
                validate_git_source_url(url, &Default::default()),
                Err(GitSourceValidationError::PrivateHost(_))
                    | Err(GitSourceValidationError::UnresolvableHost(_))
            ));
        }
    }

    #[test]
    fn explicit_policy_can_allow_internal_hosts() {
        let policy = GitSourcePolicy {
            allow_private_hosts: true,
            allowed_hosts: vec![],
        };
        assert!(validate_git_source_url("https://192.168.1.2/repo.git", &policy).is_ok());
    }
}
