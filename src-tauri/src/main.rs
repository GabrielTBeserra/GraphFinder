// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};
use std::sync::Arc;
use app_lib::engine::{clean_path_str, EmbeddingEngine, SyncPhase, SyncProgress, VaultManager};

fn format_eta(seconds: u64) -> String {
    if seconds >= 60 {
        format!("{}m {:02}s", seconds / 60, seconds % 60)
    } else {
        format!("{}s", seconds)
    }
}

fn print_sync_progress(progress: &SyncProgress) {
    use std::io::Write;

    let current_file = progress
        .current_file
        .as_deref()
        .map(|path| path.chars().take(56).collect::<String>())
        .unwrap_or_default();
    let message = match progress.phase {
        SyncPhase::Scanning => format!(
            "Scanning repository - {} files found {}",
            progress.completed, current_file
        ),
        SyncPhase::Indexing => format!(
            "Indexing {}/{} ({}%) - ETA {} {}",
            progress.completed,
            progress.total,
            progress.percent.unwrap_or(0),
            progress
                .eta_seconds
                .map(format_eta)
                .unwrap_or_else(|| "estimating".to_string()),
            current_file
        ),
        SyncPhase::Finalizing => "Finalizing index (99%)".to_string(),
        SyncPhase::Complete => "Synchronization complete (100%)".to_string(),
    };
    let mut stdout = std::io::stdout().lock();
    let _ = write!(stdout, "\r{message:<140}");
    let _ = stdout.flush();
}

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

fn install_to_os_path(exe_path: &Path) -> (bool, String) {
    let exe_dir = match exe_path.parent() {
        Some(d) => d,
        None => return (false, "Could not determine executable directory".to_string()),
    };
    let exe_dir_str = exe_dir.to_string_lossy().to_string();

    #[cfg(windows)]
    {
        // 1. Create graphfinder.cmd in the executable's directory so 'graphfinder' works identically to 'GraphFinder.exe'
        let cmd_wrapper = exe_dir.join("graphfinder.cmd");
        let cmd_content = format!("@echo off\r\n\"{}\" %*\r\n", exe_path.to_string_lossy());
        let _ = std::fs::write(&cmd_wrapper, cmd_content);

        // 2. Place a shim in ~/.cargo/bin if it exists (instant zero-restart terminal access)
        let mut shim_created = false;
        if let Some(home) = dirs::home_dir() {
            let cargo_bin = home.join(".cargo").join("bin");
            if cargo_bin.exists() {
                let cargo_shim = cargo_bin.join("graphfinder.cmd");
                let shim_content = format!("@echo off\r\n\"{}\" %*\r\n", exe_path.to_string_lossy());
                if std::fs::write(&cargo_shim, shim_content).is_ok() {
                    shim_created = true;
                }
            }
        }

        // 3. Add exe_dir to Windows User PATH via PowerShell / Registry
        let ps_script = format!(
            "$dir = '{}'; \
             $curr = [Environment]::GetEnvironmentVariable('Path', 'User'); \
             if ($curr -notlike \"*$dir*\") {{ \
                 $new = if ($curr) {{ \"$curr;$dir\" }} else {{ $dir }}; \
                 [Environment]::SetEnvironmentVariable('Path', $new, 'User'); \
                 Write-Output 'ADDED'; \
             }} else {{ \
                 Write-Output 'ALREADY_EXISTS'; \
             }}",
            exe_dir_str.replace('\'', "''")
        );

        let output = std::process::Command::new("powershell")
            .args(["-NoProfile", "-Command", &ps_script])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let mut msg = if stdout.contains("ADDED") {
                    format!("Added '{}' to User PATH", exe_dir_str)
                } else {
                    format!("'{}' is in User PATH", exe_dir_str)
                };
                if shim_created {
                    msg.push_str(" (+ active shim in ~/.cargo/bin)");
                }
                (true, msg)
            }
            Err(e) => {
                if shim_created {
                    (true, format!("Active shim created in ~/.cargo/bin (PowerShell update: {})", e))
                } else {
                    (false, format!("Failed to update User PATH: {}", e))
                }
            }
        }
    }

    #[cfg(not(windows))]
    {
        let mut linked = false;
        let mut target_used = String::new();

        if let Some(home) = dirs::home_dir() {
            let local_bin = home.join(".local").join("bin");
            let _ = std::fs::create_dir_all(&local_bin);
            let symlink = local_bin.join("graphfinder");
            let _ = std::fs::remove_file(&symlink);
            #[cfg(unix)]
            if std::os::unix::fs::symlink(exe_path, &symlink).is_ok() {
                linked = true;
                target_used = symlink.to_string_lossy().to_string();
            }
        }

        if linked {
            (true, format!("Created symlink in '{}'", target_used))
        } else {
            (false, "Could not create symlink in ~/.local/bin".to_string())
        }
    }
}

