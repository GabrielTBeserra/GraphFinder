//! MCP protocol surface for GraphFinder Server.

use serde_json::{json, Value};
use sqlx::PgPool;

use gf_core::EmbeddingProvider;
use gf_storage::Principal;

pub const MCP_PROTOCOL_VERSION: &str = "2025-03-26";

#[derive(Clone)]
pub struct McpContext<'a> {
    pub pool: &'a PgPool,
    pub principal: &'a Principal,
    pub embedder: Option<&'a dyn EmbeddingProvider>,
}

pub fn tool_definitions() -> Value {
    json!([
        {
            "name": "list_projects",
            "description": "List GraphFinder projects visible to the authenticated token.",
            "inputSchema": {"type":"object","properties":{},"additionalProperties":false}
        },
        {
            "name": "search_semantic",
            "description": "Hybrid semantic and lexical search over the active generation of a project.",
            "inputSchema": {
                "type":"object",
                "properties":{
                    "project":{"type":"string"},
                    "query":{"type":"string"},
                    "limit":{"type":"integer","minimum":1,"maximum":20}
                },
                "required":["project","query"],
                "additionalProperties":false
            }
        },
        {
            "name": "search_code",
            "description": "Search indexed code symbols and chunks in a project.",
            "inputSchema": {
                "type":"object",
                "properties":{
                    "project":{"type":"string"},
                    "query":{"type":"string"},
                    "language":{"type":"string"},
                    "limit":{"type":"integer","minimum":1,"maximum":20}
                },
                "required":["project","query"],
                "additionalProperties":false
            }
        },
        {
            "name": "find_files",
            "description": "Find indexed files by path substring or simple wildcard pattern.",
            "inputSchema": {
                "type":"object",
                "properties":{
                    "project":{"type":"string"},
                    "pattern":{"type":"string"},
                    "limit":{"type":"integer","minimum":1,"maximum":100}
                },
                "required":["project","pattern"],
                "additionalProperties":false
            }
        },
        {
            "name": "get_file_context",
            "description": "Return compact structural context for an indexed file.",
            "inputSchema": {
                "type":"object",
                "properties":{
                    "project":{"type":"string"},
                    "path":{"type":"string"}
                },
                "required":["project","path"],
                "additionalProperties":false
            }
        },
        {
            "name": "read_code_chunk",
            "description": "Read one indexed code symbol or markdown section without fetching the whole file.",
            "inputSchema": {
                "type":"object",
                "properties":{
                    "project":{"type":"string"},
                    "path":{"type":"string"},
                    "symbol":{"type":"string"}
                },
                "required":["project","path","symbol"],
                "additionalProperties":false
            }
        }
    ])
}

fn json_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc":"2.0",
        "id":id,
        "error":{"code":code,"message":message}
    })
}

fn json_result(id: Value, result: Value) -> Value {
    json!({"jsonrpc":"2.0","id":id,"result":result})
}

pub async fn handle_rpc(context: &McpContext<'_>, request: Value) -> Option<Value> {
    let id = request.get("id").cloned().unwrap_or(Value::Null);
    let is_notification = request.get("id").is_none();
    let Some(method) = request.get("method").and_then(Value::as_str) else {
        return Some(json_error(id, -32600, "invalid JSON-RPC request"));
    };

    match method {
        "initialize" => Some(json_result(
            id,
            json!({
                "protocolVersion": MCP_PROTOCOL_VERSION,
                "capabilities": {"tools": {"listChanged": false}},
                "serverInfo": {"name":"graphfinder-server","version":env!("CARGO_PKG_VERSION")}
            }),
        )),
        "notifications/initialized" | "initialized" => {
            if is_notification {
                None
            } else {
                Some(json_result(id, json!({})))
            }
        }
        "tools/list" => Some(json_result(id, json!({"tools": tool_definitions()}))),
        "tools/call" => {
            let _ = context;
            Some(json_error(id, -32601, "tool is not implemented yet"))
        }
        "ping" => Some(json_result(id, json!({}))),
        _ => Some(json_error(id, -32601, "method not found")),
    }
}
