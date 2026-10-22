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
    pub target: String,
    /// Target project/vault ID (optional)
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

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SyncVaultArgs {
    /// Target project/vault ID (optional)
    pub project: Option<String>,
    /// Local directory path to sync/index (optional). Auto-detects if already indexed or creates a new project.
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadSectionArgs {
    /// Note title or ID
    pub target: String,
    /// Section heading (H2/H3) or deep link fragment (e.g. '2. SQLite WAL Configuration')
    pub section: String,
    /// Target project/vault ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenameNoteArgs {
    /// Current title or ID of the note
    pub old_title: String,
    /// Desired new title for the note
    pub new_title: String,
    /// Target project/vault ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteNoteArgs {
    /// Note title or ID to delete
    pub target: String,
    /// Target project/vault ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetVaultStatsArgs {
    /// Target project/vault ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchCodeArgs {
    /// Semantic or exact search query for code symbols, functions, or concepts
    pub query: String,
    /// Maximum code chunks to return (default: 5)
    pub limit: Option<usize>,
    /// Filter by programming language (e.g. 'rust', 'typescript', 'python', optional)
    pub language: Option<String>,
    /// Target project/vault ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindFilesArgs {
    /// Glob pattern or file path match (e.g. '*.tsx', 'db.rs', 'engine', optional)
    pub pattern: Option<String>,
    /// Filter by programming language (e.g. 'rust', 'typescript', optional)
    pub language: Option<String>,
    /// Maximum files to return (default: 20)
    pub limit: Option<usize>,
    /// Target project/vault ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetFileContextArgs {
    /// Relative path or file name in codebase (e.g. 'src/App.tsx', 'main.rs')
    pub file_path: String,
    /// Target project/vault ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadCodeChunkArgs {
    /// Relative path or file name in codebase
    pub file_path: String,
    /// Target symbol name (function/class/struct) or line range (optional)
    pub symbol_or_lines: Option<String>,
    /// Target project/vault ID (optional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCodeDependenciesArgs {
    /// Relative path of code file (e.g. 'src/App.tsx')
    pub file_path: String,
    /// Dependency direction: 'imports' (what it imports), 'imported_by' (who imports it), or 'both' (default)
    pub direction: Option<String>,
    /// Target project/vault ID (optional)
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
        let project = match self.manager.create_project(&args.name, args.path).await {
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
        let project = match self.manager.switch_project(&args.project_id).await {
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
                    "Dependencies for `{}`:\n\n- Imported Files ({}):\n{}\n\n- Files Importing This File ({}):\n{}\n\n- External Modules/Packages ({}):\n{}",
                    deps.file_id,
                    deps.imports.len(),
                    if deps.imports.is_empty() { "  (none)".to_string() } else { deps.imports.iter().map(|i| format!("  - `{}`", i)).collect::<Vec<_>>().join("\n") },
                    deps.imported_by.len(),
                    if deps.imported_by.is_empty() { "  (none)".to_string() } else { deps.imported_by.iter().map(|i| format!("  - `{}`", i)).collect::<Vec<_>>().join("\n") },
                    deps.external.len(),
                    if deps.external.is_empty() { "  (none)".to_string() } else { deps.external.iter().map(|i| format!("  - `{}`", i)).collect::<Vec<_>>().join("\n") },
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Error retrieving dependencies: {}", e))])),
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
        .with_instructions("GraphFinder Neural Knowledge Graph & Local Semantic Vault MCP Server. Delivers atomic semantic search, structural context compression, wikilink recommendations, and local-first vault persistence.".to_string())
    }
}

pub async fn run_mcp_server(
    manager: Arc<VaultManager>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = GraphFinderMcpServer::new(manager, None);
    let (read, write) = rmcp::transport::io::stdio();
    server.serve((read, write)).await?;
    Ok(())
}

