//! ## MCP compact output grammar (v1)
//! ```text
//! search_hit := N. [[deep-link]] SCORE% ~TOKS
//! code_hit   := N. [[file#sym]] LSTART-END SCORE% ~TOKS\n```lang\nbody\n```
//! section    := [[Note#Sec]] ~TOKS\n<body>
//! list       := a, b, c   ("-" when empty)
//! pagination := +more: offset=N
//! truncation := …truncated (narrow query / lower limit / offset=N)
//! ```
//! `[[…]]` deep links, `offset=` and `read_code_chunk(...)` hints are stable
//! anchors for agents — keep them verbatim. Covered by tests in this module.

pub mod server;

use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use tauri::Emitter;

pub mod bridge;

use rmcp::{
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{
        CallToolResult, ContentBlock, GetPromptRequestParams, GetPromptResponse, GetPromptResult,
        Implementation, ListPromptsResult, ListResourceTemplatesResult, ListResourcesResult,
        PaginatedRequestParams, Prompt, PromptArgument, PromptMessage, ReadResourceRequestParams,
        ReadResourceResponse, ReadResourceResult, Resource, ResourceContents, ResourceTemplate,
        Role, ServerCapabilities, ServerConfig,
    },
    service::RequestContext,
    tool, tool_router, ErrorData as McpError, RoleServer, ServerHandler, ServiceExt,
};

use crate::engine::VaultManager;

/// Chunks larger than this many estimated tokens are truncated in `search_code`
/// output to protect the caller's context window.
const MAX_SEARCH_CHUNK_TOKENS: usize = 1500;

/// Truncates a string to at most `max_chars` bytes on a char boundary.
const DEFAULT_OUTPUT_BUDGET: usize = 8000;

/// Result diversification: keeps at most `per_parent` hits per parent
/// (note/file) and drops near-duplicates (Jaccard >= 0.8 on tokens).
fn diversify<T>(
    items: Vec<T>,
    parent: impl Fn(&T) -> String,
    content: impl Fn(&T) -> &str,
    per_parent: usize,
) -> Vec<T> {
    use std::collections::HashSet;
    let mut counts: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut seen: Vec<HashSet<String>> = Vec::new();
    let mut out = Vec::with_capacity(items.len());
    'outer: for item in items {
        let p = parent(&item);
        let c = counts.entry(p).or_insert(0);
        if *c >= per_parent {
            continue;
        }
        let tokens: HashSet<String> = content(&item)
            .split_whitespace()
            .map(|w| w.to_lowercase())
            .collect();
        for prev in &seen {
            let union = tokens.union(prev).count() as f32;
            if union > 0.0 && tokens.intersection(prev).count() as f32 / union >= 0.8 {
                continue 'outer;
            }
        }
        *c += 1;
        seen.push(tokens);
        out.push(item);
    }
    out
}

/// Appends a formatted block unless it would exceed the budget —
/// then writes an explicit truncation marker so agents can narrow.
fn push_budgeted(out: &mut String, block: &str, budget: usize) -> bool {
    if out.len() + block.len() > budget {
        out.push_str("\n…truncated (narrow query / lower limit / offset=N)\n");
        return false;
    }
    out.push_str(block);
    true
}

/// `0.731` → `"73%"`. Relevance score in compact form.
fn pct(score: f32) -> String {
    format!("{:.0}%", score * 100.0)
}

/// Token estimate suffix: `139` → `"~139t"`.
fn toks(tokens: usize) -> String {
    format!("~{}t", tokens)
}

/// Comma-joined list or `-` when empty — replaces `{:?}` debug formatting.
fn join_list(items: &[String]) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(", ")
    }
}

/// Pagination hint: `+more: offset=10`.
fn more_marker(next_offset: usize) -> String {
    format!("+more: offset={}\n", next_offset)
}

/// `21900` → `"21.9k"` for compact stat lines.
fn human(n: usize) -> String {
    if n >= 1000 {
        format!("{:.1}k", n as f64 / 1000.0)
    } else {
        n.to_string()
    }
}

fn truncate_str(s: &str, max_chars: usize) -> &str {
    if s.len() <= max_chars {
        return s;
    }
    let mut end = max_chars;
    while !s.is_char_boundary(end) {
        end -= 1;
    }
    &s[..end]
}

