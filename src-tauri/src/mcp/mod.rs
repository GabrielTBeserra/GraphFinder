pub mod server;

use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use tauri::Emitter;

use rmcp::{
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler, ServiceExt,
};

use crate::engine::VaultManager;

#[derive(Clone)]
pub struct GraphFinderMcpServer {
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
    pub manager: Arc<VaultManager>,
    pub app_handle: Option<tauri::AppHandle>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateProjectArgs {
    /// Name of the new project (e.g. 'HomeLab', 'Finances', 'Backend')
    pub name: String,
    /// Optional absolute folder path on disk
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SwitchProjectArgs {
    /// Target project ID or slug to switch to
    pub project_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchSemanticArgs {
    /// Semantic search query or natural language question
    pub query: String,
    /// Maximum number of atomic chunks to return (default: 5)
    pub limit: Option<usize>,
    /// Target project/vault ID (optional, defaults to active project)
    pub project: Option<String>,
    /// 'dense_chunks' (default, surgical 60-80 token chunks) or 'summaries' (1-line overview)
    pub format: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCompressedContextArgs {
    /// Note title or ID
    #[serde(alias = "note", alias = "title")]
    pub target: String,
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SuggestLinksArgs {
    /// Draft text or paragraph being authored
    pub text: String,
    /// Current note title or ID to exclude self-referencing (optional)
    pub current_note: Option<String>,
    /// Minimum cosine similarity threshold (0.0 to 1.0, default: 0.60)
    pub threshold: Option<f32>,
    /// Target project/vault ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StoreContextArgs {
    /// Main concept or note title
    pub title: String,
    /// Markdown body content of the note
    pub content: String,
    /// List of context tags (optional)
    pub tags: Option<Vec<String>>,
    /// List of notes to connect with explicit wikilinks (optional)
    pub links: Option<Vec<String>>,
    /// Target project/vault ID to save into (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetGraphNeighborhoodArgs {
    /// Note title or ID
    pub target: String,
    /// Graph hops of separation (1 or 2, default: 1)
    pub depth: Option<usize>,
    /// Target project/vault ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinkNotesArgs {
    /// Source note title or ID
    pub from_note: String,
    /// Destination note title
    pub to_note: String,
    /// Target project/vault ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListRecentNotesArgs {
    /// Maximum number of notes to return (default: 10)
    pub limit: Option<usize>,
    /// Target project/vault ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct SyncVaultArgs {
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
    /// Local directory path to sync/index (optional). Auto-detects if already indexed or creates a new project.
    #[serde(default, alias = "directory", alias = "dir", alias = "vault_path")]
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadSectionArgs {
    /// Note title or ID
    #[serde(alias = "note", alias = "title")]
    pub target: String,
    /// Section heading (H2/H3) or deep link fragment (e.g. '2. SQLite WAL Configuration')
    #[serde(alias = "heading")]
    pub section: String,
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenameNoteArgs {
    /// Current title or ID of the note
    #[serde(alias = "old", alias = "source")]
    pub old_title: String,
    /// Desired new title for the note
    #[serde(alias = "new", alias = "target")]
    pub new_title: String,
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteNoteArgs {
    /// Note title or ID to delete
    #[serde(alias = "note", alias = "title")]
    pub target: String,
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetVaultStatsArgs {
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchCodeArgs {
    /// Semantic or exact search query for code symbols, functions, or concepts
    pub query: String,
    /// Maximum code chunks to return (default: 5)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Filter by programming language (e.g. 'rust', 'typescript', 'python', optional)
    #[serde(default, alias = "lang")]
    pub language: Option<String>,
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct FindFilesArgs {
    /// Glob pattern or file path match (e.g. '*.tsx', 'db.rs', 'engine', optional)
    #[serde(default, alias = "query", alias = "file", alias = "name")]
    pub pattern: Option<String>,
    /// Filter by programming language (e.g. 'rust', 'typescript', optional)
    #[serde(default, alias = "lang")]
    pub language: Option<String>,
    /// Maximum files to return (default: 20)
    #[serde(default)]
    pub limit: Option<usize>,
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFileContextArgs {
    /// Relative path or file name in codebase (e.g. 'src/App.tsx', 'main.rs')
    #[serde(alias = "file", alias = "path", alias = "target")]
    pub file_path: String,
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadCodeChunkArgs {
    /// Relative path or file name in codebase
    #[serde(alias = "file", alias = "path", alias = "target")]
    pub file_path: String,
    /// Target symbol name (function/class/struct) or line range (optional)
    #[serde(default, alias = "symbol", alias = "lines", alias = "range")]
    pub symbol_or_lines: Option<String>,
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCodeDependenciesArgs {
    /// Relative path of code file (e.g. 'src/App.tsx')
    #[serde(alias = "file", alias = "path", alias = "target")]
    pub file_path: String,
    /// Dependency direction: 'imports' (what it imports), 'imported_by' (who imports it), or 'both' (default)
    #[serde(default)]
    pub direction: Option<String>,
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindPathArgs {
    /// Origin node title, file path, or symbol (e.g. 'App.tsx' or 'Note Title')
    #[serde(alias = "from", alias = "start")]
    pub source: String,
    /// Destination node title, file path, or symbol (e.g. 'db.rs')
    #[serde(alias = "to", alias = "end")]
    pub target: String,
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExplainNodeArgs {
    /// Target node title, file path, or symbol
    #[serde(alias = "node", alias = "symbol", alias = "file")]
    pub target: String,
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct GetCommunityReportArgs {
    /// 'markdown' (default human-readable audit) or 'json'
    #[serde(default)]
    pub format: Option<String>,
    /// Target project/vault ID (optional)
    #[serde(default)]
    pub project: Option<String>,
}

#[derive(Debug, Default, Deserialize, JsonSchema)]
pub struct ExportGraphArgs {
    /// Custom output directory (optional, defaults to './graphfinder-out')
    #[serde(default)]
    pub out_dir: Option<String>,
    /// Target project/vault ID (optional)
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
    /// Lists all isolated projects/vaults, note counts, and active project indicator.
    #[tool(description = "Lists all isolated projects/vaults, note counts, and active project indicator.")]
    async fn list_projects(&self) -> Result<CallToolResult, McpError> {
        let projects = self.manager.list_projects().await;
        let mut out = String::from("Projects and Vaults in GraphFinder:\n\n");
        for p in projects {
            let status = if p.is_active { " (Active)" } else { "" };
            out.push_str(&format!(
                "- **{}** (ID: `{}`){} | {} notes\n  Path: {}\n\n",
                p.name, p.id, status, p.note_count, p.path
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Creates a new isolated project with dedicated Markdown vault and SQLite vector index.
    #[tool(description = "Creates a new isolated project with dedicated Markdown vault and SQLite vector index.")]
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
            "Project '{}' (ID: '{}') created successfully!\nNote Vault: {}\nVector SQLite Database: {}",
            project.name, project.id, project.path, project.db_path
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Switches the active default vault for subsequent GraphFinder operations.
    #[tool(description = "Switches the active default vault for subsequent GraphFinder operations.")]
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
            let _ = app.emit("vault-sync-complete", project.note_count);
        }
        let out = format!(
            "Active project successfully switched to: {} (ID: '{}') with {} notes.",
            project.name, project.id, project.note_count
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Surgical section retrieval for maximum token conservation. Returns exact 2-4 line chunks with deep link [[Note#Section]] and relevance score.
    #[tool(description = "Surgical section retrieval for maximum token conservation. Returns exact 2-4 line chunks with deep link [[Note#Section]] and relevance score.")]
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
            let mut out = String::from("Semantic Overview:\n");
            for (i, h) in hits.into_iter().enumerate() {
                out.push_str(&format!(
                    "{}. [[{}]] (Relevance: {:.0}%)\n   Summary: {}\n",
                    i + 1,
                    h.title,
                    h.score * 100.0,
                    h.snippet
                ));
            }
            return Ok(CallToolResult::success(vec![ContentBlock::text(out)]));
        }

        let chunks = match engine.search_chunks(&args.query, limit).await {
            Ok(c) => c,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error during chunk search: {}", e))])),
        };
        if chunks.is_empty() {
            return Ok(CallToolResult::success(vec![ContentBlock::text("No relevant semantic chunks found in vault.")]));
        }

        let mut out = String::from("High-Density Semantic Chunks (Token-Efficient):\n\n");
        for (i, c) in chunks.into_iter().enumerate() {
            out.push_str(&format!(
                "{}. {} | Score: {:.0}% (~{} tokens)\n\"{}\"\n\n",
                i + 1,
                c.deep_link,
                c.score * 100.0,
                c.token_estimate,
                c.content.trim()
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Returns ultracompact summary (<80 tokens) of a note: executive summary, headings map, token estimates, and connected graph neighbors.
    #[tool(description = "Returns ultracompact summary (<80 tokens) of a note: executive summary, headings map, token estimates, and connected graph neighbors.")]
    async fn get_compressed_context(&self, Parameters(args): Parameters<GetCompressedContextArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let ctx = match engine.get_compressed_context(&args.target).await {
            Ok(c) => c,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error retrieving context: {}", e))])),
        };
        match ctx {
            Some(c) => {
                let mut out = format!(
                    "Note Summary: [[{}]] (~{} total tokens)\nSummary: {}\nTags: {:?}\nGraph Connections: {:?}\n\nAvailable Sections:\n",
                    c.title, c.token_estimate, c.summary, c.tags, c.connected_notes
                );
                for s in c.sections {
                    out.push_str(&format!(
                        "- #{} (~{} tokens): {}\n",
                        s.heading, s.token_estimate, s.snippet
                    ));
                }
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            None => Ok(CallToolResult::success(vec![ContentBlock::text(format!("Note '{}' not found.", args.target))])),
        }
    }

    /// Auto-Linker: analyzes draft text and recommends conceptual wikilinks with high semantic affinity.
    #[tool(description = "Auto-Linker: analyzes draft text and recommends conceptual wikilinks with high semantic affinity.")]
    async fn suggest_links(&self, Parameters(args): Parameters<SuggestLinksArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let current_note = args.current_note.as_deref().unwrap_or("");
        let threshold = args.threshold.unwrap_or(0.60);

        let suggestions = match engine.suggest_links(&args.text, current_note, threshold, 4).await {
            Ok(s) => s,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error suggesting links: {}", e))])),
        };
        if suggestions.is_empty() {
            return Ok(CallToolResult::success(vec![ContentBlock::text("No strong link recommendations found for this text.")]));
        }

        let mut out = String::from("Semantic Wikilink Suggestions for the Graph:\n\n");
        for (i, s) in suggestions.into_iter().enumerate() {
            out.push_str(&format!(
                "{}. {} (Affinity: {:.0}%)\n   Context: \"{}\"\n",
                i + 1,
                s.deep_link,
                s.similarity_score * 100.0,
                s.preview
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Saves or updates a Markdown note in the vault with CommonMark AST chunking, dense summary, and FastEmbed vectorization.
    #[tool(description = "Saves or updates a Markdown note in the vault with CommonMark AST chunking, dense summary, and FastEmbed vectorization.")]
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
        let autolinks_info = if !note.outgoing_links.is_empty() {
            format!(
                "\nAuto-links Created (≥65% affinity): {:?}",
                note.outgoing_links
            )
        } else {
            "\nAuto-links: No existing notes with ≥65% affinity found.".to_string()
        };
        let out = format!(
            "Note successfully saved, vectorized, and linked!\nTitle: [[{}]]\nFile: {}\nTags: {:?}{}",
            note.title, note.path, note.tags, autolinks_info
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Returns associative subgraph around a note (1 or 2 hops) with concise neighbor summaries.
    #[tool(description = "Returns associative subgraph around a note (1 or 2 hops) with concise neighbor summaries.")]
    async fn get_graph_neighborhood(&self, Parameters(args): Parameters<GetGraphNeighborhoodArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let depth = args.depth.unwrap_or(1);
        let n = match engine.get_neighborhood(&args.target, depth).await {
            Ok(res) => res,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to retrieve graph neighborhood: {}", e))])),
        };
        let mut out = format!("Subgraph around [[{}]]:\n\nConnected Nodes:\n", args.target);
        for node in n.nodes {
            out.push_str(&format!(
                "- [[{}]] ({} links, tags: {:?})\n",
                node.title, node.degree, node.tags
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Creates an explicit bidirectional wikilink between two notes or concepts.
    #[tool(description = "Creates an explicit bidirectional wikilink between two notes or concepts.")]
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
            "Link created successfully!\nSource: [[{}]] -> Target: [[{}]]\nUpdated Links: {:?}",
            updated.title, args.to_note, updated.outgoing_links
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Lists vault notes with 1-line summaries and token estimates.
    #[tool(description = "Lists vault notes with 1-line summaries and token estimates.")]
    async fn list_recent_notes(&self, Parameters(args): Parameters<ListRecentNotesArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let limit = args.limit.unwrap_or(10);
        let notes = match engine.list_notes().await {
            Ok(n) => n,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to list notes: {}", e))])),
        };
        let display_notes = notes.into_iter().take(limit);

        let mut out = String::from("Notes in Knowledge Vault:\n");
        for n in display_notes {
            out.push_str(&format!(
                "- [[{}]] (Tags: {:?}) - \"{}\"\n",
                n.title, n.tags, n.summary
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Synchronizes Markdown notes and code files with local SQLite vector index, re-embedding modified files and updating AST import graph. Accepts optional 'path'.
    #[tool(description = "Synchronizes Markdown notes and code files with local SQLite vector index, re-embedding modified files and updating AST import graph. Accepts optional 'path' to index any local folder.")]
    async fn sync_vault(&self, Parameters(args): Parameters<SyncVaultArgs>) -> Result<CallToolResult, McpError> {
        let (project_id, engine, is_new) = if let Some(p) = &args.path {
            match self.manager.get_or_register_project_by_path(std::path::Path::new(p)).await {
                Ok((id, eng, is_new)) => (id, eng, is_new),
                Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to access path '{}': {}", p, e))])),
            }
        } else {
            let pid = args.project.clone().unwrap_or_else(|| "default".to_string());
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
        let status_desc = if is_new {
            format!("New project '{}' registered and indexed", project_id)
        } else {
            format!("Project '{}' incrementally updated", project_id)
        };

        let mut out = format!(
            "Synchronization completed successfully!\n• Status: {}\n• Directory: {}\n• Items processed in this sync: {}\n",
            status_desc,
            engine.vault_path().display(),
            count
        );
        if let Some(s) = stats {
            out.push_str(&format!(
                "• Total Notes: {} (with {} sections/chunks)\n• Total Code Files: {} (with {} AST symbols)\n• Graph Connections: {} (wikilinks and imports)\n• Detected Languages: {:?}",
                s.total_notes, s.total_chunks, s.total_code_files, s.total_code_chunks, s.total_links, s.languages
            ));
        }

        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Surgically reads a single section (heading H2/H3) of a note, guaranteeing maximum token economy for LLMs.
    #[tool(description = "Surgically reads a single section (heading H2/H3) of a note, guaranteeing maximum token economy for LLMs.")]
    async fn read_section(&self, Parameters(args): Parameters<ReadSectionArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.read_section(&args.target, &args.section).await {
            Ok(Some(chunk)) => {
                let out = format!(
                    "Section: {} (~{} tokens)\nDeep Link: {}\n\n{}",
                    chunk.heading.as_deref().unwrap_or("General"),
                    chunk.token_estimate,
                    chunk.deep_link,
                    chunk.content.trim()
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Ok(None) => Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "Section '{}' not found in note '{}'. Use get_compressed_context to view available sections.",
                args.section, args.target
            ))])),
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error reading section: {}", e))])),
        }
    }

    /// Renames a note and its file on disk, refactoring all incoming wikilinks [[Old Title]] across the entire vault to prevent broken links.
    #[tool(description = "Renames a note and its file on disk, refactoring all incoming wikilinks [[Old Title]] across the entire vault to prevent broken links.")]
    async fn rename_note(&self, Parameters(args): Parameters<RenameNoteArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.rename_note(&args.old_title, &args.new_title).await {
            Ok(note) => {
                if let Some(app) = &self.app_handle {
                    let _ = app.emit("vault-sync-complete", 1);
                }
                let out = format!(
                    "Note renamed successfully!\nPrevious Title: {}\nNew Title: [[{}]]\nFile: {}\nAll vault wikilinks were refactored automatically.",
                    args.old_title, note.title, note.path
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to rename note: {}", e))])),
        }
    }

    /// Deletes a note from the vault (removes .md file and purges vector records from SQLite).
    #[tool(description = "Deletes a note from the vault (removes .md file and purges vector records from SQLite).")]
    async fn delete_note(&self, Parameters(args): Parameters<DeleteNoteArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.delete_note(&args.target).await {
            Ok(_) => {
                if let Some(app) = &self.app_handle {
                    let _ = app.emit("vault-sync-complete", 1);
                }
                let out = format!("Note '{}' successfully deleted from vault and vector database.", args.target);
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to delete note: {}", e))])),
        }
    }

    /// Returns health and graph statistics: total notes, code files, vectorized chunks, graph connections, tags, and estimated tokens.
    #[tool(description = "Returns health and graph statistics: total notes, code files, vectorized chunks, graph connections, tags, and estimated tokens.")]
    async fn get_vault_stats(&self, Parameters(args): Parameters<GetVaultStatsArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.get_vault_stats().await {
            Ok(stats) => {
                let langs_str = if stats.languages.is_empty() {
                    "None".to_string()
                } else {
                    stats.languages.join(", ")
                };
                let out = format!(
                    "Project and Vault Statistics:\n\n- Markdown Notes: {}\n- Code Files: {}\n- Detected Languages: {}\n- Vectorized Chunks & Sections: {}\n- Graph Connections (Wikilinks + Imports): {}\n- Unique Tags: {}\n- Total Estimated Tokens: {}",
                    stats.total_notes,
                    stats.total_code_files,
                    langs_str,
                    stats.total_chunks + stats.total_code_chunks,
                    stats.total_links + stats.total_code_links,
                    stats.total_tags,
                    stats.total_tokens
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Failed to get statistics: {}", e))])),
        }
    }

    /// Hybrid dense vector + BM25 search across project codebase. Returns function, class, or block excerpts with relevance scores and deep links [[file#symbol]].
    #[tool(description = "Hybrid dense vector + BM25 search across project codebase. Returns function, class, or block excerpts with relevance scores and deep links [[file#symbol]].")]
    async fn search_code(&self, Parameters(args): Parameters<SearchCodeArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let limit = args.limit.unwrap_or(5);

        let hits = match engine.search_code(&args.query, limit, args.language.as_deref()).await {
            Ok(h) => h,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error searching code: {}", e))])),
        };

        if hits.is_empty() {
            return Ok(CallToolResult::success(vec![ContentBlock::text("No relevant code chunks found.")]));
        }

        let mut out = String::from("Relevant Code Chunks (Neural + FTS5 Hybrid Search):\n\n");
        for (i, h) in hits.into_iter().enumerate() {
            out.push_str(&format!(
                "{}. {} (Lines {}-{}, Relevance: {:.0}%, ~{} tokens)\n   Language: {}\n```{}\n{}\n```\n\n",
                i + 1,
                h.deep_link,
                h.start_line,
                h.end_line,
                h.score * 100.0,
                h.token_estimate,
                h.language,
                h.language,
                h.content.trim()
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Quickly finds files in project codebase by name, glob pattern, or language, including connectivity degree in the dependency graph.
    #[tool(description = "Quickly finds files in project codebase by name, glob pattern, or language, including connectivity degree in the dependency graph.")]
    async fn find_files(&self, Parameters(args): Parameters<FindFilesArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let limit = args.limit.unwrap_or(20);

        let files = match engine.find_files(args.pattern.as_deref(), args.language.as_deref(), limit).await {
            Ok(f) => f,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error finding files: {}", e))])),
        };

        if files.is_empty() {
            return Ok(CallToolResult::success(vec![ContentBlock::text("No files found matching the provided filters.")]));
        }

        let mut out = format!("Files found in project (Total: {}):\n\n", files.len());
        for f in files {
            out.push_str(&format!(
                "- `{}` ({}, {} bytes) | Imports: {} | Dependents: {}\n  Summary: {}\n\n",
                f.path, f.language, f.size_bytes, f.imports_count, f.dependents_count, f.summary
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Returns compact structural summary of a code file (<80 tokens): imported files, dependents, external packages, and symbol catalog.
    #[tool(description = "Returns compact structural summary of a code file (<80 tokens): imported files, dependents, external packages, and symbol catalog.")]
    async fn get_file_context(&self, Parameters(args): Parameters<GetFileContextArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;

        match engine.get_file_context(&args.file_path).await {
            Ok(Some(ctx)) => {
                let sym_list = if ctx.symbols.is_empty() {
                    "No top-level symbols identified".to_string()
                } else {
                    ctx.symbols
                        .iter()
                        .map(|s| format!("{} (L{}-L{}, ~{}t)", s.name, s.start_line, s.end_line, s.token_estimate))
                        .collect::<Vec<_>>()
                        .join(", ")
                };

                let out = format!(
                    "File Structural Context:\n\n- File: `{}` ({}, ~{} tokens)\n- Summary: {}\n- Imported Files (Internal): {:?}\n- Dependent Files (Who imports this): {:?}\n- External Packages: {:?}\n- Symbols: {}",
                    ctx.file_id, ctx.language, ctx.token_estimate, ctx.summary, ctx.imported_files, ctx.dependent_files, ctx.external_packages, sym_list
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

    /// Surgically reads a function, class, struct, or line range from a code file without loading the entire file into context.
    #[tool(description = "Surgically reads a function, class, struct, or line range from a code file without loading the entire file into context.")]
    async fn read_code_chunk(&self, Parameters(args): Parameters<ReadCodeChunkArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;

        match engine.read_code_chunk(&args.file_path, args.symbol_or_lines.as_deref()).await {
            Ok(Some(chunk)) => {
                let out = format!(
                    "Code Chunk: {} (Lines {}-{}, ~{} tokens)\nDeep Link: {}\n\n```{}\n{}\n```",
                    chunk.symbol_name.as_deref().unwrap_or("block"),
                    chunk.start_line,
                    chunk.end_line,
                    chunk.token_estimate,
                    chunk.deep_link,
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

    /// Explores dependency and import graph of a code file (imports, dependents, external modules).
    #[tool(description = "Explores dependency and import graph of a code file (imports, dependents, external modules).")]
    async fn get_code_dependencies(&self, Parameters(args): Parameters<GetCodeDependenciesArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let direction = args.direction.as_deref().unwrap_or("both");

        match engine.get_code_dependencies(&args.file_path, direction).await {
            Ok(deps) => {
                let out = format!(
                    "Dependencies for `{}`:\n\n- Imported Files ({}):\n{}\n\n- Files Importing This File ({}):\n{}\n\n- External Modules/Packages ({}):\n{}\n\n- Outgoing Function Calls ({}):\n{}\n\n- Incoming Callers ({}):\n{}",
                    deps.file_id,
                    deps.imports.len(),
                    if deps.imports.is_empty() { "  (none)".to_string() } else { deps.imports.iter().map(|i| format!("  - `{}`", i)).collect::<Vec<_>>().join("\n") },
                    deps.imported_by.len(),
                    if deps.imported_by.is_empty() { "  (none)".to_string() } else { deps.imported_by.iter().map(|i| format!("  - `{}`", i)).collect::<Vec<_>>().join("\n") },
                    deps.external.len(),
                    if deps.external.is_empty() { "  (none)".to_string() } else { deps.external.iter().map(|i| format!("  - `{}`", i)).collect::<Vec<_>>().join("\n") },
                    deps.outgoing_calls.len(),
                    if deps.outgoing_calls.is_empty() { "  (none)".to_string() } else { deps.outgoing_calls.iter().map(|c| format!("  - `{}`", c)).collect::<Vec<_>>().join("\n") },
                    deps.incoming_callers.len(),
                    if deps.incoming_callers.is_empty() { "  (none)".to_string() } else { deps.incoming_callers.iter().map(|c| format!("  - `{}`", c)).collect::<Vec<_>>().join("\n") },
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error retrieving dependencies: {}", e))])),
        }
    }

    /// Discovers shortest multi-hop dependency path connecting any two symbols, files, or notes.
    #[tool(description = "Discovers shortest multi-hop dependency path connecting any two symbols, files, or notes.")]
    async fn find_path(&self, Parameters(args): Parameters<FindPathArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.find_path(&args.source, &args.target).await {
            Ok(Some(path)) => {
                let mut out = format!("### Path from `{}` to `{}` (Length: {} hops)\n\n", path.source, path.target, path.length);
                for (i, step) in path.steps.iter().enumerate() {
                    let dir = if i == 0 { "START" } else { &step.direction };
                    let lbl = step.edge_label.as_deref().map(|l| format!(" [{}]", l)).unwrap_or_default();
                    out.push_str(&format!("{}. {} **{}** (`{}`){}\n", i + 1, dir, step.title, step.node_type, lbl));
                }
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Ok(None) => Ok(CallToolResult::success(vec![ContentBlock::text(format!("No connection path found between `{}` and `{}`.", args.source, args.target))])),
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error finding path: {}", e))])),
        }
    }

    /// Generates a comprehensive 360° architectural context card for any symbol, file, or note.
    #[tool(description = "Generates a comprehensive 360° architectural context card for any symbol, file, or note.")]
    async fn explain_node(&self, Parameters(args): Parameters<ExplainNodeArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.explain_node(&args.target).await {
            Ok(Some(e)) => {
                let god_badge = if e.is_god_node { " 🌟 **[GOD NODE / ARCHITECTURAL HUB]**" } else { "" };
                let mut out = format!("# Architectural Context: `{}`{}\n\n", e.title, god_badge);
                out.push_str(&format!("- **Node Type**: `{}`\n", e.node_type));
                if let Some(p) = e.path {
                    out.push_str(&format!("- **File Path**: `{}`\n", p));
                }
                out.push_str(&format!("- **Community Cluster**: C{} - {}\n", e.community_id, e.community_name));
                out.push_str(&format!("- **Centrality Rank**: #{} (Degree: {} | Inbound: {} | Outbound: {})\n", e.centrality_rank, e.degree, e.in_degree, e.out_degree));
                if let Some(s) = e.summary {
                    out.push_str(&format!("- **Summary**: {}\n", s));
                }
                out.push_str(&format!("\n### Inbound Dependents ({}):\n", e.incoming_dependents.len()));
                if e.incoming_dependents.is_empty() {
                    out.push_str("- *(None)*\n");
                } else {
                    for dep in e.incoming_dependents {
                        out.push_str(&format!("- {}\n", dep));
                    }
                }
                out.push_str(&format!("\n### Outbound Dependencies ({}):\n", e.outgoing_dependencies.len()));
                if e.outgoing_dependencies.is_empty() {
                    out.push_str("- *(None)*\n");
                } else {
                    for dep in e.outgoing_dependencies {
                        out.push_str(&format!("- {}\n", dep));
                    }
                }
                out.push_str(&format!("\n### Co-located Sibling Nodes in Community C{}:\n", e.community_id));
                for sib in e.sibling_nodes {
                    out.push_str(&format!("- `{}`\n", sib));
                }
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Ok(None) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Node `{}` not found in graph.", args.target))])),
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error explaining node: {}", e))])),
        }
    }

    /// Returns high-density architectural audit of communities, god nodes, and surprises.
    #[tool(description = "Returns high-density architectural audit of communities, god nodes, and surprises.")]
    async fn get_community_report(&self, Parameters(args): Parameters<GetCommunityReportArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.analyze_graph().await {
            Ok(report) => {
                if args.format.as_deref() == Some("json") {
                    let json = serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".to_string());
                    Ok(CallToolResult::success(vec![ContentBlock::text(json)]))
                } else {
                    let mut out = String::from("# Architectural Community & Topology Report\n\n");
                    out.push_str(&format!("- **Total Nodes**: {} | **Connections**: {} | **Density**: {:.4}\n", report.total_nodes, report.total_edges, report.density));
                    out.push_str(&format!("- **Communities**: {} | **God Nodes**: {} | **Surprises**: {}\n\n", report.communities.len(), report.god_nodes.len(), report.surprises.len()));
                    
                    out.push_str("### 🌟 God Nodes (Architectural Hubs)\n");
                    for g in &report.god_nodes {
                        out.push_str(&format!("- **`{}`** (`{}`) | Degree: {} (In: {}, Out: {}) | C{} ({})\n", g.title, g.node_type, g.degree, g.in_degree, g.out_degree, g.community_id, g.role));
                    }

                    out.push_str("\n### 🧩 Modular Subsystem Communities\n");
                    for c in &report.communities {
                        out.push_str(&format!("- **C{}**: {} ({} nodes) - Key nodes: {}\n", c.id, c.name, c.node_count, c.key_nodes.join(", ")));
                    }

                    if !report.surprises.is_empty() {
                        out.push_str("\n### ⚡ Architectural Surprises (Cross-Couplings)\n");
                        for (i, s) in report.surprises.iter().take(8).enumerate() {
                            out.push_str(&format!("{}. `{}` (C{}) -> `{}` (C{}) [{}]\n   *{}*\n", i + 1, s.source_title, s.source_community, s.target_title, s.target_community, s.label, s.reason));
                        }
                    }

                    Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
                }
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error generating report: {}", e))])),
        }
    }

    /// Exports graph.json, GRAPH_REPORT.md, and standalone graph.html to disk.
    #[tool(description = "Exports graph.json, GRAPH_REPORT.md, and standalone graph.html to disk.")]
    async fn export_graph(&self, Parameters(args): Parameters<ExportGraphArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let out_path = args.out_dir.as_ref().map(std::path::Path::new);
        match engine.export_artifacts(out_path).await {
            Ok(summary) => {
                let out = format!(
                    "Knowledge graph artifacts exported successfully to `{}`:\n\n- **Audit Report**: `{}`\n- **Graph Schema**: `{}`\n- **Offline Visual Canvas**: `{}`\n\nTotal Nodes: {} | Connections: {} | Communities: {} | God Nodes: {}",
                    summary.output_dir, summary.report_markdown_path, summary.graph_json_path, summary.standalone_html_path,
                    summary.total_nodes, summary.total_edges, summary.communities_count, summary.god_nodes_count
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error exporting graph: {}", e))])),
        }
    }
}

#[tool_handler]
impl ServerHandler for GraphFinderMcpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder().enable_tools().build(),
        )
        .with_server_info(Implementation::new("graphfinder", env!("CARGO_PKG_VERSION")))
        .with_instructions("GraphFinder Neural Knowledge Graph & Local Semantic Vault MCP Server. Delivers atomic semantic search, topological community clustering, God Nodes identification, multi-hop pathfinding, structural context compression, and local-first vault persistence.".to_string())
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
        let instructions_content = r#"# GraphFinder Universal MCP Agent Instructions

GraphFinder is a neural knowledge graph and universal codebase intelligence engine. It provides 24 atomic tools across 4 categories:

## 1. Architectural & Topological Audit
- `get_community_report`: Global map of modular subsystems, architectural God Nodes (central hubs), and unexpected cross-couplings.
- `explain_node`: 360-degree context card for a file/symbol (centrality, callers, callees, community).
- `find_path`: BFS shortest dependency path connecting any two files, symbols, or notes.
- `export_graph`: Exports standalone `graph.json`, `GRAPH_REPORT.md`, and visual `graph.html`.

## 2. Multi-Language Codebase Intelligence
- `search_code`: Hybrid semantic (BM25 + neural) search for symbols, functions, and logic across code repositories.
- `find_files`: Fast glob and regex pattern matching across project files.
- `get_file_context`: High-density structural overview of a file (imports, dependents, exported symbols) without reading the entire file.
- `read_code_chunk`: Surgical extraction of a specific function, class, or line range.
- `get_code_dependencies`: Forward and reverse dependency analysis for any file.

## 3. Semantic Vault & Knowledge Search
- `search_semantic`: Atomic vector search over knowledge notes with deep links (`[[Note#Section]]`).
- `get_compressed_context`: Compressed structural map and token estimates for a note.
- `read_section`: Targeted reading of a single H2/H3 section.
- `get_vault_stats`: Metrics on indexed notes, chunks, and embeddings.
- `get_graph_neighborhood`: 1-hop or 2-hop graph adjacency.

## 4. Vault Synchronization & Mutation
- `sync_vault`: Incrementally re-indexes notes and codebases via SHA-256 change detection.
- `store_context`: Saves a new markdown note with automatic semantic chunking and tag taxonomy.
- `suggest_links`: Evaluates draft text and suggests semantic wikilinks to existing vault notes.
- `link_notes`: Creates bidirectional relationships between notes.
- `rename_note`, `delete_note`: Knowledge maintenance operations.
- `list_projects`, `switch_project`, `create_project`: Multi-workspace management.

## Golden Rules for AI Agents:
1. Prefer atomic excerpts: Use `search_semantic` or `read_code_chunk` rather than reading entire files into context.
2. Verify connections: Use `suggest_links` before saving new knowledge with `store_context`.
3. Audit architecture first: Use `get_community_report()` when onboarding or inspecting unfamiliar systems.
"#;
        let _ = std::fs::write(&instr_path, instructions_content);
    }

    names
}


