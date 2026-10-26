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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeRelationKind {
    Import,
    TypeReference,
    DiRegistration,
    DiInjection,
}

impl CodeRelationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Import => "import",
            Self::TypeReference => "type",
            Self::DiRegistration => "registration",
            Self::DiInjection => "injection",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtractedCodeRelation {
    pub kind: CodeRelationKind,
    pub raw: String,
    pub target: String,
    pub source_symbol: Option<String>,
    pub target_symbol: Option<String>,
    pub line: usize,
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
    pub calls: Vec<String>,
}

/// Parsed metadata and chunks of a source code file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCodeFile {
    pub path: String,
    pub name: String,
    pub extension: String,
    pub language: String,
    pub size_bytes: usize,
    pub namespaces: Vec<String>,
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
        "sql" => Some("sql"),
        "sh" | "bash" | "zsh" => Some("shell"),
        _ => None,
    }
}

pub fn is_supported_code_file(path: &Path) -> bool {
    let file_name = match path.file_name().and_then(|s| s.to_str()) {
        Some(n) => n.to_lowercase(),
        None => return false,
    };
    if file_name.ends_with(".min.js")
        || file_name.ends_with(".min.css")
        || file_name.ends_with(".bundle.js")
        || file_name.ends_with(".chunk.js")
        || file_name.ends_with(".d.ts")
        || file_name.ends_with(".map")
        || file_name.ends_with(".generated.ts")
        || file_name.ends_with(".generated.js")
        || file_name.ends_with(".pb.go")
        || file_name.ends_with("_pb2.py")
    {
        return false;
    }
    if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
        get_language_from_ext(ext).is_some()
    } else {
        false
    }
}

// ==========================================
// AST-Aware Lexical & Syntactic Parser for TS/JS
// ==========================================

#[derive(Debug, Clone, PartialEq, Eq)]
enum JsTokenKind {
    Ident,
    StringLit,
    Punct,
}

#[derive(Debug, Clone)]
struct JsToken {
    kind: JsTokenKind,
    text: String,
    #[allow(dead_code)]
    line: usize,
}

/// Tokenizes TypeScript / JavaScript source into an AST token stream,
/// stripping `//` line comments and `/* ... */` block comments accurately
/// while preserving string literals and scanning inside `${...}` template expressions.
fn tokenize_js_ts(source: &str) -> Vec<JsToken> {
    let chars: Vec<char> = source.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut line = 1usize;
    let mut tokens = Vec::new();

    while i < n {
        let c = chars[i];
        if c == '\n' {
            line += 1;
            i += 1;
            continue;
        }
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Single-line comment //
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            i += 2;
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Multi-line comment /* ... */
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            i += 2;
            while i + 1 < n && !(chars[i] == '*' && chars[i + 1] == '/') {
                if chars[i] == '\n' {
                    line += 1;
                }
                i += 1;
            }
            i = (i + 2).min(n);
            continue;
        }

        // String literal '...' or "..."
        if c == '\'' || c == '"' {
            let quote = c;
            let start_line = line;
            i += 1;
            let mut val = String::new();
            while i < n && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < n {
                    i += 1;
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    val.push(chars[i]);
                } else {
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    val.push(chars[i]);
                }
                i += 1;
            }
            if i < n {
                i += 1; // closing quote
            }
            tokens.push(JsToken {
                kind: JsTokenKind::StringLit,
                text: val,
                line: start_line,
            });
            continue;
        }

        // Template literal `...` (recursively tokenize ${...} expressions)
        if c == '`' {
            i += 1;
            let mut interp_content = String::new();
            while i < n && chars[i] != '`' {
                if chars[i] == '\\' && i + 1 < n {
                    if chars[i + 1] == '\n' {
                        line += 1;
                    }
                    i += 2;
                    continue;
                }
                if chars[i] == '$' && i + 1 < n && chars[i + 1] == '{' {
                    i += 2;
                    let mut depth = 1usize;
                    while i < n && depth > 0 {
                        if chars[i] == '\n' {
                            line += 1;
                        }
                        if chars[i] == '{' {
                            depth += 1;
                        } else if chars[i] == '}' {
                            depth -= 1;
                            if depth == 0 {
                                i += 1;
                                break;
                            }
                        }
                        interp_content.push(chars[i]);
                        i += 1;
                    }
                    interp_content.push(';');
                } else {
                    if chars[i] == '\n' {
                        line += 1;
                    }
                    i += 1;
                }
            }
            if i < n {
                i += 1;
            }
            if !interp_content.is_empty() {
                tokens.extend(tokenize_js_ts(&interp_content));
            }
            continue;
        }

        // Identifier or keyword
        if c.is_ascii_alphabetic() || c == '_' || c == '$' {
            let start_line = line;
            let mut ident = String::new();
            while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '$') {
                ident.push(chars[i]);
                i += 1;
            }
            tokens.push(JsToken {
                kind: JsTokenKind::Ident,
                text: ident,
                line: start_line,
            });
            continue;
        }

        // Arrow `=>`
        if c == '=' && i + 1 < n && chars[i + 1] == '>' {
            tokens.push(JsToken {
                kind: JsTokenKind::Punct,
                text: "=>".to_string(),
                line,
            });
            i += 2;
            continue;
        }

        // Punctuation
        tokens.push(JsToken {
            kind: JsTokenKind::Punct,
            text: c.to_string(),
            line,
        });
        i += 1;
    }

    tokens
}

