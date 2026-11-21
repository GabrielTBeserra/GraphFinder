// User-facing application preferences, persisted to `settings.json` inside
// `base_dir` so the GUI, the tray behaviour and Windows autostart stay in sync.

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    /// Launch GraphFinder automatically when the user signs in to Windows.
    pub autostart: bool,
    /// Closing the window hides it to the system tray instead of quitting.
    pub close_to_tray: bool,
    /// When autostarting, begin hidden in the tray (no window pop-up).
    pub start_minimized: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            autostart: false,
            close_to_tray: true,
            start_minimized: false,
        }
    }
}

pub fn settings_path(base_dir: &Path) -> PathBuf {
    base_dir.join("settings.json")
}

pub fn load(base_dir: &Path) -> AppSettings {
    fs::read_to_string(settings_path(base_dir))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

pub fn save(base_dir: &Path, settings: &AppSettings) {
    if let Ok(s) = serde_json::to_string_pretty(settings) {
        let _ = fs::write(settings_path(base_dir), s);
    }
}
