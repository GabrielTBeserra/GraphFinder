// Single-instance coordination between the GraphFinder GUI, the CLI and
// headless processes. The design uses two artifacts inside `base_dir`:
//
//  - `graphfinder.lock`: an OS-level exclusive file lock (fs2). Whoever holds
//    it is the only process allowed to run vault-touching work locally.
//    Released automatically by the OS when the owner dies — no stale locks.
//  - `instance.json`: metadata written by the GUI ({role:"gui", port, pid}).
//    CLI commands probe `GET /health` on that port; when a GUI is alive the
//    command is delegated over `POST /api/cli` so it executes inside the
//    running instance and its results are reflected in the interface.

use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::fs::{self, File, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub pid: u32,
    pub role: String,
    #[serde(default)]
    pub port: Option<u16>,
    #[serde(default)]
    pub started_at: u64,
}

#[derive(Debug, Serialize)]
pub struct CliProxyResponse {
    pub code: i32,
    pub stdout: String,
    #[serde(default)]
    pub stderr: String,
}

#[derive(Debug, Deserialize)]
struct CliProxyResponseIn {
    code: i32,
    #[serde(default)]
    stdout: String,
    #[serde(default)]
    stderr: String,
}

pub fn lock_path(base_dir: &Path) -> PathBuf {
    base_dir.join("graphfinder.lock")
}

pub fn info_path(base_dir: &Path) -> PathBuf {
    base_dir.join("instance.json")
}

/// Try to take the exclusive instance lock. Returns the open file handle —
/// the caller MUST keep it alive for the whole process lifetime, or the lock
/// is released early.
pub fn try_acquire_lock(base_dir: &Path) -> Option<File> {
    let path = lock_path(base_dir);
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(&path)
        .ok()?;
    file.try_lock_exclusive().ok()?;
    Some(file)
}

pub fn write_info(base_dir: &Path, role: &str, port: Option<u16>) {
    let info = InstanceInfo {
        pid: std::process::id(),
        role: role.to_string(),
        port,
        started_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0),
    };
    if let Ok(s) = serde_json::to_string(&info) {
        let _ = fs::write(info_path(base_dir), s);
    }
}

pub fn read_info(base_dir: &Path) -> Option<InstanceInfo> {
    let raw = fs::read_to_string(info_path(base_dir)).ok()?;
    serde_json::from_str(&raw).ok()
}

pub fn remove_info(base_dir: &Path) {
    let _ = fs::remove_file(info_path(base_dir));
}

/// Returns the HTTP port of a live GUI instance, if `instance.json` points at
/// one whose `/health` endpoint answers.
pub fn probe_gui(base_dir: &Path) -> Option<u16> {
    let info = read_info(base_dir)?;
    if info.role != "gui" {
        return None;
    }
    let port = info.port?;
    let url = format!("http://127.0.0.1:{}/health", port);
    match ureq::get(&url)
        .timeout(Duration::from_millis(800))
        .call()
    {
        Ok(resp) if resp.status() == 200 => Some(port),
        _ => None,
    }
}

/// Forward a CLI invocation to the running GUI so it executes in-process
/// (same VaultManager, same engines, same event pipeline as the interface).
pub fn delegate_to_gui(
    port: u16,
    args: &[String],
    cwd: &Path,
) -> Result<CliProxyResponse, String> {
    let url = format!("http://127.0.0.1:{}/api/cli", port);
    let body = serde_json::json!({
        "args": args,
        "cwd": cwd.to_string_lossy(),
    });

    let mut req = ureq::post(&url);
    if let Ok(token) = std::env::var("GRAPHFINDER_MCP_TOKEN") {
        if !token.trim().is_empty() {
            req = req.set("Authorization", &format!("Bearer {}", token.trim()));
        }
    }

    match req.send_json(body) {
        Ok(resp) => resp
            .into_json::<CliProxyResponseIn>()
            .map(|r| CliProxyResponse {
                code: r.code,
                stdout: r.stdout,
                stderr: r.stderr,
            })
            .map_err(|e| format!("invalid delegation response: {}", e)),
        Err(ureq::Error::Status(code, resp)) => {
            let detail = resp.into_string().unwrap_or_default();
            Err(format!("delegation rejected (HTTP {}): {}", code, detail))
        }
        Err(e) => Err(format!("delegation failed: {}", e)),
    }
}
