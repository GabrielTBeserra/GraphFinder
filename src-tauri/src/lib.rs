use std::sync::Arc;
use tauri::{Manager, State};

pub mod engine;
pub mod mcp;

use engine::{
    ChunkHit, CompressedContext, EmbeddingEngine, GraphData, LinkSuggestion, Neighborhood,
    NoteRecord, NoteSummary, ProjectInfo, SearchHit, VaultManager,
};

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
    manager.active_engine().await.get_graph().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn list_notes(manager: State<'_, Arc<VaultManager>>) -> Result<Vec<NoteSummary>, String> {
    manager.active_engine().await.list_notes().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_note(
    id: String,
    manager: State<'_, Arc<VaultManager>>,
) -> Result<Option<NoteRecord>, String> {
    manager.active_engine().await.get_note(&id).await.map_err(|e| e.to_string())
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
    manager.active_engine().await.sync_vault().await.map_err(|e| e.to_string())
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
        .unwrap_or_else(|_| "graphfinder.exe".to_string());

    let abs_vault = std::fs::canonicalize(engine.vault_path())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| engine.vault_path().to_string_lossy().to_string());
    let clean_vault = abs_vault.trim_start_matches(r"\\?\").to_string();

    let abs_db = std::fs::canonicalize(engine.db_path())
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| engine.db_path().to_string_lossy().to_string());
    let clean_db = abs_db.trim_start_matches(r"\\?\").to_string();
    let clean_exe = exe_path.trim_start_matches(r"\\?\").to_string();

    let escaped_exe = clean_exe.replace('\\', "\\\\");
    let escaped_vault = clean_vault.replace('\\', "\\\\");
    let escaped_db = clean_db.replace('\\', "\\\\");

    let config_json = format!(
r#"{{
  "mcpServers": {{
    "graphfinder": {{
      "command": "{}",
      "args": ["--mcp"],
      "env": {{
        "GRAPHFINDER_VAULT": "{}",
        "GRAPHFINDER_DB": "{}"
      }}
    }}
  }}
}}"#,
        escaped_exe, escaped_vault, escaped_db
    );

    Ok(McpConfigInfo {
        executable_path: clean_exe,
        vault_path: clean_vault,
        db_path: clean_db,
        config_json,
    })
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

            println!("[GraphFinder Tauri] Carregando motor semântico com suporte a múltiplos projetos/vaults...");
            let embeddings = Arc::new(EmbeddingEngine::new().expect("Falha ao inicializar embeddings locais"));
            let manager = Arc::new(VaultManager::new(embeddings).expect("Falha ao inicializar VaultManager"));

            let manager_for_sync = Arc::clone(&manager);
            tauri::async_runtime::spawn(async move {
                let engine = manager_for_sync.active_engine().await;
                if let Ok(count) = engine.sync_vault().await {
                    println!("[GraphFinder Tauri] {} notas sincronizadas no cofre ativo.", count);
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
            pick_directory
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
