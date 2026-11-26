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
            let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
            let name = params.get("name").and_then(Value::as_str).unwrap_or("");
            let empty = json!({});
            let arguments = params.get("arguments").unwrap_or(&empty);
            match call_tool(context, name, arguments).await {
                Ok(result) => Some(json_result(id, tool_success(result))),
                Err(error) => Some(json_error(id, -32000, &error)),
            }
        }
        "ping" => Some(json_result(id, json!({}))),
        _ => Some(json_error(id, -32601, "method not found")),
    }
}

fn tool_success(value: Value) -> Value {
    let text = serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".into());
    json!({
        "content": [{"type":"text","text":text}],
        "structuredContent": value,
        "isError": false
    })
}

fn argument<'a>(arguments: &'a Value, name: &str) -> Result<&'a str, String> {
    arguments
        .get(name)
        .and_then(Value::as_str)
        .ok_or_else(|| format!("missing required argument `{name}`"))
}

fn argument_limit(arguments: &Value, default: i64, maximum: i64) -> i64 {
    arguments
        .get("limit")
        .and_then(Value::as_i64)
        .unwrap_or(default)
        .clamp(1, maximum)
}

async fn resolve_project(
    context: &McpContext<'_>,
    input: &str,
    scope: &str,
) -> Result<gf_storage::models::Project, String> {
    let project = if let Ok(id) = uuid::Uuid::parse_str(input) {
        gf_storage::get_project(context.pool, id)
            .await
            .map_err(|e| e.to_string())?
            .filter(|project| project.tenant_id == context.principal.tenant_id)
    } else {
        sqlx::query_as::<_, gf_storage::models::Project>(
            "SELECT * FROM projects WHERE tenant_id=$1 AND slug=$2",
        )
        .bind(context.principal.tenant_id)
        .bind(input)
        .fetch_optional(context.pool)
        .await
        .map_err(|e| e.to_string())?
    }
    .ok_or_else(|| "project not found".to_string())?;

    if !gf_storage::authorize_project_scope(context.pool, context.principal, project.id, scope)
        .await
        .map_err(|e| e.to_string())?
    {
        return Err("insufficient project scope".into());
    }
    Ok(project)
}

async fn tool_list_projects(context: &McpContext<'_>) -> Result<Value, String> {
    let projects = gf_storage::list_projects(context.pool, context.principal.tenant_id)
        .await
        .map_err(|e| e.to_string())?;
    let mut visible = Vec::new();
    for project in projects {
        if gf_storage::authorize_project_scope(
            context.pool,
            context.principal,
            project.id,
            gf_storage::SCOPE_SEARCH_READ,
        )
        .await
        .unwrap_or(false)
        {
            visible.push(json!({
                "id": project.id,
                "slug": project.slug,
                "name": project.name,
                "active_generation": project.active_generation
            }));
        }
    }
    Ok(json!({"projects": visible, "count": visible.len()}))
}

async fn search_hits(
    context: &McpContext<'_>,
    project: &gf_storage::models::Project,
    query: &str,
    filters: gf_storage::SearchFilters,
    limit: i64,
) -> Result<Value, String> {
    let embedder = context
        .embedder
        .ok_or_else(|| "embedding provider unavailable".to_string())?;
    let embedding = embedder.embed_query(query).map_err(|e| e.to_string())?;
    let hits =
        gf_storage::hybrid_search(context.pool, project.id, query, &embedding, &filters, limit)
            .await
            .map_err(|e| e.to_string())?;
    let results = hits
        .into_iter()
        .map(|hit| {
            json!({
                "path": hit.path,
                "heading_path": hit.heading_path,
                "score": hit.score,
                "excerpt": hit.content.chars().take(500).collect::<String>(),
                "chunk_id": hit.chunk_id,
                "document_id": hit.document_id
            })
        })
        .collect::<Vec<_>>();
    Ok(json!({"project": project.slug, "results": results, "count": results.len()}))
}

async fn tool_find_files(context: &McpContext<'_>, arguments: &Value) -> Result<Value, String> {
    let project = resolve_project(
        context,
        argument(arguments, "project")?,
        gf_storage::SCOPE_SEARCH_READ,
    )
    .await?;
    let pattern = argument(arguments, "pattern")?;
    let limit = argument_limit(arguments, 20, 100);
    let like = if pattern.contains('*') {
        pattern.replace('*', "%")
    } else {
        format!("%{pattern}%")
    };
    let rows = sqlx::query_as::<_, (String, String, Option<String>, i64)>(
        r#"
        SELECT d.path,d.document_type,d.language,d.size_bytes
        FROM documents d
        JOIN projects p ON p.id=d.project_id AND p.active_generation=d.generation
        WHERE d.project_id=$1 AND d.path ILIKE $2
        ORDER BY d.path LIMIT $3
        "#,
    )
    .bind(project.id)
    .bind(like)
    .bind(limit)
    .fetch_all(context.pool)
    .await
    .map_err(|e| e.to_string())?;
    let files = rows
        .into_iter()
        .map(|(path, document_type, language, size_bytes)| {
            json!({"path":path,"type":document_type,"language":language,"size_bytes":size_bytes})
        })
        .collect::<Vec<_>>();
    Ok(json!({"project":project.slug,"files":files,"count":files.len()}))
}

