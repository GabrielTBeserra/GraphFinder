use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

use crate::engine::VaultManager;

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcRequest {
    jsonrpc: String,
    id: Option<Value>,
    method: String,
    params: Option<Value>,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcResponse {
    jsonrpc: String,
    id: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<JsonRpcError>,
}

#[derive(Debug, Deserialize, Serialize)]
struct JsonRpcError {
    code: i32,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    data: Option<Value>,
}

pub async fn run_mcp_server(
    manager: Arc<VaultManager>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    eprintln!(
        "[GraphFinder MCP] Servidor iniciado via stdio. Suporte a múltiplos projetos/vaults ativo."
    );

    let stdin = tokio::io::stdin();
    let mut stdout = tokio::io::stdout();
    let reader = BufReader::new(stdin);
    let mut lines = reader.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        let request: JsonRpcRequest = match serde_json::from_str(line) {
            Ok(req) => req,
            Err(e) => {
                eprintln!("[GraphFinder MCP] JSON inválido recebido: {}", e);
                let err_resp = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: None,
                    result: None,
                    error: Some(JsonRpcError {
                        code: -32700,
                        message: format!("Parse error: {}", e),
                        data: None,
                    }),
                };
                let out = serde_json::to_string(&err_resp)? + "\n";
                stdout.write_all(out.as_bytes()).await?;
                stdout.flush().await?;
                continue;
            }
        };

        let response = handle_request(&manager, request).await;
        if let Some(resp) = response {
            let out = serde_json::to_string(&resp)? + "\n";
            stdout.write_all(out.as_bytes()).await?;
            stdout.flush().await?;
        }
    }

    Ok(())
}

