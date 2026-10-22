// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use app_lib::engine::{clean_path_str, EmbeddingEngine, VaultManager};

#[cfg(windows)]
fn attach_console() {
    #[link(name = "kernel32")]
    extern "system" {
        fn AttachConsole(dwProcessId: u32) -> i32;
    }
    const ATTACH_PARENT_PROCESS: u32 = 0xFFFFFFFF;
    unsafe {
        AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(not(windows))]
fn attach_console() {}

fn print_help() {
    println!(r#"
======================================================================
  GraphFinder - Neural Knowledge Graph & Local Semantic Vault
======================================================================

USAGE:
  graphfinder [COMMAND] [OPTIONS]

COMMANDS:
  sync, syn, index [path]
      Indexes Markdown notes and codebase in the specified directory
      (or current directory if omitted).
      If already indexed, updates incrementally via SHA-256 hashes.
      If new, automatically registers a new isolated project.
      Options:
        --json      Returns structured JSON output (ideal for scripts/skills)
        --export    Also generates graph.json, GRAPH_REPORT.md, and graph.html

  report, audit [path] [--out-dir <dir>] [--json]
      Performs topological community clustering (Louvain/LPA), identifies
      God Nodes (central hubs), detects cross-community surprises and cycles,
      and produces standalone artifacts in graphfinder-out/:
        • GRAPH_REPORT.md  (High-density architectural audit)
        • graph.json       (Standardized schema consumed by AI agents)
        • graph.html       (Zero-dependency standalone canvas visualizer)

  path <from> <to> [--path <path>] [--json]
      Finds the shortest multi-hop dependency path connecting any two
      symbols, files, or notes (e.g. graphfinder path "App.tsx" "db.rs").

  explain <node> [--path <path>] [--json]
      Generates 360° architectural context card for a node: community,
      centrality rank, callers, callees, and sibling dependencies.

  export [path] [--out-dir <dir>]
      Generates graph.json, GRAPH_REPORT.md, and graph.html without terminal report.

  status, info [path]
      Displays directory indexing statistics (notes, code, graph, db).

  projects, list
      Lists all registered projects and paths in GraphFinder.

  search <query> [--path <path>] [--limit <n>]
      Executes hybrid semantic search (BM25 + neural embeddings) in project.

  mcp, --mcp
      Starts the headless stdio MCP server for AI agent integration
      (Cursor, Claude Desktop, Antigravity, Cline, etc.).

  help, --help, -h
      Displays this help information.

  (no arguments)
      Launches the desktop graphical interface (GUI).

EXAMPLES:
  graphfinder sync
  graphfinder report
  graphfinder path "GraphView.tsx" "DbManager"
  graphfinder explain "DbManager"
  graphfinder export --out-dir ./artifacts
  graphfinder search "JWT authentication"
  graphfinder mcp
======================================================================
"#);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();

    // Se nenhum argumento foi passado, inicia a interface gráfica desktop (Tauri)
    if args.len() <= 1 {
        app_lib::run();
        return Ok(());
    }

    // Argumentos de terminal presentes: conecta ao console pai (no Windows)
    attach_console();

    let cmd = args[1].to_lowercase();

    // 1. Ajuda
    if cmd == "help" || cmd == "--help" || cmd == "-h" {
        print_help();
        return Ok(());
    }

    // 2. Headless MCP Stdio Mode
    if cmd == "--mcp" || cmd == "mcp" || args.iter().any(|a| a == "--mcp" || a == "mcp") {
        eprintln!("[GraphFinder MCP] Headless stdio mode activated.");
        let embeddings = Arc::new(EmbeddingEngine::new_lazy());
        let _ = embeddings.init_model(None).await;
        let manager = Arc::new(VaultManager::new(embeddings)?);
        let engine = manager.active_engine().await;
        let _ = engine.sync_vault().await;

        app_lib::mcp::run_mcp_server(manager).await?;
        return Ok(());
    }

    // 3. Command: sync / syn / index
    if cmd == "sync" || cmd == "syn" || cmd == "index" {
        let is_json = args.iter().any(|a| a == "--json");
        // Target path argument is first positional arg after command not starting with '--'
        let target_arg = args
            .iter()
            .skip(2)
            .find(|a| !a.starts_with("--"))
            .map(|s| s.as_str())
            .unwrap_or(".");

        let target_dir = Path::new(target_arg);
        let abs_path = if target_dir.is_absolute() {
            target_dir.to_path_buf()
        } else {
            std::env::current_dir()?.join(target_dir)
        };

        let canonical = std::fs::canonicalize(&abs_path).unwrap_or(abs_path);
        let display_path = clean_path_str(&canonical);

        if !canonical.exists() {
            eprintln!("Error: Directory '{}' does not exist.", display_path);
            std::process::exit(1);
        }

        if !is_json {
            println!("┌────────────────────────────────────────────────────────────┐");
            println!("│ GraphFinder Indexer & Knowledge Engine                     │");
            println!("└────────────────────────────────────────────────────────────┘");
            println!("Target Directory : {}", display_path);
            println!("Initializing FastEmbed neural engine...");
        }

        let embeddings = Arc::new(EmbeddingEngine::new_lazy());
        if let Err(e) = embeddings.init_model(None).await {
            eprintln!("Warning loading neural embeddings: {}", e);
        }

        let manager = Arc::new(VaultManager::new(Arc::clone(&embeddings))?);

        let (project_id, engine, is_new) = match manager.get_or_register_project_by_path(&canonical).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error registering project for path: {}", e);
                std::process::exit(1);
            }
        };

        if !is_json {
            let status_msg = if is_new {
                "New project registered automatically"
            } else {
                "Existing project detected (incremental SHA-256 update)"
            };
            println!("Project          : {} ({})", project_id, status_msg);
            println!("Indexing markdown notes, source code, and AST graph...");
        }

        let start_time = std::time::Instant::now();
        let sync_result = engine.sync_vault().await;
        let elapsed = start_time.elapsed();

        match sync_result {
            Ok(changed_count) => {
                let stats = engine.get_vault_stats().await.unwrap_or(app_lib::engine::VaultStats {
                    total_notes: 0,
                    total_chunks: 0,
                    total_links: 0,
                    total_tags: 0,
                    total_tokens: 0,
                    total_code_files: 0,
                    total_code_chunks: 0,
                    total_code_links: 0,
                    languages: Vec::new(),
                });

                if is_json {
                    let out = serde_json::json!({
                        "status": "success",
                        "project_id": project_id,
                        "path": display_path,
                        "is_new": is_new,
                        "changed_items": changed_count,
                        "elapsed_ms": elapsed.as_millis(),
                        "notes": stats.total_notes,
                        "chunks": stats.total_chunks,
                        "code_files": stats.total_code_files,
                        "code_chunks": stats.total_code_chunks,
                        "total_links": stats.total_links,
                        "languages": stats.languages,
                        "db_path": clean_path_str(engine.db_path()),
                    });
                    println!("{}", serde_json::to_string_pretty(&out)?);
                } else {
                    println!("\n[OK] Synchronization completed successfully in {:.2}s!", elapsed.as_secs_f32());
                    println!("────────────────────────────────────────────────────────────");
                    println!("  Markdown Notes   : {} ({} vectorized sections/chunks)", stats.total_notes, stats.total_chunks);
                    println!("  Code Files       : {} ({} AST symbols/functions)", stats.total_code_files, stats.total_code_chunks);
                    println!("  Graph Connections: {} (wikilinks + dependency imports)", stats.total_links);
                    println!("  Languages        : {}", if stats.languages.is_empty() { "None".to_string() } else { stats.languages.join(", ") });
                    println!("  Changed Items    : {}", changed_count);
                    println!("  SQLite Database  : {}", clean_path_str(engine.db_path()));
                    if args.iter().any(|a| a == "--export") {
                        if let Ok(exp) = engine.export_artifacts(None).await {
                            println!("  Artifacts Exported: {}", exp.output_dir);
                        }
                    }
                    println!("────────────────────────────────────────────────────────────\n");
                }
            }
            Err(e) => {
                eprintln!("Error during synchronization: {}", e);
                std::process::exit(1);
            }
        }
        return Ok(());
    }

    // 4. Command: status / info
    if cmd == "status" || cmd == "info" {
        let target_arg = args.get(2).map(|s| s.as_str()).unwrap_or(".");
        let target_dir = Path::new(target_arg);
        let abs_path = if target_dir.is_absolute() {
            target_dir.to_path_buf()
        } else {
            std::env::current_dir()?.join(target_dir)
        };
        let canonical = std::fs::canonicalize(&abs_path).unwrap_or(abs_path);
        let display_path = clean_path_str(&canonical);

        let embeddings = Arc::new(EmbeddingEngine::new_lazy());
        let manager = Arc::new(VaultManager::new(embeddings)?);

        let (project_id, engine, is_new) = match manager.get_or_register_project_by_path(&canonical).await {
            Ok(res) => res,
            Err(e) => {
                eprintln!("Error checking path: {}", e);
                std::process::exit(1);
            }
        };

        let stats = engine.get_vault_stats().await.unwrap_or(app_lib::engine::VaultStats {
            total_notes: 0,
            total_chunks: 0,
            total_links: 0,
            total_tags: 0,
            total_tokens: 0,
            total_code_files: 0,
            total_code_chunks: 0,
            total_code_links: 0,
            languages: Vec::new(),
        });

        println!("┌────────────────────────────────────────────────────────────┐");
        println!("│ GraphFinder Project Status                                 │");
        println!("└────────────────────────────────────────────────────────────┘");
        println!("Path             : {}", display_path);
        println!("Project ID       : {}", project_id);
        println!("Status           : {}", if is_new { "Newly registered" } else { "Registered" });
        println!("Notes            : {} ({} sections)", stats.total_notes, stats.total_chunks);
        println!("Code             : {} files ({} symbols)", stats.total_code_files, stats.total_code_chunks);
        println!("Connections      : {} graph links", stats.total_links);
        println!("Languages        : {}", if stats.languages.is_empty() { "None".to_string() } else { stats.languages.join(", ") });
        println!("SQLite Database  : {}", clean_path_str(engine.db_path()));
        println!("────────────────────────────────────────────────────────────\n");
        return Ok(());
    }

    // 5. Command: projects / list
    if cmd == "projects" || cmd == "list" {
        let embeddings = Arc::new(EmbeddingEngine::new_lazy());
        let manager = Arc::new(VaultManager::new(embeddings)?);
        let list = manager.list_projects().await;

        println!("┌────────────────────────────────────────────────────────────┐");
        println!("│ Registered Projects in GraphFinder                         │");
        println!("└────────────────────────────────────────────────────────────┘");
        for (i, p) in list.iter().enumerate() {
            let active_mark = if p.is_active { " * (active)" } else { "" };
            println!("{}. {} [{}] {}", i + 1, p.name, p.id, active_mark);
            println!("   Path:     {}", p.path);
            println!("   Notes:    {}", p.note_count);
            println!("   Database: {}", p.db_path);
            println!();
        }
        return Ok(());
    }

    // 6. Command: search
    if cmd == "search" {
        if args.len() < 3 {
            eprintln!("Usage: graphfinder search <query> [--path <path>] [--limit <n>]");
            std::process::exit(1);
        }
        let query = &args[2];
        let limit = args
            .iter()
            .position(|a| a == "--limit")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(5);

        let target_dir = args
            .iter()
            .position(|a| a == "--path")
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let embeddings = Arc::new(EmbeddingEngine::new_lazy());
        let _ = embeddings.init_model(None).await;
        let manager = Arc::new(VaultManager::new(Arc::clone(&embeddings))?);
        let (_, engine, _) = manager.get_or_register_project_by_path(&target_dir).await?;

        println!("Searching for '{}'...", query);
        let note_hits = engine.search_chunks(query, limit).await.unwrap_or_default();
        let code_hits = engine.search_code(query, limit, None).await.unwrap_or_default();

        println!("\n=== Matching Notes (Hybrid) ===");
        if note_hits.is_empty() {
            println!("(No matching notes)");
        } else {
            for h in note_hits {
                println!("• [[{}]] (score: {:.2})", h.deep_link, h.score);
                println!("  {}", h.content.lines().next().unwrap_or(""));
            }
        }

        println!("\n=== Matching Code (AST & Semantic) ===");
        if code_hits.is_empty() {
            println!("(No matching code)");
        } else {
            for h in code_hits {
                println!("• [[{}]] - {} [{}:{}] (score: {:.2})", h.deep_link, h.language, h.start_line, h.end_line, h.score);
                println!("  {}", h.content.lines().next().unwrap_or(""));
            }
        }
        println!();
        return Ok(());
    }

    // 7. Command: report / audit
    if cmd == "report" || cmd == "audit" {
        let is_json = args.iter().any(|a| a == "--json");
        let target_arg = args
            .iter()
            .skip(2)
            .find(|a| !a.starts_with("--"))
            .map(|s| s.as_str())
            .unwrap_or(".");

        let target_dir = Path::new(target_arg);
        let abs_path = if target_dir.is_absolute() {
            target_dir.to_path_buf()
        } else {
            std::env::current_dir()?.join(target_dir)
        };
        let canonical = std::fs::canonicalize(&abs_path).unwrap_or(abs_path);

        let custom_out_dir = args
            .iter()
            .position(|a| a == "--out-dir")
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from);

        let embeddings = Arc::new(EmbeddingEngine::new_lazy());
        let manager = Arc::new(VaultManager::new(embeddings)?);
        let (project_id, engine, _) = manager.get_or_register_project_by_path(&canonical).await?;

        let report = engine.analyze_graph().await?;
        let export_summary = engine.export_artifacts(custom_out_dir.as_deref()).await?;

        if is_json {
            let out = serde_json::json!({
                "project_id": project_id,
                "report": report,
                "artifacts": export_summary,
            });
            println!("{}", serde_json::to_string_pretty(&out)?);
        } else {
            println!("┌────────────────────────────────────────────────────────────┐");
            println!("│ GraphFinder Architectural Topology Report                  │");
            println!("└────────────────────────────────────────────────────────────┘");
            println!("Project          : {}", project_id);
            println!("Total Nodes      : {} | Total Connections: {}", report.total_nodes, report.total_edges);
            println!("Graph Density    : {:.4}", report.density);
            println!("Communities (LPA): {}", report.communities.len());
            println!("God Nodes (Hubs) : {}", report.god_nodes.len());
            println!("Cross Surprises  : {}", report.surprises.len());
            println!("Dependency Cycles: {}", report.cycles.len());
            println!("────────────────────────────────────────────────────────────");
            println!("\nTop Architectural Hubs (God Nodes):");
            for (i, g) in report.god_nodes.iter().take(6).enumerate() {
                println!("  {}. {} [{}] - Degree: {} (In: {}, Out: {}) | {}", i + 1, g.title, g.node_type, g.degree, g.in_degree, g.out_degree, g.role);
            }
            println!("\nDetected Modular Subsystems:");
            for c in report.communities.iter().take(6) {
                println!("  • C{}: {} ({} nodes) - Stack: {}", c.id, c.name, c.node_count, c.dominant_type);
            }
            println!("\nStandalone Artifacts Generated:");
            println!("  • Audit Report : {}", export_summary.report_markdown_path);
            println!("  • Graph Schema : {}", export_summary.graph_json_path);
            println!("  • Visual Canvas: {}", export_summary.standalone_html_path);
            println!("────────────────────────────────────────────────────────────\n");
        }
        return Ok(());
    }

    // 8. Command: path
    if cmd == "path" {
        if args.len() < 4 {
            eprintln!("Usage: graphfinder path <from> <to> [--path <path>] [--json]");
            std::process::exit(1);
        }
        let from = &args[2];
        let to = &args[3];
        let is_json = args.iter().any(|a| a == "--json");

        let target_dir = args
            .iter()
            .position(|a| a == "--path")
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let embeddings = Arc::new(EmbeddingEngine::new_lazy());
        let manager = Arc::new(VaultManager::new(embeddings)?);
        let (_, engine, _) = manager.get_or_register_project_by_path(&target_dir).await?;

        let path_opt = engine.find_path(from, to).await?;

        if is_json {
            println!("{}", serde_json::to_string_pretty(&path_opt)?);
        } else {
            match path_opt {
                Some(p) => {
                    println!("\nShortest Dependency Path: '{}' ➔ '{}' ({} hops)", p.source, p.target, p.length);
                    println!("────────────────────────────────────────────────────────────");
                    for (i, step) in p.steps.iter().enumerate() {
                        let prefix = if i == 0 { "START" } else { &step.direction };
                        let edge_info = step.edge_label.as_deref().map(|l| format!(" [{}]", l)).unwrap_or_default();
                        println!("  {} {} [{}] {}{}", prefix, step.title, step.node_type, step.node_id, edge_info);
                    }
                    println!("────────────────────────────────────────────────────────────\n");
                }
                None => {
                    println!("\nNo path found connecting '{}' to '{}'.\n", from, to);
                }
            }
        }
        return Ok(());
    }

    // 9. Command: explain
    if cmd == "explain" {
        if args.len() < 3 {
            eprintln!("Usage: graphfinder explain <node> [--path <path>] [--json]");
            std::process::exit(1);
        }
        let target_node = &args[2];
        let is_json = args.iter().any(|a| a == "--json");

        let target_dir = args
            .iter()
            .position(|a| a == "--path")
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from)
            .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

        let embeddings = Arc::new(EmbeddingEngine::new_lazy());
        let manager = Arc::new(VaultManager::new(embeddings)?);
        let (_, engine, _) = manager.get_or_register_project_by_path(&target_dir).await?;

        let explanation = engine.explain_node(target_node).await?;

        if is_json {
            println!("{}", serde_json::to_string_pretty(&explanation)?);
        } else {
            match explanation {
                Some(e) => {
                    let god_tag = if e.is_god_node { " 🌟 [GOD NODE / CENTRAL HUB]" } else { "" };
                    println!("┌────────────────────────────────────────────────────────────┐");
                    println!("│ Node Context Card: {}{}", e.title, god_tag);
                    println!("└────────────────────────────────────────────────────────────┘");
                    println!("Type             : {}", e.node_type);
                    if let Some(p) = e.path {
                        println!("Path             : {}", p);
                    }
                    println!("Community        : C{} ({})", e.community_id, e.community_name);
                    println!("Degree           : {} (In: {}, Out: {}) - Centrality Rank #{}", e.degree, e.in_degree, e.out_degree, e.centrality_rank);
                    if let Some(s) = e.summary {
                        println!("Summary          : {}", s);
                    }
                    println!("Inbound Callers  : {}", if e.incoming_dependents.is_empty() { "None".to_string() } else { e.incoming_dependents.join(", ") });
                    println!("Dependencies     : {}", if e.outgoing_dependencies.is_empty() { "None".to_string() } else { e.outgoing_dependencies.join(", ") });
                    println!("Community Siblings: {}", if e.sibling_nodes.is_empty() { "None".to_string() } else { e.sibling_nodes.join(", ") });
                    println!("────────────────────────────────────────────────────────────\n");
                }
                None => {
                    println!("\nEntity '{}' not found in indexed graph.\n", target_node);
                }
            }
        }
        return Ok(());
    }

    // 10. Command: export
    if cmd == "export" {
        let target_arg = args
            .iter()
            .skip(2)
            .find(|a| !a.starts_with("--"))
            .map(|s| s.as_str())
            .unwrap_or(".");

        let target_dir = Path::new(target_arg);
        let abs_path = if target_dir.is_absolute() {
            target_dir.to_path_buf()
        } else {
            std::env::current_dir()?.join(target_dir)
        };
        let canonical = std::fs::canonicalize(&abs_path).unwrap_or(abs_path);

        let custom_out_dir = args
            .iter()
            .position(|a| a == "--out-dir")
            .and_then(|i| args.get(i + 1))
            .map(PathBuf::from);

        let embeddings = Arc::new(EmbeddingEngine::new_lazy());
        let manager = Arc::new(VaultManager::new(embeddings)?);
        let (_, engine, _) = manager.get_or_register_project_by_path(&canonical).await?;

        let exp = engine.export_artifacts(custom_out_dir.as_deref()).await?;
        println!("[OK] Knowledge graph artifacts exported successfully!");
        println!("  • Directory : {}", exp.output_dir);
        println!("  • Graph JSON: {}", exp.graph_json_path);
        println!("  • Report MD : {}", exp.report_markdown_path);
        println!("  • HTML View : {}", exp.standalone_html_path);
        println!();
        return Ok(());
    }

    // If command not recognized, print help
    eprintln!("Unknown command: '{}'", cmd);
    print_help();
    std::process::exit(1);
}
