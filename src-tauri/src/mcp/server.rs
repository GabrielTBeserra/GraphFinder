use axum::{
    extract::{Request, State},
    http::{header, HeaderMap, Method, StatusCode},
    middleware::{self, Next},
    response::{Html, IntoResponse, Json, Response},
    routing::get,
    Router,
};
use rmcp::transport::streamable_http_server::{
    session::local::LocalSessionManager,
    StreamableHttpServerConfig, StreamableHttpService,
};
use serde_json::json;
use std::{net::SocketAddr, sync::Arc};
use tauri::AppHandle;

use super::GraphFinderMcpServer;
use crate::engine::VaultManager;

#[derive(Clone)]
pub struct ServerState {
    pub manager: Arc<VaultManager>,
    pub app_handle: Option<AppHandle>,
    pub port: u16,
}

async fn security_middleware(req: Request, next: Next) -> Result<Response, StatusCode> {
    // 1. Host validation against DNS rebinding
    if let Some(host) = req.headers().get(header::HOST).and_then(|h| h.to_str().ok()) {
        let clean_host = host.split(':').next().unwrap_or(host).trim();
        if clean_host != "localhost" && clean_host != "127.0.0.1" && clean_host != "[::1]" {
            eprintln!("[GraphFinder Security] Rejeitando requisição com Host não-local: {}", host);
            return Err(StatusCode::FORBIDDEN);
        }
    }

    // 2. Origin validation to block malicious third-party websites in the browser
    let origin_header = req.headers().get(header::ORIGIN).and_then(|o| o.to_str().ok());
    let mut allowed_origin = None;
    if let Some(origin) = origin_header {
        let is_allowed = origin.starts_with("http://localhost")
            || origin.starts_with("http://127.0.0.1")
            || origin.starts_with("https://localhost")
            || origin.starts_with("https://127.0.0.1")
            || origin.starts_with("tauri://")
            || origin.starts_with("vscode-webview://");
        if !is_allowed {
            eprintln!("[GraphFinder Security] Rejeitando requisição cross-origin não autorizada de Origin: {}", origin);
            return Err(StatusCode::FORBIDDEN);
        }
        allowed_origin = Some(origin.to_string());
    }

    // Handle OPTIONS preflight for allowed local origins
    if req.method() == Method::OPTIONS {
        let mut res = StatusCode::NO_CONTENT.into_response();
        let headers = res.headers_mut();
        if let Some(ref origin) = allowed_origin {
            if let Ok(val) = origin.parse() {
                headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, val);
            }
        }
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_METHODS,
            "GET, POST, OPTIONS".parse().unwrap(),
        );
        headers.insert(
            header::ACCESS_CONTROL_ALLOW_HEADERS,
            "Content-Type, Authorization, Accept".parse().unwrap(),
        );
        return Ok(res);
    }

    let mut response = next.run(req).await;
    let headers = response.headers_mut();
    headers.insert(header::X_CONTENT_TYPE_OPTIONS, "nosniff".parse().unwrap());
    headers.insert(header::X_FRAME_OPTIONS, "DENY".parse().unwrap());
    if let Some(ref origin) = allowed_origin {
        if let Ok(val) = origin.parse() {
            headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, val);
        }
    }
    Ok(response)
}

