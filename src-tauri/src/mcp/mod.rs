pub mod server;

use schemars::JsonSchema;
use serde::Deserialize;
use std::sync::Arc;
use tauri::Emitter;

use rmcp::{
    handler::server::router::tool::ToolRouter,
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerConfig},
    tool, tool_handler, tool_router, ErrorData as McpError, ServerHandler, ServiceExt,
};

use crate::engine::VaultManager;

#[derive(Clone)]
pub struct GraphFinderMcpServer {
    #[allow(dead_code)]
    tool_router: ToolRouter<Self>,
    pub manager: Arc<VaultManager>,
    pub app_handle: Option<tauri::AppHandle>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct CreateProjectArgs {
    /// Nome do novo projeto (ex: 'HomeLab', 'Finanças', 'Estudos')
    pub name: String,
    /// Caminho absoluto opcional da pasta no computador
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SwitchProjectArgs {
    /// ID ou slug do projeto para o qual deseja alternar
    pub project_id: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SearchSemanticArgs {
    /// Texto ou pergunta para buscar no cofre
    pub query: String,
    /// Quantidade máxima de trechos a retornar (padrão: 5)
    pub limit: Option<usize>,
    /// ID do projeto/vault específico (opcional, usa o ativo por padrão)
    pub project: Option<String>,
    /// 'dense_chunks' (padrão, trecho exato de 60-80 tokens) ou 'summaries' (visão geral de 1 linha)
    pub format: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCompressedContextArgs {
    /// Título ou ID da nota
    pub target: String,
    /// ID do projeto/vault específico (opcional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SuggestLinksArgs {
    /// Texto ou parágrafo sendo redigido
    pub text: String,
    /// ID ou título da nota atual para ignorar auto-referência (opcional)
    pub current_note: Option<String>,
    /// Similaridade mínima de corte (0.0 a 1.0, padrão: 0.60)
    pub threshold: Option<f32>,
    /// ID do projeto/vault específico (opcional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct StoreContextArgs {
    /// Título da nota ou conceito principal
    pub title: String,
    /// Conteúdo em Markdown da nota
    pub content: String,
    /// Lista de tags (opcional)
    pub tags: Option<Vec<String>>,
    /// Lista de notas a conectar com wikilinks (opcional)
    pub links: Option<Vec<String>>,
    /// ID do projeto/vault específico onde salvar (opcional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetGraphNeighborhoodArgs {
    /// Título ou ID da nota
    pub target: String,
    /// Graus de separação no grafo (1 ou 2, padrão: 1)
    pub depth: Option<usize>,
    /// ID do projeto/vault específico (opcional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct LinkNotesArgs {
    /// Título ou ID da nota de origem
    pub from_note: String,
    /// Título da nota de destino
    pub to_note: String,
    /// ID do projeto/vault específico (opcional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ListRecentNotesArgs {
    /// Quantidade máxima de notas a retornar (padrão: 10)
    pub limit: Option<usize>,
    /// ID do projeto/vault específico (opcional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SyncVaultArgs {
    /// ID do projeto/vault específico (opcional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ReadSectionArgs {
    /// Título ou ID da nota
    pub target: String,
    /// Título da seção (H2/H3) ou fragmento do deep link (ex: '2. SQLite WAL Configuration')
    pub section: String,
    /// ID do projeto/vault específico (opcional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct RenameNoteArgs {
    /// Título ou ID atual da nota
    pub old_title: String,
    /// Novo título desejado para a nota
    pub new_title: String,
    /// ID do projeto/vault específico (opcional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct DeleteNoteArgs {
    /// Título ou ID da nota a ser excluída
    pub target: String,
    /// ID do projeto/vault específico (opcional)
    pub project: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetVaultStatsArgs {
    /// ID do projeto/vault específico (opcional)
    pub project: Option<String>,
}

impl GraphFinderMcpServer {
    pub fn new(manager: Arc<VaultManager>, app_handle: Option<tauri::AppHandle>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            manager,
            app_handle,
        }
    }
}

#[tool_router]
impl GraphFinderMcpServer {
    /// Lista todos os projetos e seus respectivos cofres (vaults) isolados, incluindo quantidade de notas e projeto ativo.
    #[tool(description = "Lista todos os projetos e seus respectivos cofres (vaults) isolados, incluindo quantidade de notas e projeto ativo.")]
    async fn list_projects(&self) -> Result<CallToolResult, McpError> {
        let projects = self.manager.list_projects().await;
        let mut out = String::from("Projetos e Vaults no GraphFinder:\n\n");
        for p in projects {
            let status = if p.is_active { " (Ativo)" } else { "" };
            out.push_str(&format!(
                "- **{}** (ID: `{}`){} | {} notas\n  Caminho: {}\n\n",
                p.name, p.id, status, p.note_count, p.path
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Cria um novo projeto com cofre (vault) e banco SQLite de embeddings 100% isolado.
    #[tool(description = "Cria um novo projeto com cofre (vault) e banco SQLite de embeddings 100% isolado.")]
    async fn create_project(&self, Parameters(args): Parameters<CreateProjectArgs>) -> Result<CallToolResult, McpError> {
        let project = match self.manager.create_project(&args.name, args.path).await {
            Ok(p) => p,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao criar projeto: {}", e))])),
        };
        if let Some(app) = &self.app_handle {
            let _ = app.emit("vault-sync-complete", 1);
        }
        let out = format!(
            "Projeto '{}' (ID: '{}') criado com sucesso!\nCofre de notas: {}\nBanco SQLite de vetores: {}",
            project.name, project.id, project.path, project.db_path
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Altera o projeto/vault ativo para as próximas operações no GraphFinder.
    #[tool(description = "Altera o projeto/vault ativo para as próximas operações no GraphFinder.")]
    async fn switch_project(&self, Parameters(args): Parameters<SwitchProjectArgs>) -> Result<CallToolResult, McpError> {
        let project = match self.manager.switch_project(&args.project_id).await {
            Ok(p) => p,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao alternar projeto: {}", e))])),
        };
        if let Some(app) = &self.app_handle {
            let _ = app.emit("vault-sync-complete", project.note_count);
        }
        let out = format!(
            "Projeto ativo alterado com sucesso para: {} (ID: '{}') com {} notas.",
            project.name, project.id, project.note_count
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Busca semântica cirúrgica por seções para máxima economia de tokens. Retorna o trecho exato de 2-4 linhas com deep link [[Nota#Seção]] e pontuação de relevância.
    #[tool(description = "Busca semântica cirúrgica por seções para máxima economia de tokens. Retorna o trecho exato de 2-4 linhas com deep link [[Nota#Seção]] e pontuação de relevância.")]
    async fn search_semantic(&self, Parameters(args): Parameters<SearchSemanticArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let limit = args.limit.unwrap_or(5);
        let format = args.format.as_deref().unwrap_or("dense_chunks");

        if format == "summaries" {
            let hits = match engine.search_semantic(&args.query, limit).await {
                Ok(h) => h,
                Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro na busca semântica: {}", e))])),
            };
            if hits.is_empty() {
                return Ok(CallToolResult::success(vec![ContentBlock::text("Nenhum resultado semântico encontrado no cofre.")]));
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
            return Ok(CallToolResult::success(vec![ContentBlock::text(out)]));
        }

        let chunks = match engine.search_chunks(&args.query, limit).await {
            Ok(c) => c,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro na busca por trechos: {}", e))])),
        };
        if chunks.is_empty() {
            return Ok(CallToolResult::success(vec![ContentBlock::text("Nenhum trecho semântico relevante encontrado no cofre.")]));
        }

        let mut out = String::from("Trechos Semânticos de Alta Densidade (Token-Efficient):\n\n");
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
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Retorna uma síntese ultracompacta (~50-80 tokens) de uma nota: resumo executivo, tópicos principais, tokens e nós vizinhos conectados.
    #[tool(description = "Retorna uma síntese ultracompacta (~50-80 tokens) de uma nota: resumo executivo, tópicos principais, tokens e nós vizinhos conectados.")]
    async fn get_compressed_context(&self, Parameters(args): Parameters<GetCompressedContextArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let ctx = match engine.get_compressed_context(&args.target).await {
            Ok(c) => c,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao recuperar contexto: {}", e))])),
        };
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
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            None => Ok(CallToolResult::success(vec![ContentBlock::text(format!("Nota '{}' não encontrada.", args.target))])),
        }
    }

    /// Auto-Linker: analisa um parágrafo de rascunho e sugere notas ou seções existentes com alta similaridade semântica para conexão imediata.
    #[tool(description = "Auto-Linker: analisa um parágrafo de rascunho e sugere notas ou seções existentes com alta similaridade semântica para conexão imediata.")]
    async fn suggest_links(&self, Parameters(args): Parameters<SuggestLinksArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let current_note = args.current_note.as_deref().unwrap_or("");
        let threshold = args.threshold.unwrap_or(0.60);

        let suggestions = match engine.suggest_links(&args.text, current_note, threshold, 4).await {
            Ok(s) => s,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao sugerir links: {}", e))])),
        };
        if suggestions.is_empty() {
            return Ok(CallToolResult::success(vec![ContentBlock::text("Nenhuma sugestão forte de link encontrada para este trecho.")]));
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
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Salva ou atualiza uma nota no vault. Realiza chunking semântico por seções, gera resumo denso e calcula embeddings automaticamente.
    #[tool(description = "Salva ou atualiza uma nota no vault. Realiza chunking semântico por seções, gera resumo denso e calcula embeddings automaticamente.")]
    async fn store_context(&self, Parameters(args): Parameters<StoreContextArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let mut content = args.content;

        if let Some(links) = args.links {
            if !links.is_empty() {
                content.push_str("\n\n## Conexões Relacionadas\n");
                for l in links {
                    content.push_str(&format!("- [[{}]]\n", l));
                }
            }
        }

        if let Some(tags) = args.tags {
            if !tags.is_empty() {
                content.push_str("\n\nTags: ");
                for t in tags {
                    let tag_clean = t.trim_start_matches('#');
                    content.push_str(&format!("#{} ", tag_clean));
                }
                content.push('\n');
            }
        }

        let note = match engine.save_note(&args.title, &content, None).await {
            Ok(n) => n,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao salvar nota: {}", e))])),
        };
        if let Some(app) = &self.app_handle {
            let _ = app.emit("vault-sync-complete", 1);
            let _ = app.emit("mcp-node-updated", &note.id);
        }
        let autolinks_info = if !note.outgoing_links.is_empty() {
            format!(
                "\nAuto-links Criados (≥65% afinidade): {:?}",
                note.outgoing_links
            )
        } else {
            "\nAuto-links: Nenhuma nota com afinidade ≥ 65% encontrada.".to_string()
        };
        let out = format!(
            "Nota salva, vetorizada e autovinculada com sucesso!\nTítulo: [[{}]]\nArquivo: {}\nTags: {:?}{}",
            note.title, note.path, note.tags, autolinks_info
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Retorna o subgrafo associativo ao redor de uma nota (1 ou 2 graus de separação) com resumo conciso de cada nó vizinho.
    #[tool(description = "Retorna o subgrafo associativo ao redor de uma nota (1 ou 2 graus de separação) com resumo conciso de cada nó vizinho.")]
    async fn get_graph_neighborhood(&self, Parameters(args): Parameters<GetGraphNeighborhoodArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let depth = args.depth.unwrap_or(1);
        let n = match engine.get_neighborhood(&args.target, depth).await {
            Ok(res) => res,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao buscar subgrafo: {}", e))])),
        };
        let mut out = format!("Subgrafo ao redor de [[{}]]:\n\nNós Conectados:\n", args.target);
        for node in n.nodes {
            out.push_str(&format!(
                "- [[{}]] ({} links, tags: {:?})\n",
                node.title, node.degree, node.tags
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Cria explicitamente uma ligação bidirecional entre duas notas ou conceitos.
    #[tool(description = "Cria explicitamente uma ligação bidirecional entre duas notas ou conceitos.")]
    async fn link_notes(&self, Parameters(args): Parameters<LinkNotesArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let updated = match engine.link_notes(&args.from_note, &args.to_note).await {
            Ok(u) => u,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao vincular notas: {}", e))])),
        };
        if let Some(app) = &self.app_handle {
            let _ = app.emit("vault-sync-complete", 1);
            let _ = app.emit("mcp-node-updated", &updated.id);
        }
        let out = format!(
            "Ligação criada com sucesso!\nOrigem: [[{}]] -> Destino: [[{}]]\nLinks atualizados: {:?}",
            updated.title, args.to_note, updated.outgoing_links
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Lista as notas do cofre com seus resumos de 1 linha e estimativas de tokens.
    #[tool(description = "Lista as notas do cofre com seus resumos de 1 linha e estimativas de tokens.")]
    async fn list_recent_notes(&self, Parameters(args): Parameters<ListRecentNotesArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let limit = args.limit.unwrap_or(10);
        let notes = match engine.list_notes().await {
            Ok(n) => n,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao listar notas: {}", e))])),
        };
        let display_notes = notes.into_iter().take(limit);

        let mut out = String::from("Notas no cofre de conhecimento:\n");
        for n in display_notes {
            out.push_str(&format!(
                "- [[{}]] (Tags: {:?}) - \"{}\"\n",
                n.title, n.tags, n.summary
            ));
        }
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Sincroniza o cofre de arquivos .md com o banco vetorial SQLite local, recalculando embeddings e expurgando notas deletadas.
    #[tool(description = "Sincroniza o cofre de arquivos .md com o banco vetorial SQLite local, recalculando embeddings e expurgando notas deletadas.")]
    async fn sync_vault(&self, Parameters(args): Parameters<SyncVaultArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        let count = match engine.sync_vault().await {
            Ok(c) => c,
            Err(e) => return Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao sincronizar cofre: {}", e))])),
        };
        if let Some(app) = &self.app_handle {
            let _ = app.emit("vault-sync-complete", count);
        }
        let out = format!(
            "Sincronização concluída com sucesso! {} notas ativas e indexadas no vault.",
            count
        );
        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }

    /// Lê cirurgicamente uma única seção (heading H2/H3) de uma nota, garantindo economia máxima de tokens para LLMs.
    #[tool(description = "Lê cirurgicamente uma única seção (heading H2/H3) de uma nota, garantindo economia máxima de tokens para LLMs.")]
    async fn read_section(&self, Parameters(args): Parameters<ReadSectionArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.read_section(&args.target, &args.section).await {
            Ok(Some(chunk)) => {
                let out = format!(
                    "Seção: {} (~{} tokens)\nDeep Link: {}\n\n{}",
                    chunk.heading.as_deref().unwrap_or("Geral"),
                    chunk.token_estimate,
                    chunk.deep_link,
                    chunk.content.trim()
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Ok(None) => Ok(CallToolResult::success(vec![ContentBlock::text(format!(
                "Seção '{}' não encontrada na nota '{}'. Use get_compressed_context para ver as seções disponíveis.",
                args.section, args.target
            ))])),
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao ler seção: {}", e))])),
        }
    }

    /// Renomeia uma nota e seu arquivo no disco, atualizando todos os wikilinks [[Nota Antiga]] no cofre inteiro para evitar links quebrados.
    #[tool(description = "Renomeia uma nota e seu arquivo no disco, atualizando todos os wikilinks [[Nota Antiga]] no cofre inteiro para evitar links quebrados.")]
    async fn rename_note(&self, Parameters(args): Parameters<RenameNoteArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.rename_note(&args.old_title, &args.new_title).await {
            Ok(note) => {
                if let Some(app) = &self.app_handle {
                    let _ = app.emit("vault-sync-complete", 1);
                }
                let out = format!(
                    "Nota renomeada com sucesso!\nTítulo anterior: {}\nNovo título: [[{}]]\nArquivo: {}\nTodos os wikilinks no cofre foram refatorados automaticamente.",
                    args.old_title, note.title, note.path
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao renomear nota: {}", e))])),
        }
    }

    /// Exclui uma nota do cofre (remove arquivo .md e expurga vetores do banco SQLite).
    #[tool(description = "Exclui uma nota do cofre (remove arquivo .md e expurga vetores do banco SQLite).")]
    async fn delete_note(&self, Parameters(args): Parameters<DeleteNoteArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.delete_note(&args.target).await {
            Ok(_) => {
                if let Some(app) = &self.app_handle {
                    let _ = app.emit("vault-sync-complete", 1);
                }
                let out = format!("Nota '{}' excluída com sucesso do cofre e do banco vetorial.", args.target);
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao excluir nota: {}", e))])),
        }
    }

    /// Retorna estatísticas de integridade do cofre: total de notas, seções vetorizadas, total de conexões, tags e tokens estimados.
    #[tool(description = "Retorna estatísticas de integridade do cofre: total de notas, seções vetorizadas, total de conexões, tags e tokens estimados.")]
    async fn get_vault_stats(&self, Parameters(args): Parameters<GetVaultStatsArgs>) -> Result<CallToolResult, McpError> {
        let engine = self.manager.get_engine(args.project.as_deref()).await;
        match engine.get_vault_stats().await {
            Ok(stats) => {
                let out = format!(
                    "Estatísticas do Cofre:\n\n- Notas Ativas: {}\n- Seções Vetorizadas: {}\n- Conexões (Wikilinks): {}\n- Tags Únicas: {}\n- Estimativa Total de Tokens: {}",
                    stats.total_notes,
                    stats.total_chunks,
                    stats.total_links,
                    stats.total_tags,
                    stats.total_tokens
                );
                Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
            }
            Err(e) => Ok(CallToolResult::error(vec![ContentBlock::text(format!("Erro ao obter estatísticas: {}", e))])),
        }
    }
}

#[tool_handler]
impl ServerHandler for GraphFinderMcpServer {
    fn get_info(&self) -> ServerConfig {
        ServerConfig::new(
            ServerCapabilities::builder().enable_tools().build(),
        )
        .with_server_info(Implementation::new("graphfinder", env!("CARGO_PKG_VERSION")))
        .with_instructions("GraphFinder Neural Knowledge Graph & Local Semantic Vault MCP Server. Permite busca semântica atômica, compressão estrutural de contexto, sugestão de wikilinks e persistência de notas no cofre.".to_string())
    }
}

pub async fn run_mcp_server(
    manager: Arc<VaultManager>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let server = GraphFinderMcpServer::new(manager, None);
    let (read, write) = rmcp::transport::io::stdio();
    server.serve((read, write)).await?;
    Ok(())
}

