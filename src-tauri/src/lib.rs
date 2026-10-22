use std::sync::Arc;
use tauri::{Emitter, Manager, State};

pub mod engine;
pub mod mcp;

use engine::{
    ChunkHit, CodeChunkHit, CodeDependencies, CodeFileRecord, CompressedContext, EmbeddingEngine,
    ExportSummary, FileContext, GraphAnalysisReport, GraphData, LinkSuggestion, Neighborhood,
    NodeExplanation, NoteRecord, NoteSummary, PathResult, ProjectInfo, SearchHit, VaultManager,
    VaultStats,
};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use std::time::Duration;

#[tauri::command]
async fn list_projects(manager: State<'_, Arc<VaultManager>>) -> Result<Vec<ProjectInfo>, String> {
    Ok(manager.list_projects().await)
}

#[tauri::command]
#[allow(non_snake_case)]
async fn switch_project(
    project_id: Option<String>,
    projectId: Option<String>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<ProjectInfo, String> {
    let id = project_id.or(projectId).unwrap_or_else(|| "default".to_string());
    manager.switch_project(&id).await
}

#[tauri::command]
async fn create_project(
    name: String,
    path: Option<String>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<ProjectInfo, String> {
    manager.create_project(&name, path).await
}

#[tauri::command]
async fn pick_directory() -> Result<Option<String>, String> {
    let folder = rfd::AsyncFileDialog::new()
        .set_title("Select Folder for Knowledge Vault")
        .pick_folder()
        .await;
    Ok(folder.map(|f| f.path().to_string_lossy().to_string()))
}

#[tauri::command]
async fn get_graph(manager: State<'_, Arc<VaultManager>>) -> Result<GraphData, String> {
    manager
        .active_engine()
        .await
        .get_graph()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_notes(manager: State<'_, Arc<VaultManager>>) -> Result<Vec<NoteSummary>, String> {
    manager
        .active_engine()
        .await
        .list_notes()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_note(
    id: String,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Option<NoteRecord>, String> {
    manager
        .active_engine()
        .await
        .get_note(&id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_note(
    title: String,
    content: String,
    filename: Option<String>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<NoteRecord, String> {
    manager
        .active_engine()
        .await
        .save_note(&title, &content, filename.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn rename_note(
    old_title: String,
    new_title: String,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<NoteRecord, String> {
    manager
        .active_engine()
        .await
        .rename_note(&old_title, &new_title)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn delete_note(
    target: String,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<(), String> {
    manager
        .active_engine()
        .await
        .delete_note(&target)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_vault_stats(
    manager: State<'_, Arc<VaultManager>>,
) -> Result<VaultStats, String> {
    manager
        .active_engine()
        .await
        .get_vault_stats()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_section(
    target: String,
    section: String,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Option<ChunkHit>, String> {
    manager
        .active_engine()
        .await
        .read_section(&target, &section)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_semantic(
    query: String,
    limit: Option<usize>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Vec<SearchHit>, String> {
    let limit = limit.unwrap_or(10);
    manager
        .active_engine()
        .await
        .search_semantic(&query, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn search_chunks(
    query: String,
    limit: Option<usize>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Vec<ChunkHit>, String> {
    let limit = limit.unwrap_or(8);
    manager
        .active_engine()
        .await
        .search_chunks(&query, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn suggest_links(
    text: String,
    current_note: Option<String>,
    threshold: Option<f32>,
    limit: Option<usize>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Vec<LinkSuggestion>, String> {
    let current_id = current_note.unwrap_or_default();
    let threshold = threshold.unwrap_or(0.60);
    let limit = limit.unwrap_or(5);
    manager
        .active_engine()
        .await
        .suggest_links(&text, &current_id, threshold, limit)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_compressed_context(
    target: String,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Option<CompressedContext>, String> {
    manager
        .active_engine()
        .await
        .get_compressed_context(&target)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_neighborhood(
    target: String,
    depth: Option<usize>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Neighborhood, String> {
    let depth = depth.unwrap_or(1);
    manager
        .active_engine()
        .await
        .get_neighborhood(&target, depth)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn link_notes(
    from: String,
    to: String,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<NoteRecord, String> {
    manager
        .active_engine()
        .await
        .link_notes(&from, &to)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn sync_vault(manager: State<'_, Arc<VaultManager>>) -> Result<usize, String> {
    manager
        .active_engine()
        .await
        .sync_vault()
        .await
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
pub struct McpConfigInfo {
    pub executable_path: String,
    pub vault_path: String,
    pub db_path: String,
    pub config_json: String,
}

#[tauri::command]
async fn get_mcp_config(manager: State<'_, Arc<VaultManager>>) -> Result<McpConfigInfo, String> {
    let engine = manager.active_engine().await;
    let exe_path = std::env::current_exe()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| "GraphFinder.exe".to_string());

    let abs_vault = std::fs::canonicalize(engine.vault_path())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| engine.vault_path().to_string_lossy().to_string());
    let clean_vault = abs_vault.trim_start_matches(r"\\?\").to_string();

    let abs_db = std::fs::canonicalize(engine.db_path())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| engine.db_path().to_string_lossy().to_string());
    let clean_db = abs_db.trim_start_matches(r"\\?\").to_string();
    let clean_exe = exe_path.trim_start_matches(r"\\?\").to_string();

    let port = std::env::var("GRAPHFINDER_MCP_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(45123);

    let config_json = format!(
        r#"{{
  "mcpServers": {{
    "graphfinder": {{
      "type": "http",
      "url": "http://127.0.0.1:{}/mcp"
    }}
  }}
}}"#,
        port
    );

    Ok(McpConfigInfo {
        executable_path: clean_exe,
        vault_path: clean_vault,
        db_path: clean_db,
        config_json,
    })
}

#[tauri::command]
async fn get_engine_status(manager: State<'_, Arc<VaultManager>>) -> Result<serde_json::Value, String> {
    Ok(serde_json::json!({
        "status": manager.embeddings().get_status(),
        "is_ready": manager.embeddings().is_ready()
    }))
}

#[tauri::command]
async fn search_code(
    query: String,
    limit: Option<usize>,
    language: Option<String>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Vec<CodeChunkHit>, String> {
    manager
        .active_engine()
        .await
        .search_code(&query, limit.unwrap_or(5), language.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn find_files(
    pattern: Option<String>,
    language: Option<String>,
    limit: Option<usize>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Vec<CodeFileRecord>, String> {
    manager
        .active_engine()
        .await
        .find_files(pattern.as_deref(), language.as_deref(), limit.unwrap_or(20))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_file_context(
    file: Option<String>,
    file_path: Option<String>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Option<FileContext>, String> {
    let target = file.or(file_path).unwrap_or_default();
    manager
        .active_engine()
        .await
        .get_file_context(&target)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn read_code_chunk(
    file: Option<String>,
    file_path: Option<String>,
    symbol: Option<String>,
    symbol_or_lines: Option<String>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Option<CodeChunkHit>, String> {
    let target = file.or(file_path).unwrap_or_default();
    let sym = symbol.or(symbol_or_lines);
    manager
        .active_engine()
        .await
        .read_code_chunk(&target, sym.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_code_dependencies(
    file: Option<String>,
    file_path: Option<String>,
    direction: Option<String>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<CodeDependencies, String> {
    let target = file.or(file_path).unwrap_or_default();
    manager
        .active_engine()
        .await
        .get_code_dependencies(&target, direction.as_deref().unwrap_or("both"))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_code_files(
    limit: Option<usize>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Vec<CodeFileRecord>, String> {
    manager
        .active_engine()
        .await
        .list_code_files(limit.unwrap_or(50))
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn analyze_graph(manager: State<'_, Arc<VaultManager>>) -> Result<GraphAnalysisReport, String> {
    manager
        .active_engine()
        .await
        .analyze_graph()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn find_path(
    source: String,
    target: String,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Option<PathResult>, String> {
    manager
        .active_engine()
        .await
        .find_path(&source, &target)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn explain_node(
    target: String,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Option<NodeExplanation>, String> {
    manager
        .active_engine()
        .await
        .explain_node(&target)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn export_artifacts(
    out_dir: Option<String>,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<ExportSummary, String> {
    let path = out_dir.as_ref().map(std::path::Path::new);
    manager
        .active_engine()
        .await
        .export_artifacts(path)
        .await
        .map_err(|e| e.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            println!("[GraphFinder Tauri] Loading semantic engine with asynchronous initialization...");
            let embeddings = Arc::new(EmbeddingEngine::new_lazy());
            let manager = Arc::new(VaultManager::new(embeddings.clone()).expect("Failed to initialize VaultManager"));

            // Starts Native Localhost MCP HTTP Server
            let mcp_port = std::env::var("GRAPHFINDER_MCP_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(45123);

            let manager_for_mcp = Arc::clone(&manager);
            let app_handle_for_mcp = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = mcp::server::start_http_mcp_server(manager_for_mcp, Some(app_handle_for_mcp), mcp_port).await {
                    eprintln!("[GraphFinder MCP Error] Failed to run HTTP server on port {}: {}", mcp_port, e);
                }
            });

            // Initializes neural model in background asynchronously with progress notifications
            let embeddings_for_init = Arc::clone(&embeddings);
            let manager_for_sync = Arc::clone(&manager);
            let app_handle_for_init = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = embeddings_for_init.init_model(Some(app_handle_for_init.clone())).await {
                    eprintln!("[GraphFinder Error] Failed to load neural model: {}", e);
                    let _ = app_handle_for_init.emit("engine-status", serde_json::json!({
                        "status": "error",
                        "message": format!("Error loading neural model: {}", e)
                    }));
                } else {
                    let engine = manager_for_sync.active_engine().await;
                    if let Ok(count) = engine.sync_vault().await {
                        println!("[GraphFinder Tauri] {} notes synchronized in active vault.", count);
                        let _ = app_handle_for_init.emit("vault-sync-complete", count);
                    }
                }
            });

            // File Watcher with Debounce (500ms) for automatic synchronization when files are edited externally (Obsidian, VS Code, etc)
            let app_handle_for_watch = app.handle().clone();
            let manager_for_watch = Arc::clone(&manager);
            tauri::async_runtime::spawn_blocking(move || {
                let (tx, rx) = std::sync::mpsc::channel();
                let mut debouncer = match new_debouncer(Duration::from_millis(500), tx) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("[GraphFinder Watcher Error] Failed to create debouncer: {}", e);
                        return;
                    }
                };

                let base_dir = crate::engine::project::resolve_base_dir();
                let vaults_dir = base_dir.join("vaults");
                let default_vault = base_dir.join("vault");

                if default_vault.exists() {
                    let _ = debouncer.watcher().watch(&default_vault, RecursiveMode::Recursive);
                }
                if vaults_dir.exists() {
                    let _ = debouncer.watcher().watch(&vaults_dir, RecursiveMode::Recursive);
                }

                while let Ok(res) = rx.recv() {
                    match res {
                        Ok(events) => {
                            let has_md = events.iter().any(|e| {
                                e.path.extension().map(|ext| ext == "md").unwrap_or(false)
                            });
                            if has_md {
                                let manager = Arc::clone(&manager_for_watch);
                                let handle = app_handle_for_watch.clone();
                                tauri::async_runtime::spawn(async move {
                                    let engine = manager.active_engine().await;
                                    if let Ok(count) = engine.sync_vault().await {
                                        if count > 0 {
                                            let _ = handle.emit("vault-sync-complete", count);
                                        }
                                    }
                                });
                            }
                        }
                        Err(err) => {
                            eprintln!("[GraphFinder Watcher] Error: {:?}", err);
                        }
                    }
                }
            });

            if let Some(window) = app.get_webview_window("main") {
                if let Some(icon) = app.default_window_icon() {
                    let _ = window.set_icon(icon.clone());
                }
            }

            app.manage(manager);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_graph,
            list_notes,
            get_note,
            save_note,
            rename_note,
            delete_note,
            get_vault_stats,
            read_section,
            search_semantic,
            search_chunks,
            suggest_links,
            get_compressed_context,
            get_neighborhood,
            link_notes,
            sync_vault,
            get_mcp_config,
            list_projects,
            switch_project,
            create_project,
            pick_directory,
            get_engine_status,
            search_code,
            find_files,
            get_file_context,
            read_code_chunk,
            get_code_dependencies,
            list_code_files,
            analyze_graph,
            find_path,
            explain_node,
            export_artifacts
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
