//! Markdown parsing, semantic chunking and wikilink utilities.
//!
//! Uses a real CommonMark AST (`pulldown-cmark`) so that:
//! - only real ATX/Setext headings (`# Title`, not `#tag`) create section boundaries;
//! - `#` comments, wikilinks and tags inside fenced/inline code are ignored;
//! - oversized sections are sub-split into overlapping windows that fit the embedding model.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use regex::Regex;
use std::collections::BTreeSet;
use std::ops::Range;
use std::sync::LazyLock;

static WIKILINK_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(!?)\[\[([^\[\]\n]+?)\]\]").expect("valid regex for wikilinks")
});

/// A tag must start with a letter or underscore, so `#123` (issue refs) and `##` are ignored.
static TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(?:^|[\s(,;])#([\p{L}_][\p{L}\p{N}_\-/]*)").expect("valid regex for tags")
});

/// Max tokens per chunk. MiniLM truncates at 256 word-pieces and E5 at 512; ~320 heuristic
/// tokens keeps every chunk fully inside the model window for both.
pub const MAX_CHUNK_TOKENS: usize = 320;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedChunk {
    pub chunk_index: usize,
    pub heading: Option<String>,
    pub content: String,
    pub token_estimate: usize,
    pub links: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedNote {
    pub title: String,
    pub summary: String,
    pub links: Vec<String>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub chunks: Vec<ParsedChunk>,
}

/// Splits optional YAML frontmatter. Returns (frontmatter_lines, body, body_offset_in_content).
fn split_frontmatter(content: &str) -> (Vec<&str>, &str, usize) {
    let mut lines = content.split_inclusive('\n');
    let first = match lines.next() {
        Some(l) => l,
        None => return (Vec::new(), content, 0),
    };
    if first.trim() != "---" {
        return (Vec::new(), content, 0);
    }
    let mut offset = first.len();
    let mut fm = Vec::new();
    for line in lines {
        offset += line.len();
        if line.trim() == "---" {
            return (fm, &content[offset..], offset);
        }
        fm.push(line.trim_end_matches(['\r', '\n']));
    }
    // Unterminated frontmatter: treat everything as body.
    (Vec::new(), content, 0)
}

fn clean_scalar(v: &str) -> String {
    v.trim().trim_matches('"').trim_matches('\'').trim().to_string()
}

/// Reads a scalar `key: value` from YAML-lite frontmatter.
fn frontmatter_scalar(fm: &[&str], key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    fm.iter().find_map(|l| {
        l.trim()
            .strip_prefix(&prefix)
            .map(clean_scalar)
            .filter(|v| !v.is_empty() && !v.starts_with('['))
    })
}

/// Reads a list from `key: [a, b]`, `key: a, b` or a block list (`- a`).
fn frontmatter_list(fm: &[&str], key: &str) -> Vec<String> {
    let prefix = format!("{key}:");
    let mut out = Vec::new();
    let mut i = 0;
    while i < fm.len() {
        let line = fm[i].trim();
        if let Some(rest) = line.strip_prefix(&prefix) {
            let rest = rest.trim();
            if rest.is_empty() {
                // Block list
                i += 1;
                while i < fm.len() {
                    let item = fm[i].trim();
                    if let Some(v) = item.strip_prefix('-') {
                        let v = clean_scalar(v);
                        if !v.is_empty() {
                            out.push(v);
                        }
                        i += 1;
                    } else {
                        break;
                    }
                }
                continue;
            }
            let inner = rest.trim_start_matches('[').trim_end_matches(']');
            for part in inner.split(',') {
                let v = clean_scalar(part);
                if !v.is_empty() {
                    out.push(v);
                }
            }
        }
        i += 1;
    }
    out
}

struct HeadingInfo {
    range: Range<usize>,
    level: HeadingLevel,
    text: String,
}

struct BodyScan {
    headings: Vec<HeadingInfo>,
    code_ranges: Vec<Range<usize>>,
    first_paragraph: Option<String>,
}

fn scan_body(body: &str) -> BodyScan {
    let mut headings = Vec::new();
    let mut code_ranges = Vec::new();
    let mut first_paragraph: Option<String> = None;

    let mut current_heading: Option<(Range<usize>, HeadingLevel, String)> = None;
    let mut paragraph_buf: Option<String> = None;
    let mut code_block_depth = 0usize;

    let parser = Parser::new_ext(body, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH);
    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                current_heading = Some((range, level, String::new()));
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((r, level, text)) = current_heading.take() {
                    headings.push(HeadingInfo {
                        range: r,
                        level,
                        text: text.trim().to_string(),
                    });
                }
            }
            Event::Start(Tag::CodeBlock(_)) => {
                code_block_depth += 1;
                code_ranges.push(range);
            }
            Event::End(TagEnd::CodeBlock) => {
                code_block_depth = code_block_depth.saturating_sub(1);
            }
            Event::Start(Tag::Paragraph) if first_paragraph.is_none() && current_heading.is_none() => {
                paragraph_buf = Some(String::new());
            }
            Event::End(TagEnd::Paragraph) => {
                if let Some(p) = paragraph_buf.take() {
                    let p = p.split_whitespace().collect::<Vec<_>>().join(" ");
                    if !p.is_empty() && !is_tag_only_line(&p) {
                        first_paragraph = Some(p);
                    }
                }
            }
            Event::Code(text) => {
                code_ranges.push(range);
                if let Some((_, _, h)) = current_heading.as_mut() {
                    h.push_str(&text);
                }
                if let Some(p) = paragraph_buf.as_mut() {
                    p.push_str(&text);
                }
            }
            Event::Text(text) if code_block_depth == 0 => {
                if let Some((_, _, h)) = current_heading.as_mut() {
                    h.push_str(&text);
                }
                if let Some(p) = paragraph_buf.as_mut() {
                    p.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(p) = paragraph_buf.as_mut() {
                    p.push(' ');
                }
            }
            _ => {}
        }
    }

    BodyScan {
        headings,
        code_ranges,
        first_paragraph,
    }
}