async fn handle_request(
    manager: &Arc<VaultManager>,
    req: JsonRpcRequest,
) -> Option<JsonRpcResponse> {
    let id = req.id;
    let method = req.method.as_str();
    eprintln!("[GraphFinder MCP Incoming] Method: {}", method);

    match method {
        "initialize" => {
            let client_protocol = req
                .params
                .as_ref()
                .and_then(|p| p.get("protocolVersion"))
                .and_then(|v| v.as_str())
                .unwrap_or("2024-11-05");

            let result = json!({
                "protocolVersion": client_protocol,
                "capabilities": {
                    "tools": {
                        "listChanged": false
                    },
                    "resources": {},
                    "prompts": {}
                },
                "serverInfo": {
                    "name": "graphfinder",
                    "version": env!("CARGO_PKG_VERSION")
                }
            });
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(result),
                error: None,
            })
        }
        "initialized" | "notifications/initialized" | "notifications/cancelled" => None,
        "ping" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({})),
            error: None,
        }),
        "resources/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({ "resources": [] })),
            error: None,
        }),
        "prompts/list" => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(json!({ "prompts": [] })),
            error: None,
        }),
        "tools/list" => {
            let tools = json!({
                "tools": [
                    {
                        "name": "list_projects",
                        "description": "Lista todos os projetos e seus respectivos cofres (vaults) isolados, incluindo quantidade de notas e projeto ativo.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "create_project",
                        "description": "Cria um novo projeto com cofre (vault) e banco SQLite de embeddings 100% isolado.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string", "description": "Nome do novo projeto (ex: 'HomeLab', 'Finanças', 'Estudos')" }
                            },
                            "required": ["name"]
                        }
                    },
                    {
                        "name": "switch_project",
                        "description": "Altera o projeto/vault ativo para as próximas operações no GraphFinder.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "project_id": { "type": "string", "description": "ID ou slug do projeto para o qual deseja alternar" }
                            },
                            "required": ["project_id"]
                        }
                    },
                    {
                        "name": "search_semantic",
                        "description": "Busca semântica cirúrgica por seções para máxima economia de tokens. Retorna o trecho exato de 2-4 linhas com deep link [[Nota#Seção]] e pontuação de relevância.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "query": { "type": "string", "description": "Texto ou pergunta para buscar no cofre" },
                                "limit": { "type": "integer", "description": "Quantidade máxima de trechos a retornar (padrão: 5)" },
                                "project": { "type": "string", "description": "ID do projeto/vault específico (opcional, usa o ativo por padrão)" },
                                "format": {
                                    "type": "string",
                                    "enum": ["dense_chunks", "summaries"],
                                    "description": "'dense_chunks' (padrão, trecho exato de 60-80 tokens) ou 'summaries' (visão geral de 1 linha)"
                                }
                            },
                            "required": ["query"]
                        }
                    },
                    {
                        "name": "get_compressed_context",
                        "description": "Retorna uma síntese ultracompacta (~50-80 tokens) de uma nota: resumo executivo, tópicos principais, tokens e nós vizinhos conectados.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "target": { "type": "string", "description": "Título ou ID da nota" },
                                "project": { "type": "string", "description": "ID do projeto/vault específico (opcional)" }
                            },
                            "required": ["target"]
                        }
                    },
                    {
                        "name": "suggest_links",
                        "description": "Auto-Linker: analisa um parágrafo de rascunho e sugere notas ou seções existentes com alta similaridade semântica para conexão imediata.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "text": { "type": "string", "description": "Texto ou parágrafo sendo redigido" },
                                "current_note": { "type": "string", "description": "ID ou título da nota atual para ignorar auto-referência (opcional)" },
                                "threshold": { "type": "number", "description": "Similaridade mínima de corte (0.0 a 1.0, padrão: 0.60)" },
                                "project": { "type": "string", "description": "ID do projeto/vault específico (opcional)" }
                            },
                            "required": ["text"]
                        }
                    },
                    {
                        "name": "store_context",
                        "description": "Salva ou atualiza uma nota no vault. Realiza chunking semântico por seções, gera resumo denso e calcula embeddings automaticamente.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "title": { "type": "string", "description": "Título da nota ou conceito principal" },
                                "content": { "type": "string", "description": "Conteúdo em Markdown da nota" },
                                "tags": { "type": "array", "items": { "type": "string" }, "description": "Lista de tags (opcional)" },
                                "links": { "type": "array", "items": { "type": "string" }, "description": "Lista de notas a conectar com wikilinks (opcional)" },
                                "project": { "type": "string", "description": "ID do projeto/vault específico onde salvar (opcional)" }
                            },
                            "required": ["title", "content"]
                        }
                    },
                    {
                        "name": "get_graph_neighborhood",
                        "description": "Retorna o subgrafo associativo ao redor de uma nota (1 ou 2 graus de separação) com resumo conciso de cada nó vizinho.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "target": { "type": "string", "description": "Título ou ID da nota" },
                                "depth": { "type": "integer", "description": "Graus de separação no grafo (1 ou 2)" },
                                "project": { "type": "string", "description": "ID do projeto/vault específico (opcional)" }
                            },
                            "required": ["target"]
                        }
                    },
                    {
                        "name": "link_notes",
                        "description": "Cria explicitamente uma ligação bidirecional entre duas notas ou conceitos.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "from_note": { "type": "string", "description": "Título ou ID da nota de origem" },
                                "to_note": { "type": "string", "description": "Título da nota de destino" },
                                "project": { "type": "string", "description": "ID do projeto/vault específico (opcional)" }
                            },
                            "required": ["from_note", "to_note"]
                        }
                    },
                    {
                        "name": "list_recent_notes",
                        "description": "Lista as notas do cofre com seus resumos de 1 linha e estimativas de tokens.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "limit": { "type": "integer", "description": "Quantidade máxima de notas a retornar" },
                                "project": { "type": "string", "description": "ID do projeto/vault específico (opcional)" }
                            }
                        }
                    },
                    {
                        "name": "sync_vault",
                        "description": "Sincroniza o cofre de arquivos .md com o banco vetorial SQLite local, recalculando embeddings e expurgando notas deletadas.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "project": { "type": "string", "description": "ID do projeto/vault específico (opcional)" }
                            }
                        }
                    },
                    {
                        "name": "list_projects",
                        "description": "Lista todos os projetos e cofres de notas disponíveis no GraphFinder.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {}
                        }
                    },
                    {
                        "name": "create_project",
                        "description": "Cria um novo cofre de notas em qualquer diretório do computador ou na pasta padrão de vaults.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "name": { "type": "string", "description": "Nome do projeto ou cofre" },
                                "path": { "type": "string", "description": "Caminho absoluto opcional da pasta no computador" }
                            },
                            "required": ["name"]
                        }
                    },
                    {
                        "name": "switch_project",
                        "description": "Alterna o cofre ativo no GraphFinder.",
                        "inputSchema": {
                            "type": "object",
                            "properties": {
                                "project_id": { "type": "string", "description": "ID do projeto a ser ativado" }
                            },
                            "required": ["project_id"]
                        }
                    }
                ]
            });
            Some(JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id,
                result: Some(tools),
                error: None,
            })
        }
        "tools/call" => {
            let params = req.params.unwrap_or(json!({}));
            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            let call_res = execute_tool(manager, tool_name, arguments).await;
            match call_res {
                Ok(content) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({
                        "content": [
                            {
                                "type": "text",
                                "text": content
                            }
                        ],
                        "isError": false
                    })),
                    error: None,
                }),
                Err(err) => Some(JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id,
                    result: Some(json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Erro ao executar ferramenta: {}", err)
                            }
                        ],
                        "isError": true
                    })),
                    error: None,
                }),
            }
        }
        _ => Some(JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(JsonRpcError {
                code: -32601,
                message: format!("Método '{}' não encontrado", method),
                data: None,
            }),
        }),
    }
}

