use std::sync::{OnceLock, RwLock};
use std::time::{Duration, Instant};

use serde::Serialize;

const CACHE_TTL: Duration = Duration::from_secs(30 * 60);
const LATEST_RELEASE_API_URL: &str =
    "https://api.github.com/repos/GabrielTBeserra/GraphFinder/releases/latest";
const RELEASES_PAGE_URL: &str = "https://github.com/GabrielTBeserra/GraphFinder/releases";

#[derive(Debug, Clone, Serialize)]
pub struct UpdateStatusResponse {
    pub current_version: String,
    pub latest_version: Option<String>,
    pub tag_name: Option<String>,
    pub name: Option<String>,
    pub has_update: bool,
    pub severity: Option<String>,
    pub release_url: Option<String>,
    pub release_notes: Option<String>,
    pub published_at: Option<String>,
    pub checked_at: String,
    pub check_failed: bool,
    pub channel: String,
}

struct CacheEntry {
    fetched_at: Instant,
    status: UpdateStatusResponse,
}

static CACHE: OnceLock<RwLock<Option<CacheEntry>>> = OnceLock::new();

fn cache() -> &'static RwLock<Option<CacheEntry>> {
    CACHE.get_or_init(|| RwLock::new(None))
}

pub fn current_version() -> String {
    std::env::var("GF_VERSION")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
}

fn parse_semver(version: &str) -> (u64, u64, u64) {
    let cleaned = version.trim().trim_start_matches('v').split('-').next().unwrap_or("");
    let mut parts = cleaned.split('.');
    let parse_part = |part: Option<&str>| part.and_then(|p| p.parse::<u64>().ok()).unwrap_or(0);
    (
        parse_part(parts.next()),
        parse_part(parts.next()),
        parse_part(parts.next()),
    )
}

fn compare_semver(a: &str, b: &str) -> std::cmp::Ordering {
    parse_semver(a).cmp(&parse_semver(b))
}

fn severity_for(current: &str, latest: &str, name: &str, body: &str) -> String {
    let haystack = format!("{} {}", name.to_lowercase(), body.to_lowercase());
    if haystack.contains("security") || haystack.contains("cve") || haystack.contains("vulnerab") {
        return "security".into();
    }
    let (cmaj, cmin, _) = parse_semver(current);
    let (lmaj, lmin, _) = parse_semver(latest);
    if lmaj != cmaj {
        "major".into()
    } else if lmin != cmin {
        "minor".into()
    } else {
        "patch".into()
    }
}

fn failure_status() -> UpdateStatusResponse {
    UpdateStatusResponse {
        current_version: current_version(),
        latest_version: None,
        tag_name: None,
        name: None,
        has_update: false,
        severity: None,
        release_url: Some(RELEASES_PAGE_URL.into()),
        release_notes: None,
        published_at: None,
        checked_at: chrono::Utc::now().to_rfc3339(),
        check_failed: true,
        channel: "stable".into(),
    }
}

#[derive(serde::Deserialize)]
struct GithubRelease {
    tag_name: Option<String>,
    name: Option<String>,
    body: Option<String>,
    html_url: Option<String>,
    published_at: Option<String>,
    draft: Option<bool>,
    prerelease: Option<bool>,
}

/// Returns the update status, using a 30-minute in-process cache so the panel
/// never hammers the GitHub API. Network or API failures degrade to
/// `check_failed = true` instead of erroring the request — update awareness
/// must never take the control plane down.
pub async fn update_status(force: bool) -> UpdateStatusResponse {
    if !force {
        if let Ok(guard) = cache().read() {
            if let Some(entry) = guard.as_ref() {
                if entry.fetched_at.elapsed() < CACHE_TTL {
                    return entry.status.clone();
                }
            }
        }
    }

    match fetch_latest_release().await {
        Ok(status) => {
            if let Ok(mut guard) = cache().write() {
                *guard = Some(CacheEntry {
                    fetched_at: Instant::now(),
                    status: status.clone(),
                });
            }
            status
        }
        Err(err) => {
            tracing::warn!("update check failed: {err}");
            if let Ok(guard) = cache().read() {
                if let Some(entry) = guard.as_ref() {
                    let mut stale = entry.status.clone();
                    stale.check_failed = true;
                    return stale;
                }
            }
            failure_status()
        }
    }
}

async fn fetch_latest_release() -> Result<UpdateStatusResponse, String> {
    let client = reqwest::Client::builder()
        .user_agent("graphfinder-server/update-check")
        .timeout(Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(LATEST_RELEASE_API_URL)
        .header("accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("github api returned {}", response.status()));
    }

    let release: GithubRelease = response.json().await.map_err(|e| e.to_string())?;
    if release.draft.unwrap_or(false) || release.prerelease.unwrap_or(false) {
        return Err("latest release is a draft or prerelease".into());
    }

    let current = current_version();
    let tag = release.tag_name.clone().unwrap_or_default();
    let latest = tag.trim_start_matches('v').to_string();
    let has_update = !latest.is_empty() && compare_semver(&latest, &current) == std::cmp::Ordering::Greater;
    let name = release.name.clone().unwrap_or_else(|| tag.clone());
    let body = release.body.clone().unwrap_or_default();

    Ok(UpdateStatusResponse {
        current_version: current.clone(),
        latest_version: if latest.is_empty() { None } else { Some(latest.clone()) },
        tag_name: if tag.is_empty() { None } else { Some(tag) },
        name: Some(name.clone()),
        has_update,
        severity: if has_update {
            Some(severity_for(&current, &latest, &name, &body))
        } else {
            None
        },
        release_url: Some(release.html_url.unwrap_or_else(|| RELEASES_PAGE_URL.into())),
        release_notes: if body.is_empty() { None } else { Some(body) },
        published_at: release.published_at,
        checked_at: chrono::Utc::now().to_rfc3339(),
        check_failed: false,
        channel: "stable".into(),
    })
}