fn auto_install_mcp_config(exe_path: &Path) -> Vec<(String, PathBuf, bool, String)> {
    let exe_str = exe_path.to_string_lossy().to_string();
    let mut results = Vec::new();

    // 0. Install to OS System PATH
    let (path_ok, path_msg) = install_to_os_path(exe_path);
    results.push((
        "System OS PATH".to_string(),
        exe_path.parent().unwrap_or(exe_path).to_path_buf(),
        path_ok,
        path_msg,
    ));

    let mut candidates: Vec<(&str, PathBuf)> = Vec::new();

    // 1. Current workspace Cursor directory: .cursor/mcp.json
    let local_cursor = PathBuf::from(".cursor");
    if local_cursor.exists() {
        candidates.push(("Cursor (Workspace)", local_cursor.join("mcp.json")));
    }

    // 2. Claude Desktop
    #[cfg(windows)]
    if let Some(appdata) = dirs::data_dir() {
        candidates.push(("Claude Desktop", appdata.join("Claude").join("claude_desktop_config.json")));
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = dirs::home_dir() {
        candidates.push(("Claude Desktop", home.join("Library").join("Application Support").join("Claude").join("claude_desktop_config.json")));
    }
    #[cfg(target_os = "linux")]
    if let Some(config) = dirs::config_dir() {
        candidates.push(("Claude Desktop", config.join("Claude").join("claude_desktop_config.json")));
    }

    // 3. Cursor Global User Settings
    #[cfg(windows)]
    if let Some(appdata) = dirs::data_dir() {
        candidates.push(("Cursor (User Global)", appdata.join("Cursor").join("User").join("globalStorage").join("cursor.mcp").join("mcp.json")));
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = dirs::home_dir() {
        candidates.push(("Cursor (User Global)", home.join("Library").join("Application Support").join("Cursor").join("User").join("globalStorage").join("cursor.mcp").join("mcp.json")));
    }
    #[cfg(target_os = "linux")]
    if let Some(config) = dirs::config_dir() {
        candidates.push(("Cursor (User Global)", config.join("Cursor").join("User").join("globalStorage").join("cursor.mcp").join("mcp.json")));
    }
    if let Some(home) = dirs::home_dir() {
        let cursor_home = home.join(".cursor");
        if cursor_home.exists() {
            candidates.push(("Cursor (Home)", cursor_home.join("mcp.json")));
        }
    }

    // 4. Windsurf (Codeium)
    if let Some(home) = dirs::home_dir() {
        let windsurf_dir = home.join(".codeium").join("windsurf");
        if windsurf_dir.exists() {
            candidates.push(("Windsurf", windsurf_dir.join("mcp_config.json")));
        }
    }

    // 5. VS Code (Cline / Roo Code)
    #[cfg(windows)]
    if let Some(appdata) = dirs::data_dir() {
        let roo_dir = appdata.join("Code").join("User").join("globalStorage").join("rooveterinaryinc.roo-cline").join("settings");
        if roo_dir.exists() {
            candidates.push(("VS Code Roo-Cline", roo_dir.join("cline_mcp_settings.json")));
        }
        let cline_dir = appdata.join("Code").join("User").join("globalStorage").join("saoudrizwan.claude-dev").join("settings");
        if cline_dir.exists() {
            candidates.push(("VS Code Cline", cline_dir.join("cline_mcp_settings.json")));
        }
    }
    #[cfg(target_os = "macos")]
    if let Some(home) = dirs::home_dir() {
        let roo_dir = home.join("Library").join("Application Support").join("Code").join("User").join("globalStorage").join("rooveterinaryinc.roo-cline").join("settings");
        if roo_dir.exists() {
            candidates.push(("VS Code Roo-Cline", roo_dir.join("cline_mcp_settings.json")));
        }
        let cline_dir = home.join("Library").join("Application Support").join("Code").join("User").join("globalStorage").join("saoudrizwan.claude-dev").join("settings");
        if cline_dir.exists() {
            candidates.push(("VS Code Cline", cline_dir.join("cline_mcp_settings.json")));
        }
    }
    #[cfg(target_os = "linux")]
    if let Some(config) = dirs::config_dir() {
        let roo_dir = config.join("Code").join("User").join("globalStorage").join("rooveterinaryinc.roo-cline").join("settings");
        if roo_dir.exists() {
            candidates.push(("VS Code Roo-Cline", roo_dir.join("cline_mcp_settings.json")));
        }
        let cline_dir = config.join("Code").join("User").join("globalStorage").join("saoudrizwan.claude-dev").join("settings");
        if cline_dir.exists() {
            candidates.push(("VS Code Cline", cline_dir.join("cline_mcp_settings.json")));
        }
    }

    // 6. Generic Gemini / Antigravity (if present)
    if let Some(home) = dirs::home_dir() {
        let gemini_cfg_dir = home.join(".gemini").join("config");
        if gemini_cfg_dir.exists() {
            candidates.push(("Antigravity / Gemini", gemini_cfg_dir.join("mcp_config.json")));
        }
    }

    for (name, path) in candidates {
        if let Some(parent) = path.parent() {
            if parent.exists() {
                match update_mcp_json_file(&path, &exe_str) {
                    Ok(_) => results.push((name.to_string(), path, true, "Configured successfully".to_string())),
                    Err(e) => results.push((name.to_string(), path, false, format!("Error: {}", e))),
                }
            }
        }
    }

    results
}

fn update_mcp_json_file(path: &Path, exe_str: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut root: serde_json::Value = if path.exists() {
        let content = std::fs::read_to_string(path)?;
        serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({}))
    } else {
        serde_json::json!({})
    };

    if !root.is_object() {
        root = serde_json::json!({});
    }

    let obj = root.as_object_mut().unwrap();
    if !obj.contains_key("mcpServers") || !obj["mcpServers"].is_object() {
        obj.insert("mcpServers".to_string(), serde_json::json!({}));
    }

    let servers = obj.get_mut("mcpServers").unwrap().as_object_mut().unwrap();
    servers.insert(
        "graphfinder".to_string(),
        serde_json::json!({
            "command": exe_str,
            "args": ["--mcp"]
        }),
    );

    let formatted = serde_json::to_string_pretty(&root)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, formatted)?;
    Ok(())
}

