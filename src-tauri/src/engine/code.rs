//! Multi-language code parsing, import extraction, path resolution and semantic chunking.
//!
//! Supports JavaScript, TypeScript, Python, Rust, Go, C/C++, Java, Kotlin, C#, PHP, Ruby, Swift, and more.

use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::LazyLock;

/// Represents an import or module reference extracted from source code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedImport {
    pub raw: String,
    pub target: String,
    pub is_relative: bool,
}

/// A semantic chunk of code (function, class, struct, or windowed block).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeChunk {
    pub chunk_index: usize,
    pub symbol_name: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub token_estimate: usize,
}

/// Parsed metadata and chunks of a source code file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCodeFile {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub language: String,
    pub size_bytes: usize,
    pub imports: Vec<ExtractedImport>,
    pub chunks: Vec<CodeChunk>,
    pub summary: String,
}

// ==========================================
// Language and Extension Detection
// ==========================================

pub fn get_language_from_ext(ext: &str) -> Option<&'static str> {
    match ext.to_lowercase().as_str() {
        "ts" | "mts" | "cts" => Some("typescript"),
        "tsx" => Some("tsx"),
        "js" | "mjs" | "cjs" => Some("javascript"),
        "jsx" => Some("jsx"),
        "py" | "pyi" => Some("python"),
        "rs" => Some("rust"),
        "go" => Some("go"),
        "c" | "h" => Some("c"),
        "cpp" | "hpp" | "cc" | "cxx" | "hh" => Some("cpp"),
        "java" => Some("java"),
        "kt" | "kts" => Some("kotlin"),
        "cs" => Some("csharp"),
        "fs" => Some("fsharp"),
        "php" => Some("php"),
        "rb" => Some("ruby"),
        "swift" => Some("swift"),
        "html" | "htm" => Some("html"),
        "css" | "scss" | "sass" | "less" => Some("css"),
        "json" => Some("json"),
        "yaml" | "yml" => Some("yaml"),
        "toml" => Some("toml"),
        "sql" => Some("sql"),
        "sh" | "bash" | "zsh" => Some("shell"),
        _ => None,
    }
}

pub fn is_supported_code_file(path: &Path) -> bool {
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        get_language_from_ext(ext).is_some()
    } else {
        false
    }
}

// ==========================================
// Regular Expressions for Import Extraction
// ==========================================

// JS/TS: import ... from '...'; export ... from '...'; require('...'); import('...')
static JS_IMPORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)(?:import|export)\s+(?:(?:[\w*\s{},]+)\s+from\s+)?['"]([^'"]+)['"]|(?:require|import)\s*\(\s*['"]([^'"]+)['"]\s*\)"#)
        .expect("valid regex for js imports")
});

// Python: from foo.bar import ...; import foo.bar
static PY_IMPORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^\s*(?:from\s+([.\w]+)\s+import|import\s+([.\w]+))"#)
        .expect("valid regex for python imports")
});

// Rust: mod foo; pub mod foo; use crate::foo::bar; use super::foo;
static RS_MOD_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^\s*(?:pub\s+)?mod\s+([a-zA-Z0-9_]+)\s*;"#)
        .expect("valid regex for rust mod")
});

static RS_USE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^\s*(?:pub\s+)?use\s+([a-zA-Z0-9_:]+)"#)
        .expect("valid regex for rust use")
});

// Go: import "foo/bar" or inside import ( "foo" \n "bar" )
static GO_SINGLE_IMPORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)import\s+(?:[a-zA-Z0-9_.]+\s+)?["']([^"']+)["']"#)
        .expect("valid regex for go single import")
});

static GO_BLOCK_IMPORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?s)import\s*\((.*?)\)"#)
        .expect("valid regex for go import block")
});

static GO_ITEM_IN_BLOCK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"["']([^"']+)["']"#)
        .expect("valid regex for go item")
});

// C/C++: #include "foo.h" or #include <foo.h>
static C_INCLUDE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^\s*#\s*include\s*[<"]([^>"]+)[>"]"#)
        .expect("valid regex for c include")
});

