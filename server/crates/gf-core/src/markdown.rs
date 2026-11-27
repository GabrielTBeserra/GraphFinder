//! Markdown parsing and semantic chunking using a CommonMark AST.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};
use regex::Regex;
use std::collections::BTreeSet;
use std::ops::Range;
use std::sync::LazyLock;

static WIKILINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"!?\[\[([^\[\]\n]+?)\]\]").expect("valid wikilink regex"));
static TAG_RE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(?:^|[\s(,;])#([\p{L}_][\p{L}\p{N}_\-/]*)").expect("valid tag regex")
});

pub const MAX_CHUNK_TOKENS: usize = 320;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownChunk {
    pub chunk_index: usize,
    pub heading: Option<String>,
    pub content: String,
    pub token_estimate: usize,
    pub links: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownNote {
    pub title: String,
    pub summary: String,
    pub links: Vec<String>,
    pub tags: Vec<String>,
    pub aliases: Vec<String>,
    pub chunks: Vec<MarkdownChunk>,
}

#[derive(Debug)]
struct Heading {
    range: Range<usize>,
    level: HeadingLevel,
    text: String,
}

#[derive(Debug, Default)]
struct Scan {
    headings: Vec<Heading>,
    code_ranges: Vec<Range<usize>>,
    first_paragraph: Option<String>,
}

pub fn parse_markdown(filename_stem: &str, content: &str) -> MarkdownNote {
    let (frontmatter, body) = split_frontmatter(content);
    let scan = scan_body(body);
    let title = frontmatter_value(&frontmatter, "title")
        .or_else(|| {
            scan.headings
                .iter()
                .find(|h| h.level == HeadingLevel::H1 && !h.text.is_empty())
                .map(|h| h.text.clone())
        })
        .unwrap_or_else(|| filename_stem.to_string());

    let mut tags = extract_tags(body, 0, &scan.code_ranges);
    for tag in frontmatter_list(&frontmatter, "tags") {
        let tag = tag.trim_start_matches('#').to_lowercase();
        if !tag.is_empty() {
            tags.insert(tag);
        }
    }
    let links = extract_links(body, 0, &scan.code_ranges);

    let boundaries: Vec<&Heading> = scan
        .headings
        .iter()
        .filter(|h| {
            matches!(
                h.level,
                HeadingLevel::H1 | HeadingLevel::H2 | HeadingLevel::H3
            )
        })
        .collect();
    let mut sections = Vec::new();
    let first = boundaries
        .first()
        .map(|h| h.range.start)
        .unwrap_or(body.len());
    if first > 0 {
        sections.push((None, 0..first));
    }
    for (i, heading) in boundaries.iter().enumerate() {
        let end = boundaries
            .get(i + 1)
            .map(|h| h.range.start)
            .unwrap_or(body.len());
        sections.push((
            (!heading.text.is_empty()).then(|| heading.text.clone()),
            heading.range.end.min(end)..end,
        ));
    }

    let mut chunks = Vec::new();
    for (heading, range) in sections {
        let text = body[range.clone()].trim();
        if text.is_empty() || is_tag_only_line(text) {
            continue;
        }
        for window in window_section(text, MAX_CHUNK_TOKENS) {
            chunks.push(MarkdownChunk {
                chunk_index: chunks.len(),
                heading: heading.clone(),
                token_estimate: estimate_tokens(&window),
                links: extract_links(&window, range.start, &scan.code_ranges)
                    .into_iter()
                    .collect(),
                tags: extract_tags(&window, range.start, &scan.code_ranges)
                    .into_iter()
                    .collect(),
                content: window,
            });
        }
    }

    MarkdownNote {
        title,
        summary: scan
            .first_paragraph
            .map(|p| truncate_chars(&p, 200))
            .unwrap_or_default(),
        links: links.into_iter().collect(),
        tags: tags.into_iter().collect(),
        aliases: frontmatter_list(&frontmatter, "aliases"),
        chunks,
    }
}

fn scan_body(body: &str) -> Scan {
    let mut scan = Scan::default();
    let mut heading: Option<(Range<usize>, HeadingLevel, String)> = None;
    let mut paragraph: Option<String> = None;
    let mut code_depth = 0usize;
    let parser = Parser::new_ext(body, Options::ENABLE_TABLES | Options::ENABLE_STRIKETHROUGH);
    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::Heading { level, .. }) => {
                heading = Some((range, level, String::new()))
            }
            Event::End(TagEnd::Heading(_)) => {
                if let Some((range, level, text)) = heading.take() {
                    scan.headings.push(Heading {
                        range,
                        level,
                        text: text.trim().into(),
                    });
                }
            }
            Event::Start(Tag::CodeBlock(_)) => {
                code_depth += 1;
                scan.code_ranges.push(range);
            }
            Event::End(TagEnd::CodeBlock) => code_depth = code_depth.saturating_sub(1),
            Event::Start(Tag::Paragraph) if scan.first_paragraph.is_none() && heading.is_none() => {
                paragraph = Some(String::new())
            }
            Event::End(TagEnd::Paragraph) => {
                if let Some(text) = paragraph.take() {
                    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
                    if !text.is_empty() && !is_tag_only_line(&text) {
                        scan.first_paragraph = Some(text);
                    }
                }
            }
            Event::Code(text) => {
                scan.code_ranges.push(range);
                if let Some((_, _, h)) = heading.as_mut() {
                    h.push_str(&text);
                }
                if let Some(p) = paragraph.as_mut() {
                    p.push_str(&text);
                }
            }
            Event::Text(text) if code_depth == 0 => {
                if let Some((_, _, h)) = heading.as_mut() {
                    h.push_str(&text);
                }
                if let Some(p) = paragraph.as_mut() {
                    p.push_str(&text);
                }
            }
            Event::SoftBreak | Event::HardBreak => {
                if let Some(p) = paragraph.as_mut() {
                    p.push(' ');
                }
            }
            _ => {}
        }
    }
    scan
}