fn in_ranges(pos: usize, ranges: &[Range<usize>]) -> bool {
    ranges.iter().any(|r| r.contains(&pos))
}

/// Returns true when a line only contains hashtags (e.g. `#rust #mcp` or `Tags: #a #b`).
fn is_tag_only_line(line: &str) -> bool {
    let l = line.trim();
    let l = l
        .strip_prefix("Tags:")
        .or_else(|| l.strip_prefix("tags:"))
        .unwrap_or(l)
        .trim();
    !l.is_empty() && l.split_whitespace().all(|w| w.starts_with('#') && w.len() > 1)
}

/// Normalizes a raw wikilink body (`Note#Section|Alias`) to its link target (`Note#Section`).
/// Returns None for pure self-references like `[[#Section]]`.
fn normalize_link_target(raw: &str) -> Option<String> {
    let target = raw.split_once('|').map(|(t, _)| t).unwrap_or(raw).trim();
    if target.is_empty() || target.starts_with('#') {
        return None;
    }
    Some(target.to_string())
}

fn extract_links(text: &str, offset: usize, code: &[Range<usize>]) -> BTreeSet<String> {
    WIKILINK_RE
        .captures_iter(text)
        .filter(|c| !in_ranges(offset + c.get(0).map(|m| m.start()).unwrap_or(0), code))
        .filter_map(|c| c.get(2).and_then(|m| normalize_link_target(m.as_str())))
        .collect()
}

fn extract_tags(text: &str, offset: usize, code: &[Range<usize>]) -> BTreeSet<String> {
    TAG_RE
        .captures_iter(text)
        .filter_map(|c| c.get(1))
        .filter(|m| !in_ranges(offset + m.start(), code))
        .map(|m| m.as_str().trim_end_matches(['-', '/']).to_lowercase())
        .filter(|t| !t.is_empty())
        .collect()
}

pub fn estimate_tokens(text: &str) -> usize {
    let chars = text.chars().count();
    if chars == 0 {
        return 0;
    }
    let words = text.split_whitespace().count();
    // Max of the char heuristic (~4 chars/token) and word heuristic (~0.75 words/token);
    // the word heuristic is more accurate for accented / non-English text.
    chars.div_ceil(4).max((words * 4).div_ceil(3)).max(1)
}

/// Splits text into paragraphs on blank lines, never breaking inside fenced code.
fn split_paragraphs(text: &str) -> Vec<String> {
    let mut paras = Vec::new();
    let mut current: Vec<&str> = Vec::new();
    let mut in_fence = false;
    for line in text.lines() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
        }
        if line.trim().is_empty() && !in_fence {
            if !current.is_empty() {
                paras.push(current.join("\n"));
                current.clear();
            }
        } else {
            current.push(line);
        }
    }
    if !current.is_empty() {
        paras.push(current.join("\n"));
    }
    paras
}

