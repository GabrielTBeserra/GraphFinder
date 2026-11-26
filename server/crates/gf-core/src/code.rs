//! Lightweight source-code parsing for indexing.
//!
//! This is an AST-lite parser: it extracts imports and common symbol boundaries
//! without executing or compiling the target repository.

use crate::markdown::content_hash;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeSymbol {
    pub name: String,
    pub kind: String,
    pub start_line: usize,
    pub end_line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedCode {
    pub language: String,
    pub imports: Vec<String>,
    pub symbols: Vec<CodeSymbol>,
    pub content_hash: String,
}

pub fn detect_language(path: &str) -> Option<&'static str> {
    match path.rsplit('.').next()?.to_ascii_lowercase().as_str() {
        "rs" => Some("rust"),
        "ts" | "tsx" => Some("typescript"),
        "js" | "jsx" | "mjs" | "cjs" => Some("javascript"),
        "py" => Some("python"),
        "go" => Some("go"),
        _ => None,
    }
}

pub fn parse_code(path: &str, content: &str) -> Option<ParsedCode> {
    let language = detect_language(path)?.to_string();
    let mut imports = Vec::new();
    let mut symbols = Vec::new();

    for (index, line) in content.lines().enumerate() {
        let line_no = index + 1;
        let trimmed = line.trim();
        if let Some(import) = parse_import(&language, trimmed) {
            imports.push(import);
        }
        if let Some((kind, name)) = parse_symbol(&language, trimmed) {
            symbols.push(CodeSymbol {
                name,
                kind,
                start_line: line_no,
                end_line: line_no,
            });
        }
    }

    for i in 0..symbols.len() {
        let next_start = symbols.get(i + 1).map(|s| s.start_line);
        symbols[i].end_line = next_start
            .map(|line| line.saturating_sub(1).max(symbols[i].start_line))
            .unwrap_or_else(|| content.lines().count().max(symbols[i].start_line));
    }

    imports.sort();
    imports.dedup();
    Some(ParsedCode {
        language,
        imports,
        symbols,
        content_hash: content_hash(content),
    })
}

fn parse_import(language: &str, line: &str) -> Option<String> {
    match language {
        "rust" => line
            .strip_prefix("use ")
            .map(|v| v.trim_end_matches(';').trim().to_string()),
        "typescript" | "javascript" if line.starts_with("import ") => {
            Some(line.trim_end_matches(';').to_string())
        }
        "python" if line.starts_with("import ") || line.starts_with("from ") => {
            Some(line.to_string())
        }
        "go" if line.starts_with("import ") => Some(
            line.trim_start_matches("import ")
                .trim_matches('"')
                .to_string(),
        ),
        _ => None,
    }
    .filter(|v| !v.is_empty())
}

fn parse_symbol(language: &str, line: &str) -> Option<(String, String)> {
    match language {
        "rust" => rust_symbol(line),
        "typescript" | "javascript" => js_symbol(line),
        "python" => python_symbol(line),
        "go" => go_symbol(line),
        _ => None,
    }
}

fn rust_symbol(line: &str) -> Option<(String, String)> {
    for (prefix, kind) in [
        ("pub fn ", "function"),
        ("fn ", "function"),
        ("pub struct ", "struct"),
        ("struct ", "struct"),
        ("pub enum ", "enum"),
        ("enum ", "enum"),
        ("impl ", "impl"),
        ("pub trait ", "trait"),
        ("trait ", "trait"),
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name = rest.split(['(', '<', ' ', '{']).next()?.trim();
            if !name.is_empty() {
                return Some((kind.into(), name.into()));
            }
        }
    }
    None
}

fn js_symbol(line: &str) -> Option<(String, String)> {
    let line = line
        .strip_prefix("export default ")
        .or_else(|| line.strip_prefix("export "))
        .unwrap_or(line);
    for (prefix, kind) in [
        ("async function ", "function"),
        ("function ", "function"),
        ("class ", "class"),
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name = rest.split(['(', ' ', '{']).next()?.trim();
            if !name.is_empty() {
                return Some((kind.into(), name.into()));
            }
        }
    }
    for prefix in ["const ", "let ", "var "] {
        if let Some(rest) = line.strip_prefix(prefix) {
            if rest.contains("=>") {
                let name = rest.split('=').next()?.trim();
                if !name.is_empty() {
                    return Some(("function".into(), name.into()));
                }
            }
        }
    }
    None
}

fn python_symbol(line: &str) -> Option<(String, String)> {
    for (prefix, kind) in [
        ("def ", "function"),
        ("async def ", "function"),
        ("class ", "class"),
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            let name = rest.split(['(', ':', ' ']).next()?.trim();
            if !name.is_empty() {
                return Some((kind.into(), name.into()));
            }
        }
    }
    None
}

fn go_symbol(line: &str) -> Option<(String, String)> {
    if let Some(rest) = line.strip_prefix("func ") {
        let rest = rest.trim_start_matches('(').trim_start();
        let name = rest.split(['(', ' ']).next()?.trim();
        if !name.is_empty() {
            return Some(("function".into(), name.into()));
        }
    }
    if let Some(rest) = line.strip_prefix("type ") {
        let mut parts = rest.split_whitespace();
        if let (Some(name), Some(kind)) = (parts.next(), parts.next()) {
            return Some((kind.into(), name.into()));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_typescript_imports_and_symbols() {
        let parsed = parse_code(
            "sample.ts",
            include_str!("../../../fixtures/code/sample.ts"),
        )
        .unwrap();
        assert_eq!(parsed.language, "typescript");
        assert_eq!(parsed.imports.len(), 2);
        assert!(parsed
            .symbols
            .iter()
            .any(|s| s.name == "Indexer" && s.kind == "class"));
        assert!(parsed
            .symbols
            .iter()
            .any(|s| s.name == "scanRepository" && s.kind == "function"));
        assert!(parsed.symbols.iter().any(|s| s.name == "helper"));
    }

    #[test]
    fn parses_rust_imports_and_symbols() {
        let parsed = parse_code(
            "sample.rs",
            include_str!("../../../fixtures/code/sample.rs"),
        )
        .unwrap();
        assert_eq!(parsed.language, "rust");
        assert!(parsed.imports.iter().any(|i| i.contains("HashMap")));
        assert!(parsed
            .symbols
            .iter()
            .any(|s| s.name == "Indexer" && s.kind == "struct"));
        assert!(parsed
            .symbols
            .iter()
            .any(|s| s.name == "scan" && s.kind == "function"));
        assert!(parsed.symbols.iter().any(|s| s.name == "detect_language"));
    }

    #[test]
    fn detects_supported_languages() {
        assert_eq!(detect_language("x.rs"), Some("rust"));
        assert_eq!(detect_language("x.tsx"), Some("typescript"));
        assert_eq!(detect_language("x.py"), Some("python"));
        assert_eq!(detect_language("x.go"), Some("go"));
        assert_eq!(detect_language("x.txt"), None);
    }
}