fn print_help() {
    println!(r#"
======================================================================
  GraphFinder - Universal Neural Knowledge Graph & Code Intelligence
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

  report, audit [path] [--out-dir <dir>] [--json] [--sync]
                [--fail-on-cycles] [--max-god-nodes <n>]
      Performs topological community clustering (Louvain/LPA), identifies
      God Nodes (central hubs), detects cross-community surprises and cycles,
      and produces standalone artifacts in graphfinder-out/:
        • GRAPH_REPORT.md  (High-density architectural audit)
        • graph.json       (Standardized schema consumed by AI agents)
        • graph.html       (Zero-dependency standalone canvas visualizer)
      CI/CD Quality Gate Options:
        --fail-on-cycles      Exits with code 1 if any dependency cycle is found
        --max-god-nodes <n>   Exits with code 1 if God Nodes exceed <n>
        --sync                Synchronizes codebase/vault before running audit

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

  install-cli, install-path
      Registers GraphFinder in your operating system's PATH (and creates
      command shims) so you can run 'graphfinder' from any terminal anywhere.

  config, mcp-config, setup [--install] [--json]
      Generates ready-to-use MCP configuration for ANY AI editor:
      Cursor, Claude Desktop, VS Code (Cline / Roo Code), Windsurf, Zed, and agents.
      Options:
        --install   Auto-detects installed editors on this system, registers GraphFinder,
                    and adds the CLI to the operating system's PATH.
        --json      Outputs the raw JSON configuration snippet for scripting

  tools, schemas [path]
      Exports all 24 MCP tool schemas, the unified tools.json manifest,
      and instructions.md to ./mcp-tools (or an optional custom directory).
      Completely generic and consumable by any AI editor, LLM, or agent framework.

  mcp, --mcp
      Starts the headless stdio MCP server for universal AI agent integration
      (Cursor, Claude Desktop, VS Code, Windsurf, Zed, Antigravity, Cline, etc.).

  help, --help, -h
      Displays this help information.

  (no arguments)
      Launches the desktop graphical interface (GUI).

EXAMPLES:
  graphfinder config
  graphfinder config --install
  graphfinder tools
  graphfinder tools ./custom-mcp-dir
  graphfinder sync
  graphfinder report
  graphfinder path "GraphView.tsx" "DbManager"
  graphfinder explain "DbManager"
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

    let cmd = args[1].to_lowercase();
    let is_mcp = cmd == "--mcp" || cmd == "mcp" || args.iter().any(|a| a == "--mcp" || a == "mcp");

    // Argumentos de terminal presentes: conecta ao console pai (no Windows), exceto em modo MCP stdio
    // para não interferir nas pipes de entrada e saída padrão (JSON-RPC)
    if !is_mcp {
        attach_console();
    }

    // 1. Ajuda
    if cmd == "help" || cmd == "--help" || cmd == "-h" {
        print_help();
        return Ok(());
    }

    // 2. Headless MCP Stdio Mode
    if is_mcp {
        eprintln!("[GraphFinder MCP] Headless stdio mode activated.");
        let embeddings = Arc::new(EmbeddingEngine::new_lazy());
        let manager = Arc::new(VaultManager::new(Arc::clone(&embeddings))?);

        // Inicializa o modelo FastEmbed e sincronização inicial em background
        // para que o handshake MCP (initialize) responda instantaneamente (<50ms)
        let bg_manager = Arc::clone(&manager);
        let bg_embeddings = Arc::clone(&embeddings);
        tokio::spawn(async move {
            let _ = bg_embeddings.init_model(None).await;
            let engine = bg_manager.active_engine().await;
            let _ = engine.sync_vault().await;
        });

        app_lib::mcp::run_mcp_server(manager).await?;
        return Ok(());
    }

    // 2.2. Dedicated CLI System Installer
    if cmd == "install-cli" || cmd == "install-path" {
        let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("GraphFinder.exe"));
        println!("======================================================================");
        println!("  GraphFinder CLI System Installer");
        println!("======================================================================");
        let (ok, msg) = install_to_os_path(&exe_path);
        if ok {
            println!("  ✓ {}\n", msg);
            println!("You can now run 'graphfinder' from any terminal or command prompt!");
        } else {
            println!("  ✗ Failed to install to OS PATH: {}\n", msg);
        }
        println!("======================================================================");
        return Ok(());
    }

    // 2.3. Universal MCP Configuration for ANY AI Editor
    if cmd == "config" || cmd == "mcp-config" || cmd == "setup" {
        let exe_path = std::env::current_exe().unwrap_or_else(|_| PathBuf::from("GraphFinder.exe"));
        let exe_str = exe_path.to_string_lossy().to_string();

        let is_json = args.iter().any(|a| a == "--json");
        let is_install = args.iter().any(|a| a == "--install" || a == "-i");

        if is_json {
            let json_config = serde_json::json!({
                "mcpServers": {
                    "graphfinder": {
                        "command": exe_str,
                        "args": ["--mcp"]
                    }
                }
            });
            println!("{}", serde_json::to_string_pretty(&json_config).unwrap());
            return Ok(());
        }

        if is_install {
            println!("======================================================================");
            println!("  GraphFinder Universal MCP Auto-Installer");
            println!("======================================================================");
            println!("Target Executable: {}\n", exe_str);

            let results = auto_install_mcp_config(&exe_path);
            if results.is_empty() {
                println!("No standard AI editor configuration directories were detected on this system.");
                println!("You can manually copy the configuration below into your editor:\n");
            } else {
                println!("Detected and configured {} target(s):", results.len());
                for (name, path, ok, msg) in &results {
                    if *ok {
                        println!("  ✓ {:<22} -> {}", name, path.display());
                    } else {
                        println!("  ✗ {:<22} -> {} ({})", name, path.display(), msg);
                    }
                }
                println!("\nConfiguration complete! Restart your editor(s) to activate GraphFinder.");
                return Ok(());
            }
        }

        // Generic guide for any editor:
        println!("======================================================================");
        println!("  GraphFinder Universal MCP Configuration (For Any AI Editor)");
        println!("======================================================================");
        println!("\nExecutable Path:");
        println!("  {}\n", exe_str);

        println!("1. Universal Configuration (Cursor, Claude Desktop, Windsurf, VS Code Cline/Roo)");
        println!("   Paste this block into your editor's MCP config file:");
        let escaped_exe = exe_str.replace('\\', "\\\\");
        println!(r#"{{
  "mcpServers": {{
    "graphfinder": {{
      "command": "{}",
      "args": ["--mcp"]
    }}
  }}
}}"#, escaped_exe);

        println!("\n2. Zed Editor (settings.json)");
        println!(r#"{{
  "context_servers": {{
    "graphfinder": {{
      "command": {{
        "path": "{}",
        "args": ["--mcp"]
      }}
    }}
  }}
}}"#, escaped_exe);

        println!("\n3. Streamable HTTP Mode (for Remote Agents or Web Dashboards)");
        println!("   Endpoint: http://127.0.0.1:45123/mcp");

        println!("\nTip: Run 'graphfinder config --install' to automatically register GraphFinder");
        println!("     into all installed AI editors on this machine.");
        println!("======================================================================");
        return Ok(());
    }

    // 2.5. Universal MCP Schemas & Tools Export for ANY AI Editor/Agent
    if cmd == "export-schemas" || cmd == "tools" || cmd == "mcp-tools" || cmd == "schemas" {
        let mut target_dirs: Vec<PathBuf> = Vec::new();

        // 1. Permite passar um diretório arbitrário como argumento: graphfinder tools <caminho>
        let custom_dir = args
            .iter()
            .skip(2)
            .find(|a| !a.starts_with("--"))
            .map(|s| PathBuf::from(s));

        let primary_dir = if let Some(custom) = custom_dir {
            custom
        } else {
            // Diretório padrão genérico e universal: ./mcp-tools
            PathBuf::from("./mcp-tools")
        };
        target_dirs.push(primary_dir.clone());

        // 2. Sincroniza opcionalmente com outras pastas conhecidas se existirem (ex: .gemini/...)
        if let Some(home) = dirs::home_dir() {
            let gemini_ide = home.join(".gemini").join("antigravity-ide").join("mcp").join("graphfinder");
            if gemini_ide.exists() || home.join(".gemini").join("antigravity-ide").exists() {
                target_dirs.push(gemini_ide);
            }
        }

        let dir_refs: Vec<&Path> = target_dirs.iter().map(|p| p.as_path()).collect();
        let names = app_lib::mcp::export_all_schemas(&dir_refs);
        println!("======================================================================");
        println!("  GraphFinder Universal MCP Tools & Schemas (All 24 Tools)");
        println!("======================================================================");
        println!("Exported {} universal MCP tool schemas:", names.len());
        for (i, name) in names.iter().enumerate() {
            println!("  {:2}. {}", i + 1, name);
        }
        println!("\nGenerated Universal Artifacts in '{}':", primary_dir.display());
        println!("  • 24 Individual tool schemas: {}/<tool_name>.json", primary_dir.display());
        println!("  • Unified tools manifest:     {}/tools.json (OpenAI/LangChain format)", primary_dir.display());
        println!("  • Agent instructions:         {}/instructions.md", primary_dir.display());
        if target_dirs.len() > 1 {
            println!("\nAlso synchronized to detected editor directories:");
            for d in &target_dirs[1..] {
                println!("  • {}", d.display());
            }
        }
        println!("\nCompatible with: Cursor, Claude Desktop, VS Code, Windsurf, Zed, Antigravity, and all AI Agents.");
        println!("======================================================================");
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
        let sync_result = engine
            .sync_vault_with_progress(move |progress| {
                if !is_json {
                    print_sync_progress(&progress);
                }
            })
            .await;
        let elapsed = start_time.elapsed();
        if !is_json {
            println!();
        }

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
        let fail_on_cycles = args.iter().any(|a| a == "--fail-on-cycles");
        let force_sync = args.iter().any(|a| a == "--sync");

        let mut max_god_nodes: Option<usize> = None;
        let mut custom_out_dir: Option<PathBuf> = None;
        let mut target_arg: Option<String> = None;

        let mut i = 2;
        while i < args.len() {
            let arg = &args[i];
            if arg == "--out-dir" {
                if let Some(val) = args.get(i + 1) {
                    custom_out_dir = Some(PathBuf::from(val));
                    i += 2;
                    continue;
                }
            } else if let Some(val) = arg.strip_prefix("--out-dir=") {
                custom_out_dir = Some(PathBuf::from(val));
            } else if arg == "--max-god-nodes" {
                if let Some(val) = args.get(i + 1) {
                    max_god_nodes = val.parse::<usize>().ok();
                    i += 2;
                    continue;
                }
            } else if let Some(val) = arg.strip_prefix("--max-god-nodes=") {
                max_god_nodes = val.parse::<usize>().ok();
            } else if !arg.starts_with("--") && target_arg.is_none() {
                target_arg = Some(arg.clone());
            }
            i += 1;
        }

        let target_str = target_arg.as_deref().unwrap_or(".");
        let target_dir = Path::new(target_str);
        let abs_path = if target_dir.is_absolute() {
            target_dir.to_path_buf()
        } else {
            std::env::current_dir()?.join(target_dir)
        };
        let canonical = std::fs::canonicalize(&abs_path).unwrap_or(abs_path);

        let embeddings = Arc::new(EmbeddingEngine::new_lazy());
        let manager = Arc::new(VaultManager::new(embeddings)?);
        let (project_id, engine, is_new) = manager.get_or_register_project_by_path(&canonical).await?;

        if force_sync || is_new {
            let _ = engine.sync_vault().await;
        }

        let mut report = engine.analyze_graph().await?;
        if report.total_nodes == 0 && !force_sync {
            let _ = engine.sync_vault().await;
            report = engine.analyze_graph().await?;
        }

        let export_summary = engine.export_artifacts(custom_out_dir.as_deref()).await?;

        let mut violations: Vec<String> = Vec::new();
        if fail_on_cycles && !report.cycles.is_empty() {
            violations.push(format!(
                "Dependency cycles detected: {} cycle(s) found (--fail-on-cycles)",
                report.cycles.len()
            ));
        }
        if let Some(max_gods) = max_god_nodes {
            if report.god_nodes.len() > max_gods {
                violations.push(format!(
                    "God Nodes count ({}) exceeds threshold of {} (--max-god-nodes)",
                    report.god_nodes.len(),
                    max_gods
                ));
            }
        }
        let ci_passed = violations.is_empty();

        if is_json {
            let out = serde_json::json!({
                "project_id": project_id,
                "ci_passed": ci_passed,
                "violations": violations,
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

            if fail_on_cycles || max_god_nodes.is_some() {
                println!("\nCI/CD Architectural Quality Gate:");
                if ci_passed {
                    println!("  ✓ [PASS] All architectural quality checks passed.");
                } else {
                    for v in &violations {
                        println!("  ✗ [FAIL] {}", v);
                    }
                }
            }
            println!("────────────────────────────────────────────────────────────\n");
        }

        if !ci_passed {
            std::process::exit(1);
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