// Java/Kotlin/C#: import foo.bar.Baz; package foo.bar; using Foo.Bar;
static JAVA_KT_IMPORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^\s*import\s+(?:static\s+)?([a-zA-Z0-9_.]+)"#)
        .expect("valid regex for java/kt import")
});

static CSHARP_USING_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^\s*using\s+(?:static\s+)?([a-zA-Z0-9_.]+)\s*;"#)
        .expect("valid regex for csharp using")
});

// PHP: require 'foo.php'; include 'foo.php'; use Foo\Bar;
static PHP_IMPORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)(?:require|require_once|include|include_once)\s*\(?['"]([^'"]+)['"]\)?|^\s*use\s+([a-zA-Z0-9_\\]+)\s*;"#)
        .expect("valid regex for php import")
});

// Ruby: require 'foo'; require_relative 'foo'
static RUBY_IMPORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^\s*(?:require|require_relative)\s*['"]([^'"]+)['"]"#)
        .expect("valid regex for ruby import")
});

// Swift: import Foo
static SWIFT_IMPORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^\s*import\s+([a-zA-Z0-9_]+)"#)
        .expect("valid regex for swift import")
});

// Web/CSS: @import "foo.css"; or @import url("foo.css");
static CSS_IMPORT_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)@import\s+(?:url\()?['"]([^'"]+)['"]"#)
        .expect("valid regex for css import")
});

// ==========================================
// Import Extraction Function
// ==========================================

