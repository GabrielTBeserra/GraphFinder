use std::sync::Arc;
use tauri::{Emitter, Manager, State};

pub mod engine;
pub mod mcp;

use engine::{
    ChunkHit, CompressedContext, EmbeddingEngine, GraphData, LinkSuggestion, Neighborhood,
    NoteRecord, NoteSummary, ProjectInfo, SearchHit, VaultManager, VaultStats,
};
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode};
use std::time::Duration;

#[tauri::command]
async fn list_projects(manager: State<'_, Arc<VaultManager>>) -> Result<Vec<ProjectInfo>, String> {
    Ok(manager.list_projects().await)
}

#[tauri::command]
async fn switch_project(
    project_id: String,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<ProjectInfo, String> {
    manager.switch_project(&project_id).await
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
        .set_title("Selecionar Pasta para o Cofre de Notas")
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

            println!("[GraphFinder Tauri] Carregando motor semântico com inicialização assíncrona...");
            let embeddings = Arc::new(EmbeddingEngine::new_lazy());
            let manager = Arc::new(VaultManager::new(embeddings.clone()).expect("Falha ao inicializar VaultManager"));

            // Inicia o Servidor MCP HTTP Nativo (Localhost)
            let mcp_port = std::env::var("GRAPHFINDER_MCP_PORT")
                .ok()
                .and_then(|p| p.parse::<u16>().ok())
                .unwrap_or(45123);

            let manager_for_mcp = Arc::clone(&manager);
            let app_handle_for_mcp = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = mcp::server::start_http_mcp_server(manager_for_mcp, Some(app_handle_for_mcp), mcp_port).await {
                    eprintln!("[GraphFinder MCP Error] Falha ao rodar servidor HTTP na porta {}: {}", mcp_port, e);
                }
            });

            // Inicializa modelo neural em background de forma assíncrona com notificações de progresso
            let embeddings_for_init = Arc::clone(&embeddings);
            let manager_for_sync = Arc::clone(&manager);
            let app_handle_for_init = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = embeddings_for_init.init_model(Some(app_handle_for_init.clone())).await {
                    eprintln!("[GraphFinder Error] Falha ao carregar modelo neural: {}", e);
                    let _ = app_handle_for_init.emit("engine-status", serde_json::json!({
                        "status": "error",
                        "message": format!("Erro ao carregar modelo neural: {}", e)
                    }));
                } else {
                    let engine = manager_for_sync.active_engine().await;
                    if let Ok(count) = engine.sync_vault().await {
                        println!("[GraphFinder Tauri] {} notas sincronizadas no cofre ativo.", count);
                        let _ = app_handle_for_init.emit("vault-sync-complete", count);
                    }
                }
            });

            // File Watcher com Debounce (500ms) para sincronização automática quando arquivos são alterados externamente (Obsidian, VS Code, etc)
            let app_handle_for_watch = app.handle().clone();
            let manager_for_watch = Arc::clone(&manager);
            tauri::async_runtime::spawn_blocking(move || {
                let (tx, rx) = std::sync::mpsc::channel();
                let mut debouncer = match new_debouncer(Duration::from_millis(500), tx) {
                    Ok(d) => d,
                    Err(e) => {
                        eprintln!("[GraphFinder Watcher Error] Falha ao criar debouncer: {}", e);
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
                            eprintln!("[GraphFinder Watcher] Erro: {:?}", err);
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
            get_engine_status
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
