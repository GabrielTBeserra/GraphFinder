use std::collections::HashMap;
use std::path::Path;

use globset::{Glob, GlobSet, GlobSetBuilder};
use sha2::{Digest, Sha256};
use thiserror::Error;
use walkdir::WalkDir;

const DEFAULT_IGNORED_DIRS: &[&str] = &[".git", "node_modules", "target", "dist", "build", ".venv"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScannedFile {
    pub relative_path: String,
    pub size_bytes: u64,
    pub content_hash: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ScanDiff {
    pub added: Vec<String>,
    pub changed: Vec<String>,
    pub unchanged: Vec<String>,
    pub removed: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ScanError {
    #[error("invalid glob pattern `{pattern}`: {source}")]
    InvalidGlob {
        pattern: String,
        source: globset::Error,
    },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("filesystem walk error: {0}")]
    Walk(#[from] walkdir::Error),
}

pub fn scan_repository(
    root: &Path,
    include_patterns: &[String],
    exclude_patterns: &[String],
) -> Result<Vec<ScannedFile>, ScanError> {
    let includes = build_globset(include_patterns, true)?;
    let excludes = build_globset(exclude_patterns, false)?;
    let mut files = Vec::new();

    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        let path = entry.path();
        if entry.file_type().is_symlink() {
            continue;
        }
        if entry.file_type().is_dir() {
            continue;
        }
        let relative = path.strip_prefix(root).unwrap_or(path);
        if has_ignored_component(relative) {
            continue;
        }
        let relative = normalize_path(relative);
        if !includes.is_match(&relative) || excludes.is_match(&relative) {
            continue;
        }
        let bytes = std::fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        files.push(ScannedFile {
            relative_path: relative,
            size_bytes: bytes.len() as u64,
            content_hash: format!("{:x}", hasher.finalize()),
        });
    }
    files.sort_by(|a, b| a.relative_path.cmp(&b.relative_path));
    Ok(files)
}

pub fn diff_scan(current: &[ScannedFile], previous: &[ScannedFile]) -> ScanDiff {
    let current = current
        .iter()
        .map(|file| (file.relative_path.as_str(), file.content_hash.as_str()))
        .collect::<HashMap<_, _>>();
    let previous = previous
        .iter()
        .map(|file| (file.relative_path.as_str(), file.content_hash.as_str()))
        .collect::<HashMap<_, _>>();
    let mut diff = ScanDiff::default();

    for (path, hash) in &current {
        match previous.get(path) {
            None => diff.added.push((*path).to_string()),
            Some(old_hash) if old_hash != hash => diff.changed.push((*path).to_string()),
            _ => diff.unchanged.push((*path).to_string()),
        }
    }
    for path in previous.keys() {
        if !current.contains_key(path) {
            diff.removed.push((*path).to_string());
        }
    }
    diff.added.sort();
    diff.changed.sort();
    diff.unchanged.sort();
    diff.removed.sort();
    diff
}

fn build_globset(patterns: &[String], default_all: bool) -> Result<GlobSet, ScanError> {
    let mut builder = GlobSetBuilder::new();
    if patterns.is_empty() && default_all {
        builder.add(Glob::new("**/*").map_err(|source| ScanError::InvalidGlob {
            pattern: "**/*".into(),
            source,
        })?);
    }
    for pattern in patterns {
        builder.add(Glob::new(pattern).map_err(|source| ScanError::InvalidGlob {
            pattern: pattern.clone(),
            source,
        })?);
    }
    Ok(builder.build().expect("globset construction"))
}

fn has_ignored_component(path: &Path) -> bool {
    path.components().any(|component| {
        DEFAULT_IGNORED_DIRS.contains(&component.as_os_str().to_string_lossy().as_ref())
    })
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scans_with_includes_excludes_and_hashes() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("docs")).unwrap();
        std::fs::create_dir_all(temp.path().join("node_modules/pkg")).unwrap();
        std::fs::write(temp.path().join("docs/a.md"), "hello").unwrap();
        std::fs::write(temp.path().join("docs/b.txt"), "skip").unwrap();
        std::fs::write(temp.path().join("node_modules/pkg/x.js"), "skip").unwrap();

        let files =
            scan_repository(temp.path(), &["docs/**".into()], &["**/*.txt".into()]).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].relative_path, "docs/a.md");
        assert_eq!(files[0].content_hash.len(), 64);
    }

    #[test]
    fn diffs_incremental_file_states() {
        let previous = vec![
            ScannedFile {
                relative_path: "same".into(),
                size_bytes: 1,
                content_hash: "a".into(),
            },
            ScannedFile {
                relative_path: "changed".into(),
                size_bytes: 1,
                content_hash: "b".into(),
            },
            ScannedFile {
                relative_path: "removed".into(),
                size_bytes: 1,
                content_hash: "c".into(),
            },
        ];
        let current = vec![
            ScannedFile {
                relative_path: "same".into(),
                size_bytes: 1,
                content_hash: "a".into(),
            },
            ScannedFile {
                relative_path: "changed".into(),
                size_bytes: 1,
                content_hash: "new".into(),
            },
            ScannedFile {
                relative_path: "added".into(),
                size_bytes: 1,
                content_hash: "d".into(),
            },
        ];
        let diff = diff_scan(&current, &previous);
        assert_eq!(diff.added, ["added"]);
        assert_eq!(diff.changed, ["changed"]);
        assert_eq!(diff.unchanged, ["same"]);
        assert_eq!(diff.removed, ["removed"]);
    }
}