#[derive(Clone)]
pub struct GraphFinderMcpServer {
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
    pub manager: Arc<VaultManager>,
    pub app_handle: Option<tauri::AppHandle>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateProjectArgs {
    /// Project name (e.g. 'Backend')
    pub name: String,
    /// Absolute folder path (optional)
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SwitchProjectArgs {
    /// Project ID or slug
    pub project_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchSemanticArgs {
    /// Search query
    pub query: String,
    /// Max chunks (default: 5)
    pub limit: Option<usize>,
    /// Project ID (optional)
    pub project: Option<String>,
    /// 'dense_chunks' (default) or 'summaries'
    pub format: Option<String>,
    /// Pagination offset (default: 0)
    #[serde(default)]
    pub offset: Option<usize>,
    /// 'ids': links + scores only
    #[serde(default)]
    pub fields: Option<String>,
    /// Batch: multiple queries in one call
    #[serde(default)]
    pub queries: Option<Vec<String>>,
    /// Response budget in chars (default: 8000)
    #[serde(default)]
    pub max_chars: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCompressedContextArgs {
    /// Note title or ID
    #[serde(alias = "note", alias = "title")]
    pub target: String,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SuggestLinksArgs {
    /// Draft text to link
    pub text: String,
    /// Note to exclude (optional)
    pub current_note: Option<String>,
    /// Min similarity 0-1 (default: 0.65)
    pub threshold: Option<f32>,
    /// Project ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UpdateSectionArgs {
    /// Note title or ID
    #[serde(alias = "note", alias = "title")]
    pub target: String,
    /// Section heading to replace (## / ###)
    pub heading: String,
    /// New section body (heading preserved)
    pub content: String,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ValidateVaultArgs {
    /// Single note title/ID (optional)
    #[serde(default)]
    pub target: Option<String>,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StoreContextArgs {
    /// Note title
    pub title: String,
    /// Markdown body
    pub content: String,
    /// Context tags (optional)
    pub tags: Option<Vec<String>>,
    /// Wikilink targets (optional)
    pub links: Option<Vec<String>>,
    /// Project ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetGraphNeighborhoodArgs {
    /// Note title or ID
    pub target: String,
    /// Hops: 1 or 2 (default: 1)
    pub depth: Option<usize>,
    /// Project ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinkNotesArgs {
    /// Source note title or ID
    pub from_note: String,
    /// Destination note title
    pub to_note: String,
    /// Project ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListRecentNotesArgs {
    /// Max notes (default: 10)
    pub limit: Option<usize>,
    /// Project ID (optional)
    pub project: Option<String>,
    /// Pagination offset (default: 0)
    #[serde(default)]
    pub offset: Option<usize>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct SyncVaultArgs {
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
    /// Local dir to index (optional)
    #[serde(default, alias = "directory", alias = "dir", alias = "vault_path")]
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadSectionArgs {
    /// Note title or ID
    #[serde(alias = "note", alias = "title")]
    pub target: String,
    /// H2/H3 heading or deep-link fragment
    #[serde(alias = "heading")]
    pub section: String,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenameNoteArgs {
    /// Current title or ID
    #[serde(alias = "old", alias = "source")]
    pub old_title: String,
    /// New title
    #[serde(alias = "new", alias = "target")]
    pub new_title: String,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteNoteArgs {
    /// Note title or ID to delete
    #[serde(alias = "note", alias = "title")]
    pub target: String,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetVaultStatsArgs {
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchCodeArgs {
    /// Code search query
    pub query: String,
    /// Max chunks (default: 5)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Language filter (optional)
    #[serde(default, alias = "lang")]
    pub language: Option<String>,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
    /// Pagination offset (default: 0)
    #[serde(default)]
    pub offset: Option<usize>,
    /// 'ids': [[file#symbol]] + scores only
    #[serde(default)]
    pub fields: Option<String>,
    /// Batch: multiple queries in one call
    #[serde(default)]
    pub queries: Option<Vec<String>>,
    /// Response budget in chars (default: 8000)
    #[serde(default)]
    pub max_chars: Option<usize>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct FindFilesArgs {
    /// Glob/name match (e.g. '*.tsx', optional)
    #[serde(default, alias = "query", alias = "file", alias = "name")]
    pub pattern: Option<String>,
    /// Language filter (optional)
    #[serde(default, alias = "lang")]
    pub language: Option<String>,
    /// Max files (default: 20)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
    /// Pagination offset (default: 0)
    #[serde(default)]
    pub offset: Option<usize>,
    /// 'ids': paths only
    #[serde(default)]
    pub fields: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFileContextArgs {
    /// File path or name
    #[serde(alias = "file", alias = "path", alias = "target")]
    pub file_path: String,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadCodeChunkArgs {
    /// File path or name
    #[serde(alias = "file", alias = "path", alias = "target")]
    pub file_path: String,
    /// Symbol name or line range (optional)
    #[serde(default, alias = "symbol", alias = "lines", alias = "range")]
    pub symbol_or_lines: Option<String>,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCodeDependenciesArgs {
    /// File path
    #[serde(alias = "file", alias = "path", alias = "target")]
    pub file_path: String,
    /// 'imports' | 'imported_by' | 'both' (default)
    #[serde(default)]
    pub direction: Option<String>,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindPathArgs {
    /// Origin node (file, symbol, note)
    #[serde(alias = "from", alias = "start")]
    pub source: String,
    /// Destination node
    #[serde(alias = "to", alias = "end")]
    pub target: String,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExplainNodeArgs {
    /// Node (file, symbol, note)
    #[serde(alias = "node", alias = "symbol", alias = "file")]
    pub target: String,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetCommunityReportArgs {
    /// 'markdown' (default) or 'json'
    #[serde(default)]
    pub format: Option<String>,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ExportGraphArgs {
    /// Output dir (default: './graphfinder-out')
    #[serde(default)]
    pub out_dir: Option<String>,
    /// Project ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

impl GraphFinderMcpServer {
    pub fn new(manager: Arc<VaultManager>, app_handle: Option<tauri::AppHandle>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            manager,
            app_handle,
        }
    }
}

#[tool_router]
impl GraphFinderMcpServer {
    /// Lists vault projects with note/code counts and the active flag. Use to discover project IDs for the 'project' param.
    #[tool(description = "Lists vault projects with note/code counts and the active flag. Use to discover project IDs for the 'project' param.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn list_projects(&self) -> Result<CallToolResult, McpError> {
        let projects = self.manager.list_projects().await;
        let mut out = format!("projects({}):\n", projects.len());
        for p in projects {
            let status = if p.is_active { ", active" } else { "" };
            out.push_str(&format!(
                "- {} ({}{}) notes={} code={} → {}\n",
                p.name, p.id, status, p.note_count, p.code_count, p.path
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Creates an isolated project (markdown vault + vector index) at a path.
    #[tool(description = "Creates an isolated project (markdown vault + vector index) at a path.", annotations(open_world_hint = false))]
    async fn create_project(&self, Parameters(args): Parameters<CreateProjectArgs>) -> Result<CallToolResult, McpError> {
        let project = match self
            .manager
            .create_project_with_handle(&args.name, args.path, self.app_handle.clone())
            .await
        {
            Ok(p) => p,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to create project: {}", e))])),
        };
        if let Some(app) = &self.app_handle {
            let _ = app.emit("vault-sync-complete", 1);
        }
        let out = format!(
            "created {} ({}) | vault: {} | db: {}",
            project.name, project.id, project.path, project.db_path
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Switches the active default vault. Most tools also accept project='<id>' per call.
    #[tool(description = "Switches the active default vault. Most tools also accept project='<id>' per call.", annotations(idempotent_hint = true, open_world_hint = false))]
    async fn switch_project(&self, Parameters(args): Parameters<SwitchProjectArgs>) -> Result<CallToolResult, McpError> {
        let project = match self
            .manager
            .switch_project_with_handle(&args.project_id, self.app_handle.clone())
            .await
        {
            Ok(p) => p,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to switch project: {}", e))])),
        };
        if let Some(app) = &self.app_handle {
            let _ = app.emit("vault-sync-complete", project.note_count + project.code_count);
        }
        let out = format!(
            "active → {} ({}) notes={} code={}",
            project.name, project.id, project.note_count, project.code_count
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Atomic vector search over vault notes; returns ranked [[Note#Section]] excerpts. Use to answer questions; get_compressed_context for structure. fields='ids' for cheap 2-stage lookup.
    #[tool(description = "Atomic vector search over vault notes; returns ranked [[Note#Section]] excerpts. Use to answer questions; get_compressed_context for structure. fields='ids' for cheap 2-stage lookup.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn search_semantic(&self, Parameters(args): Parameters<SearchSemanticArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let limit = args.limit.unwrap_or(5);
        let format = args.format.as_deref().unwrap_or("dense_chunks");

        if format == "summaries" {
            let hits = match engine.search_semantic(&args.query, limit).await {
                Ok(h) => h,
                Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error during semantic search: {}", e))])),
            };
            if hits.is_empty() {
                return Ok(CallToolResult::success(vec![ContentBlock::text("No semantic results found in vault.")]));
            }
            let mut out = String::new();
            for (i, h) in hits.into_iter().enumerate() {
                out.push_str(&format!(
                    "{}. [[{}]] {} — {}\n",
                    i + 1,
                    h.title,
                    pct(h.score),
                    h.snippet
                ));
            }
            return Ok(CallToolResult::success(vec![ContentBlock::text(out)]));
        }

        let offset = args.offset.unwrap_or(0);
        let ids_only = matches!(args.fields.as_deref(), Some("ids") | Some("id"));
        let budget = args.max_chars.unwrap_or(DEFAULT_OUTPUT_BUDGET);
        let queries: Vec<String> = args
            .queries
            .clone()
            .filter(|qs| !qs.is_empty())
            .unwrap_or_else(|| vec![args.query.clone()]);
        let fetch_n = offset + limit + 1;

        let mut out = String::new();
        let mut any = false;
        for q in &queries {
            if queries.len() > 1 {
                out.push_str(&format!("=== '{}' ===\n", q));
            }
            let chunks = match engine.search_chunks(q, fetch_n).await {
                Ok(c) => c,
                Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error during chunk search: {}", e))])),
            };
            let has_more = chunks.len() >= fetch_n;
            let chunks = diversify(
                chunks,
                |c: &crate::engine::ChunkHit| c.note_id.clone(),
                |c: &crate::engine::ChunkHit| c.content.as_str(),
                2,
            );
            let idx = offset;
            for (i, c) in chunks.into_iter().skip(offset).take(limit).enumerate() {
                any = true;
                let block = if ids_only {
                    format!("{}. {} {}\n", idx + i + 1, c.deep_link, pct(c.score))
                } else {
                    format!(
                        "{}. {} {} {}\n{}\n\n",
                        idx + i + 1,
                        c.deep_link,
                        pct(c.score),
                        toks(c.token_estimate),
                        c.content.trim()
                    )
                };
                if !push_budgeted(&mut out, &block, budget) {
                    return Ok(CallToolResult::success(vec![ContentBlock::text(out)]));
                }
            }
            if has_more {
                out.push_str(&more_marker(offset + limit));
            }
        }
        if !any {
            return Ok(CallToolResult::success(vec![ContentBlock::text("No relevant semantic chunks found in vault.")]));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Returns a note's summary, section map with token estimates, tags and graph links. Use to pick a section, then read_section.
    #[tool(description = "Returns a note's summary, section map with token estimates, tags and graph links. Use to pick a section, then read_section.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn get_compressed_context(&self, Parameters(args): Parameters<GetCompressedContextArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let ctx = match engine.get_compressed_context(&args.target).await {
            Ok(c) => c,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error retrieving context: {}", e))])),
        };
        match ctx {
            Some(c) => {
                let mut out = format!(
                    "[[{}]] {} | tags: {} | links: {}\n{}\n",
                    c.title,
                    toks(c.token_estimate),
                    join_list(&c.tags),
                    join_list(&c.connected_notes),
                    c.summary
                );
                for s in c.sections {
                    out.push_str(&format!(
                        "- {} ({}): {}\n",
                        s.heading,
                        toks(s.token_estimate),
                        s.snippet
                    ));
                }
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            None => {
                let hint = match self
                    .did_you_mean(args.project.as_deref(), &args.target)
                    .await
                {
                    Some(s) => format!(" Did you mean [[{}]]?", s),
                    None => String::new(),
                };
                Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                    "Note '{}' not found.{}",
                    args.target, hint
                ))]))
            }
        }
    }

    /// Scores draft text and suggests wikilinks to existing notes. Run before store_context.
    #[tool(description = "Scores draft text and suggests wikilinks to existing notes. Run before store_context.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn suggest_links(&self, Parameters(args): Parameters<SuggestLinksArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let current_note = args.current_note.as_deref().unwrap_or("");
        let threshold = args.threshold.unwrap_or(0.65);

        let suggestions = match engine.suggest_links(&args.text, current_note, threshold, 4).await {
            Ok(s) => s,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error suggesting links: {}", e))])),
        };
        if suggestions.is_empty() {
            return Ok(CallToolResult::success(vec![ContentBlock::text("No strong link recommendations found for this text.")]));
        }

        let mut out = String::new();
        for (i, s) in suggestions.into_iter().enumerate() {
            out.push_str(&format!(
                "{}. {} {} — {}\n",
                i + 1,
                s.deep_link,
                pct(s.similarity_score),
                s.preview
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Saves or updates a vault note (chunking, tags, vectors). For edits to an existing section prefer update_section.
    #[tool(description = "Saves or updates a vault note (chunking, tags, vectors). For edits to an existing section prefer update_section.", annotations(idempotent_hint = true, open_world_hint = false))]
    async fn store_context(&self, Parameters(args): Parameters<StoreContextArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let mut content = args.content;

        if let Some(links) = args.links {
            if !links.is_empty() {
                content.push_str("\n\n## Related Connections\n");
                for l in links {
                    content.push_str(&format!("- [[{}]]\n", l));
                }
            }
        }

        if let Some(tags) = args.tags {
            if !tags.is_empty() {
                content.push_str("\n\nTags: ");
                for t in tags {
                    let tag_clean = t.trim_start_matches('#');
                    content.push_str(&format!("#{} ", tag_clean));
                }
                content.push('\n');
            }
        }

        let note = match engine.save_note(&args.title, &content, None).await {
            Ok(n) => n,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to save note: {}", e))])),
        };
        if let Some(app) = &self.app_handle {
            let _ = app.emit("vault-sync-complete", 1);
            let _ = app.emit("mcp-node-updated", &note.id);
        }
        let out = format!(
            "saved [[{}]] → {} | tags: {} | autolinks: {}",
            note.title,
            note.path,
            join_list(&note.tags),
            join_list(&note.outgoing_links)
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Lists 1-2 hop wikilink neighbors of a note. For the global map use get_community_report.
    #[tool(description = "Lists 1-2 hop wikilink neighbors of a note. For the global map use get_community_report.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn get_graph_neighborhood(&self, Parameters(args): Parameters<GetGraphNeighborhoodArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let depth = args.depth.unwrap_or(1);
        let n = match engine.get_neighborhood(&args.target, depth).await {
            Ok(res) => res,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to retrieve graph neighborhood: {}", e))])),
        };
        let mut out = format!(
            "[[{}]] depth={} neighbors({}):\n",
            args.target,
            depth,
            n.nodes.len()
        );
        for node in n.nodes {
            out.push_str(&format!(
                "- [[{}]] links={} tags: {}\n",
                node.title,
                node.degree,
                join_list(&node.tags)
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Creates a bidirectional wikilink between two notes.
    #[tool(description = "Creates a bidirectional wikilink between two notes.", annotations(idempotent_hint = true, open_world_hint = false))]
    async fn link_notes(&self, Parameters(args): Parameters<LinkNotesArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let updated = match engine.link_notes(&args.from_note, &args.to_note).await {
            Ok(u) => u,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to link notes: {}", e))])),
        };
        if let Some(app) = &self.app_handle {
            let _ = app.emit("vault-sync-complete", 1);
            let _ = app.emit("mcp-node-updated", &updated.id);
        }
        let out = format!(
            "linked [[{}]] → [[{}]] | links: {}",
            updated.title,
            args.to_note,
            join_list(&updated.outgoing_links)
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Lists vault notes with 1-line summaries and token estimates. Use to browse; search_semantic to query.
    #[tool(description = "Lists vault notes with 1-line summaries and token estimates. Use to browse; search_semantic to query.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn list_recent_notes(&self, Parameters(args): Parameters<ListRecentNotesArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let limit = args.limit.unwrap_or(10);
        let offset = args.offset.unwrap_or(0);
        let notes = match engine.list_notes().await {
            Ok(n) => n,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to list notes: {}", e))])),
        };
        let total = notes.len();
        let page: Vec<_> = notes.into_iter().skip(offset).take(limit).collect();
        let remaining = total.saturating_sub(offset + page.len());
        let display_notes = page.into_iter();

        let mut out = String::new();
        for n in display_notes {
            out.push_str(&format!(
                "- [[{}]] tags: {} — {}\n",
                n.title,
                join_list(&n.tags),
                n.summary
            ));
        }
        if remaining > 0 {
            out.push_str(&more_marker(offset + limit));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Incrementally re-indexes notes and code (SHA-256 diff) and rebuilds the import graph. 'path' indexes a new folder.
    #[tool(description = "Incrementally re-indexes notes and code (SHA-256 diff) and rebuilds the import graph. 'path' indexes a new folder.", annotations(idempotent_hint = true, open_world_hint = false))]
    async fn sync_vault(&self, Parameters(args): Parameters<SyncVaultArgs>) -> Result<CallToolResult, McpError> {
        let (project_id, engine, is_new) = if let Some(p) = &args.path {
            match self.manager.get_or_register_project_by_path(std::path::Path::new(p)).await {
                Ok((id, eng, is_new)) => (id, eng, is_new),
                Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to access path '{}': {}", p, e))])),
            }
        } else {
            let pid = match &args.project {
                Some(p) if !p.trim().is_empty() => p.clone(),
                _ => self.manager.active_project_id().await,
            };
            let eng = self.manager.get_engine(args.project.as_deref()).await;
            (pid, eng, false)
        };

        let count = match engine.sync_vault().await {
            Ok(c) => c,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to sync vault: {}", e))])),
        };
        if let Some(app) = &self.app_handle {
            let _ = app.emit("vault-sync-complete", count);
        }

        let stats = engine.get_vault_stats().await.ok();
        let mut out = format!(
            "synced '{}' ({}) | processed={} | {}\n",
            project_id,
            if is_new { "new" } else { "updated" },
            count,
            engine.vault_path().display()
        );
        if let Some(s) = stats {
            out.push_str(&format!(
                "notes={} code={} chunks={} symbols={} links={} imports={} langs={}",
                s.total_notes, s.total_code_files, s.total_chunks, s.total_code_chunks,
                s.total_links, s.total_code_links, join_list(&s.languages)
            ));
        }

        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Reads one ##/### section of a note. Use after get_compressed_context maps the headings.
    #[tool(description = "Reads one ##/### section of a note. Use after get_compressed_context maps the headings.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn read_section(&self, Parameters(args): Parameters<ReadSectionArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.read_section(&args.target, &args.section).await {
            Ok(Some(chunk)) => {
                let out = format!(
                    "{} {}\n\n{}",
                    chunk.deep_link,
                    toks(chunk.token_estimate),
                    chunk.content.trim()
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Ok(None) => {
                let hint = engine
                    .get_compressed_context(&args.target)
                    .await
                    .ok()
                    .flatten()
                    .map(|c| {
                        format!(
                            " Available sections: {}",
                            c.sections
                                .iter()
                                .map(|s| s.heading.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )
                    })
                    .unwrap_or_else(|| {
                        " Use get_compressed_context to view available sections.".to_string()
                    });
                Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                    "Section '{}' not found in note '{}'.{}",
                    args.section, args.target, hint
                ))]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error reading section: {}", e))])),
        }
    }

    /// Renames a note and rewrites all incoming [[wikilinks]] vault-wide.
    #[tool(description = "Renames a note and rewrites all incoming [[wikilinks]] vault-wide.", annotations(open_world_hint = false))]
    async fn rename_note(&self, Parameters(args): Parameters<RenameNoteArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.rename_note(&args.old_title, &args.new_title).await {
            Ok(note) => {
                if let Some(app) = &self.app_handle {
                    let _ = app.emit("vault-sync-complete", 1);
                }
                let out = format!(
                    "renamed {} → [[{}]] → {} | wikilinks refactored",
                    args.old_title, note.title, note.path
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to rename note: {}", e))])),
        }
    }

    /// Deletes a note file and purges its vectors. Destructive.
    #[tool(description = "Deletes a note file and purges its vectors. Destructive.", annotations(destructive_hint = true, open_world_hint = false))]
    async fn delete_note(&self, Parameters(args): Parameters<DeleteNoteArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.delete_note(&args.target).await {
            Ok(_) => {
                if let Some(app) = &self.app_handle {
                    let _ = app.emit("vault-sync-complete", 1);
                }
                let out = format!("deleted '{}' (file + vectors)", args.target);
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to delete note: {}", e))])),
        }
    }

    /// Returns counts: notes, code files, chunks, links, tags, tokens.
    #[tool(description = "Returns counts: notes, code files, chunks, links, tags, tokens.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn get_vault_stats(&self, Parameters(args): Parameters<GetVaultStatsArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.get_vault_stats().await {
            Ok(stats) => {
                let out = format!(
                    "notes={} code={} chunks={} links={} imports={} tags={} tokens={} langs={}",
                    stats.total_notes,
                    stats.total_code_files,
                    stats.total_chunks + stats.total_code_chunks,
                    stats.total_links,
                    stats.total_code_links,
                    stats.total_tags,
                    human(stats.total_tokens as usize),
                    join_list(&stats.languages)
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to get statistics: {}", e))])),
        }
    }

    /// Hybrid BM25+vector code search; returns ranked [[file#symbol]] excerpts. Use to locate code; read full symbols with read_code_chunk. fields='ids' for cheap 2-stage lookup.
    #[tool(description = "Hybrid BM25+vector code search; returns ranked [[file#symbol]] excerpts. Use to locate code; read full symbols with read_code_chunk. fields='ids' for cheap 2-stage lookup.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn search_code(&self, Parameters(args): Parameters<SearchCodeArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let limit = args.limit.unwrap_or(5);

        let offset = args.offset.unwrap_or(0);
        let ids_only = matches!(args.fields.as_deref(), Some("ids") | Some("id"));
        let budget = args.max_chars.unwrap_or(DEFAULT_OUTPUT_BUDGET);
        let queries: Vec<String> = args
            .queries
            .clone()
            .filter(|qs| !qs.is_empty())
            .unwrap_or_else(|| vec![args.query.clone()]);
        let fetch_n = offset + limit + 1;

        let mut out = String::new();
        let mut any = false;
        for q in &queries {
            if queries.len() > 1 {
                out.push_str(&format!("=== '{}' ===\n", q));
            }
            let hits = match engine.search_code(q, fetch_n, args.language.as_deref()).await {
                Ok(h) => h,
                Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error searching code: {}", e))])),
            };
            let has_more = hits.len() >= fetch_n;
            let hits = diversify(
                hits,
                |h: &crate::engine::CodeChunkHit| h.file_id.clone(),
                |h: &crate::engine::CodeChunkHit| h.content.as_str(),
                2,
            );
            let idx = offset;
            for (i, h) in hits.into_iter().skip(offset).take(limit).enumerate() {
                any = true;
                let block = if ids_only {
                    format!(
                        "{}. {} L{}-{} {}\n",
                        idx + i + 1, h.deep_link, h.start_line, h.end_line, pct(h.score)
                    )
                } else {
                    let content = h.content.trim();
                    let body = if h.token_estimate > MAX_SEARCH_CHUNK_TOKENS {
                        let truncated = truncate_str(content, MAX_SEARCH_CHUNK_TOKENS * 4);
                        let target = h
                            .symbol_name
                            .clone()
                            .unwrap_or_else(|| format!("L{}-L{}", h.start_line, h.end_line));
                        format!(
                            "{}\n…truncated (~{}t total; read_code_chunk(\"{}\", \"{}\"))",
                            truncated, h.token_estimate, h.file_id, target
                        )
                    } else {
                        content.to_string()
                    };
                    format!(
                        "{}. {} L{}-{} {} {}\n```{}\n{}\n```\n\n",
                        idx + i + 1,
                        h.deep_link,
                        h.start_line,
                        h.end_line,
                        pct(h.score),
                        toks(h.token_estimate),
                        h.language,
                        body
                    )
                };
                if !push_budgeted(&mut out, &block, budget) {
                    return Ok(CallToolResult::success(vec![ContentBlock::text(out)]));
                }
            }
            if has_more {
                out.push_str(&more_marker(offset + limit));
            }
        }
        if !any {
            return Ok(CallToolResult::success(vec![ContentBlock::text("No relevant code chunks found.")]));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Finds files by glob/name/language with import-degree stats. Use for paths; search_code for content.
    #[tool(description = "Finds files by glob/name/language with import-degree stats. Use for paths; search_code for content.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn find_files(&self, Parameters(args): Parameters<FindFilesArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let limit = args.limit.unwrap_or(20);

        let offset = args.offset.unwrap_or(0);
        let ids_only = matches!(args.fields.as_deref(), Some("ids") | Some("id"));
        let files = match engine.find_files(args.pattern.as_deref(), args.language.as_deref(), offset + limit + 1).await {
            Ok(f) => f,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error finding files: {}", e))])),
        };
        let has_more = files.len() > offset + limit;
        let files: Vec<_> = files.into_iter().skip(offset).take(limit).collect();

        if files.is_empty() {
            return Ok(CallToolResult::success(vec![ContentBlock::text("No files found matching the provided filters.")]));
        }

        let mut out = format!("files({}):\n", files.len());
        for f in files {
            if ids_only {
                out.push_str(&format!("- {}\n", f.path));
            } else {
                out.push_str(&format!(
                    "- {} ({}, {}B) imports={} ext={} deps={} — {}\n",
                    f.path, f.language, human(f.size_bytes as usize), f.imports_count, f.external_count, f.dependents_count, f.summary
                ));
            }
        }
        if has_more {
            out.push_str(&more_marker(offset + limit));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Structural map of one file: imports, dependents, external pkgs, symbols. Use before reading code; then read_code_chunk.
    #[tool(description = "Structural map of one file: imports, dependents, external pkgs, symbols. Use before reading code; then read_code_chunk.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn get_file_context(&self, Parameters(args): Parameters<GetFileContextArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;

        match engine.get_file_context(&args.file_path).await {
            Ok(Some(ctx)) => {
                let sym_list = if ctx.symbols.is_empty() {
                    "-".to_string()
                } else {
                    ctx.symbols
                        .iter()
                        .map(|s| format!("{} L{}-{} {}", s.name, s.start_line, s.end_line, toks(s.token_estimate)))
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                let out = format!(
                    "`{}` ({}, {})\n{}\nimports: {}\ndependents: {}\nexternal: {}\nsymbols: {}",
                    ctx.file_id,
                    ctx.language,
                    toks(ctx.token_estimate),
                    ctx.summary,
                    join_list(&ctx.imported_files),
                    join_list(&ctx.dependent_files),
                    join_list(&ctx.external_packages),
                    sym_list
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Ok(None) => Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "File '{}' not found in project. Use find_files to list available files.",
                args.file_path
            ))])),
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error retrieving file context: {}", e))])),
        }
    }

    /// Reads one symbol or line range from a file. Use after search_code/get_file_context locate the target.
    #[tool(description = "Reads one symbol or line range from a file. Use after search_code/get_file_context locate the target.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn read_code_chunk(&self, Parameters(args): Parameters<ReadCodeChunkArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;

        match engine.read_code_chunk(&args.file_path, args.symbol_or_lines.as_deref()).await {
            Ok(Some(chunk)) => {
                let out = format!(
                    "{} L{}-{} {}\n\n```{}\n{}\n```",
                    chunk.deep_link,
                    chunk.start_line,
                    chunk.end_line,
                    toks(chunk.token_estimate),
                    chunk.language,
                    chunk.content.trim()
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Ok(None) => Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "Chunk or symbol not found in file '{}'. Use get_file_context to view available symbols.",
                args.file_path
            ))])),
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error reading code chunk: {}", e))])),
        }
    }

    /// Forward/reverse dependency report for a file: imports, importers, external pkgs, calls. Use for blast-radius analysis.
    #[tool(description = "Forward/reverse dependency report for a file: imports, importers, external pkgs, calls. Use for blast-radius analysis.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn get_code_dependencies(&self, Parameters(args): Parameters<GetCodeDependenciesArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let direction = args.direction.as_deref().unwrap_or("both");

        match engine.get_code_dependencies(&args.file_path, direction).await {
            Ok(deps) => {
                let out = format!(
                    "deps `{}`:\nimports({}): {}\nimported_by({}): {}\nexternal({}): {}\ncalls({}): {}\ncallers({}): {}",
                    deps.file_id,
                    deps.imports.len(),
                    join_list(&deps.imports),
                    deps.imported_by.len(),
                    join_list(&deps.imported_by),
                    deps.external.len(),
                    join_list(&deps.external),
                    deps.outgoing_calls.len(),
                    join_list(&deps.outgoing_calls),
                    deps.incoming_callers.len(),
                    join_list(&deps.incoming_callers),
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error retrieving dependencies: {}", e))])),
        }
    }

    /// BFS shortest path between two nodes across imports and wikilinks.
    #[tool(description = "BFS shortest path between two nodes across imports and wikilinks.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn find_path(&self, Parameters(args): Parameters<FindPathArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.find_path(&args.source, &args.target).await {
            Ok(Some(path)) => {
                let mut out = String::new();
                for (i, step) in path.steps.iter().enumerate() {
                    if i == 0 {
                        out.push_str(&format!("`{}`", step.title));
                        continue;
                    }
                    let lbl = step.edge_label.as_deref().unwrap_or(step.direction.as_str());
                    out.push_str(&format!(" -[{}]-> `{}`", lbl, step.title));
                }
                out.push_str(&format!(" ({} hops)", path.length));
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Ok(None) => Ok(CallToolResult::success(vec![ContentBlock::text(format!("No connection path found between `{}` and `{}`.", args.source, args.target))])),
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error finding path: {}", e))])),
        }
    }

    /// 360-degree context card: centrality, community, in/out edges, siblings. For the global map use get_community_report.
    #[tool(description = "360-degree context card: centrality, community, in/out edges, siblings. For the global map use get_community_report.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn explain_node(&self, Parameters(args): Parameters<ExplainNodeArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.explain_node(&args.target).await {
            Ok(Some(e)) => {
                let god = if e.is_god_node { " GOD-NODE" } else { "" };
                let mut out = format!(
                    "`{}` ({}){} | C{} {}\n",
                    e.title, e.node_type, god, e.community_id, e.community_name
                );
                if let Some(p) = e.path {
                    out.push_str(&format!("path: `{}`\n", p));
                }
                out.push_str(&format!(
                    "rank=#{} degree={} in={} out={}\n",
                    e.centrality_rank, e.degree, e.in_degree, e.out_degree
                ));
                if let Some(s) = e.summary {
                    out.push_str(&format!("{}\n", s));
                }
                out.push_str(&format!(
                    "in({}): {}\n",
                    e.incoming_dependents.len(),
                    join_list(&e.incoming_dependents)
                ));
                out.push_str(&format!(
                    "out({}): {}\n",
                    e.outgoing_dependencies.len(),
                    join_list(&e.outgoing_dependencies)
                ));
                out.push_str(&format!(
                    "siblings C{}: {}\n",
                    e.community_id,
                    join_list(&e.sibling_nodes)
                ));
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Ok(None) => {
                let hint = match self
                    .did_you_mean(args.project.as_deref(), &args.target)
                    .await
                {
                    Some(s) => format!(" Did you mean `{}`?", s),
                    None => String::new(),
                };
                Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                    "Node `{}` not found in graph.{}",
                    args.target, hint
                ))]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error explaining node: {}", e))])),
        }
    }

    /// Topological audit: communities, God Nodes, cross-coupling surprises. Start here for architecture questions.
    #[tool(description = "Topological audit: communities, God Nodes, cross-coupling surprises. Start here for architecture questions.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn get_community_report(&self, Parameters(args): Parameters<GetCommunityReportArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.analyze_graph().await {
            Ok(report) => {
                if args.format.as_deref() == Some("json") {
                    let json = serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string());
                    Ok(CallToolResult::success(vec![ContentBlock::text(json)]))
                } else {
                    let mut out = format!(
                        "# Community Report\nnodes={} edges={} density={:.4} | communities={} god_nodes={} surprises={}\n",
                        report.total_nodes, report.total_edges, report.density,
                        report.communities.len(), report.god_nodes.len(), report.surprises.len()
                    );

                    out.push_str("\nGod Nodes:\n");
                    for g in &report.god_nodes {
                        out.push_str(&format!("- `{}` ({}) degree={} in={} out={} | C{} {}\n", g.title, g.node_type, g.degree, g.in_degree, g.out_degree, g.community_id, g.role));
                    }

                    out.push_str("\nCommunities:\n");
                    for c in &report.communities {
                        out.push_str(&format!("- C{} {} ({} nodes): {}\n", c.id, c.name, c.node_count, c.key_nodes.join(", ")));
                    }

                    if !report.surprises.is_empty() {
                        out.push_str("\nSurprises:\n");
                        for (i, s) in report.surprises.iter().enumerate() {
                            out.push_str(&format!("{}. `{}` (C{}) -> `{}` (C{}) [{}] — {}\n", i + 1, s.source_title, s.source_community, s.target_title, s.target_community, s.label, s.reason));
                        }
                    }

                    Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error generating report: {}", e))])),
        }
    }

    /// Exports graph.json, GRAPH_REPORT.md and standalone graph.html to disk.
    #[tool(description = "Exports graph.json, GRAPH_REPORT.md and standalone graph.html to disk.", annotations(idempotent_hint = true, open_world_hint = false))]
    async fn export_graph(&self, Parameters(args): Parameters<ExportGraphArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let out_path = args.out_dir.as_ref().map(std::path::Path::new);
        match engine.export_artifacts(out_path).await {
            Ok(summary) => {
                let out = format!(
                    "exported → {} | report={} graph={} html={} | nodes={} edges={} communities={} god={}",
                    summary.output_dir, summary.report_markdown_path, summary.graph_json_path, summary.standalone_html_path,
                    summary.total_nodes, summary.total_edges, summary.communities_count, summary.god_nodes_count
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error exporting graph: {}", e))])),
        }
    }
    /// Replaces the body of one ##/### section (heading preserved) and re-indexes. Prefer over store_context for edits.
    #[tool(description = "Replaces the body of one ##/### section (heading preserved) and re-indexes. Prefer over store_context for edits.", annotations(idempotent_hint = true, open_world_hint = false))]
    async fn update_section(&self, Parameters(args): Parameters<UpdateSectionArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine
            .update_section(&args.target, &args.heading, &args.content)
            .await
        {
            Ok(note) => {
                if let Some(app) = &self.app_handle {
                    let _ = app.emit("vault-sync-complete", 1);
                    let _ = app.emit("mcp-node-updated", &note.id);
                }
                Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                    "updated '{}' in [[{}]] → {}",
                    args.heading, note.title, note.path
                ))]))
            }
            Err(e) => {
                let hint = if e.to_string().contains("not found") {
                    match self
                        .did_you_mean(args.project.as_deref(), &args.target)
                        .await
                    {
                        Some(s) => format!(" Did you mean [[{}]]?", s),
                        None => String::new(),
                    }
                } else {
                    String::new()
                };
                Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                    "Failed to update section: {}.{}",
                    e, hint
                ))]))
            }
        }
    }

    /// Audits notes against the vault schema: missing H1/summary, section size, broken wikilinks, orphans, tags. 'target' checks one note.
    #[tool(description = "Audits notes against the vault schema: missing H1/summary, section size, broken wikilinks, orphans, tags. 'target' checks one note.", annotations(read_only_hint = true, open_world_hint = false))]
    async fn validate_vault(&self, Parameters(args): Parameters<ValidateVaultArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let summaries = match engine.list_notes().await {
            Ok(n) => n,
            Err(e) => {
                return Ok(CallToolResult::error(vec![ContentBlock::text(format!(
                    "Failed to list notes: {}",
                    e
                ))]))
            }
        };
        let graph = engine.get_graph().await.ok();

        // Known link targets: note titles and ids (with and without .md)
        let mut known: std::collections::HashSet<String> = std::collections::HashSet::new();
        for n in &summaries {
            known.insert(n.title.to_lowercase());
            known.insert(n.title.trim_end_matches(".md").to_lowercase());
            known.insert(n.id.to_lowercase());
            known.insert(n.id.trim_end_matches(".md").to_lowercase());
        }

        // Degree map for orphan detection
        let mut degrees: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        if let Some(g) = &graph {
            for n in &g.nodes {
                degrees.insert(n.id.to_lowercase(), n.degree);
                degrees.insert(n.title.to_lowercase(), n.degree);
            }
        }

        let mut errors: Vec<String> = Vec::new();
        let mut warnings: Vec<String> = Vec::new();
        let mut checked = 0usize;

        for s in &summaries {
            if let Some(ref t) = args.target {
                if !s.id.eq_ignore_ascii_case(t)
                    && !s.title.eq_ignore_ascii_case(t)
                    && !s.id.trim_end_matches(".md").eq_ignore_ascii_case(t)
                    && !s.title.trim_end_matches(".md").eq_ignore_ascii_case(t)
                {
                    continue;
                }
            }
            checked += 1;
            let note = match engine.get_note(&s.id).await {
                Ok(Some(n)) => n,
                _ => continue,
            };
            let label = note.title.trim_end_matches(".md");

            // --- H1 + executive summary ---
            let lines: Vec<&str> = note.content.lines().collect();
            let h1_idx = lines.iter().position(|l| l.starts_with("# "));
            if h1_idx.is_none() {
                errors.push(format!("[[{}]]: missing H1 title", label));
            }
            let first_h2 = lines.iter().position(|l| l.starts_with("## "));
            let summary_start = h1_idx.map(|i| i + 1).unwrap_or(0);
            let summary_end = first_h2.unwrap_or(lines.len());
            let summary_text: String = lines[summary_start..summary_end.min(lines.len())]
                .join(" ")
                .trim()
                .to_string();
            if summary_text.is_empty() {
                errors.push(format!("[[{}]]: missing executive summary after H1", label));
            } else if summary_text.len() / 4 > 120 {
                warnings.push(format!(
                    "[[{}]]: executive summary too long (~{} tokens, keep 2-3 lines)",
                    label,
                    summary_text.len() / 4
                ));
            }

            // --- Section token bands (40-150 tokens) ---
            let mut sec_start: Option<(usize, String)> = None;
            for (i, l) in lines.iter().enumerate() {
                if l.starts_with("## ") || l.starts_with("### ") {
                    if let Some((si, heading)) = sec_start.take() {
                        let body: String = lines[si + 1..i].join(" ");
                        let tokens = body.trim().len() / 4;
                        if tokens < 40 {
                            warnings.push(format!(
                                "[[{}]]: section \"{}\" too small (~{} tokens, min 40)",
                                label, heading, tokens
                            ));
                        } else if tokens > 150 {
                            warnings.push(format!(
                                "[[{}]]: section \"{}\" too large (~{} tokens, max 150)",
                                label, heading, tokens
                            ));
                        }
                    }
                    let heading = l.trim_start_matches('#').trim().to_string();
                    sec_start = Some((i, heading));
                }
            }
            if let Some((si, heading)) = sec_start {
                let body: String = lines[si + 1..].join(" ");
                let tokens = body.trim().len() / 4;
                if tokens < 40 {
                    warnings.push(format!(
                        "[[{}]]: section \"{}\" too small (~{} tokens, min 40)",
                        label, heading, tokens
                    ));
                } else if tokens > 150 {
                    warnings.push(format!(
                        "[[{}]]: section \"{}\" too large (~{} tokens, max 150)",
                        label, heading, tokens
                    ));
                }
            }

            // --- Broken wikilinks ---
            let mut rest = note.content.as_str();
            while let Some(open) = rest.find("[[") {
                if let Some(close) = rest[open..].find("]]") {
                    let link = &rest[open + 2..open + close];
                    let target = link
                        .split('#')
                        .next()
                        .unwrap_or("")
                        .split('|')
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_lowercase();
                    if !target.is_empty() && !known.contains(&target) {
                        errors.push(format!(
                            "[[{}]]: broken wikilink [[{}]]",
                            label,
                            link.split('|').next().unwrap_or(link)
                        ));
                    }
                    rest = &rest[open + close + 2..];
                } else {
                    break;
                }
            }

            // --- Tags ---
            if note.tags.is_empty() {
                warnings.push(format!("[[{}]]: no tags", label));
            }

            // --- Orphan check ---
            let deg = degrees
                .get(&note.id.to_lowercase())
                .or_else(|| degrees.get(&note.title.to_lowercase()))
                .copied()
                .unwrap_or(0);
            if deg == 0 {
                warnings.push(format!(
                    "[[{}]]: orphan note (0 graph connections — add wikilinks)",
                    label
                ));
            }
        }

        if checked == 0 {
            let hint = match args.target.as_deref() {
                Some(t) => match self.did_you_mean(args.project.as_deref(), t).await {
                    Some(s) => format!(" Did you mean [[{}]]?", s),
                    None => String::new(),
                },
                None => String::new(),
            };
            return Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "No notes matched target.{hint}"
            ))]));
        }

        let mut out = format!("checked={} errors({}):\n", checked, errors.len());
        for e in &errors {
            out.push_str(&format!("- {}\n", e));
        }
        out.push_str(&format!("warnings({}):\n", warnings.len()));
        for w in &warnings {
            out.push_str(&format!("- {}\n", w));
        }
        let ok_count = checked
            .saturating_sub(
                errors
                    .iter()
                    .map(|e| e.split("]]:").next().unwrap_or(""))
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    .max(1),
            )
            .max(0);
        out.push_str(&format!(
            "result: {} errors, {} warnings — {} compliant",
            errors.len(),
            warnings.len(),
            ok_count
        ));
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }
}

const AGENT_RULES_RESOURCE: &str = r#"# GraphFinder Agent Rules (AGENTS.md)

1. NEVER inject full Markdown files or whole source files into context unless explicitly asked. Prefer atomic excerpts via `search_semantic` / `read_section` / `read_code_chunk`, cited as deep links `[[Note#Section]]` / `[[file#symbol]]`.
2. Every note MUST: start with a single H1 + 2-3 line executive summary; be split into `##`/`###` sections of 40-150 tokens with descriptive titles; keep 1-3 `[[wikilinks]]` to other notes; carry lowercase `#tags`.
3. Before saving new knowledge, run `suggest_links` on the draft and use `store_context` (or `update_section` for surgical edits).
4. For architecture questions, run `get_community_report` first; use `explain_node` for 360° context and `find_path` for dependency chains.
5. All vault/code tools accept an optional `project: "<id>"` parameter for multi-vault targeting.
"#;

impl GraphFinderMcpServer {
    fn prompt_defs() -> Vec<Prompt> {
        vec![
            Prompt::new(
                "onboard_project",
                Some("Architecture audit of the active project: communities, god nodes, cycles, and cross-coupling surprises."),
                None,
            ),
            Prompt::new(
                "answer_question",
                Some("Answer a question using atomic vault excerpts with deep-link citations."),
                Some(vec![
                    PromptArgument::new("question")
                        .with_description("The question to answer")
                        .with_required(true),
                ]),
            ),
            Prompt::new(
                "investigate_code",
                Some("Locate and explain a feature, symbol, or bug across the indexed codebase."),
                Some(vec![
                    PromptArgument::new("feature")
                        .with_description("Feature, symbol, or behavior to investigate")
                        .with_required(true),
                ]),
            ),
            Prompt::new(
                "write_note",
                Some("Write a GraphFinder-compliant note: H1 + summary, 40-150 token sections, wikilinks, tags."),
                Some(vec![
                    PromptArgument::new("topic")
                        .with_description("Topic of the note to write")
                        .with_required(true),
                ]),
            ),
            Prompt::new(
                "audit_vault",
                Some("Audit vault notes against the AGENTS.md schema and fix violations."),
                None,
            ),
        ]
    }

    fn prompt_message(name: &str, args: Option<&rmcp::model::JsonObject>) -> Option<String> {
        let get = |k: &str| args.and_then(|a| a.get(k)).and_then(|v| v.as_str()).unwrap_or("");
        let text = match name {
            "onboard_project" => "Audit the architecture of the active GraphFinder project:\n1. Run `get_community_report()` for the global map (LPA communities, God Nodes, surprises, cycles).\n2. For each God Node / hub, run `explain_node(target)` to get centrality, callers, callees, and role.\n3. Summarize modular boundaries, risky couplings, and cycles. Cite nodes with deep links.".to_string(),
            "answer_question" => format!("Answer this question using the GraphFinder vault with maximum token economy:\n\n\"{}\"\n\nPipeline:\n1. Run `search_semantic(query)` for the top atomic chunks.\n2. If structure is needed, run `get_compressed_context(note_id)` or `read_section(note, heading)`.\n3. Respond concisely, citing every fact as `[[Note#Section]]`.", get("question")),
            "investigate_code" => format!("Investigate \"{}\" in the indexed codebase:\n1. `search_code(query)` for candidate symbols/files (hybrid BM25 + neural).\n2. `get_file_context(file)` for structural overview (imports, dependents, symbols).\n3. `read_code_chunk(file, symbol)` for the surgical implementation.\n4. `get_code_dependencies(file)` to assess blast radius before changes.\nCite locations as `[[file#symbol]]` or file:line ranges.", get("feature")),
            "write_note" => format!("Write a vault note about \"{}\" following the GraphFinder schema:\n- Single `# Title` + 2-3 line executive summary.\n- `##`/`###` sections of 40-150 tokens with descriptive titles.\n- 1-3 explicit `[[wikilinks]]` to notes consulted (run `suggest_links` on the draft first).\n- Lowercase `#tags` at the end.\nSave via `store_context`; for edits to existing sections use `update_section`.", get("topic")),
            "audit_vault" => "Audit vault compliance against the GraphFinder note schema:\n1. Run `validate_vault()` for the full report (missing summaries, oversized/undersized sections, broken wikilinks, orphans, missing tags).\n2. Fix each violation with `update_section` (surgical) or `store_context`, adding `[[wikilinks]]` to orphan notes via `suggest_links` + `link_notes`.\n3. Re-run `validate_vault()` to confirm a clean report.".to_string(),
            _ => return None,
        };
        Some(text)
    }

    /// Fuzzy "did you mean" suggestion for failed note lookups.
    async fn did_you_mean(&self, project: Option<&str>, target: &str) -> Option<String> {
        let engine = self.manager.get_engine(project).await;
        let notes = engine.list_notes().await.ok()?;
        let mut best: Option<(f64, String)> = None;
        for n in notes {
            for cand in [n.title.as_str(), n.id.as_str()] {
                let s = similarity_score(target, cand);
                if s > 0.45 && best.as_ref().map_or(true, |(bs, _)| s > *bs) {
                    best = Some((s, n.title.clone()));
                }
            }
        }
        best.map(|(_, t)| t)
    }

    fn uri_encode(s: &str) -> String {
        let mut out = String::with_capacity(s.len());
        for b in s.bytes() {
            match b {
                b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' => {
                    out.push(b as char)
                }
                _ => out.push_str(&format!("%{:02X}", b)),
            }
        }
        out
    }

    fn uri_decode(s: &str) -> String {
        let bytes = s.as_bytes();
        let mut out = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                if let Ok(hex) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                    out.push(hex);
                    i += 3;
                    continue;
                }
            }
            out.push(bytes[i]);
            i += 1;
        }
        String::from_utf8_lossy(&out).to_string()
    }
}

impl ServerHandler for GraphFinderMcpServer {
    async fn call_tool(
        &self,
        request: rmcp::model::CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::CallToolResponse, McpError> {
        let tcc = rmcp::handler::server::tool::ToolCallContext::new(self, request, context);
        self.tool_router.call(tcc).await
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<rmcp::model::ListToolsResult, McpError> {
        let mut tools = self.tool_router.list_all();
        // GRAPHFINDER_MCP_PROFILE=core exposes only the essential toolset —
        // cuts the schema token cost agents pay on every session init.
        if std::env::var("GRAPHFINDER_MCP_PROFILE").ok().as_deref() == Some("core") {
            const CORE: &[&str] = &[
                "search_semantic",
                "read_section",
                "store_context",
                "sync_vault",
                "search_code",
                "read_code_chunk",
                "find_files",
                "list_projects",
                "get_compressed_context",
                "suggest_links",
            ];
            tools.retain(|t| CORE.contains(&t.name.as_ref()));
        }
        Ok(rmcp::model::ListToolsResult {
            tools,
            ..Default::default()
        })
    }

    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder()
                .enable_tools()
                .enable_prompts()
                .enable_resources()
                .build(),
        )
        .with_server_info(Implementation::new("graphfinder", env!("CARGO_PKG_VERSION")))
        .with_instructions("GraphFinder: semantic vault + code intelligence. Pipeline: search_semantic/search_code (fields='ids' for cheap scans) -> get_compressed_context/get_file_context -> read_section/read_code_chunk; suggest_links before store_context; get_community_report for architecture. Most tools accept project='<id>'. Resources: graphfinder://instructions,://stats,://note/{id},://code/{path}. Prompts: onboard_project, answer_question, investigate_code, write_note, audit_vault.".to_string())
    }

    async fn list_prompts(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListPromptsResult, McpError> {
        Ok(ListPromptsResult {
            prompts: Self::prompt_defs(),
            ..Default::default()
        })
    }

    async fn get_prompt(
        &self,
        request: GetPromptRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<GetPromptResponse, McpError> {
        let text = Self::prompt_message(&request.name, request.arguments.as_ref())
            .ok_or_else(|| McpError::invalid_params(format!("Unknown prompt: {}", request.name), None))?;
        Ok(GetPromptResponse::Complete(
            GetPromptResult::new(vec![PromptMessage::new_text(Role::User, text)])
                .with_description(format!("GraphFinder pipeline: {}", request.name)),
        ))
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        let mut resources = vec![
            Resource::new("graphfinder://instructions", "agent-instructions")
                .with_title("Agent Instructions")
                .with_description("Mandatory GraphFinder vault rules (AGENTS.md): atomic retrieval, note schema, wikilinks.")
                .with_mime_type("text/markdown"),
            Resource::new("graphfinder://stats", "vault-stats")
                .with_title("Vault Statistics")
                .with_description("Live vault and codebase index metrics.")
                .with_mime_type("application/json"),
        ];

        let engine = self.manager.get_engine(None).await;
        if let Ok(notes) = engine.list_notes().await {
            for n in notes.iter().take(500) {
                resources.push(
                    Resource::new(
                        format!("graphfinder://note/{}", Self::uri_encode(&n.id)),
                        n.id.clone(),
                    )
                    .with_title(n.title.trim_end_matches(".md"))
                    .with_description(&n.summary)
                    .with_mime_type("text/markdown"),
                );
            }
        }

        Ok(ListResourcesResult {
            resources,
            ..Default::default()
        })
    }

    async fn list_resource_templates(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListResourceTemplatesResult, McpError> {
        Ok(ListResourceTemplatesResult {
            resource_templates: vec![
                ResourceTemplate::new("graphfinder://note/{id}", "note")
                    .with_title("Vault Note")
                    .with_description("Read a vault note by id or title.")
                    .with_mime_type("text/markdown"),
                ResourceTemplate::new("graphfinder://code/{path}", "code-file")
                    .with_title("Code File")
                    .with_description("Read an indexed code file by project-relative path.")
                    .with_mime_type("text/plain"),
            ],
            ..Default::default()
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResponse, McpError> {
        let uri = request.uri.clone();
        let path = uri.strip_prefix("graphfinder://").unwrap_or("");

        let mk = |text: String, _mime: &str| {
            Ok(ReadResourceResponse::Complete(ReadResourceResult::new(
                vec![ResourceContents::text(text, uri.clone())],
            )))
        };

        if path == "instructions" {
            return mk(AGENT_RULES_RESOURCE.to_string(), "text/markdown");
        }

        let engine = self.manager.get_engine(None).await;

        if path == "stats" {
            let stats = engine
                .get_vault_stats()
                .await
                .map_err(|e| McpError::internal_error(format!("Failed to get stats: {}", e), None))?;
            let text = serde_json::to_string_pretty(&stats)
                .unwrap_or_else(|_| "{}".to_string());
            return mk(text, "application/json");
        }

        if let Some(note_id) = path.strip_prefix("note/") {
            let id = Self::uri_decode(note_id);
            let note = engine
                .get_note(&id)
                .await
                .map_err(|e| McpError::internal_error(format!("Failed to read note: {}", e), None))?
                .ok_or_else(|| McpError::invalid_params(format!("Note not found: {}", id), None))?;
            return mk(note.content, "text/markdown");
        }

        if let Some(code_path) = path.strip_prefix("code/") {
            let rel = Self::uri_decode(code_path);
            if rel.contains("..") {
                return Err(McpError::invalid_params("Path traversal is not allowed".to_string(), None));
            }
            let project_id = self.manager.active_project_id().await;
            let project = self
                .manager
                .get_project_info(&project_id)
                .await
                .map_err(|e| McpError::internal_error(format!("Failed to resolve project: {}", e), None))?;
            let file_path = {
                let p = std::path::Path::new(&rel);
                if p.is_absolute() {
                    p.to_path_buf()
                } else {
                    std::path::Path::new(&project.path).join(p)
                }
            };
            let content = std::fs::read_to_string(&file_path).map_err(|e| {
                McpError::invalid_params(format!("Cannot read {}: {}", rel, e), None)
            })?;
            const MAX: usize = 64 * 1024;
            let text = if content.len() > MAX {
                format!("{}\n\n... [truncated at 64KB — use read_code_chunk for surgical reads]", &content[..MAX])
            } else {
                content
            };
            return mk(text, "text/plain");
        }

        Err(McpError::invalid_params(
            format!("Unknown resource URI: {}", uri),
            None,
        ))
    }
}

pub async fn run_mcp_server(
    manager: Arc<VaultManager>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = GraphFinderMcpServer::new(manager, None);
    let (read, write) = rmcp::transport::io::stdio();
    let running = server.serve((read, write)).await?;
    running.waiting().await?;
    Ok(())
}

pub fn export_all_schemas(dirs: &[&std::path::Path]) -> Vec<String> {
    let router = GraphFinderMcpServer::tool_router();
    let tools = router.list_all();
    let mut names = Vec::new();
    let mut all_tools_json = Vec::new();

    for tool in &tools {
        names.push(tool.name.to_string());
        let schema_obj = serde_json::json!({
            "name": tool.name.as_ref(),
            "description": tool.description.as_deref().unwrap_or(""),
            "parameters": tool.input_schema.as_ref(),
        });
        all_tools_json.push(schema_obj.clone());

        let json_str = serde_json::to_string_pretty(&schema_obj).unwrap_or_default();

        for dir in dirs {
            let _ = std::fs::create_dir_all(dir);
            let file_path = dir.join(format!("{}.json", tool.name));
            let _ = std::fs::write(&file_path, &json_str);
        }
    }

    // Export unified tools.json manifest for generic agents (Cursor, Claude, LangChain, etc.)
    let manifest_obj = serde_json::json!({
        "schema_version": "1.0",
        "server": {
            "name": "graphfinder",
            "version": env!("CARGO_PKG_VERSION"),
            "description": "GraphFinder Neural Knowledge Graph & Universal Code Intelligence MCP Server",
        },
        "tools_count": tools.len(),
        "tools": all_tools_json,
    });
    let manifest_str = serde_json::to_string_pretty(&manifest_obj).unwrap_or_default();

    for dir in dirs {
        let _ = std::fs::create_dir_all(dir);
        let manifest_path = dir.join("tools.json");
        let _ = std::fs::write(&manifest_path, &manifest_str);

        let instr_path = dir.join("instructions.md");
        let instructions_content = r#"# GraphFinder MCP Agent Instructions

Neural knowledge graph + universal code intelligence: 26 tools in 4 groups.

## 1. Architecture & Topology
- `get_community_report`: LPA communities, God Nodes, cross-coupling surprises — start here for architecture questions.
- `explain_node`: 360-degree card for a file/symbol/note (centrality, community, edges).
- `find_path`: BFS shortest dependency path between two nodes.
- `export_graph`: graph.json + GRAPH_REPORT.md + standalone graph.html.

## 2. Code Intelligence
- `search_code`: hybrid BM25+vector search returning [[file#symbol]] excerpts; `read_code_chunk` for full symbols; `fields='ids'` for cheap scans.
- `find_files`: glob/name/language lookup.
- `get_file_context`: imports, dependents, external pkgs, symbol map.
- `read_code_chunk`: one symbol or line range (`L10-40`, `10-40`, `L10`); unknown symbols error with the available list.
- `get_code_dependencies`: forward/reverse dependency report for a file.

## 3. Vault & Knowledge
- `search_semantic`: vector search returning [[Note#Section]] excerpts; `fields='ids'` for cheap scans.
- `get_compressed_context`: note summary + section map with token estimates.
- `read_section`: one ##/### section.
- `get_graph_neighborhood`: 1-2 hop wikilink neighbors.
- `get_vault_stats`: index counts.
- `validate_vault`: schema audit (H1, summary, section size, wikilinks, orphans, tags).
- `list_recent_notes`: browse notes with summaries.

## 4. Mutation & Projects
- `store_context`: save/update a note — run `suggest_links` on the draft first.
- `update_section`: replace one section body (heading preserved).
- `suggest_links`: wikilink suggestions for draft text.
- `link_notes`, `rename_note`, `delete_note`: graph maintenance.
- `sync_vault`: incremental re-index; `path` registers a new folder.
- `list_projects`, `create_project`, `switch_project`: multi-vault management.

## Golden Rules
1. Prefer atomic excerpts over whole files: search -> surgical read -> cite deep links [[Note#Section]] / [[file#symbol]].
2. Run `suggest_links` before `store_context`.
3. Run `get_community_report` first when onboarding unfamiliar systems.
4. Most tools accept `project: "<id>"`.
"#;
        let _ = std::fs::write(&instr_path, instructions_content);
    }

    names
}


/// Normalized similarity (bigram Jaccard + prefix bonus) for fuzzy "did you mean" suggestions.
fn similarity_score(a: &str, b: &str) -> f64 {
    let norm = |s: &str| s.trim().trim_end_matches(".md").to_lowercase();
    let (a, b) = (norm(a), norm(b));
    if a == b {
        return 1.0;
    }
    if a.is_empty() || b.is_empty() {
        return 0.0;
    }
    let bigrams = |s: &str| -> std::collections::HashSet<(char, char)> {
        s.chars().zip(s.chars().skip(1)).collect()
    };
    let (sa, sb) = (bigrams(&a), bigrams(&b));
    let inter = sa.intersection(&sb).count() as f64;
    let union = (sa.len() + sb.len()) as f64 - inter;
    let jaccard = if union > 0.0 { inter / union } else { 0.0 };
    let prefix = if b.starts_with(&a) || a.starts_with(&b) {
        0.25
    } else {
        0.0
    };
    (jaccard + prefix).min(1.0)
}