async fn active_document(
    context: &McpContext<'_>,
    project_id: uuid::Uuid,
    path: &str,
) -> Result<gf_storage::models::Document, String> {
    sqlx::query_as::<_, gf_storage::models::Document>(
        r#"
        SELECT d.* FROM documents d
        JOIN projects p ON p.id=d.project_id AND p.active_generation=d.generation
        WHERE d.project_id=$1 AND d.path=$2
        "#,
    )
    .bind(project_id)
    .bind(path)
    .fetch_optional(context.pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "file not found in active generation".into())
}

async fn tool_get_file_context(
    context: &McpContext<'_>,
    arguments: &Value,
) -> Result<Value, String> {
    let project = resolve_project(
        context,
        argument(arguments, "project")?,
        gf_storage::SCOPE_CODE_READ,
    )
    .await?;
    let path = argument(arguments, "path")?;
    let document = active_document(context, project.id, path).await?;
    let chunks = gf_storage::list_chunks(context.pool, document.id)
        .await
        .map_err(|e| e.to_string())?;
    Ok(json!({
        "project": project.slug,
        "path": document.path,
        "type": document.document_type,
        "language": document.language,
        "size_bytes": document.size_bytes,
        "metadata": document.metadata,
        "chunks": chunks.iter().map(|chunk| json!({
            "heading_path": chunk.heading_path,
            "token_count": chunk.token_count,
            "start_line": chunk.start_line,
            "end_line": chunk.end_line
        })).collect::<Vec<_>>()
    }))
}

async fn tool_read_code_chunk(
    context: &McpContext<'_>,
    arguments: &Value,
) -> Result<Value, String> {
    let project = resolve_project(
        context,
        argument(arguments, "project")?,
        gf_storage::SCOPE_CODE_READ,
    )
    .await?;
    let path = argument(arguments, "path")?;
    let symbol = argument(arguments, "symbol")?;
    let document = active_document(context, project.id, path).await?;
    let chunk = sqlx::query_as::<_, gf_storage::models::Chunk>(
        r#"
        SELECT * FROM chunks
        WHERE document_id=$1 AND ($2 = ANY(heading_path) OR metadata->>'symbol'=$2)
        ORDER BY chunk_index LIMIT 1
        "#,
    )
    .bind(document.id)
    .bind(symbol)
    .fetch_optional(context.pool)
    .await
    .map_err(|e| e.to_string())?
    .ok_or_else(|| "chunk not found".to_string())?;
    Ok(json!({
        "project": project.slug,
        "path": document.path,
        "symbol": symbol,
        "start_line": chunk.start_line,
        "end_line": chunk.end_line,
        "content": chunk.content
    }))
}

async fn call_tool(
    context: &McpContext<'_>,
    name: &str,
    arguments: &Value,
) -> Result<Value, String> {
    match name {
        "list_projects" => tool_list_projects(context).await,
        "search_semantic" => {
            let project = resolve_project(
                context,
                argument(arguments, "project")?,
                gf_storage::SCOPE_SEARCH_READ,
            )
            .await?;
            search_hits(
                context,
                &project,
                argument(arguments, "query")?,
                gf_storage::SearchFilters::default(),
                argument_limit(arguments, 5, 20),
            )
            .await
        }
        "search_code" => {
            let project = resolve_project(
                context,
                argument(arguments, "project")?,
                gf_storage::SCOPE_CODE_READ,
            )
            .await?;
            search_hits(
                context,
                &project,
                argument(arguments, "query")?,
                gf_storage::SearchFilters {
                    language: arguments
                        .get("language")
                        .and_then(Value::as_str)
                        .map(str::to_string),
                    document_type: Some("code".into()),
                },
                argument_limit(arguments, 5, 20),
            )
            .await
        }
        "find_files" => tool_find_files(context, arguments).await,
        "get_file_context" => tool_get_file_context(context, arguments).await,
        "read_code_chunk" => tool_read_code_chunk(context, arguments).await,
        _ => Err("unknown tool".into()),
    }
}