fn split_frontmatter(content: &str) -> (Vec<&str>, &str) {
    let mut lines = content.split_inclusive('\n');
    let Some(first) = lines.next() else {
        return (vec![], content);
    };
    if first.trim() != "---" {
        return (vec![], content);
    }
    let mut offset = first.len();
    let mut frontmatter = vec![];
    for line in lines {
        offset += line.len();
        if line.trim() == "---" {
            return (frontmatter, &content[offset..]);
        }
        frontmatter.push(line.trim_end_matches(['\r', '\n']));
    }
    (vec![], content)
}

fn frontmatter_value(frontmatter: &[&str], key: &str) -> Option<String> {
    let prefix = format!("{key}:");
    frontmatter.iter().find_map(|line| {
        line.trim()
            .strip_prefix(&prefix)
            .map(|v| v.trim().trim_matches('"').to_string())
            .filter(|v| !v.is_empty() && !v.starts_with('['))
    })
}

fn frontmatter_list(frontmatter: &[&str], key: &str) -> Vec<String> {
    let prefix = format!("{key}:");
    let mut out = vec![];
    for line in frontmatter {
        if let Some(rest) = line.trim().strip_prefix(&prefix) {
            let inner = rest.trim().trim_start_matches('[').trim_end_matches(']');
            out.extend(
                inner
                    .split(',')
                    .map(|v| v.trim().trim_matches('"').to_string())
                    .filter(|v| !v.is_empty()),
            );
        }
    }
    out
}

fn extract_links(text: &str, offset: usize, code: &[Range<usize>]) -> BTreeSet<String> {
    WIKILINK_RE
        .captures_iter(text)
        .filter(|c| !in_ranges(offset + c.get(0).map(|m| m.start()).unwrap_or(0), code))
        .filter_map(|c| c.get(1).map(|m| m.as_str()))
        .filter_map(|raw| {
            let target = raw.split_once('|').map(|(t, _)| t).unwrap_or(raw).trim();
            (!target.is_empty() && !target.starts_with('#')).then(|| target.to_string())
        })
        .collect()
}

fn extract_tags(text: &str, offset: usize, code: &[Range<usize>]) -> BTreeSet<String> {
    TAG_RE
        .captures_iter(text)
        .filter_map(|c| c.get(1))
        .filter(|m| !in_ranges(offset + m.start(), code))
        .map(|m| m.as_str().trim_end_matches(['-', '/']).to_lowercase())
        .filter(|tag| !tag.is_empty())
        .collect()
}