/// Hard-splits a single oversized paragraph on whitespace.
fn hard_split(text: &str, max_tokens: usize) -> Vec<String> {
    let max_chars = max_tokens * 3;
    let mut out = Vec::new();
    let mut buf = String::new();
    for word in text.split_inclusive(char::is_whitespace) {
        if buf.chars().count() + word.chars().count() > max_chars && !buf.trim().is_empty() {
            out.push(buf.trim().to_string());
            buf.clear();
        }
        buf.push_str(word);
    }
    if !buf.trim().is_empty() {
        out.push(buf.trim().to_string());
    }
    out
}

/// Splits a section into windows of at most `max_tokens`, overlapping by one paragraph.
pub fn window_section(content: &str, max_tokens: usize) -> Vec<String> {
    if estimate_tokens(content) <= max_tokens {
        return vec![content.to_string()];
    }
    let mut paras: Vec<String> = Vec::new();
    for p in split_paragraphs(content) {
        if estimate_tokens(&p) > max_tokens {
            paras.extend(hard_split(&p, max_tokens));
        } else {
            paras.push(p);
        }
    }

    let mut windows = Vec::new();
    let mut current: Vec<String> = Vec::new();
    let mut current_tokens = 0;
    for p in paras {
        let pt = estimate_tokens(&p);
        if current_tokens + pt > max_tokens && !current.is_empty() {
            windows.push(current.join("\n\n"));
            // Overlap: carry the last paragraph forward if it is small enough.
            let last = current.pop().unwrap_or_default();
            current.clear();
            current_tokens = 0;
            let lt = estimate_tokens(&last);
            if lt <= max_tokens / 3 && lt + pt <= max_tokens {
                current.push(last);
                current_tokens = lt;
            }
        }
        current_tokens += pt;
        current.push(p);
    }
    if !current.is_empty() {
        windows.push(current.join("\n\n"));
    }
    windows
}

pub fn parse_markdown(filename_stem: &str, content: &str) -> ParsedNote {
    let (fm, body, _) = split_frontmatter(content);
    let scan = scan_body(body);

    // 1. Title: frontmatter > first H1 > filename
    let title = frontmatter_scalar(&fm, "title")
        .or_else(|| {
            scan.headings
                .iter()
                .find(|h| h.level == HeadingLevel::H1 && !h.text.is_empty())
                .map(|h| h.text.clone())
        })
        .unwrap_or_else(|| filename_stem.to_string());

    // 2. Note-level links & tags (code excluded)
    let links = extract_links(body, 0, &scan.code_ranges);
    let mut tags = extract_tags(body, 0, &scan.code_ranges);
    for t in frontmatter_list(&fm, "tags") {
        let t = t.trim_start_matches('#').to_lowercase();
        if !t.is_empty() {
            tags.insert(t);
        }
    }
    let aliases = frontmatter_list(&fm, "aliases");

    // 3. Section boundaries: H1-H3 split chunks; H4+ stay inside their parent section.
    let boundaries: Vec<&HeadingInfo> = scan
        .headings
        .iter()
        .filter(|h| matches!(h.level, HeadingLevel::H1 | HeadingLevel::H2 | HeadingLevel::H3))
        .collect();

    let mut sections: Vec<(Option<String>, Range<usize>)> = Vec::new();
    let first_start = boundaries.first().map(|b| b.range.start).unwrap_or(body.len());
    if first_start > 0 {
        sections.push((None, 0..first_start));
    }
    for (i, b) in boundaries.iter().enumerate() {
        let end = boundaries
            .get(i + 1)
            .map(|n| n.range.start)
            .unwrap_or(body.len());
        let heading = (!b.text.is_empty()).then(|| b.text.clone());
        sections.push((heading, b.range.end.min(end)..end));
    }

    let mut chunks = Vec::new();
    for (heading, range) in sections {
        let raw = &body[range.clone()];
        let section_text = raw.trim();
        if section_text.is_empty() || is_tag_only_line(section_text) {
            continue;
        }
        let lead = raw.len() - raw.trim_start().len();
        let section_offset = range.start + lead;
        let section_links = extract_links(section_text, section_offset, &scan.code_ranges);
        let section_tags = extract_tags(section_text, section_offset, &scan.code_ranges);

        for window in window_section(section_text, MAX_CHUNK_TOKENS) {
            // Only keep links/tags that actually occur in this window.
            let w_links: Vec<String> = section_links
                .iter()
                .filter(|l| window.contains(l.as_str()))
                .cloned()
                .collect();
            let w_tags: Vec<String> = section_tags
                .iter()
                .filter(|t| window.to_lowercase().contains(&format!("#{t}")))
                .cloned()
                .collect();
            chunks.push(ParsedChunk {
                chunk_index: chunks.len(),
                heading: heading.clone(),
                token_estimate: estimate_tokens(&window),
                content: window,
                links: w_links,
                tags: w_tags,
            });
        }
    }

    let summary = scan
        .first_paragraph
        .map(|p| truncate_chars(&p, 200))
        .unwrap_or_default();

    ParsedNote {
        title,
        summary,
        links: links.into_iter().collect(),
        tags: tags.into_iter().collect(),
        aliases,
        chunks,
    }
}