pub fn extract_imports(language: &str, content: &str) -> Vec<ExtractedImport> {
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    match language {
        "typescript" | "tsx" | "javascript" | "jsx" => {
            for cap in JS_IMPORT_RE.captures_iter(content) {
                let target = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str());
                if let Some(t) = target {
                    let clean = t.trim();
                    if !clean.is_empty() && seen.insert(clean.to_string()) {
                        results.push(ExtractedImport {
                            raw: cap[0].to_string(),
                            target: clean.to_string(),
                            is_relative: clean.starts_with('.') || clean.starts_with('/'),
                        });
                    }
                }
            }
        }
        "python" => {
            for cap in PY_IMPORT_RE.captures_iter(content) {
                let target = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str());
                if let Some(t) = target {
                    let clean = t.trim();
                    if !clean.is_empty() && seen.insert(clean.to_string()) {
                        results.push(ExtractedImport {
                            raw: cap[0].to_string(),
                            target: clean.to_string(),
                            is_relative: clean.starts_with('.'),
                        });
                    }
                }
            }
        }
        "rust" => {
            // mod foo;
            for cap in RS_MOD_RE.captures_iter(content) {
                if let Some(m) = cap.get(1) {
                    let name = m.as_str().trim();
                    if seen.insert(format!("mod:{}", name)) {
                        results.push(ExtractedImport {
                            raw: cap[0].to_string(),
                            target: name.to_string(),
                            is_relative: true,
                        });
                    }
                }
            }
            // use crate::... or use super::...
            for cap in RS_USE_RE.captures_iter(content) {
                if let Some(m) = cap.get(1) {
                    let full_path = m.as_str().trim();
                    if seen.insert(full_path.to_string()) {
                        let is_rel = full_path.starts_with("crate::") || full_path.starts_with("super::");
                        results.push(ExtractedImport {
                            raw: cap[0].to_string(),
                            target: full_path.to_string(),
                            is_relative: is_rel,
                        });
                    }
                }
            }
        }
        "go" => {
            for cap in GO_SINGLE_IMPORT_RE.captures_iter(content) {
                if let Some(m) = cap.get(1) {
                    let t = m.as_str().trim();
                    if seen.insert(t.to_string()) {
                        results.push(ExtractedImport {
                            raw: cap[0].to_string(),
                            target: t.to_string(),
                            is_relative: t.starts_with('.') || t.starts_with('/'),
                        });
                    }
                }
            }
            for block in GO_BLOCK_IMPORT_RE.captures_iter(content) {
                if let Some(b) = block.get(1) {
                    for cap in GO_ITEM_IN_BLOCK_RE.captures_iter(b.as_str()) {
                        if let Some(m) = cap.get(1) {
                            let t = m.as_str().trim();
                            if seen.insert(t.to_string()) {
                                results.push(ExtractedImport {
                                    raw: format!("import \"{}\"", t),
                                    target: t.to_string(),
                                    is_relative: t.starts_with('.') || t.starts_with('/'),
                                });
                            }
                        }
                    }
                }
            }
        }
        "c" | "cpp" => {
            for cap in C_INCLUDE_RE.captures_iter(content) {
                if let Some(m) = cap.get(1) {
                    let t = m.as_str().trim();
                    if seen.insert(t.to_string()) {
                        results.push(ExtractedImport {
                            raw: cap[0].to_string(),
                            target: t.to_string(),
                            is_relative: !t.starts_with('<'),
                        });
                    }
                }
            }
        }
        "java" | "kotlin" => {
            for cap in JAVA_KT_IMPORT_RE.captures_iter(content) {
                if let Some(m) = cap.get(1) {
                    let t = m.as_str().trim();
                    if seen.insert(t.to_string()) {
                        results.push(ExtractedImport {
                            raw: cap[0].to_string(),
                            target: t.to_string(),
                            is_relative: false,
                        });
                    }
                }
            }
        }
        "csharp" => {
            for cap in CSHARP_USING_RE.captures_iter(content) {
                if let Some(m) = cap.get(1) {
                    let t = m.as_str().trim();
                    if seen.insert(t.to_string()) {
                        results.push(ExtractedImport {
                            raw: cap[0].to_string(),
                            target: t.to_string(),
                            is_relative: false,
                        });
                    }
                }
            }
        }
        "php" => {
            for cap in PHP_IMPORT_RE.captures_iter(content) {
                let target = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str());
                if let Some(t) = target {
                    let clean = t.trim();
                    if !clean.is_empty() && seen.insert(clean.to_string()) {
                        results.push(ExtractedImport {
                            raw: cap[0].to_string(),
                            target: clean.to_string(),
                            is_relative: clean.starts_with('.') || clean.starts_with('/'),
                        });
                    }
                }
            }
        }
        "ruby" => {
            for cap in RUBY_IMPORT_RE.captures_iter(content) {
                if let Some(m) = cap.get(1) {
                    let t = m.as_str().trim();
                    if seen.insert(t.to_string()) {
                        let is_rel = cap[0].contains("require_relative") || t.starts_with('.');
                        results.push(ExtractedImport {
                            raw: cap[0].to_string(),
                            target: t.to_string(),
                            is_relative: is_rel,
                        });
                    }
                }
            }
        }
        "swift" => {
            for cap in SWIFT_IMPORT_RE.captures_iter(content) {
                if let Some(m) = cap.get(1) {
                    let t = m.as_str().trim();
                    if seen.insert(t.to_string()) {
                        results.push(ExtractedImport {
                            raw: cap[0].to_string(),
                            target: t.to_string(),
                            is_relative: false,
                        });
                    }
                }
            }
        }
        "css" => {
            for cap in CSS_IMPORT_RE.captures_iter(content) {
                if let Some(m) = cap.get(1) {
                    let t = m.as_str().trim();
                    if seen.insert(t.to_string()) {
                        results.push(ExtractedImport {
                            raw: cap[0].to_string(),
                            target: t.to_string(),
                            is_relative: t.starts_with('.') || t.starts_with('/'),
                        });
                    }
                }
            }
        }
        _ => {}
    }

    results
}

// ==========================================
// Path Resolution
// ==========================================