/// Syntactic AST import extractor for TypeScript / JavaScript.
/// Handles:
/// - Single and multiline `import ... from '...'` and `import type ... from '...'`
/// - Side-effect `import '...'`
/// - Single and multiline re-exports: `export { ... } from '...'`, `export * from '...'`, `export * as ns from '...'`, `export type { ... } from '...'`
/// - Dynamic `import('...')` (including inline webpack comments) and TS `import('...').Type`
/// - CommonJS `require('...')` and TS `import x = require('...')`
pub fn extract_js_ts_imports_ast(content: &str) -> Vec<ExtractedImport> {
    let tokens = tokenize_js_ts(content);
    let len = tokens.len();
    let mut results = Vec::new();
    let mut seen = HashSet::new();
    let mut i = 0;

    let mut push_import = |raw: String, target: &str| {
        let clean = target.trim();
        if !clean.is_empty() && seen.insert(clean.to_string()) {
            results.push(ExtractedImport {
                raw,
                target: clean.to_string(),
                is_relative: clean.starts_with('.') || clean.starts_with('/'),
            });
        }
    };

    while i < len {
        let tok = &tokens[i];

        // 1. `import` statements and `import(...)` dynamic calls / type queries
        if tok.kind == JsTokenKind::Ident && tok.text == "import" {
            // Case 1a: Dynamic import `import('...')`
            if i + 2 < len && tokens[i + 1].text == "(" {
                if tokens[i + 2].kind == JsTokenKind::StringLit {
                    let target = &tokens[i + 2].text;
                    push_import(format!("import('{}')", target), target);
                    i += 3;
                    continue;
                }
            }

            // Case 1b: Side-effect import `import '...'`
            if i + 1 < len && tokens[i + 1].kind == JsTokenKind::StringLit {
                let target = &tokens[i + 1].text;
                push_import(format!("import '{}'", target), target);
                i += 2;
                continue;
            }

            // Case 1c: `import ... from '...'` or `import x = require('...')` (potentially multiline)
            let mut j = i + 1;
            let mut brace_depth = 0isize;
            let mut found = false;
            while j < len {
                let t = &tokens[j];
                if t.text == "{" {
                    brace_depth += 1;
                } else if t.text == "}" {
                    brace_depth = (brace_depth - 1).max(0);
                } else if brace_depth == 0 && t.kind == JsTokenKind::Ident && t.text == "from" {
                    if j + 1 < len && tokens[j + 1].kind == JsTokenKind::StringLit {
                        let target = &tokens[j + 1].text;
                        push_import(format!("import ... from '{}'", target), target);
                        i = j + 2;
                        found = true;
                        break;
                    }
                } else if brace_depth == 0
                    && t.kind == JsTokenKind::Ident
                    && t.text == "require"
                    && j + 2 < len
                    && tokens[j + 1].text == "("
                    && tokens[j + 2].kind == JsTokenKind::StringLit
                {
                    let target = &tokens[j + 2].text;
                    push_import(format!("import = require('{}')", target), target);
                    i = j + 3;
                    found = true;
                    break;
                } else if brace_depth == 0 && (t.text == ";" || (t.kind == JsTokenKind::Ident && (t.text == "import" || t.text == "export" || t.text == "const" || t.text == "let" || t.text == "var" || t.text == "function" || t.text == "class"))) {
                    break;
                }
                j += 1;
            }
            if found {
                continue;
            }
        }

        // 2. Re-exports: `export * from '...'`, `export * as ns from '...'`, `export { ... } from '...'`, `export type { ... } from '...'`
        if tok.kind == JsTokenKind::Ident && tok.text == "export" {
            let mut j = i + 1;
            if j < len && tokens[j].kind == JsTokenKind::Ident && tokens[j].text == "type" {
                j += 1;
            }

            if j < len && (tokens[j].text == "*" || tokens[j].text == "{") {
                if tokens[j].text == "{" {
                    let mut depth = 1usize;
                    j += 1;
                    while j < len && depth > 0 {
                        if tokens[j].text == "{" {
                            depth += 1;
                        } else if tokens[j].text == "}" {
                            depth -= 1;
                        }
                        j += 1;
                    }
                } else {
                    // `*` or `* as ns`
                    j += 1;
                    if j + 1 < len && tokens[j].kind == JsTokenKind::Ident && tokens[j].text == "as" {
                        j += 2;
                    }
                }

                if j + 1 < len
                    && tokens[j].kind == JsTokenKind::Ident
                    && tokens[j].text == "from"
                    && tokens[j + 1].kind == JsTokenKind::StringLit
                {
                    let target = &tokens[j + 1].text;
                    push_import(format!("export ... from '{}'", target), target);
                    i = j + 2;
                    continue;
                }
            }
        }

        // 3. CommonJS `require('...')`
        if tok.kind == JsTokenKind::Ident && tok.text == "require" {
            if i + 2 < len
                && tokens[i + 1].text == "("
                && tokens[i + 2].kind == JsTokenKind::StringLit
                && (i + 3 >= len || tokens[i + 3].text == ")" || tokens[i + 3].text == ",")
            {
                let target = &tokens[i + 2].text;
                push_import(format!("require('{}')", target), target);
                i += 3;
                continue;
            }
        }

        i += 1;
    }

    results
}