async fn execute_tool(
    manager: &Arc<VaultManager>,
    tool_name: &str,
    args: Value,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    match tool_name {
        "list_projects" => {
            let projects = manager.list_projects().await;
            let mut out = String::from("Projetos e Vaults no GraphFinder:\n\n");
            for p in projects {
                let status = if p.is_active { " (Ativo)" } else { "" };
                out.push_str(&format!(
                    "- **{}** (ID: `{}`){} | {} notas\n  Caminho: {}\n\n",
                    p.name, p.id, status, p.note_count, p.path
                ));
            }
            Ok(out)
        }
        "create_project" => {
            let name = args
                .get("name")
                .and_then(|v| v.as_str())
                .ok_or("Campo 'name' é obrigatório")?;
            let custom_path = args
                .get("path")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let project = manager.create_project(name, custom_path).await?;
            Ok(format!(
                "Projeto '{}' (ID: '{}') criado com sucesso!\nCofre de notas: {}\nBanco SQLite de vetores: {}",
                project.name, project.id, project.path, project.db_path
            ))
        }
        "switch_project" => {
            let project_id = args
                .get("project_id")
                .and_then(|v| v.as_str())
                .ok_or("Campo 'project_id' é obrigatório")?;
            let project = manager.switch_project(project_id).await?;
            Ok(format!(
                "Projeto ativo alterado com sucesso para: {} (ID: '{}') com {} notas.",
                project.name, project.id, project.note_count
            ))
        }
        "search_semantic" => {
            let project = args.get("project").and_then(|v| v.as_str());
            let engine = manager.get_engine(project).await;

            let query = args
                .get("query")
                .and_then(|v| v.as_str())
                .ok_or("Campo 'query' é obrigatório")?;
            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;
            let format = args
                .get("format")
                .and_then(|v| v.as_str())
                .unwrap_or("dense_chunks");

            if format == "summaries" {
                let hits = engine.search_semantic(query, limit).await?;
                if hits.is_empty() {
                    return Ok("Nenhum resultado semântico encontrado no cofre.".to_string());
                }
                let mut out = String::from("Visão Geral Semântica:\n");
                for (i, h) in hits.into_iter().enumerate() {
                    out.push_str(&format!(
                        "{}. [[{}]] (Relevância: {:.0}%)\n   Síntese: {}\n",
                        i + 1,
                        h.title,
                        h.score * 100.0,
                        h.snippet
                    ));
                }
                return Ok(out);
            }

            // Default: Dense Chunks (granular section-level search)
            let chunks = engine.search_chunks(query, limit).await?;
            if chunks.is_empty() {
                return Ok("Nenhum trecho semântico relevante encontrado no cofre.".to_string());
            }

            let mut out =
                String::from("Trechos Semânticos de Alta Densidade (Token-Efficient):\n\n");
            for (i, c) in chunks.into_iter().enumerate() {
                out.push_str(&format!(
                    "{}. {} | Score: {:.0}% (~{} tokens)\n\"{}\"\n\n",
                    i + 1,
                    c.deep_link,
                    c.score * 100.0,
                    c.token_estimate,
                    c.content.trim()
                ));
            }
            Ok(out)
        }
        "get_compressed_context" => {
            let project = args.get("project").and_then(|v| v.as_str());
            let engine = manager.get_engine(project).await;

            let target = args
                .get("target")
                .and_then(|v| v.as_str())
                .ok_or("Campo 'target' é obrigatório")?;

            let ctx = engine.get_compressed_context(target).await?;
            match ctx {
                Some(c) => {
                    let mut out = format!(
                        "Síntese da Nota: [[{}]] (~{} tokens totais)\nResumo: {}\nTags: {:?}\nConexões no Grafo: {:?}\n\nSeções Disponíveis:\n",
                        c.title, c.token_estimate, c.summary, c.tags, c.connected_notes
                    );
                    for s in c.sections {
                        out.push_str(&format!(
                            "- #{} (~{} tokens): {}\n",
                            s.heading, s.token_estimate, s.snippet
                        ));
                    }
                    Ok(out)
                }
                None => Ok(format!("Nota '{}' não encontrada.", target)),
            }
        }
        "suggest_links" => {
            let project = args.get("project").and_then(|v| v.as_str());
            let engine = manager.get_engine(project).await;

            let text = args
                .get("text")
                .and_then(|v| v.as_str())
                .ok_or("Campo 'text' é obrigatório")?;
            let current_note = args
                .get("current_note")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let threshold = args
                .get("threshold")
                .and_then(|v| v.as_f64())
                .unwrap_or(0.60) as f32;

            let suggestions = engine
                .suggest_links(text, current_note, threshold, 4)
                .await?;
            if suggestions.is_empty() {
                return Ok(
                    "Nenhuma sugestão forte de link encontrada para este trecho.".to_string(),
                );
            }

            let mut out = String::from("Sugestões Semânticas de Wikilinks para o Grafo:\n\n");
            for (i, s) in suggestions.into_iter().enumerate() {
                out.push_str(&format!(
                    "{}. {} (Afinidade: {:.0}%)\n   Contexto: \"{}\"\n",
                    i + 1,
                    s.deep_link,
                    s.similarity_score * 100.0,
                    s.preview
                ));
            }
            Ok(out)
        }
        "store_context" => {
            let project = args.get("project").and_then(|v| v.as_str());
            let engine = manager.get_engine(project).await;

            let title = args
                .get("title")
                .and_then(|v| v.as_str())
                .ok_or("Campo 'title' é obrigatório")?;
            let mut content = args
                .get("content")
                .and_then(|v| v.as_str())
                .ok_or("Campo 'content' é obrigatório")?
                .to_string();

            // Append optional links
            if let Some(links) = args.get("links").and_then(|v| v.as_array()) {
                if !links.is_empty() {
                    content.push_str("\n\n## Conexões Relacionadas\n");
                    for l in links {
                        if let Some(s) = l.as_str() {
                            content.push_str(&format!("- [[{}]]\n", s));
                        }
                    }
                }
            }

            // Append optional tags
            if let Some(tags) = args.get("tags").and_then(|v| v.as_array()) {
                if !tags.is_empty() {
                    content.push_str("\n\nTags: ");
                    for t in tags {
                        if let Some(s) = t.as_str() {
                            let tag_clean = s.trim_start_matches('#');
                            content.push_str(&format!("#{} ", tag_clean));
                        }
                    }
                    content.push('\n');
                }
            }

            let note = engine.save_note(title, &content, None).await?;
            let autolinks_info = if !note.outgoing_links.is_empty() {
                format!(
                    "\nAuto-links Criados (≥65% afinidade): {:?}",
                    note.outgoing_links
                )
            } else {
                "\nAuto-links: Nenhuma nota com afinidade ≥ 65% encontrada.".to_string()
            };
            Ok(format!(
                "Nota salva, vetorizada e autovinculada com sucesso!\nTítulo: [[{}]]\nArquivo: {}\nTags: {:?}{}",
                note.title, note.path, note.tags, autolinks_info
            ))
        }
        "get_graph_neighborhood" => {
            let project = args.get("project").and_then(|v| v.as_str());
            let engine = manager.get_engine(project).await;

            let target = args
                .get("target")
                .and_then(|v| v.as_str())
                .ok_or("Campo 'target' é obrigatório")?;
            let depth = args.get("depth").and_then(|v| v.as_u64()).unwrap_or(1) as usize;

            let n = engine.get_neighborhood(target, depth).await?;
            let mut out = format!("Subgrafo ao redor de [[{}]]:\n\nNós Conectados:\n", target);
            for node in n.nodes {
                out.push_str(&format!(
                    "- [[{}]] ({} links, tags: {:?})\n",
                    node.title, node.degree, node.tags
                ));
            }
            Ok(out)
        }
        "link_notes" => {
            let project = args.get("project").and_then(|v| v.as_str());
            let engine = manager.get_engine(project).await;

            let from_note = args
                .get("from_note")
                .and_then(|v| v.as_str())
                .ok_or("Campo 'from_note' é obrigatório")?;
            let to_note = args
                .get("to_note")
                .and_then(|v| v.as_str())
                .ok_or("Campo 'to_note' é obrigatório")?;

            let updated = engine.link_notes(from_note, to_note).await?;
            Ok(format!(
                "Ligação criada com sucesso!\nOrigem: [[{}]] -> Destino: [[{}]]\nLinks atualizados: {:?}",
                updated.title, to_note, updated.outgoing_links
            ))
        }
        "list_recent_notes" => {
            let project = args.get("project").and_then(|v| v.as_str());
            let engine = manager.get_engine(project).await;

            let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(10) as usize;
            let notes = engine.list_notes().await?;
            let display_notes = notes.into_iter().take(limit);

            let mut out = String::from("Notas no cofre de conhecimento:\n");
            for n in display_notes {
                out.push_str(&format!(
                    "- [[{}]] (Tags: {:?}) - \"{}\"\n",
                    n.title, n.tags, n.summary
                ));
            }
            Ok(out)
        }
        "sync_vault" => {
            let project = args.get("project").and_then(|v| v.as_str());
            let engine = manager.get_engine(project).await;
            let count = engine.sync_vault().await?;
            Ok(format!(
                "Sincronização concluída com sucesso! {} notas ativas e indexadas no vault.",
                count
            ))
        }
        _ => Err(format!("Ferramenta '{}' não implementada", tool_name).into()),
    }
}
