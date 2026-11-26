# GraphFinder Server — Checklist de implementação por etapas e commits

Este documento é o plano operacional para construir o **GraphFinder Server** como uma variante aditiva do projeto, sem alterar o comportamento do GraphFinder desktop atual.

## 0. Contrato de execução

### Regras obrigatórias

- [ ] Não modificar `src/`, `src-tauri/`, workflows existentes ou o pipeline de release desktop nesta etapa.
- [ ] Todo código novo da versão servidor deve ficar dentro de `server/`.
- [ ] O servidor não pode depender de Tauri, WebKit, X11, xvfb ou interface gráfica.
- [ ] Cada commit deve deixar a branch em estado compilável ou explicitamente documentar o contrário antes do commit.
- [ ] Nenhum endpoint fica público sem autenticação, TLS ou proxy reverso.
- [ ] Tokens nunca são persistidos em plaintext.
- [ ] Repositórios indexados por Git são tratados como entrada não confiável.
- [ ] O índice publicado deve ser consistente durante buscas concorrentes; nenhuma consulta pode ler uma geração parcial.

### Decisões-base assumidas

- [ ] Banco principal: PostgreSQL 16 + pgvector.
- [ ] Fila inicial: PostgreSQL com `SELECT ... FOR UPDATE SKIP LOCKED`.
- [ ] Redis/Valkey: somente quando houver necessidade real de cache, rate limit distribuído ou pub/sub.
- [ ] Object storage/S3: somente quando volume local não for suficiente.
- [ ] Modelo inicial de embeddings: `multilingual-e5-small`, CPU por padrão.
- [ ] Projetos podem ter fontes Git readonly; vault gerenciado/editável entra em fase posterior.
- [ ] Multi-tenancy é modelado desde o início, mesmo que o primeiro deploy tenha um único tenant.
- [ ] MCP inicial é readonly/search-first; ferramentas de escrita só entram depois de auth/RBAC estarem prontos.

### Não-objetivos iniciais

- [ ] Não migrar o desktop para PostgreSQL.
- [ ] Não remover SQLite do produto desktop.
- [ ] Não adicionar Neo4j, Elasticsearch ou Qdrant no MVP.
- [ ] Não implementar multi-tenancy com banco separado por tenant no MVP.
- [ ] Não executar código, scripts ou hooks de repositórios indexados.
- [ ] Não expor PostgreSQL, Redis ou painel interno diretamente na rede pública.

---

## 1. Estrutura alvo

```text
server/
├── Cargo.toml
├── crates/
│   ├── gf-core/
│   ├── gf-storage/
│   ├── gf-indexer/
│   ├── gf-mcp/
│   └── gf-server/
├── apps/
│   └── admin/
├── migrations/
├── fixtures/
└── deploy/
    ├── Dockerfile.server
    ├── Dockerfile.admin
    ├── docker-compose.yml
    └── Caddyfile
```

### Comandos do binário servidor

```text
graphfinder-server migrate
graphfinder-server api
graphfinder-server worker
```

---

## 2. Sequência de commits planejada

Cada item abaixo representa um commit ou um pequeno grupo de commits coesos.

### Fase 0 — Planejamento e fundação

#### Commit 0 — planejamento

- [x] Criar branch `feature/server`.
- [x] Adicionar este checklist em `server/IMPLEMENTATION_CHECKLIST.md`.
- [ ] Commit sugerido:
  ```text
  docs(server): add implementation checklist
  ```

**Critério de aceite**
- [ ] Checklist versionado na branch.
- [ ] Nenhum arquivo de `src/` ou `src-tauri/` alterado.

#### Commit 1 — workspace independente

- [x] Criar `server/Cargo.toml` como workspace independente.
- [x] Criar crates vazios: `gf-core`, `gf-storage`, `gf-indexer`, `gf-mcp`, `gf-server`.
- [x] Criar binário inicial `graphfinder-server` com subcomandos `migrate`, `api` e `worker` ainda retornando “not implemented”.
- [x] Adicionar `server/README.md` com escopo e comandos básicos.
- [ ] Commit sugerido:
  ```text
  chore(server): initialize independent rust workspace
  ```

**Critério de aceite**
- [x] `cargo check --manifest-path server/Cargo.toml` passa.
- [x] Nenhuma dependência de Tauri foi adicionada.

#### Commit 2 — fixtures e testes de compatibilidade