// ==========================================
// AST-Aware Lexical & Syntactic Parser for Rust & Python
// ==========================================

fn strip_rust_comments_and_strings(source: &str) -> String {
    let chars: Vec<char> = source.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut out = String::with_capacity(source.len());

    while i < n {
        let c = chars[i];

        // Line comment //
        if c == '/' && i + 1 < n && chars[i + 1] == '/' {
            i += 2;
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Nested block comment /* ... */
        if c == '/' && i + 1 < n && chars[i + 1] == '*' {
            i += 2;
            let mut depth = 1usize;
            while i + 1 < n && depth > 0 {
                if chars[i] == '/' && chars[i + 1] == '*' {
                    depth += 1;
                    i += 2;
                } else if chars[i] == '*' && chars[i + 1] == '/' {
                    depth -= 1;
                    i += 2;
                } else {
                    if chars[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
            }
            out.push(' ');
            continue;
        }

        // Raw string r#"..."# or r"..."
        if c == 'r' && i + 1 < n && (chars[i + 1] == '#' || chars[i + 1] == '"') {
            let mut j = i + 1;
            let mut hashes = 0usize;
            while j < n && chars[j] == '#' {
                hashes += 1;
                j += 1;
            }
            if j < n && chars[j] == '"' {
                j += 1;
                while j < n {
                    if chars[j] == '\n' {
                        out.push('\n');
                    }
                    if chars[j] == '"' {
                        let mut k = j + 1;
                        let mut matched_hashes = 0usize;
                        while k < n && matched_hashes < hashes && chars[k] == '#' {
                            matched_hashes += 1;
                            k += 1;
                        }
                        if matched_hashes == hashes {
                            j = k;
                            break;
                        }
                    }
                    j += 1;
                }
                out.push_str("\"\"");
                i = j;
                continue;
            }
        }

        // Standard string "..."
        if c == '"' {
            i += 1;
            while i < n && chars[i] != '"' {
                if chars[i] == '\\' && i + 1 < n {
                    if chars[i + 1] == '\n' {
                        out.push('\n');
                    }
                    i += 2;
                } else {
                    if chars[i] == '\n' {
                        out.push('\n');
                    }
                    i += 1;
                }
            }
            if i < n {
                i += 1;
            }
            out.push_str("\"\"");
            continue;
        }

        out.push(c);
        i += 1;
    }

    out
}

/// Recursively expands a Rust `use` tree expression (e.g. `crate::engine::{code::{parse_code_file, ExtractedImport as Imp}, db::DbManager}`)
/// into individual resolved module/symbol paths.
fn expand_rust_use_tree(prefix: &str, tree_expr: &str, out: &mut Vec<String>) {
    let trimmed = tree_expr.trim().trim_end_matches(';').trim();
    if trimmed.is_empty() {
        return;
    }

    // Split top-level comma-separated items (respecting `{ ... }` nesting)
    let mut parts = Vec::new();
    let mut depth = 0isize;
    let mut start = 0usize;
    for (idx, ch) in trimmed.char_indices() {
        if ch == '{' {
            depth += 1;
        } else if ch == '}' {
            depth = (depth - 1).max(0);
        } else if ch == ',' && depth == 0 {
            parts.push(&trimmed[start..idx]);
            start = idx + 1;
        }
    }
    parts.push(&trimmed[start..]);

    if parts.len() > 1 {
        for part in parts {
            expand_rust_use_tree(prefix, part, out);
        }
        return;
    }

    let item = parts[0].trim();
    if item.is_empty() {
        return;
    }

    // Check if item contains a nested group `path::{...}` or `{...}`
    if let Some(brace_open) = item.find('{') {
        if let Some(brace_close) = item.rfind('}') {
            let sub_prefix_raw = item[..brace_open].trim().trim_end_matches("::");
            let new_prefix = if prefix.is_empty() {
                sub_prefix_raw.to_string()
            } else if sub_prefix_raw.is_empty() {
                prefix.to_string()
            } else {
                format!("{}::{}", prefix, sub_prefix_raw)
            };
            if !new_prefix.is_empty() {
                out.push(new_prefix.clone());
            }
            let inner = &item[brace_open + 1..brace_close];
            expand_rust_use_tree(&new_prefix, inner, out);
            return;
        }
    }

    // Strip ` as Alias`
    let base_item = if let Some(as_idx) = item.find(" as ") {
        item[..as_idx].trim()
    } else {
        item
    };

    let clean_item = base_item.trim_start_matches("::").trim();
    if clean_item == "self" {
        if !prefix.is_empty() {
            out.push(prefix.to_string());
        }
        return;
    }
    if clean_item == "*" {
        if !prefix.is_empty() {
            out.push(prefix.to_string());
        }
        return;
    }

    let full = if prefix.is_empty() {
        clean_item.to_string()
    } else {
        format!("{}::{}", prefix, clean_item)
    };
    if !full.is_empty() {
        out.push(full);
    }
}

pub fn extract_rust_imports_ast(content: &str) -> Vec<ExtractedImport> {
    let cleaned = strip_rust_comments_and_strings(content);
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    // Scan statements separated by `;` or `{` / `}` blocks
    let chars: Vec<char> = cleaned.chars().collect();
    let n = chars.len();
    let mut i = 0;

    while i < n {
        if chars[i].is_whitespace() {
            i += 1;
            continue;
        }

        // Skip attributes `#[...]` or `#![...]`
        if chars[i] == '#' && i + 1 < n && (chars[i + 1] == '[' || (chars[i + 1] == '!' && i + 2 < n && chars[i + 2] == '[')) {
            let mut bracket_depth = 0isize;
            while i < n {
                if chars[i] == '[' {
                    bracket_depth += 1;
                } else if chars[i] == ']' {
                    bracket_depth -= 1;
                    if bracket_depth <= 0 {
                        i += 1;
                        break;
                    }
                }
                i += 1;
            }
            continue;
        }

        // Read word
        if chars[i].is_ascii_alphabetic() || chars[i] == '_' {
            let start = i;
            while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();

            // Skip optional `pub` or `pub(...)` visibility modifier
            let mut effective_word = word;
            if effective_word == "pub" {
                while i < n && chars[i].is_whitespace() {
                    i += 1;
                }
                if i < n && chars[i] == '(' {
                    let mut pdepth = 0isize;
                    while i < n {
                        if chars[i] == '(' {
                            pdepth += 1;
                        } else if chars[i] == ')' {
                            pdepth -= 1;
                            if pdepth <= 0 {
                                i += 1;
                                break;
                            }
                        }
                        i += 1;
                    }
                    while i < n && chars[i].is_whitespace() {
                        i += 1;
                    }
                }
                let w_start = i;
                while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                effective_word = chars[w_start..i].iter().collect();
            }

            if effective_word == "mod" {
                while i < n && chars[i].is_whitespace() {
                    i += 1;
                }
                let m_start = i;
                while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let mod_name: String = chars[m_start..i].iter().collect();
                while i < n && chars[i].is_whitespace() {
                    i += 1;
                }
                // External module declaration ends with `;` (not `{` inline module)
                if !mod_name.is_empty() && i < n && chars[i] == ';' {
                    if seen.insert(format!("mod:{}", mod_name)) {
                        results.push(ExtractedImport {
                            raw: format!("mod {};", mod_name),
                            target: mod_name,
                            is_relative: true,
                        });
                    }
                    i += 1;
                }
                continue;
            }

            if effective_word == "use" {
                let u_start = i;
                let mut brace_depth = 0isize;
                while i < n {
                    if chars[i] == '{' {
                        brace_depth += 1;
                    } else if chars[i] == '}' {
                        brace_depth = (brace_depth - 1).max(0);
                    } else if chars[i] == ';' && brace_depth == 0 {
                        break;
                    }
                    i += 1;
                }
                let use_expr: String = chars[u_start..i].iter().collect();
                if i < n && chars[i] == ';' {
                    i += 1;
                }
                let mut expanded = Vec::new();
                expand_rust_use_tree("", &use_expr, &mut expanded);
                for full_path in expanded {
                    let clean = full_path.trim().trim_start_matches("::");
                    if !clean.is_empty() && seen.insert(clean.to_string()) {
                        let is_rel = clean.starts_with("crate::")
                            || clean.starts_with("super::")
                            || clean.starts_with("self::");
                        results.push(ExtractedImport {
                            raw: format!("use {};", clean),
                            target: clean.to_string(),
                            is_relative: is_rel,
                        });
                    }
                }
                continue;
            }
            continue;
        }

        i += 1;
    }

    results
}

/// Strips Python `#` comments and triple/single quoted string literals,
/// joining parenthesized and backslash-continued lines into logical statements.
fn extract_python_logical_lines(source: &str) -> Vec<String> {
    let chars: Vec<char> = source.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut cleaned = String::with_capacity(source.len());
    let mut paren_depth = 0isize;

    while i < n {
        let c = chars[i];

        // Triple-quoted strings """...""" or '''...'''
        if (c == '"' || c == '\'') && i + 2 < n && chars[i + 1] == c && chars[i + 2] == c {
            let q = c;
            i += 3;
            while i + 2 < n && !(chars[i] == q && chars[i + 1] == q && chars[i + 2] == q) {
                i += 1;
            }
            i = (i + 3).min(n);
            cleaned.push_str("\"\"");
            continue;
        }

        // Single-quoted strings '...' or "..."
        if c == '"' || c == '\'' {
            let q = c;
            i += 1;
            while i < n && chars[i] != q && chars[i] != '\n' {
                if chars[i] == '\\' && i + 1 < n {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < n && chars[i] == q {
                i += 1;
            }
            cleaned.push_str("\"\"");
            continue;
        }

        // Comment #...
        if c == '#' {
            while i < n && chars[i] != '\n' {
                i += 1;
            }
            continue;
        }

        // Explicit line continuation `\` + newline
        if c == '\\' && i + 1 < n && (chars[i + 1] == '\n' || (chars[i + 1] == '\r' && i + 2 < n && chars[i + 2] == '\n')) {
            i += if chars[i + 1] == '\r' { 3 } else { 2 };
            cleaned.push(' ');
            continue;
        }

        if c == '(' || c == '[' || c == '{' {
            paren_depth += 1;
            cleaned.push(' ');
            i += 1;
            continue;
        }
        if c == ')' || c == ']' || c == '}' {
            paren_depth = (paren_depth - 1).max(0);
            cleaned.push(' ');
            i += 1;
            continue;
        }

        if c == '\n' {
            if paren_depth > 0 {
                cleaned.push(' ');
            } else {
                cleaned.push('\n');
            }
            i += 1;
            continue;
        }

        if c == ';' && paren_depth == 0 {
            cleaned.push('\n');
            i += 1;
            continue;
        }

        cleaned.push(c);
        i += 1;
    }

    cleaned
        .lines()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty())
        .collect()
}

pub fn extract_python_imports_ast(content: &str) -> Vec<ExtractedImport> {
    let lines = extract_python_logical_lines(content);
    let mut results = Vec::new();
    let mut seen = HashSet::new();

    let mut push_py = |raw: String, target: &str| {
        let clean = target.trim();
        if !clean.is_empty() && seen.insert(clean.to_string()) {
            results.push(ExtractedImport {
                raw,
                target: clean.to_string(),
                is_relative: clean.starts_with('.'),
            });
        }
    };

    for line in lines {
        // Case 1: `import a, b as c, d.e`
        if let Some(rest) = line.strip_prefix("import ") {
            for item in rest.split(',') {
                let item_trim = item.trim();
                let target = if let Some(as_idx) = item_trim.find(" as ") {
                    item_trim[..as_idx].trim()
                } else {
                    item_trim.split_whitespace().next().unwrap_or("")
                };
                if !target.is_empty() {
                    push_py(format!("import {}", target), target);
                }
            }
            continue;
        }

        // Case 2: `from <module> import <items>`
        if let Some(rest) = line.strip_prefix("from ") {
            if let Some(imp_idx) = rest.find(" import ") {
                let module_part = rest[..imp_idx].trim();
                let items_part = rest[imp_idx + " import ".len()..].trim();

                if !module_part.is_empty() && module_part.chars().any(|c| c != '.') {
                    push_py(format!("from {} import ...", module_part), module_part);
                }

                for item in items_part.split(',') {
                    let item_trim = item.trim();
                    if item_trim.is_empty() || item_trim == "*" {
                        continue;
                    }
                    let sym = if let Some(as_idx) = item_trim.find(" as ") {
                        item_trim[..as_idx].trim()
                    } else {
                        item_trim.split_whitespace().next().unwrap_or("")
                    };
                    if !sym.is_empty() {
                        let sub_target = if module_part.ends_with('.') {
                            format!("{}{}", module_part, sym)
                        } else if !module_part.is_empty() {
                            format!("{}.{}", module_part, sym)
                        } else {
                            sym.to_string()
                        };
                        push_py(
                            format!("from {} import {}", module_part, sym),
                            &sub_target,
                        );
                    }
                }
            }
        }
    }

    results
}

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
    Regex::new(
        r#"(?m)^\s*(?:global\s+)?using\s+(?:static\s+)?(?:[A-Za-z_][A-Za-z0-9_]*\s*=\s*)?(?:global::)?([a-zA-Z0-9_.]+)\s*;"#,
    )
        .expect("valid regex for csharp using")
});

static CSHARP_NAMESPACE_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)^\s*namespace\s+([a-zA-Z_][a-zA-Z0-9_.]*)\s*(?:;|\{)")
        .expect("valid regex for csharp namespace")
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
            return extract_js_ts_imports_ast(content);
        }
        "python" => {
            return extract_python_imports_ast(content);
        }
        "rust" => {
            return extract_rust_imports_ast(content);
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

pub fn extract_csharp_namespaces(content: &str) -> Vec<String> {
    let mut namespaces = Vec::new();
    let mut seen = HashSet::new();

    for cap in CSHARP_NAMESPACE_RE.captures_iter(content) {
        if let Some(namespace) = cap.get(1).map(|m| m.as_str()) {
            if seen.insert(namespace.to_string()) {
                namespaces.push(namespace.to_string());
            }
        }
    }

    namespaces
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
        .trim_start_matches('~')
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

    // 4. Rust crate::, super::, or self:: resolution
    if import_target.starts_with("crate::")
        || import_target.starts_with("super::")
        || import_target.starts_with("self::")
    {
        let (rel_rust, is_super, is_self) = if let Some(rest) = import_target.strip_prefix("crate::") {
            (rest.replace("::", "/"), false, false)
        } else if let Some(rest) = import_target.strip_prefix("super::") {
            (rest.replace("::", "/"), true, false)
        } else {
            let rest = import_target.trim_start_matches("self::");
            (rest.replace("::", "/"), false, true)
        };

        let parts: Vec<&str> = rel_rust.split('/').filter(|s| !s.is_empty()).collect();
        // Try progressively shorter module prefixes (e.g. engine/db/GraphData -> engine/db -> engine)
        for prefix_len in (1..=parts.len()).rev() {
            let mod_path = parts[..prefix_len].join("/");
            if is_super || is_self {
                let mut base_dirs = vec![parent_dir.to_path_buf()];
                if is_super {
                    if let Some(grandparent) = parent_dir.parent() {
                        base_dirs.push(grandparent.to_path_buf());
                    }
                }
                for base in base_dirs {
                    let candidates = [
                        base.join(format!("{}.rs", mod_path)),
                        base.join(format!("{}/mod.rs", mod_path)),
                    ];
                    for cand in &candidates {
                        let norm = normalize_path(cand);
                        if project_files_set.contains(&norm) {
                            return Some(norm);
                        }
                    }
                }
            } else {
                let rust_candidates = [
                    format!("src/{}.rs", mod_path),
                    format!("src/{}/mod.rs", mod_path),
                    format!("src-tauri/src/{}.rs", mod_path),
                    format!("src-tauri/src/{}/mod.rs", mod_path),
                    format!("{}.rs", mod_path),
                    format!("{}/mod.rs", mod_path),
                ];
                for c in &rust_candidates {
                    if project_files_set.contains(c) {
                        return Some(c.clone());
                    }
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
    Regex::new(r#"(?m)^\s*(?:(?:export|pub(?:\([^)]*\))?|async|static|default|final|public|private|protected)\s+)*(?:(?:function|fn|def|func|class|struct|interface|type|enum|trait|impl)\s+([a-zA-Z0-9_]+)|(?:const|let|var)\s+([a-zA-Z0-9_]+)\s*=\s*(?:async\s*)?(?:\([^)]*\)|[a-zA-Z0-9_]+)\s*=>)"#)
        .expect("valid regex for symbol def")
});

static FUNC_CALL_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\("#)
        .expect("valid regex for function calls")
});

const CALL_IGNORE_KEYWORDS: &[&str] = &[
    "if", "else", "for", "while", "loop", "match", "switch", "case", "return", "catch",
    "try", "typeof", "sizeof", "alignof", "new", "delete", "super", "this", "self", "Self",
    "function", "fn", "def", "func", "class", "struct", "enum", "trait", "impl", "where",
    "pub", "import", "export", "require", "from", "await", "yield", "async",
    "Some", "None", "Ok", "Err", "Box", "Arc", "Rc", "Mutex", "RwLock", "Vec", "String",
    "HashMap", "HashSet", "Option", "Result", "println", "eprintln", "format", "write",
    "writeln", "vec", "panic", "assert", "assert_eq", "assert_ne", "cfg", "derive",
    "allow", "warn", "deny", "test", "console", "log", "warn", "error", "len", "is_empty",
    "clone", "to_string", "as_str", "unwrap", "expect", "unwrap_or", "unwrap_or_default",
    "unwrap_or_else", "map", "and_then", "filter", "collect", "into", "from", "iter",
    "into_iter", "push", "insert", "get", "set", "ok", "err", "default",
];

/// Extracts unique function/method call names invoked within a code block.
pub fn extract_function_calls(content: &str, self_symbol: Option<&str>) -> Vec<String> {
    let cleaned = strip_rust_comments_and_strings(content);
    let mut calls = Vec::new();
    let mut seen = HashSet::new();

    for cap in FUNC_CALL_RE.captures_iter(&cleaned) {
        if let Some(m) = cap.get(1) {
            let name = m.as_str();
            if name.len() < 2 {
                continue;
            }
            if let Some(self_sym) = self_symbol {
                if name == self_sym {
                    continue;
                }
            }
            if CALL_IGNORE_KEYWORDS.contains(&name) {
                continue;
            }
            if seen.insert(name.to_string()) {
                calls.push(name.to_string());
            }
        }
    }

    calls
}

pub fn chunk_code_file(_path: &str, content: &str) -> Vec<CodeChunk> {
    let lines: Vec<&str> = content.lines().collect();
    if lines.is_empty() {
        return Vec::new();
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

    // If small file (<= 80 lines) and has at most 1 symbol, return as single atomic chunk
    if lines.len() <= 80 && symbol_boundaries.len() <= 1 {
        let token_estimate = estimate_code_tokens(content);
        let sym = symbol_boundaries.into_iter().next().map(|(_, s)| s);
        let calls = extract_function_calls(content, sym.as_deref());
        return vec![CodeChunk {
            chunk_index: 0,
            symbol_name: sym,
            start_line: 1,
            end_line: lines.len(),
            content: content.to_string(),
            token_estimate,
            calls,
        }];
    }

    let mut chunks = Vec::new();

    // If we detected structured symbol boundaries
    if !symbol_boundaries.is_empty() {
        let mut prev_idx = 0;
        let mut prev_sym: Option<String> = None;

        for (sym_idx, sym_name) in symbol_boundaries {
            let min_gap = if prev_sym.is_some() { 18 } else { 15 };
            if sym_idx > prev_idx && sym_idx - prev_idx >= min_gap {
                let slice = lines[prev_idx..sym_idx].join("\n");
                let token_estimate = estimate_code_tokens(&slice);
                let calls = extract_function_calls(&slice, prev_sym.as_deref());
                chunks.push(CodeChunk {
                    chunk_index: chunks.len(),
                    symbol_name: prev_sym.clone(),
                    start_line: prev_idx + 1,
                    end_line: sym_idx,
                    content: slice,
                    token_estimate,
                    calls,
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
            let calls = extract_function_calls(&slice, prev_sym.as_deref());
            chunks.push(CodeChunk {
                chunk_index: chunks.len(),
                symbol_name: prev_sym,
                start_line: prev_idx + 1,
                end_line: lines.len(),
                content: slice,
                token_estimate,
                calls,
            });
        }
    } else {
        // Fallback: sliding window of ~100 lines with 10 lines overlap
        let window_size = 100;
        let step = 90;
        let mut start = 0;

        while start < lines.len() && chunks.len() < 20 {
            let end = (start + window_size).min(lines.len());
            let slice = lines[start..end].join("\n");
            let token_estimate = estimate_code_tokens(&slice);
            let calls = extract_function_calls(&slice, None);
            chunks.push(CodeChunk {
                chunk_index: chunks.len(),
                symbol_name: None,
                start_line: start + 1,
                end_line: end,
                content: slice,
                token_estimate,
                calls,
            });
            if end == lines.len() {
                break;
            }
            start += step;
        }
    }

    if chunks.len() > 24 {
        chunks.truncate(24);
    }

    // If no chunks generated, fallback to 1 chunk
    if chunks.is_empty() {
        let calls = extract_function_calls(content, None);
        chunks.push(CodeChunk {
            chunk_index: 0,
            symbol_name: None,
            start_line: 1,
            end_line: lines.len(),
            content: content.to_string(),
            token_estimate: estimate_code_tokens(content),
            calls,
        });
    }

    chunks
}

pub fn estimate_code_tokens(text: &str) -> usize {
    // Code has punctuation and tokens roughly every 3.5 - 4 characters
    (text.len() / 4).max(1)
}

const CSHARP_NAMESPACE_HASH_PREFIX: &str = "csharp-namespaces-v1:";

pub fn needs_code_metadata_refresh(rel_path: &str, stored_hash: &str) -> bool {
    let extension = Path::new(rel_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    get_language_from_ext(extension) == Some("csharp")
        && !stored_hash.starts_with(CSHARP_NAMESPACE_HASH_PREFIX)
}

pub fn indexed_code_content_hash(rel_path: &str, content: &str) -> String {
    let source_hash = crate::engine::parser::content_hash(content);
    let extension = Path::new(rel_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default();
    if get_language_from_ext(extension) == Some("csharp") {
        format!("{CSHARP_NAMESPACE_HASH_PREFIX}{source_hash}")
    } else {
        source_hash
    }
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

    let namespaces = if language == "csharp" {
        extract_csharp_namespaces(content)
    } else {
        Vec::new()
    };
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
        namespaces,
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
    fn test_code_relation_kind_labels_are_stable() {
        assert_eq!(CodeRelationKind::Import.as_str(), "import");
        assert_eq!(CodeRelationKind::TypeReference.as_str(), "type");
        assert_eq!(CodeRelationKind::DiRegistration.as_str(), "registration");
        assert_eq!(CodeRelationKind::DiInjection.as_str(), "injection");
    }

    #[test]
    fn test_extract_js_imports() {
        let code = r#"
            import React, {
                useState,
                useEffect,
                type FC,
            } from 'react';
            import { GraphView } from './components/GraphView';
            import type {
                GraphData,
                PathResult
            } from '../types';
            export {
                 ArchitecturalReportModal,
                type ReportProps
            } from './ArchitecturalReportModal';
            export * as utils from './utils/index';
            const LazyComp = React.lazy(() => import(/* webpackChunkName: "lazy" */ './LazyComp'));
            type DynamicType = typeof import('./dynamic/type').MyType;
            const { helper } = require('./utils/helper');
            // import FakeComment from './fake-comment';
            /* import FakeBlock from './fake-block'; */
            const str = "import FakeString from './fake-string'";
        "#;
        let imports = extract_imports("typescript", code);
        let targets: Vec<&str> = imports.iter().map(|i| i.target.as_str()).collect();
        assert!(targets.contains(&"react"));
        assert!(targets.contains(&"./components/GraphView"));
        assert!(targets.contains(&"../types"));
        assert!(targets.contains(&"./ArchitecturalReportModal"));
        assert!(targets.contains(&"./utils/index"));
        assert!(targets.contains(&"./LazyComp"));
        assert!(targets.contains(&"./dynamic/type"));
        assert!(targets.contains(&"./utils/helper"));
        assert!(!targets.contains(&"./fake-comment"));
        assert!(!targets.contains(&"./fake-block"));
        assert!(!targets.contains(&"./fake-string"));
    }

    #[test]
    fn test_extract_rust_imports() {
        let code = r#"
            pub mod parser;
            pub(crate) mod db;
            use crate::engine::{
                db::DbManager as Db,
                code::{parse_code_file, ExtractedImport},
            };
            use super::embeddings::EmbeddingEngine;
            use serde::{Serialize, Deserialize};
            // use crate::fake::CommentedOut;
            /* use crate::fake::BlockCommented; */
            let s = "use crate::fake::InsideString;";
        "#;
        let imports = extract_imports("rust", code);
        let targets: Vec<&str> = imports.iter().map(|i| i.target.as_str()).collect();
        assert!(targets.contains(&"parser"));
        assert!(targets.contains(&"db"));
        assert!(targets.contains(&"crate::engine::db::DbManager"));
        assert!(targets.contains(&"crate::engine::code::parse_code_file"));
        assert!(targets.contains(&"crate::engine::code::ExtractedImport"));
        assert!(targets.contains(&"super::embeddings::EmbeddingEngine"));
        assert!(!targets.contains(&"crate::fake::CommentedOut"));
        assert!(!targets.contains(&"crate::fake::BlockCommented"));
        assert!(!targets.contains(&"crate::fake::InsideString"));
    }

    #[test]
    fn test_extract_python_imports() {
        let code = r#"
            import os, sys as system
            from utils.helpers import (
                format_date as fmt,
                parse_json,
            )
            from .models import User
            from . import config
            # import fake_comment
            """
            import fake_docstring
            """
        "#;
        let imports = extract_imports("python", code);
        let targets: Vec<&str> = imports.iter().map(|i| i.target.as_str()).collect();
        assert!(targets.contains(&"os"));
        assert!(targets.contains(&"sys"));
        assert!(targets.contains(&"utils.helpers"));
        assert!(targets.contains(&"utils.helpers.format_date"));
        assert!(targets.contains(&".models"));
        assert!(targets.contains(&".config"));
        assert!(!targets.contains(&"fake_comment"));
        assert!(!targets.contains(&"fake_docstring"));
    }

    #[test]
    fn test_extract_csharp_namespaces() {
        let code = r#"
            using System;
            namespace Example.Services;

            // namespace Ignored.Commented;
            namespace Example.Handlers
            {
                public class Handler {}
            }
        "#;

        assert_eq!(
            extract_csharp_namespaces(code),
            vec!["Example.Services", "Example.Handlers"]
        );
    }

    #[test]
    fn test_extract_csharp_global_and_alias_usings() {
        let code = r#"
            global using Example.Services;
            using Services = Example.Services;
            using static Example.Services.ServiceDefaults;
        "#;

        let imports = extract_imports("csharp", code);
        let targets: Vec<&str> = imports.iter().map(|import| import.target.as_str()).collect();
        assert!(targets.contains(&"Example.Services"));
        assert!(targets.contains(&"Example.Services.ServiceDefaults"));
    }

    #[test]
    fn test_csharp_metadata_hash_forces_one_time_reindex() {
        let content = "namespace Example.Services;";
        let source_hash = crate::engine::parser::content_hash(content);
        let indexed_hash = indexed_code_content_hash("Service.cs", content);

        assert!(needs_code_metadata_refresh("Service.cs", &source_hash));
        assert!(!needs_code_metadata_refresh("Service.cs", &indexed_hash));
        assert_eq!(
            indexed_hash,
            format!("csharp-namespaces-v1:{source_hash}")
        );
        assert_eq!(
            indexed_code_content_hash("Service.ts", content),
            source_hash
        );
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

        // Test @/ and ~/ path aliases
        assert_eq!(
            resolve_import_path("src/App.tsx", "@/components/GraphView", &files_set, &files_by_name),
            Some("src/components/GraphView.tsx".to_string())
        );
        assert_eq!(
            resolve_import_path("src/App.tsx", "~/types", &files_set, &files_by_name),
            Some("src/types.ts".to_string())
        );

        // Test Rust super:: and crate:: module resolution
        files_set.insert("src-tauri/src/engine/db.rs".to_string());
        files_set.insert("src-tauri/src/engine/graph_analytics.rs".to_string());
        assert_eq!(
            resolve_import_path(
                "src-tauri/src/engine/graph_analytics.rs",
                "super::db::GraphData",
                &files_set,
                &files_by_name,
            ),
            Some("src-tauri/src/engine/db.rs".to_string())
        );
        assert_eq!(
            resolve_import_path(
                "src-tauri/src/mcp/mod.rs",
                "crate::engine::db::DbManager",
                &files_set,
                &files_by_name,
            ),
            Some("src-tauri/src/engine/db.rs".to_string())
        );
    }
}