async fn info_handler(
    headers: HeaderMap,
    State(state): State<ServerState>,
) -> Response {
    let active_id = state.manager.active_project_id().await;
    let engine = state.manager.active_engine().await;
    let note_count = engine.list_notes().await.map(|n| n.len()).unwrap_or(0);
    let vault_path = engine.vault_path().to_string_lossy().to_string();

    let accept = headers
        .get(header::ACCEPT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if accept.contains("text/html") {
        let html_content = format!(
            r#"<!DOCTYPE html>
<html lang="pt-BR">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>GraphFinder MCP Server</title>
  <style>
    :root {{
      --bg: #090a0f;
      --card: #12151e;
      --border: #232838;
      --text: #f1f5f9;
      --muted: #8b9bb4;
      --accent: #6366f1;
      --emerald: #10b981;
    }}
    body {{
      margin: 0;
      padding: 2rem;
      font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Helvetica, Arial, sans-serif;
      background: var(--bg);
      color: var(--text);
      display: flex;
      justify-content: center;
      align-items: center;
      min-height: 100vh;
      box-sizing: border-box;
    }}
    .container {{
      max-width: 680px;
      width: 100%;
      background: var(--card);
      border: 1px solid var(--border);
      border-radius: 16px;
      padding: 2.2rem;
      box-shadow: 0 25px 50px -12px rgba(0,0,0,0.5);
    }}
    .header {{
      display: flex;
      align-items: center;
      justify-content: space-between;
      border-bottom: 1px solid var(--border);
      padding-bottom: 1.2rem;
      margin-bottom: 1.5rem;
    }}
    h1 {{
      margin: 0;
      font-size: 1.5rem;
      display: flex;
      align-items: center;
      gap: 0.6rem;
    }}
    .badge {{
      display: inline-flex;
      align-items: center;
      gap: 0.4rem;
      padding: 0.3rem 0.75rem;
      border-radius: 9999px;
      font-size: 0.8rem;
      font-weight: 600;
      background: rgba(16, 185, 129, 0.15);
      color: var(--emerald);
      border: 1px solid rgba(16, 185, 129, 0.3);
    }}
    .dot {{
      width: 8px;
      height: 8px;
      border-radius: 50%;
      background: var(--emerald);
      box-shadow: 0 0 10px var(--emerald);
    }}
    .card {{
      background: rgba(255,255,255,0.02);
      border: 1px solid var(--border);
      border-radius: 10px;
      padding: 1rem 1.2rem;
      margin-bottom: 1.2rem;
    }}
    .card h3 {{
      margin: 0 0 0.5rem 0;
      font-size: 0.95rem;
      color: var(--muted);
      text-transform: uppercase;
      letter-spacing: 0.05em;
    }}
    .stat-row {{
      display: flex;
      justify-content: space-between;
      padding: 0.35rem 0;
      border-bottom: 1px dashed rgba(255,255,255,0.06);
    }}
    .stat-row:last-child {{ border-bottom: none; }}
    .stat-label {{ color: var(--muted); font-size: 0.9rem; }}
    .stat-val {{ font-weight: 500; font-family: monospace; font-size: 0.9rem; }}
    .notice {{
      background: rgba(99, 102, 241, 0.08);
      border-left: 3px solid var(--accent);
      padding: 0.9rem 1.2rem;
      border-radius: 0 8px 8px 0;
      margin-bottom: 1.5rem;
      font-size: 0.88rem;
      line-height: 1.45;
      color: #cbd5e1;
    }}
    pre {{
      background: #0d0f17;
      border: 1px solid var(--border);
      border-radius: 8px;
      padding: 1rem;
      overflow-x: auto;
      font-size: 0.82rem;
      color: #a5b4fc;
      margin: 0;
    }}
    .endpoint-tag {{
      display: inline-block;
      font-size: 0.72rem;
      font-weight: 700;
      padding: 0.15rem 0.4rem;
      border-radius: 4px;
      margin-right: 0.4rem;
    }}
    .tag-post {{ background: #2563eb; color: #fff; }}
    .tag-get {{ background: #059669; color: #fff; }}
  </style>
</head>
<body>
  <div class="container">
    <div class="header">
      <h1>🕸️ GraphFinder MCP Server</h1>
      <div class="badge"><span class="dot"></span> Online</div>
    </div>

    <div class="notice">
      💡 <strong>Sobre este link:</strong> O endpoint <code>/mcp</code> é uma rota de protocolo <strong>JSON-RPC 2.0 (Streamable HTTP)</strong> utilizada por clientes e agentes de IA como Cursor, Antigravity e Claude. Quando acessado via navegador, este painel exibe o status de operação e parâmetros do cofre neural.
    </div>

    <div class="card">
      <h3>Estado do Cofre Local</h3>
      <div class="stat-row">
        <span class="stat-label">Projeto Ativo:</span>
        <span class="stat-val">{active_id}</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Notas Indexadas:</span>
        <span class="stat-val">{note_count}</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">Caminho do Vault:</span>
        <span class="stat-val">{vault_path}</span>
      </div>
      <div class="stat-row">
        <span class="stat-label">SDK:</span>
        <span class="stat-val">Official rmcp (Rust Model Context Protocol)</span>
      </div>
    </div>

    <div class="card">
      <h3>Endpoints Disponíveis</h3>
      <div class="stat-row">
        <span class="stat-label"><span class="endpoint-tag tag-post">POST</span> /mcp</span>
        <span class="stat-val">Streamable HTTP (JSON-RPC 2.0)</span>
      </div>
      <div class="stat-row">
        <span class="stat-label"><span class="endpoint-tag tag-get">GET</span> /mcp</span>
        <span class="stat-val">Painel Web de Diagnóstico</span>
      </div>
      <div class="stat-row">
        <span class="stat-label"><span class="endpoint-tag tag-get">GET</span> /health</span>
        <span class="stat-val">JSON Health Check</span>
      </div>
    </div>

    <h3 style="color: var(--muted); font-size: 0.95rem; text-transform: uppercase; margin-top: 1.5rem; margin-bottom: 0.5rem;">Configuração MCP (Cursor, Claude, Antigravity)</h3>
    <pre>{{{{
  "mcpServers": {{{{
    "graphfinder": {{{{
      "type": "http",
      "url": "http://127.0.0.1:{port}/mcp"
    }}}}
  }}}}
}}}}</pre>
  </div>
</body>
</html>"#,
            active_id = active_id,
            note_count = note_count,
            vault_path = vault_path,
            port = state.port
        );
        return Html(html_content).into_response();
    }

    Json(json!({
        "status": "online",
        "service": "GraphFinder MCP Server (Official rmcp SDK)",
        "version": env!("CARGO_PKG_VERSION"),
        "transport": ["streamable-http"],
        "endpoints": {
            "mcp": format!("http://127.0.0.1:{}/mcp", state.port),
            "health": format!("http://127.0.0.1:{}/health", state.port)
        },
        "vault": {
            "active_project": active_id,
            "indexed_notes": note_count,
            "vault_path": vault_path
        }
    }))
    .into_response()
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
        .route("/", get(info_handler))
        .route("/health", get(info_handler))
        .route("/mcp", get(info_handler).post_service(mcp_service))
        .route("/sse", get(info_handler).post_service(sse_service))
        .layer(middleware::from_fn(security_middleware))
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("[GraphFinder MCP] Servidor HTTP Oficial (rmcp) iniciando em http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("[GraphFinder MCP] Servidor HTTP Oficial (rmcp) ativo e escutando em http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