- [x] Criar `server/fixtures/` com exemplos Markdown, TypeScript, Rust, Python e repositório misto.
- [x] Criar testes de chunking esperado para headings `##`/`###`.
- [x] Criar testes de deep links no formato `[[Nota#Seção]]` e `[[arquivo#simbolo]]`.
- [ ] Commit sugerido:
  ```text
  test(server): add parser compatibility fixtures
  ```

**Critério de aceite**
- [ ] Fixtures cobrem Markdown, código e casos com links/tags.

---

### Fase 1 — Core de parsing, chunking e embeddings

#### Commit 3 — parser Markdown

- [x] Implementar parser CommonMark em `gf-core`.
- [x] Extrair headings, seções, wikilinks e tags.
- [x] Gerar chunks semânticos por `##` e `###`.
- [x] Implementar estimativa de tokens.
- [ ] Commit sugerido:
  ```text
  feat(server-core): implement markdown parsing and chunking
  ```

**Critério de aceite**
- [x] Fixtures Markdown passam.
- [x] Chunks preservam heading, deep link, conteúdo e estimativa de tokens.

#### Commit 4 — parser de código

- [x] Implementar detecção de linguagens suportadas no MVP: Rust, TypeScript, JavaScript, Python, Go e Markdown/documentação.
- [x] Extrair imports/includes/requires.
- [x] Extrair símbolos principais: functions, classes, structs, impls/métodos quando possível.
- [x] Preservar path, linguagem, linha inicial/final e nome do símbolo.
- [ ] Commit sugerido:
  ```text
  feat(server-core): implement source code parsing
  ```

**Critério de aceite**
- [x] Fixtures de código passam.
- [x] Nenhum arquivo é executado durante parsing.

#### Commit 5 — grafo interno

- [x] Criar representação de nós e links para notas, arquivos e símbolos.
- [x] Implementar adjacências para wikilinks e imports.
- [x] Implementar consultas básicas de neighborhood/path em memória ou via storage.
- [ ] Commit sugerido:
  ```text
  feat(server-core): implement graph model
  ```

**Critério de aceite**
- [x] É possível construir grafo de fixture e encontrar caminho simples entre dois nós.

#### Commit 6 — embeddings

- [x] Implementar abstração `EmbeddingProvider`.
- [x] Implementar provider FastEmbed CPU.
- [x] Carregar modelo `multilingual-e5-small` por cache local configurável.
- [x] Implementar batching com limite configurável.
- [ ] Implementar timeout e tratamento de erro.
- [ ] Commit sugerido:
  ```text
  feat(server-core): add local embedding provider
  ```

**Critério de aceite**
- [x] Teste gera embedding de dimensão esperada.
- [x] Servidor não depende de API externa para embeddings.

---

### Fase 2 — PostgreSQL, schema e gerações de índice

#### Commit 7 — migrations base

- [x] Adicionar migrations SQL para tenants, users, projects, memberships e api_tokens.
- [x] Adicionar índice único para slugs/emails quando aplicável.
- [x] Usar hashes de senha/token, nunca plaintext.
- [ ] Commit sugerido:
  ```text
  feat(server-db): add identity and authorization schema
  ```

**Critério de aceite**
- [x] `graphfinder-server migrate` aplica migrations em PostgreSQL limpo. *(validado via migrator em PostgreSQL/pgvector real no Ubuntu)*

#### Commit 8 — schema de conteúdo

- [x] Adicionar tabelas `sources`, `documents`, `chunks`, `chunk_embeddings`, `links`, `index_jobs` e `index_generations`.
- [x] Habilitar extensão `vector`.
- [x] Criar índices por `project_id`, `generation`, `source_id` e `content_hash`.
- [ ] Commit sugerido:
  ```text
  feat(server-db): add content and vector index schema
  ```

**Critério de aceite**
- [x] pgvector está disponível.
- [x] Schema suporta múltiplas gerações por projeto.

#### Commit 9 — repositories de storage

- [x] Implementar acesso a projetos, fontes, documentos, chunks e links em `gf-storage`.
- [x] Implementar queries com `sqlx` ou ORM escolhido.
- [x] Implementar testes de integração com PostgreSQL em Docker quando disponível. *(teste ignorado criado; Docker indisponível neste ambiente)*
- [ ] Commit sugerido:
  ```text
  feat(server-db): implement storage repositories
  ```

