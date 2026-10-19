use regex::Regex;
use std::collections::HashSet;
use std::sync::LazyLock;

static WIKILINK_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"\[\[(.*?)\]\]").expect("valid regex for wikilinks"));

static TAG_RE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?:^|\s)#([a-zA-Z0-9_\-\p{L}]+)").expect("valid regex for tags"));

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
    pub chunks: Vec<ParsedChunk>,
}

pub fn parse_markdown(filename_stem: &str, content: &str) -> ParsedNote {
    // 1. Separate frontmatter from body
    let mut title = String::new();
    let mut _in_frontmatter = false;
    let mut frontmatter_lines: Vec<&str> = Vec::new();
    let mut body_lines: Vec<&str> = Vec::new();

    let lines: Vec<&str> = content.lines().collect();
    let mut i = 0;

    if lines.first().is_some_and(|l| l.trim() == "---") {
        _in_frontmatter = true;
        i = 1;
        while i < lines.len() {
            let line = lines[i];
            if line.trim() == "---" {
                _in_frontmatter = false;
                i += 1;
                break;
            }
            frontmatter_lines.push(line);
            i += 1;
        }
    }

    while i < lines.len() {
        body_lines.push(lines[i]);
        i += 1;
    }

    // Check frontmatter for title
    for line in &frontmatter_lines {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("title:") {
            let t = rest.trim().trim_matches('"').trim_matches('\'').trim();
            if !t.is_empty() {
                title = t.to_string();
                break;
            }
        }
    }

    // If no frontmatter title, search for first # Heading
    if title.is_empty() {
        for line in &body_lines {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix('#') {
                if !rest.starts_with('#') {
                    let t = rest.trim();
                    if !t.is_empty() {
                        title = t.to_string();
                        break;
                    }
                }
            }
        }
    }

    // Fallback to filename stem
    if title.is_empty() {
        title = filename_stem.to_string();
    }

    // 2. Global note-level wikilinks
    let mut note_links_set = HashSet::new();
    for cap in WIKILINK_RE.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            let raw = m.as_str().trim();
            let target = match raw.split_once('|') {
                Some((target, _alias)) => target.trim(),
                None => raw,
            };
            if !target.is_empty() {
                note_links_set.insert(target.to_string());
            }
        }
    }

    // 3. Global note-level tags
    let mut note_tags_set = HashSet::new();
    for line in &frontmatter_lines {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("tags:") {
            let rest = rest.trim().trim_matches('[').trim_matches(']');
            for part in rest.split(',') {
                let tag = part
                    .trim()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .trim_start_matches('#')
                    .trim();
                if !tag.is_empty() {
                    note_tags_set.insert(tag.to_lowercase());
                }
            }
        }
    }

    for cap in TAG_RE.captures_iter(content) {
        if let Some(m) = cap.get(1) {
            let tag = m.as_str().trim().to_lowercase();
            if !tag.is_empty() {
                note_tags_set.insert(tag);
            }
        }
    }

    // 4. Section & Paragraph Chunking for Token Efficiency
    let mut raw_chunks: Vec<(Option<String>, Vec<&str>)> = Vec::new();
    let mut current_heading: Option<String> = None;
    let mut current_lines: Vec<&str> = Vec::new();

    for line in &body_lines {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            let heading_text = trimmed.trim_start_matches('#').trim().to_string();
            // If current lines already have meaningful text, push as chunk
            if !current_lines.is_empty() {
                raw_chunks.push((current_heading.clone(), current_lines));
                current_lines = Vec::new();
            }
            current_heading = Some(heading_text);
        } else {
            current_lines.push(line);
        }
    }

    if !current_lines.is_empty() {
        raw_chunks.push((current_heading, current_lines));
    }

    // If note had no headings at all, raw_chunks has 1 item or is empty
    if raw_chunks.is_empty() && !content.trim().is_empty() {
        raw_chunks.push((None, body_lines.clone()));
    }

    let mut chunks = Vec::new();
    for (idx, (heading, clines)) in raw_chunks.into_iter().enumerate() {
        let chunk_content = clines.join("\n").trim().to_string();
        if chunk_content.is_empty() && heading.is_none() {
            continue;
        }

        // Estimate token count (~1 token per 4 chars or 0.75 words)
        let token_estimate = estimate_tokens(&chunk_content);

        // Extract chunk-level wikilinks
        let mut chunk_links = HashSet::new();
        for cap in WIKILINK_RE.captures_iter(&chunk_content) {
            if let Some(m) = cap.get(1) {
                let raw = m.as_str().trim();
                let target = match raw.split_once('|') {
                    Some((target, _alias)) => target.trim(),
                    None => raw,
                };
                if !target.is_empty() {
                    chunk_links.insert(target.to_string());
                }
            }
        }

        // Extract chunk-level tags
        let mut chunk_tags = HashSet::new();
        for cap in TAG_RE.captures_iter(&chunk_content) {
            if let Some(m) = cap.get(1) {
                let tag = m.as_str().trim().to_lowercase();
                if !tag.is_empty() {
                    chunk_tags.insert(tag);
                }
            }
        }

        let mut sorted_links: Vec<String> = chunk_links.into_iter().collect();
        sorted_links.sort();

        let mut sorted_tags: Vec<String> = chunk_tags.into_iter().collect();
        sorted_tags.sort();

        chunks.push(ParsedChunk {
            chunk_index: idx,
            heading,
            content: chunk_content,
            token_estimate,
            links: sorted_links,
            tags: sorted_tags,
        });
    }

    // Compute dense note summary (1-2 sentences for compressed context)
    let summary = extract_summary(&body_lines);

    let mut sorted_note_links: Vec<String> = note_links_set.into_iter().collect();
    sorted_note_links.sort();

    let mut sorted_note_tags: Vec<String> = note_tags_set.into_iter().collect();
    sorted_note_tags.sort();

    ParsedNote {
        title,
        summary,
        links: sorted_note_links,
        tags: sorted_note_tags,
        chunks,
    }
}

fn estimate_tokens(text: &str) -> usize {
    let char_count = text.chars().count();
    if char_count == 0 {
        return 0;
    }
    // Standard rule of thumb: ~4 characters per token in English/Portuguese, min 1
    (char_count / 4).max(1)
}

fn extract_summary(lines: &[&str]) -> String {
    let mut clean_sentences = Vec::new();
    for line in lines {
        let trimmed = line.trim();
        // Skip headings, horizontal rules, frontmatter
        if trimmed.starts_with('#')
            || trimmed.starts_with("---")
            || trimmed.starts_with("```")
            || trimmed.is_empty()
        {
            continue;
        }
        clean_sentences.push(trimmed);
        if clean_sentences.len() >= 2 {
            break;
        }
    }

    let joined = clean_sentences.join(" ");
    if joined.chars().count() > 200 {
        let mut s: String = joined.chars().take(200).collect();
        s.push_str("...");
        s
    } else {
        joined
    }
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
        assert_eq!(parsed.chunks.len(), 3);
        assert_eq!(parsed.chunks[1].heading.as_deref(), Some("Seção de Rust"));
        assert_eq!(parsed.chunks[1].tags, vec!["rust"]);
        assert_eq!(parsed.chunks[1].links, vec!["Linguagens"]);
        assert_eq!(parsed.chunks[2].heading.as_deref(), Some("Seção de IA"));
        assert_eq!(parsed.chunks[2].tags, vec!["ia"]);
    }
}
