use axum::{
    extract::State,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager,
    StreamableHttpServerConfig, StreamableHttpService,
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tauri::AppHandle;
use tower_http::cors::CorsLayer;

use super::GraphFinderMcpServer;
use crate::engine::VaultManager;

#[derive(Clone)]
pub struct ServerState {
    pub manager: Arc<VaultManager>,
    pub app_handle: Option<AppHandle>,
    pub port: u16,
}

async fn health_handler(State(state): State<ServerState>) -> impl IntoResponse {
    let active_id = state.manager.active_project_id().await;
    let engine = state.manager.active_engine().await;
    let note_count = engine.list_notes().await.map(|n| n.len()).unwrap_or(0);

    Json(json!({
        "status": "online",
        "service": "GraphFinder MCP Server (Official rmcp SDK)",
        "version": env!("CARGO_PKG_VERSION"),
        "transport": ["stdio", "streamable-http"],
        "endpoints": {
            "mcp": format!("http://127.0.0.1:{}/mcp", state.port),
            "sse": format!("http://127.0.0.1:{}/sse", state.port),
            "health": format!("http://127.0.0.1:{}/health", state.port)
        },
        "vault": {
            "active_project": active_id,
            "indexed_notes": note_count
        }
    }))
}

pub async fn start_http_mcp_server(
    manager: Arc<VaultManager>,
    app_handle: Option<AppHandle>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = ServerState {
        manager: Arc::clone(&manager),
        app_handle: app_handle.clone(),
        port,
    };

    let server = GraphFinderMcpServer::new(Arc::clone(&manager), app_handle);

    let mcp_service = StreamableHttpService::new(
        {
            let server = server.clone();
            move || Ok(server.clone())
        },
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );

    let sse_service = StreamableHttpService::new(
        {
            let server = server.clone();
            move || Ok(server.clone())
        },
        Arc::new(LocalSessionManager::default()),
        StreamableHttpServerConfig::default(),
    );

    let app = Router::new()
        .route("/", get(health_handler))
        .route("/health", get(health_handler))
        .nest_service("/mcp", mcp_service)
        .nest_service("/sse", sse_service)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("[GraphFinder MCP] Servidor HTTP Oficial (rmcp) iniciando em http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("[GraphFinder MCP] Servidor HTTP Oficial (rmcp) ativo e escutando em http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
