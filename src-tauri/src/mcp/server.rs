use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::sse::{Event, KeepAlive, Sse},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use serde_json::json;
use std::{collections::HashMap, convert::Infallible, net::SocketAddr, sync::Arc, time::Duration};
use tauri::AppHandle;
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use tower_http::cors::CorsLayer;

use super::{handle_request, JsonRpcRequest};
use crate::engine::VaultManager;

#[derive(Clone)]
pub struct ServerState {
    pub manager: Arc<VaultManager>,
    pub app_handle: Option<AppHandle>,
    pub sessions: Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Event>>>>,
    pub port: u16,
}

#[derive(Deserialize)]
struct SessionQuery {
    #[serde(default)]
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
}

async fn health_handler(State(state): State<ServerState>) -> impl IntoResponse {
    let active_id = state.manager.active_project_id().await;
    let engine = state.manager.active_engine().await;
    let note_count = engine.list_notes().await.map(|n| n.len()).unwrap_or(0);

    Json(json!({
        "status": "online",
        "service": "GraphFinder MCP Server",
        "version": env!("CARGO_PKG_VERSION"),
        "transport": ["sse", "http"],
        "endpoints": {
            "sse": format!("http://127.0.0.1:{}/sse", state.port),
            "message": format!("http://127.0.0.1:{}/message?sessionId={{sessionId}}", state.port),
            "direct_http": format!("http://127.0.0.1:{}/mcp", state.port)
        },
        "vault": {
            "active_project": active_id,
            "indexed_notes": note_count
        }
    }))
}

async fn sse_handler(
    State(state): State<ServerState>,
) -> Sse<impl tokio_stream::Stream<Item = Result<Event, Infallible>>> {
    let session_id = uuid::Uuid::new_v4().to_string();
    let (tx, rx) = mpsc::unbounded_channel::<Event>();

    // Envia o endpoint relativo com sessionId conforme especificação do protocolo MCP SSE
    let endpoint_url = format!("/message?sessionId={}", session_id);
    let _ = tx.send(Event::default().event("endpoint").data(endpoint_url));

    {
        let mut sessions = state.sessions.write().await;
        sessions.insert(session_id.clone(), tx);
    }

    let stream = UnboundedReceiverStream::new(rx).map(Ok::<Event, Infallible>);

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

async fn message_handler(
    State(state): State<ServerState>,
    Query(query): Query<SessionQuery>,
    headers: HeaderMap,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let session_id = query
        .session_id
        .or_else(|| {
            headers
                .get("x-session-id")
                .and_then(|h| h.to_str().ok())
                .map(|s| s.to_string())
        })
        .unwrap_or_default();

    if session_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32600,
                    "message": "Parâmetro 'sessionId' ausente na requisição"
                }
            })),
        );
    }

    let sender = {
        let sessions = state.sessions.read().await;
        sessions.get(&session_id).cloned()
    };

    let tx = match sender {
        Some(s) => s,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32000,
                        "message": format!("Sessão MCP '{}' não encontrada", session_id)
                    }
                })),
            );
        }
    };

    let manager = Arc::clone(&state.manager);
    let app_handle = state.app_handle.clone();
    let sessions_map = Arc::clone(&state.sessions);
    let session_id_clone = session_id.clone();

    // Processa a requisição assincronamente e envia a resposta pelo canal SSE
    tokio::spawn(async move {
        let response = handle_request(&manager, app_handle.as_ref(), req).await;
        if let Some(resp) = response {
            if let Ok(serialized) = serde_json::to_string(&resp) {
                let event = Event::default().event("message").data(serialized);
                if tx.send(event).is_err() {
                    // Se o envio falhar, o cliente desconectou da stream SSE
                    let mut sessions = sessions_map.write().await;
                    sessions.remove(&session_id_clone);
                }
            }
        }
    });

    (StatusCode::ACCEPTED, Json(json!({ "status": "accepted" })))
}

async fn direct_mcp_handler(
    State(state): State<ServerState>,
    Json(req): Json<JsonRpcRequest>,
) -> impl IntoResponse {
    let response = handle_request(&state.manager, state.app_handle.as_ref(), req).await;
    match response {
        Some(resp) => (
            StatusCode::OK,
            Json(serde_json::to_value(resp).unwrap_or_default()),
        ),
        None => (StatusCode::NO_CONTENT, Json(json!({}))),
    }
}

pub async fn start_http_mcp_server(
    manager: Arc<VaultManager>,
    app_handle: Option<AppHandle>,
    port: u16,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = ServerState {
        manager,
        app_handle,
        sessions: Arc::new(RwLock::new(HashMap::new())),
        port,
    };

    let app = Router::new()
        .route("/", get(health_handler))
        .route("/health", get(health_handler))
        .route("/sse", get(sse_handler))
        .route("/message", post(message_handler))
        .route("/mcp", post(direct_mcp_handler))
        .route("/api/mcp", post(direct_mcp_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    eprintln!("[GraphFinder MCP] Servidor HTTP/SSE iniciando em http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    eprintln!("[GraphFinder MCP] Servidor HTTP/SSE ativo e escutando em http://{}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