/// Resolves an import target to an internal project file path, if it matches.
pub fn resolve_import_path(
    current_file_path: &str,
    import_target: &str,
    project_files_set: &HashSet<String>,
    project_files_by_name: &HashMap<String, Vec<String>>,
) -> Option<String> {
    let current_path = Path::new(current_file_path);
    let parent_dir = current_path.parent().unwrap_or(Path::new(""));

    let target_clean = import_target
        .trim_start_matches('@')
        .trim_start_matches('/')
        .replace('\\', "/");

    // 1. If it's a relative path (e.g. "./components/GraphView" or "../types")
    if import_target.starts_with('.') {
        let joined = parent_dir.join(import_target);
        if let Some(resolved) = normalize_and_match(&joined, project_files_set) {
            return Some(resolved);
        }
    }

    // 2. Exact match in project files set
    if project_files_set.contains(&target_clean) {
        return Some(target_clean);
    }

    // 3. Match with common extensions or /index variations
    let extensions = [
        "", ".ts", ".tsx", ".js", ".jsx", ".rs", ".py", ".go", ".c", ".h", ".cpp", ".hpp",
        "/index.ts", "/index.tsx", "/index.js", "/index.jsx", "/mod.rs", "/__init__.py",
    ];

    for ext in &extensions {
        let candidate = format!("{}{}", target_clean, ext);
        if project_files_set.contains(&candidate) {
            return Some(candidate);
        }
        // Also try prefixed with "src/"
        let src_candidate = format!("src/{}{}", target_clean, ext);
        if project_files_set.contains(&src_candidate) {
            return Some(src_candidate);
        }
    }

    // 4. Rust crate:: or super:: resolution
    if import_target.starts_with("crate::") {
        let rel_rust = import_target.trim_start_matches("crate::").replace("::", "/");
        let rust_candidates = [
            format!("src/{}.rs", rel_rust),
            format!("src/{}/mod.rs", rel_rust),
            format!("src-tauri/src/{}.rs", rel_rust),
            format!("src-tauri/src/{}/mod.rs", rel_rust),
            format!("{}.rs", rel_rust),
            format!("{}/mod.rs", rel_rust),
        ];
        for c in &rust_candidates {
            if project_files_set.contains(c) {
                return Some(c.clone());
            }
        }
        // Also try matching the module prefix if it references a sub-item like `crate::engine::db::DbManager`
        let parts: Vec<&str> = rel_rust.split('/').collect();
        if parts.len() > 1 {
            let parent_mod = parts[..parts.len() - 1].join("/");
            let rust_parent_candidates = [
                format!("src/{}.rs", parent_mod),
                format!("src/{}/mod.rs", parent_mod),
                format!("src-tauri/src/{}.rs", parent_mod),
                format!("src-tauri/src/{}/mod.rs", parent_mod),
            ];
            for c in &rust_parent_candidates {
                if project_files_set.contains(c) {
                    return Some(c.clone());
                }
            }
        }
    }

    // 5. Rust mod foo; resolution (look for foo.rs or foo/mod.rs in parent_dir)
    let rust_mod_candidates = [
        parent_dir.join(format!("{}.rs", import_target)),
        parent_dir.join(format!("{}/mod.rs", import_target)),
    ];
    for cand in &rust_mod_candidates {
        let norm = normalize_path(cand);
        if project_files_set.contains(&norm) {
            return Some(norm);
        }
    }

    // 6. Python dotted imports: e.g. "engine.db" -> "engine/db.py"
    let py_path = import_target.replace('.', "/");
    let py_candidates = [
        format!("{}.py", py_path),
        format!("{}/__init__.py", py_path),
    ];
    for c in &py_candidates {
        if project_files_set.contains(c) {
            return Some(c.clone());
        }
        let joined_py = parent_dir.join(c);
        let norm = normalize_path(&joined_py);
        if project_files_set.contains(&norm) {
            return Some(norm);
        }
    }

    // 7. C/C++ includes: e.g. "parser.h" matching any file by filename
    let file_name = Path::new(import_target)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(import_target);
    if let Some(matches) = project_files_by_name.get(file_name) {
        if matches.len() == 1 {
            return Some(matches[0].clone());
        }
    }

    None
}

fn normalize_path(p: &Path) -> String {
    let mut components = Vec::new();
    for comp in p.components() {
        match comp {
            std::path::Component::Normal(c) => components.push(c.to_string_lossy().to_string()),
            std::path::Component::ParentDir => {
                components.pop();
            }
            std::path::Component::CurDir => {}
            _ => {}
        }
    }
    components.join("/")
}

fn normalize_and_match(path: &Path, project_files_set: &HashSet<String>) -> Option<String> {
    let norm = normalize_path(path);
    if project_files_set.contains(&norm) {
        return Some(norm);
    }
    let extensions = [
        ".ts", ".tsx", ".js", ".jsx", ".rs", ".py", ".go", ".c", ".h", ".cpp", ".hpp",
        "/index.ts", "/index.tsx", "/index.js", "/index.jsx", "/mod.rs", "/__init__.py",
    ];
    for ext in &extensions {
        let candidate = format!("{}{}", norm, ext);
        if project_files_set.contains(&candidate) {
            return Some(candidate);
        }
    }
    None
}