pub fn truncate_chars(s: &str, max: usize) -> String {
    if s.chars().count() <= max {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(max).collect();
        out.push_str("...");
        out
    }
}

/// Stable 64-bit FNV-1a hash (hex). Used for change detection; stable across Rust versions.
pub fn content_hash(text: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in text.as_bytes() {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

/// Rewrites every `[[old]]`, `[[old#sec]]`, `[[old|alias]]` (case-insensitive, outside code)
/// to point at `new_title`. Returns the new content and the number of links rewritten.
pub fn rewrite_wikilinks(content: &str, old_title: &str, new_title: &str) -> (String, usize) {
    let (_, body, body_offset) = split_frontmatter(content);
    let code = scan_body(body).code_ranges;
    let old_lower = old_title.trim().to_lowercase();
    let mut count = 0;
    let mut out = String::with_capacity(content.len());
    let mut last = 0;

    for cap in WIKILINK_RE.captures_iter(content) {
        let whole = cap.get(0).expect("match");
        if whole.start() < body_offset || in_ranges(whole.start() - body_offset, &code) {
            continue;
        }
        let bang = cap.get(1).map(|m| m.as_str()).unwrap_or("");
        let inner = cap.get(2).map(|m| m.as_str()).unwrap_or("");
        let (target, alias) = match inner.split_once('|') {
            Some((t, a)) => (t, Some(a)),
            None => (inner, None),
        };
        let (base, section) = match target.split_once('#') {
            Some((b, s)) => (b, Some(s)),
            None => (target, None),
        };
        if base.trim().to_lowercase() != old_lower {
            continue;
        }
        let mut replacement = format!("{bang}[[{new_title}");
        if let Some(s) = section {
            replacement.push('#');
            replacement.push_str(s);
        }
        if let Some(a) = alias {
            replacement.push('|');
            replacement.push_str(a);
        }
        replacement.push_str("]]");
        out.push_str(&content[last..whole.start()]);
        out.push_str(&replacement);
        last = whole.end();
        count += 1;
    }
    out.push_str(&content[last..]);
    (out, count)
}

/// Sets the note title: updates frontmatter `title:` if present, else the first H1,
/// else prepends a new H1.
pub fn replace_title(content: &str, new_title: &str) -> String {
    let (fm, body, body_offset) = split_frontmatter(content);
    if frontmatter_scalar(&fm, "title").is_some() {
        let head = &content[..body_offset];
        let new_head: String = head
            .split_inclusive('\n')
            .map(|line| {
                if line.trim_start().starts_with("title:") {
                    let nl = if line.ends_with("\r\n") {
                        "\r\n"
                    } else if line.ends_with('\n') {
                        "\n"
                    } else {
                        ""
                    };
                    format!("title: {new_title}{nl}")
                } else {
                    line.to_string()
                }
            })
            .collect();
        return format!("{new_head}{body}");
    }

    let scan = scan_body(body);
    if let Some(h1) = scan.headings.iter().find(|h| h.level == HeadingLevel::H1) {
        let line_end = body[h1.range.start..]
            .find('\n')
            .map(|i| h1.range.start + i)
            .unwrap_or(body.len());
        let line = &body[h1.range.start..line_end];
        if line.trim_start().starts_with('#') {
            let cr = if line.ends_with('\r') { "\r" } else { "" };
            return format!(
                "{}{}# {new_title}{cr}{}",
                &content[..body_offset],
                &body[..h1.range.start],
                &body[line_end..]
            );
        }
    }
    format!("{}# {new_title}\n\n{body}", &content[..body_offset])
}

/// Finds the first plain-text (unlinked, outside code) mention of `title`.
/// Returns the byte range of the mention.
pub fn find_unlinked_mention(content: &str, title: &str) -> Option<Range<usize>> {
    let title = title.trim();
    if title.chars().count() < 3 {
        return None;
    }
    let (_, body, body_offset) = split_frontmatter(content);
    let scan = scan_body(body);
    let mut excluded = scan.code_ranges.clone();
    for m in WIKILINK_RE.find_iter(body) {
        excluded.push(m.range());
    }
    for h in &scan.headings {
        if h.level == HeadingLevel::H1 {
            excluded.push(h.range.clone());
        }
    }
    let pattern = format!(r"(?i)(?:^|[^\p{{L}}\p{{N}}_])({})(?:[^\p{{L}}\p{{N}}_]|$)", regex::escape(title));
    let re = Regex::new(&pattern).ok()?;
    let mut start = 0;
    while start < body.len() {
        let cap = re.captures_at(body, start)?;
        let m = cap.get(1)?;
        if !in_ranges(m.start(), &excluded) {
            return Some(body_offset + m.start()..body_offset + m.end());
        }
        start = m.end();
    }
    None
}

/// Converts the first unlinked mention of `title` into a `[[title]]` wikilink.
pub fn link_first_mention(content: &str, title: &str) -> Option<String> {
    let r = find_unlinked_mention(content, title)?;
    let original = &content[r.clone()];
    let link = if original == title {
        format!("[[{title}]]")
    } else {
        format!("[[{title}|{original}]]")
    };
    Some(format!("{}{}{}", &content[..r.start], link, &content[r.end..]))
}

/// Builds a note body with YAML frontmatter (used by MCP `store_context`).
pub fn compose_note(title: &str, body: &str, tags: &[String], aliases: &[String]) -> String {
    let mut out = String::from("---\n");
    out.push_str(&format!("title: {}\n", title.trim()));
    let clean_tags: Vec<String> = tags
        .iter()
        .map(|t| t.trim().trim_start_matches('#').to_lowercase())
        .filter(|t| !t.is_empty())
        .collect();
    if !clean_tags.is_empty() {
        out.push_str(&format!("tags: [{}]\n", clean_tags.join(", ")));
    }
    if !aliases.is_empty() {
        out.push_str(&format!("aliases: [{}]\n", aliases.join(", ")));
    }
    out.push_str("---\n");
    let body = body.trim_start();
    let has_h1 = scan_body(body)
        .headings
        .iter()
        .any(|h| h.level == HeadingLevel::H1);
    if !has_h1 {
        out.push_str(&format!("# {}\n\n", title.trim()));
    }
    out.push_str(body);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_chunking() {
        let content = r#"# Nota Principal
Texto de introdução da nota.

## Seção de Rust
Rust é uma linguagem de sistemas focada em segurança.
Possui ligação com [[Linguagens]]. #rust

## Seção de IA
Inteligência artificial moderna com embeddings e RAG. #ia
"#;
        let parsed = parse_markdown("teste", content);
        assert_eq!(parsed.title, "Nota Principal");
        assert_eq!(parsed.summary, "Texto de introdução da nota.");
        assert_eq!(parsed.chunks.len(), 3);
        assert_eq!(parsed.chunks[0].heading.as_deref(), Some("Nota Principal"));
        assert_eq!(parsed.chunks[1].heading.as_deref(), Some("Seção de Rust"));
        assert_eq!(parsed.chunks[1].tags, vec!["rust"]);
        assert_eq!(parsed.chunks[1].links, vec!["Linguagens"]);
        assert_eq!(parsed.chunks[2].heading.as_deref(), Some("Seção de IA"));
        assert_eq!(parsed.chunks[2].tags, vec!["ia"]);
    }

    #[test]
    fn tag_lines_are_not_headings() {
        let content = "# Title\n\nBody text here.\n\n#architecture #mcp\n";
        let parsed = parse_markdown("t", content);
        assert_eq!(parsed.chunks.len(), 1);
        assert_eq!(parsed.chunks[0].heading.as_deref(), Some("Title"));
        assert_eq!(parsed.tags, vec!["architecture", "mcp"]);
    }

    #[test]
    fn code_blocks_are_ignored_for_headings_links_and_tags() {
        let content = "# Script\n\nIntro.\n\n```bash\n# not a heading\necho [[NotALink]] #nottag\n```\n\nReal [[Link]] #realtag\n";
        let parsed = parse_markdown("s", content);
        assert_eq!(parsed.chunks.len(), 1);
        assert_eq!(parsed.links, vec!["Link"]);
        assert_eq!(parsed.tags, vec!["realtag"]);
        assert!(parsed.chunks[0].content.contains("# not a heading"));
    }

    #[test]
    fn frontmatter_title_tags_and_aliases() {
        let content = "---\ntitle: \"FM Title\"\ntags:\n  - alpha\n  - Beta\naliases: [One, Two]\n---\n# Ignored H1\n\nBody.\n";
        let parsed = parse_markdown("f", content);
        assert_eq!(parsed.title, "FM Title");
        assert_eq!(parsed.tags, vec!["alpha", "beta"]);
        assert_eq!(parsed.aliases, vec!["One", "Two"]);
    }

    #[test]
    fn wikilink_alias_section_and_self_links() {
        let content = "# A\n\nSee [[B|bee]], [[C#Sec]], ![[D]] and [[#Local]].\n";
        let parsed = parse_markdown("a", content);
        assert_eq!(parsed.links, vec!["B", "C#Sec", "D"]);
    }

    #[test]
    fn issue_refs_and_hex_like_numbers_are_not_tags() {
        let parsed = parse_markdown("x", "# X\n\nFixes #123 and #4f, keeps #ok_tag.\n");
        assert_eq!(parsed.tags, vec!["ok_tag"]);
    }

    #[test]
    fn oversized_sections_are_windowed() {
        let para = "palavra ".repeat(120);
        let content = format!("# Big\n\n## Huge\n\n{para}\n\n{para}\n\n{para}\n\n{para}\n");
        let parsed = parse_markdown("b", &content);
        assert!(parsed.chunks.len() >= 2, "expected sub-split, got {}", parsed.chunks.len());
        for c in &parsed.chunks {
            assert!(c.token_estimate <= MAX_CHUNK_TOKENS, "chunk too large: {}", c.token_estimate);
            assert_eq!(c.heading.as_deref(), Some("Huge"));
        }
        let idx: Vec<usize> = parsed.chunks.iter().map(|c| c.chunk_index).collect();
        assert_eq!(idx, (0..parsed.chunks.len()).collect::<Vec<_>>());
    }

    #[test]
    fn rewrite_links_preserves_sections_and_aliases() {
        let content = "# N\n\n[[Old]] [[old#S1]] [[Old|x]] [[Other]]\n\n`[[Old]]`\n";
        let (out, n) = rewrite_wikilinks(content, "Old", "New");
        assert_eq!(n, 3);
        assert!(out.contains("[[New]] [[New#S1]] [[New|x]] [[Other]]"));
        assert!(out.contains("`[[Old]]`"));
    }

    #[test]
    fn replace_title_variants() {
        assert_eq!(replace_title("# Old\n\nBody\n", "New"), "# New\n\nBody\n");
        assert_eq!(
            replace_title("---\ntitle: Old\n---\nBody\n", "New"),
            "---\ntitle: New\n---\nBody\n"
        );
        assert_eq!(replace_title("Body\n", "New"), "# New\n\nBody\n");
    }

    #[test]
    fn unlinked_mentions() {
        let content = "# Me\n\nWe use SQLite here. Also [[SQLite]] linked.\n";
        let r = find_unlinked_mention(content, "sqlite").expect("mention");
        assert_eq!(&content[r], "SQLite");
        let linked = link_first_mention(content, "SQLite").expect("linked");
        assert!(linked.contains("We use [[SQLite]] here."));
        assert!(find_unlinked_mention("# X\n\n[[SQLite]] only\n", "SQLite").is_none());
        assert!(find_unlinked_mention("# X\n\nSQLiteX word\n", "SQLite").is_none());
    }

    #[test]
    fn compose_note_adds_frontmatter_and_h1() {
        let out = compose_note("My Note", "Body text.", &["#Rust".into()], &[]);
        assert!(out.starts_with("---\ntitle: My Note\ntags: [rust]\n---\n# My Note\n\nBody text."));
        let parsed = parse_markdown("my-note", &out);
        assert_eq!(parsed.title, "My Note");
        assert_eq!(parsed.tags, vec!["rust"]);
    }

    #[test]
    fn hash_is_stable() {
        assert_eq!(content_hash("abc"), "e71fa2190541574b");
    }
}