fn in_ranges(position: usize, ranges: &[Range<usize>]) -> bool {
    ranges.iter().any(|range| range.contains(&position))
}

fn is_tag_only_line(line: &str) -> bool {
    let line = line
        .trim()
        .strip_prefix("Tags:")
        .unwrap_or(line.trim())
        .trim();
    !line.is_empty()
        && line
            .split_whitespace()
            .all(|w| w.starts_with('#') && w.len() > 1)
}

pub fn estimate_tokens(text: &str) -> usize {
    let chars = text.chars().count();
    if chars == 0 {
        return 0;
    }
    let words = text.split_whitespace().count();
    chars.div_ceil(4).max((words * 4).div_ceil(3)).max(1)
}

pub fn window_section(content: &str, max_tokens: usize) -> Vec<String> {
    if estimate_tokens(content) <= max_tokens {
        return vec![content.to_string()];
    }
    let mut windows = vec![];
    let mut current = vec![];
    let mut current_tokens = 0;
    for paragraph in content.split("\n\n") {
        let tokens = estimate_tokens(paragraph);
        if current_tokens + tokens > max_tokens && !current.is_empty() {
            windows.push(current.join("\n\n"));
            current.clear();
            current_tokens = 0;
        }
        current_tokens += tokens;
        current.push(paragraph);
    }
    if !current.is_empty() {
        windows.push(current.join("\n\n"));
    }
    windows
}

pub fn truncate_chars(value: &str, max_chars: usize) -> String {
    if value.chars().count() <= max_chars {
        value.to_string()
    } else {
        let mut out: String = value.chars().take(max_chars).collect();
        out.push_str("...");
        out
    }
}

pub fn content_hash(text: &str) -> String {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[cfg(test)]
mod tests {
    use super::*;
    const FIXTURE: &str = include_str!("../../../fixtures/markdown/semantic-note.md");

    #[test]
    fn parses_frontmatter_title_tags_and_aliases() {
        let note = parse_markdown("semantic-note", FIXTURE);
        assert_eq!(note.title, "Semantic Indexing");
        assert!(note.tags.contains(&"rag".to_string()));
        assert!(note.tags.contains(&"architecture".to_string()));
        assert_eq!(note.aliases, vec!["semantic-index"]);
    }

    #[test]
    fn extracts_links_and_ignores_code_blocks() {
        let note = parse_markdown("semantic-note", FIXTURE);
        assert!(note.links.contains(&"Graph Model".to_string()));
        assert!(note.links.contains(&"Atomic Notes#Headers".to_string()));
        assert!(!note.links.contains(&"Fake Link".to_string()));
        assert!(!note.tags.contains(&"fake".to_string()));
    }

    #[test]
    fn chunks_by_h1_h2_h3_but_not_h4() {
        let note = parse_markdown("semantic-note", FIXTURE);
        assert_eq!(note.chunks.len(), 4);
        assert_eq!(note.chunks[0].heading.as_deref(), Some("Semantic Indexing"));
        assert_eq!(note.chunks[1].heading.as_deref(), Some("1. Chunking Rules"));
        assert_eq!(note.chunks[2].heading.as_deref(), Some("1.1 Deep Links"));
        assert!(note.chunks[2].content.contains("This H4 heading"));
        assert_eq!(
            note.chunks[3].heading.as_deref(),
            Some("2. Operational Concerns")
        );
    }

    #[test]
    fn extracts_summary_and_splits_large_sections() {
        let note = parse_markdown("semantic-note", FIXTURE);
        assert!(note.summary.starts_with("Semantic indexing splits"));
        let repeated = "alpha beta gamma delta\n\n".repeat(100);
        assert!(window_section(&repeated, 100).len() > 1);
    }

    #[test]
    fn content_hash_is_stable() {
        assert_eq!(content_hash("graphfinder"), content_hash("graphfinder"));
        assert_ne!(content_hash("graphfinder"), content_hash("server"));
    }
}