// ==========================================
// Semantic Code Chunking & Symbol Extraction
// ==========================================

/// Regular expressions for identifying symbol definitions across languages.
static SYMBOL_DEF_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"(?m)^\s*(?:(?:export|pub|async|static|default|final|public|private|protected)\s+)*(?:(?:function|fn|def|func|class|struct|interface|type|enum|trait|impl)\s+([a-zA-Z0-9_]+)|(?:const|let|var)\s+([a-zA-Z0-9_]+)\s*=\s*(?:async\s*)?(?:\([^)]*\)|[a-zA-Z0-9_]+)\s*=>)"#)
        .expect("valid regex for symbol def")
});

pub fn chunk_code_file(_path: &str, content: &str) -> Vec<CodeChunk> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Vec::new();
    }

    // If small file (< 80 lines), return as single atomic chunk
    if lines.len() <= 80 {
        let token_estimate = estimate_code_tokens(content);
        return vec![CodeChunk {
            chunk_index: 0,
            symbol_name: None,
            start_line: 1,
            end_line: lines.len(),
            content: content.to_string(),
            token_estimate,
        }];
    }

    // Find symbol definition boundary lines
    let mut symbol_boundaries: Vec<(usize, String)> = Vec::new();
    for (idx, line) in lines.iter().enumerate() {
        if let Some(cap) = SYMBOL_DEF_RE.captures(line) {
            let symbol = cap.get(1).or_else(|| cap.get(2)).map(|m| m.as_str());
            if let Some(s) = symbol {
                symbol_boundaries.push((idx, s.to_string()));
            }
        }
    }

    let mut chunks = Vec::new();

    // If we detected structured symbol boundaries
    if !symbol_boundaries.is_empty() {
        let mut prev_idx = 0;
        let mut prev_sym: Option<String> = None;

        for (sym_idx, sym_name) in symbol_boundaries {
            if sym_idx > prev_idx && sym_idx - prev_idx >= 15 {
                let slice = lines[prev_idx..sym_idx].join("\n");
                let token_estimate = estimate_code_tokens(&slice);
                chunks.push(CodeChunk {
                    chunk_index: chunks.len(),
                    symbol_name: prev_sym.clone(),
                    start_line: prev_idx + 1,
                    end_line: sym_idx,
                    content: slice,
                    token_estimate,
                });
                prev_idx = sym_idx;
                prev_sym = Some(sym_name);
            } else if prev_sym.is_none() {
                prev_sym = Some(sym_name);
            }
        }

        // Remaining tail
        if prev_idx < lines.len() {
            let slice = lines[prev_idx..lines.len()].join("\n");
            let token_estimate = estimate_code_tokens(&slice);
            chunks.push(CodeChunk {
                chunk_index: chunks.len(),
                symbol_name: prev_sym,
                start_line: prev_idx + 1,
                end_line: lines.len(),
                content: slice,
                token_estimate,
            });
        }
    } else {
        // Fallback: sliding window of ~60 lines with 10 lines overlap
        let window_size = 60;
        let step = 50;
        let mut start = 0;

        while start < lines.len() {
            let end = (start + window_size).min(lines.len());
            let slice = lines[start..end].join("\n");
            let token_estimate = estimate_code_tokens(&slice);
            chunks.push(CodeChunk {
                chunk_index: chunks.len(),
                symbol_name: None,
                start_line: start + 1,
                end_line: end,
                content: slice,
                token_estimate,
            });
            if end == lines.len() {
                break;
            }
            start += step;
        }
    }

    // If no chunks generated, fallback to 1 chunk
    if chunks.is_empty() {
        chunks.push(CodeChunk {
            chunk_index: 0,
            symbol_name: None,
            start_line: 1,
            end_line: lines.len(),
            content: content.to_string(),
            token_estimate: estimate_code_tokens(content),
        });
    }

    chunks
}

pub fn estimate_code_tokens(text: &str) -> usize {
    // Code has punctuation and tokens roughly every 3.5 - 4 characters
    (text.len() / 4).max(1)
}

