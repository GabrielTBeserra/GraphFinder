// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();

    // Modo stdio headless para integração direta com clientes MCP (Cursor, Claude Desktop, Antigravity, etc.)
    if args.iter().any(|a| a == "--mcp" || a == "mcp") {
        eprintln!("[GraphFinder MCP] Modo stdio headless ativado.");
        let embeddings = std::sync::Arc::new(app_lib::engine::EmbeddingEngine::new_lazy());
        let _ = embeddings.init_model(None).await;
        let manager = std::sync::Arc::new(app_lib::engine::VaultManager::new(embeddings)?);
        let engine = manager.active_engine().await;
        let _ = engine.sync_vault().await;

        app_lib::mcp::run_mcp_server(manager).await?;
        return Ok(());
    }

    // Modo padrão: Janela desktop nativa do Tauri com servidor MCP HTTP embutido
    app_lib::run();
    Ok(())
}