**Critério de aceite**
- [x] CRUD interno de projeto/fonte/documento/chunk funciona.

#### Commit 10 — publicação atômica de gerações

- [x] Implementar criação de geração `pending`.
- [x] Implementar escrita de documentos/chunks/embeddings por geração.
- [x] Implementar publicação atômica em `projects.active_generation`.
- [x] Implementar limpeza de gerações antigas.
- [ ] Commit sugerido:
  ```text
  feat(server-db): implement atomic index generation publishing
  ```

**Critério de aceite**
- [x] Busca nunca lê geração `pending`.
- [x] Rollback manual para geração anterior é possível por operação administrativa interna.

#### Commit 11 — busca híbrida

- [x] Implementar busca lexical PostgreSQL FTS.
- [x] Implementar busca vetorial pgvector com filtro por projeto/geração.
- [x] Implementar fusão/Ranking inicial, preferencialmente RRF.
- [x] Implementar filtros por linguagem e tipo de documento.
- [ ] Commit sugerido:
  ```text
  feat(server-db): implement hybrid lexical and vector search
  ```

**Critério de aceite**
- [x] Consultas retornam apenas projeto autorizado e geração ativa.
- [x] Busca lexical e vetorial produzem resultados consistentes em fixtures indexados.

---

### Fase 3 — Indexação Git e workers

#### Commit 12 — validação segura de fontes Git

- [x] Validar esquema `https` inicialmente; `ssh` pode entrar depois.
- [x] Bloquear IPs privados, loopback e link-local por padrão.
- [x] Criar configuração explícita para permitir hosts internos.
- [x] Não permitir credenciais embutidas na URL.
- [ ] Commit sugerido:
  ```text
  feat(server-indexer): add safe git source validation
  ```

**Critério de aceite**
- [x] URLs internas são rejeitadas por padrão.
- [x] URLs com credenciais embutidas são rejeitadas.

#### Commit 13 — clone/fetch seguro

- [x] Implementar clone shallow/fetch usando CLI `git` sem shell interpolation ou `git2` com credenciais controladas.
- [x] Resolver e armazenar commit SHA exato.
- [x] Implementar cache de repositório por fonte.
- [x] Implementar limite de tamanho e timeout.
- [ ] Commit sugerido:
  ```text
  feat(server-indexer): implement safe git clone and fetch
  ```

**Critério de aceite**
- [x] Clone ocorre em diretório isolado.
- [x] Nenhum hook/script do repositório é executado.
- [x] Logs não contêm credenciais.

#### Commit 14 — scan e incremental hashing

- [x] Implementar scan respeitando include/exclude.
- [x] Ignorar diretórios pesados por padrão: `.git`, `node_modules`, `target`, `dist`, `build`, `.venv`.
- [x] Calcular hash por arquivo e por chunk.
- [x] Implementar índice incremental baseado em hash.
- [ ] Commit sugerido:
  ```text
  feat(server-indexer): implement incremental repository scanning
  ```

**Critério de aceite**
- [ ] Reindexação sem mudanças não reprocessa todos os chunks. *(será validado no pipeline completo)*
- [ ] Arquivos removidos desaparecem da nova geração. *(será validado no pipeline completo)*

#### Commit 15 — pipeline completo de indexação

- [x] Integrar clone → scan → parse → chunk → embeddings → staging → publish.
- [x] Registrar métricas de arquivos, chunks, erros e duração.
- [x] Implementar modo dry-run para validar configuração da fonte.
- [ ] Commit sugerido:
  ```text
  feat(server-indexer): implement end-to-end indexing pipeline
  ```

**Critério de aceite**
- [x] Fixture completo é indexado no PostgreSQL.
- [ ] Segunda execução usa caminho incremental. *(diff_scan existe; falta reutilizar embeddings/chunks inalterados)*

#### Commit 16 — fila de jobs no PostgreSQL

- [x] Implementar estados `queued`, `running`, `succeeded`, `failed`, `cancelled`.
- [x] Implementar aquisição com `FOR UPDATE SKIP LOCKED`.
- [x] Implementar lock por `project_id + source_id`.
- [x] Implementar retry com backoff simples.
- [x] Implementar heartbeat/timeout de worker.
- [ ] Commit sugerido:
  ```text
  feat(server-indexer): implement postgres-backed job queue
  ```

