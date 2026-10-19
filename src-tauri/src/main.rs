use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = std::env::args().collect();

    // Se iniciado com flag --mcp, entra em modo servidor MCP sobre stdio
    if args.iter().any(|a| a == "--mcp" || a == "mcp") {
        let embeddings = Arc::new(app_lib::engine::EmbeddingEngine::new()?);
        let manager = Arc::new(app_lib::engine::VaultManager::new(embeddings)?);

        let engine = manager.active_engine().await;
        let _ = engine.sync_vault().await;
        app_lib::mcp::run_mcp_server(manager).await?;
        return Ok(());
    }

    // Modo padrão: Janela desktop nativa do Tauri
    app_lib::run();
    Ok(())
}