/// Parses a source code file into a structured representation.
pub fn parse_code_file(rel_path: &str, content: &str) -> ParsedCodeFile {
    let path_obj = Path::new(rel_path);
    let name = path_obj
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("file")
        .to_string();
    let extension = path_obj
        .extension()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_lowercase();
    let language = get_language_from_ext(&extension)
        .unwrap_or("unknown")
        .to_string();

    let imports = extract_imports(&language, content);
    let chunks = chunk_code_file(rel_path, content);

    // Compute summary
    let symbols: Vec<String> = chunks
        .iter()
        .filter_map(|c| c.symbol_name.clone())
        .collect();

    let summary = if !symbols.is_empty() {
        let sym_preview = symbols.iter().take(6).cloned().collect::<Vec<_>>().join(", ");
        format!(
            "{} file ({}) defining: {}{}",
            language,
            name,
            sym_preview,
            if symbols.len() > 6 { "..." } else { "" }
        )
    } else {
        format!("{} source file: {}", language, rel_path)
    };

    ParsedCodeFile {
        path: rel_path.to_string(),
        name,
        extension,
        language,
        size_bytes: content.len(),
        imports,
        chunks,
        summary,
    }
}

// ==========================================
// Unit Tests
// ==========================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_detection() {
        assert_eq!(get_language_from_ext("rs"), Some("rust"));
        assert_eq!(get_language_from_ext("tsx"), Some("tsx"));
        assert_eq!(get_language_from_ext("py"), Some("python"));
        assert_eq!(get_language_from_ext("go"), Some("go"));
        assert_eq!(get_language_from_ext("unknown"), None);
    }

    #[test]
    fn test_extract_js_imports() {
        let code = r#"
            import React, { useState } from 'react';
            import { GraphView } from './components/GraphView';
            import type { GraphData } from '../types';
            const { helper } = require('./utils/helper');
        "#;
        let imports = extract_imports("typescript", code);
        let targets: Vec<&str> = imports.iter().map(|i| i.target.as_str()).collect();
        assert!(targets.contains(&"react"));
        assert!(targets.contains(&"./components/GraphView"));
        assert!(targets.contains(&"../types"));
        assert!(targets.contains(&"./utils/helper"));
    }

    #[test]
    fn test_extract_rust_imports() {
        let code = r#"
            pub mod parser;
            mod db;
            use crate::engine::db::DbManager;
            use super::embeddings::EmbeddingEngine;
            use serde::{Serialize, Deserialize};
        "#;
        let imports = extract_imports("rust", code);
        let targets: Vec<&str> = imports.iter().map(|i| i.target.as_str()).collect();
        assert!(targets.contains(&"parser"));
        assert!(targets.contains(&"db"));
        assert!(targets.contains(&"crate::engine::db::DbManager"));
        assert!(targets.contains(&"super::embeddings::EmbeddingEngine"));
    }

    #[test]
    fn test_extract_python_imports() {
        let code = r#"
            import os
            import sys
            from utils.helpers import format_date
            from .models import User
        "#;
        let imports = extract_imports("python", code);
        let targets: Vec<&str> = imports.iter().map(|i| i.target.as_str()).collect();
        assert!(targets.contains(&"os"));
        assert!(targets.contains(&"sys"));
        assert!(targets.contains(&"utils.helpers"));
        assert!(targets.contains(&".models"));
    }

    #[test]
    fn test_resolve_import_path() {
        let mut files_set = HashSet::new();
        files_set.insert("src/App.tsx".to_string());
        files_set.insert("src/types.ts".to_string());
        files_set.insert("src/components/GraphView.tsx".to_string());

        let mut files_by_name = HashMap::new();
        files_by_name.insert("GraphView.tsx".to_string(), vec!["src/components/GraphView.tsx".to_string()]);

        let resolved = resolve_import_path(
            "src/App.tsx",
            "./components/GraphView",
            &files_set,
            &files_by_name,
        );
        assert_eq!(resolved, Some("src/components/GraphView.tsx".to_string()));

        let resolved_types = resolve_import_path(
            "src/components/GraphView.tsx",
            "../types",
            &files_set,
            &files_by_name,
        );
        assert_eq!(resolved_types, Some("src/types.ts".to_string()));
    }
}