**Critério de aceite**
- [x] Dois workers não processam o mesmo job.
- [x] Duas execuções simultâneas da mesma fonte são impedidas.

#### Commit 17 — logs e progresso de jobs

- [x] Persistir progresso percentual e etapa atual.
- [ ] Persistir erros estruturados por arquivo/fase. *(schema criado; falta granularidade por arquivo)*
- [x] Redigir segredos de URLs e logs.
- [ ] Commit sugerido:
  ```text
  feat(server-indexer): add job progress and redacted logs
  ```

**Critério de aceite**
- [ ] Painel/API consegue acompanhar job sem ler logs de processo. *(pendente da API/painel)*

---

### Fase 4 — API REST e autenticação

#### Commit 18 — bootstrap da API

- [ ] Criar servidor HTTP com Axum em `gf-server`.
- [ ] Adicionar `/health` e `/ready`.
- [ ] Adicionar middleware de request id, logging e métricas básicas.
- [ ] Implementar configuração por variáveis de ambiente.
- [ ] Commit sugerido:
  ```text
  feat(server-api): bootstrap http api server
  ```

**Critério de aceite**
- [ ] API inicia sem GUI e responde `/health`.

#### Commit 19 — autenticação de usuários

- [ ] Implementar criação inicial de usuário admin por variável de ambiente ou comando bootstrap.
- [ ] Implementar login com senha hash Argon2/bcrypt.
- [ ] Implementar sessões seguras ou JWT curto para painel.
- [ ] Implementar logout/revogação de sessão.
- [ ] Commit sugerido:
  ```text
  feat(server-auth): implement user authentication
  ```

**Critério de aceite**
- [ ] Painel pode autenticar usuário admin inicial.
- [ ] Senha nunca é persistida em plaintext.

#### Commit 20 — tokens PAT e service tokens

- [ ] Implementar criação de tokens com formato prefixado.
- [ ] Persistir somente hash.
- [ ] Implementar scopes, projetos permitidos e expiração.
- [ ] Implementar revogação e `last_used_at`.
- [ ] Commit sugerido:
  ```text
  feat(server-auth): implement scoped api tokens
  ```

**Critério de aceite**
- [ ] Token completo é exibido apenas uma vez.
- [ ] Token expirado/revogado é rejeitado.

#### Commit 21 — RBAC e autorização

- [ ] Implementar roles de tenant e projeto.
- [ ] Implementar middleware de autorização.
- [ ] Validar acesso por projeto em todas as rotas.
- [ ] Implementar escopo `search:read`, `code:read`, `project:write`, `index:trigger`, `tokens:manage`.
- [ ] Commit sugerido:
  ```text
  feat(server-auth): enforce rbac and project acl
  ```

**Critério de aceite**
- [ ] Usuário sem acesso ao projeto recebe 403/404 consistente.
- [ ] Token com escopo restrito não executa operação fora do escopo.

#### Commit 22 — endpoints REST de projetos e fontes

- [ ] `GET/POST /api/v1/projects`
- [ ] `GET/PATCH/DELETE /api/v1/projects/{id}`
- [ ] `GET/POST /api/v1/projects/{id}/sources`
- [ ] `GET/PATCH/DELETE /api/v1/sources/{id}`
- [ ] Commit sugerido:
  ```text
  feat(server-api): add project and source endpoints
  ```

**Critério de aceite**
- [ ] CRUD de projeto/fonte respeita RBAC.

#### Commit 23 — endpoints REST de jobs e busca

- [ ] `POST /api/v1/projects/{id}/index-jobs`
- [ ] `GET /api/v1/projects/{id}/index-jobs`
- [ ] `GET /api/v1/index-jobs/{id}`
- [ ] `POST /api/v1/index-jobs/{id}/cancel`
- [ ] `POST /api/v1/search`
- [ ] Commit sugerido:
  ```text
  feat(server-api): add indexing and search endpoints
  ```

**Critério de aceite**
- [ ] É possível cadastrar fonte, disparar indexação e consultar resultados via REST.

---

### Fase 5 — MCP Server multiusuário

#### Commit 24 — transporte MCP HTTP

- [ ] Implementar endpoint `/mcp` compatível com Streamable HTTP.
- [ ] Implementar initialize, tools/list e tools/call.
- [ ] Associar cada sessão/chamada ao principal autenticado.
- [ ] Commit sugerido:
  ```text
  feat(server-mcp): implement authenticated streamable http transport
  ```

**Critério de aceite**
- [ ] Cliente MCP consegue listar ferramentas usando Bearer token.

#### Commit 25 — ferramentas readonly principais

- [ ] Implementar `list_projects`.
- [ ] Implementar `search_semantic`.
- [ ] Implementar `search_code`.
- [ ] Implementar `find_files`.
- [ ] Implementar `get_file_context`.
- [ ] Implementar `read_code_chunk`.
- [ ] Commit sugerido:
  ```text
  feat(server-mcp): add readonly project and code tools
  ```

**Critério de aceite**
- [ ] Ferramentas retornam apenas projetos autorizados.
- [ ] Resultados seguem formato compacto orientado a agentes.

#### Commit 26 — ferramentas de contexto e grafo

- [ ] Implementar `get_compressed_context`.
- [ ] Implementar `get_code_dependencies`.
- [ ] Implementar `get_graph_neighborhood`.
- [ ] Implementar `find_path`.
- [ ] Implementar `explain_node`.
- [ ] Commit sugerido:
  ```text
  feat(server-mcp): add context and graph tools
  ```

**Critério de aceite**
- [ ] Consultas de grafo usam a geração ativa consistente.

#### Commit 27 — ferramentas administrativas MCP

- [ ] Implementar `trigger_index`.
- [ ] Implementar `get_index_status`.
- [ ] Implementar `create_project` apenas com scope adequado.
- [ ] Definir compatibilidade ou alias para `sync_vault` como trigger de indexação.
- [ ] Commit sugerido:
  ```text
  feat(server-mcp): add administrative indexing tools
  ```

**Critério de aceite**
- [ ] Agentes autorizados podem disparar reindexação sem acesso administrativo completo.

---

### Fase 6 — Painel administrativo

#### Commit 28 — bootstrap do painel

- [ ] Criar `server/apps/admin` com React + TypeScript + Vite.
- [ ] Criar layout base com navegação.
- [ ] Implementar cliente API com tratamento de autenticação.
- [ ] Commit sugerido:
  ```text
  feat(server-admin): bootstrap admin web application
  ```

**Critério de aceite**
- [ ] `npm run build` em `server/apps/admin` passa.

#### Commit 29 — login e sessão

- [ ] Implementar tela de login.
- [ ] Implementar logout.
- [ ] Implementar guarda de rotas autenticadas.
- [ ] Commit sugerido:
  ```text
  feat(server-admin): add authentication flow
  ```

**Critério de aceite**
- [ ] Usuário autenticado acessa painel; não autenticado é redirecionado.

#### Commit 30 — projetos e fontes

- [ ] Listar projetos.
- [ ] Criar/editar projeto.
- [ ] Adicionar fonte Git com validação básica.
- [ ] Exibir último commit, status e geração ativa.
- [ ] Commit sugerido:
  ```text
  feat(server-admin): add project and git source management
  ```

**Critério de aceite**
- [ ] Usuário consegue configurar um projeto completo pelo painel.

#### Commit 31 — jobs de indexação

- [ ] Listar jobs por projeto.
- [ ] Exibir progresso, etapa, erros e duração.
- [ ] Implementar retry/cancel quando permitido.
- [ ] Implementar polling simples ou SSE.
- [ ] Commit sugerido:
  ```text
  feat(server-admin): add indexing job monitoring
  ```

**Critério de aceite**
- [ ] Operador acompanha indexação sem acessar logs do container.

#### Commit 32 — tokens e usuários

- [ ] Criar/listar/revogar tokens.
- [ ] Exibir escopo, projetos, expiração e último uso.
- [ ] Gerar snippet MCP pronto para copiar.
- [ ] Gerenciar usuários e papéis básicos.
- [ ] Commit sugerido:
  ```text
  feat(server-admin): add token and user management
  ```

**Critério de aceite**
- [ ] Token criado no painel funciona em cliente MCP.

#### Commit 33 — busca de teste

- [ ] Implementar página para testar busca semântica/código por projeto.
- [ ] Exibir deep links, scores e trechos.
- [ ] Commit sugerido:
  ```text
  feat(server-admin): add search inspection page
  ```

**Critério de aceite**
- [ ] Operador valida indexação sem depender de agente externo.

---

### Fase 7 — Docker, deploy e operação

#### Commit 34 — Dockerfile do servidor

- [ ] Criar build multi-stage enxuto.
- [ ] Incluir `git` no runtime apenas se o worker usar CLI Git.
- [ ] Executar como usuário não-root.
- [ ] Preparar diretório de cache de modelos/repositórios.
- [ ] Commit sugerido:
  ```text
  build(server): add production server dockerfile
  ```

**Critério de aceite**
- [ ] `docker build` da imagem servidor passa.
- [ ] Imagem executa `migrate`, `api` e `worker`.

#### Commit 35 — Dockerfile do painel

- [ ] Criar build multi-stage do frontend.
- [ ] Servir assets estaticamente via imagem leve ou pelo proxy.
- [ ] Commit sugerido:
  ```text
  build(server): add admin web dockerfile
  ```

**Critério de aceite**
- [ ] Painel é servido sem depender de Node em runtime.

#### Commit 36 — Docker Compose

- [ ] Adicionar `server/deploy/docker-compose.yml`.
- [ ] Configurar PostgreSQL + pgvector, api, worker, admin e proxy.
- [ ] Adicionar volumes persistentes.
- [ ] Adicionar healthchecks.
- [ ] Expor publicamente apenas proxy.
- [ ] Commit sugerido:
  ```text
  build(server): add docker compose deployment
  ```

**Critério de aceite**
- [ ] `docker compose config` é válido.
- [ ] Ambiente sobe com migrations, API, worker e painel.

#### Commit 37 — proxy e TLS

- [ ] Adicionar `Caddyfile` ou configuração equivalente.
- [ ] Rotear `/api` para API, `/mcp` para API e `/` para painel.
- [ ] Configurar headers de segurança.
- [ ] Configurar timeouts adequados para MCP/streaming.
- [ ] Commit sugerido:
  ```text
  build(server): add reverse proxy configuration
  ```

**Critério de aceite**
- [ ] Painel, API e MCP funcionam pelo mesmo domínio.

#### Commit 38 — observabilidade

- [ ] Implementar logs estruturados JSON.
- [ ] Implementar métricas básicas: requests, jobs, duração, falhas, chunks, embeddings.
- [ ] Adicionar correlation/request ids.
- [ ] Implementar `/metrics` interno opcional.
- [ ] Commit sugerido:
  ```text
  feat(server-ops): add structured observability
  ```

**Critério de aceite**
- [ ] Logs não contêm tokens, senhas ou credenciais Git.

#### Commit 39 — backup e restore

- [ ] Documentar backup PostgreSQL.
- [ ] Adicionar script ou procedimento de `pg_dump`.
- [ ] Definir política para cache repositório/modelos como artefatos reconstruíveis.
- [ ] Testar restore em ambiente limpo.
- [ ] Commit sugerido:
  ```text
  docs(server): add backup and restore procedures
  ```

**Critério de aceite**
- [ ] Restore recria ambiente funcional a partir de backup.

---

### Fase 8 — Vault gerenciado e escrita controlada

#### Commit 40 — modelo de vault gerenciado

- [ ] Adicionar fonte tipo `managed_vault`.
- [ ] Implementar documentos editáveis com revision/optimistic concurrency.
- [ ] Indexar alterações em nova geração ou incrementalmente.
- [ ] Commit sugerido:
  ```text
  feat(server-vault): add managed vault source type
  ```

**Critério de aceite**
- [ ] Fontes Git permanecem readonly.
- [ ] Edições concorrentes são detectadas.

#### Commit 41 — ferramentas MCP de escrita

- [ ] Implementar `store_context`.
- [ ] Implementar `update_section`.
- [ ] Implementar `link_notes`.
- [ ] Implementar `delete_note` com scope específico.
- [ ] Commit sugerido:
  ```text
  feat(server-mcp): add managed vault write tools
  ```

**Critério de aceite**
- [ ] Escrita exige scope explícito.
- [ ] Repositórios Git readonly não podem ser modificados por essas ferramentas.

#### Commit 42 — auditoria de mutações

- [ ] Registrar autor, token, ação, recurso e diff/resumo.
- [ ] Exibir eventos no painel.
- [ ] Commit sugerido:
  ```text
  feat(server-auth): add mutation audit trail
  ```

**Critério de aceite**
- [ ] Toda mutação possui evento auditável.

---

### Fase 9 — Recursos avançados

#### Commit 43 — reindexação agendada

- [ ] Implementar schedule por fonte/projeto.
- [ ] Implementar política de deduplicação.
- [ ] Commit sugerido:
  ```text
  feat(server-indexer): add scheduled indexing
  ```

#### Commit 44 — webhooks Git

- [ ] Implementar endpoint assinado para webhooks.
- [ ] Validar assinatura por provider.
- [ ] Criar job incremental após push.
- [ ] Commit sugerido:
  ```text
  feat(server-indexer): add git webhook indexing
  ```

#### Commit 45 — rollback operacional de índice

- [ ] Expor rollback para geração anterior no painel.
- [ ] Implementar proteção contra rollback acidental.
- [ ] Commit sugerido:
  ```text
  feat(server-admin): add index generation rollback
  ```

#### Commit 46 — Redis/cache quando necessário

- [ ] Introduzir Redis/Valkey somente após medir necessidade.
- [ ] Implementar cache de metadados e invalidação por geração.
- [ ] Implementar rate limiting distribuído se necessário.
- [ ] Commit sugerido:
  ```text
  feat(server-ops): add redis-backed caching and rate limiting
  ```

#### Commit 47 — OIDC/SSO

- [ ] Implementar login OIDC para humanos.
- [ ] Mapear grupos/claims para roles quando aplicável.
- [ ] Manter PATs para MCP/agentes.
- [ ] Commit sugerido:
  ```text
  feat(server-auth): add oidc single sign-on
  ```

---

## 3. Quality gates por commit

Antes de cada commit Rust:

```bash
cargo fmt --manifest-path server/Cargo.toml --check
cargo clippy --manifest-path server/Cargo.toml --workspace -- -D warnings
cargo test --manifest-path server/Cargo.toml --workspace
```

Antes de commits do painel:

```bash
cd server/apps/admin
npm ci
npm run lint
npm run build
```

Antes de commits de deploy:

```bash
cd server/deploy
docker compose config
```

---

## 4. Critérios de aceite por fase

### Fase 1 concluída

- [ ] Parser Markdown/code gera chunks e links compatíveis em fixtures.
- [ ] Embeddings locais funcionam em CPU.

### Fase 2 concluída

- [ ] PostgreSQL armazena projetos, chunks e embeddings.
- [ ] Busca híbrida respeita projeto e geração ativa.
- [ ] Publicação de índice é atômica.

### Fase 3 concluída

- [ ] Uma URL Git pública pode ser indexada de ponta a ponta.
- [ ] Jobs concorrentes não corrompem estado nem duplicam trabalho.
- [ ] Logs são redigidos e progresso é persistido.

### Fase 4 concluída

- [ ] Usuários e tokens têm autenticação e autorização funcionais.
- [ ] CRUD de projetos/fontes e disparo de jobs funcionam via REST.

### Fase 5 concluída

- [ ] Cliente MCP externo conecta em `/mcp` com Bearer token.
- [ ] Ferramentas readonly retornam resultados apenas de projetos autorizados.

### Fase 6 concluída

- [ ] Operador administra projetos, fontes, jobs, usuários e tokens pelo painel.
- [ ] Operador consegue testar busca sem usar cliente MCP externo.

### Fase 7 concluída

- [ ] Docker Compose sobe ambiente completo.
- [ ] Somente proxy está exposto.
- [ ] Backup/restore documentado e testado.

### Fase 8 concluída

- [ ] Vault gerenciado permite escrita segura via MCP.
- [ ] Mutações são auditáveis.

### Fase 9 concluída

- [ ] Indexação pode ser contínua por schedule/webhook.
- [ ] Operação pode escalar com workers adicionais e cache quando necessário.

---

## 5. Demonstração final do MVP

O MVP pode ser considerado funcional quando este fluxo passar de ponta a ponta:

- [ ] Subir `docker compose`.
- [ ] Criar usuário admin.
- [ ] Criar projeto.
- [ ] Cadastrar repositório Git público.
- [ ] Disparar indexação.
- [ ] Ver job concluir com sucesso.
- [ ] Criar token MCP com escopo readonly.
- [ ] Configurar Cursor/Claude/outro cliente com URL única do servidor.
- [ ] Executar `list_projects`.
- [ ] Executar `search_code` e `search_semantic`.
- [ ] Confirmar que resultados retornam somente do projeto autorizado.
- [ ] Reindexar após commit novo no repositório.
- [ ] Confirmar que a nova geração foi publicada sem downtime.
