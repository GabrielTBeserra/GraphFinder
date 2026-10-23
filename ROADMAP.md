# GraphFinder Architecture & Evolution Roadmap

Este documento estabelece o planejamento técnico e o roadmap detalhado de evolução do **GraphFinder**, quebrado em tarefas atômicas (1 commit por tarefa).

---

## 🚀 Bloco 1: Correção do Startup, Boot Instantâneo & Limpeza

Objetivo: Eliminar a lentidão na abertura do app (carregar projetos, notas e grafo em <50ms direto do SQLite) e remover todas as referências ao Obsidian.

- [x] **Commit 1: Boot instantâneo não-bloqueante no Frontend**
  - Remover bloqueio de tela total da `LoadingScreen` no `src/App.tsx`.
  - Converter o status do `FastEmbed` em indicador sutil não-bloqueante na barra de status/cabeçalho.
  - Carregar UI com o banco local imediatamente enquanto o modelo neural carrega em background.
- [x] **Commit 2: Paralelização e priorização no carregamento inicial (`Promise.allSettled`)**
  - Refatorar `initApp()` e `loadData()` no `src/App.tsx` para buscar dados em paralelo.
  - Priorizar `fetchProjects()` e `fetchNotes()` para que o seletor de vaults/projetos apareça instantaneamente.
- [x] **Commit 3: Eliminação de lock contention de banco no Rust**
  - Em `src-tauri/src/lib.rs`, adiar a sincronização em disco inicial (`sync_vault`) para após as leituras iniciais da UI.
  - Evitar que queries IPC do frontend fiquem presas aguardando o mutex do `DbManager`.
- [x] **Commit 4: Limpeza e remoção de referências ao Obsidian**
  - Limpar menções, comentários e documentação em `src-tauri/src/lib.rs`, docs e READMEs.
  - Focar a documentação e integrações exclusivamente em editores de código e IDEs (VS Code, Cursor, Windsurf, Zed).

---

## ⚡ Bloco 2: Motor de Busca Vetorial & RAG de Alta Precisão

Objetivo: Escalar a busca semântica para dezenas de milhares de chunks com latência sub-milissegundo.

- [x] **Commit 5: Otimização de Busca Vetorial por Pré-filtragem no SQLite**
  - Em `src-tauri/src/engine/db.rs`, filtrar candidatos via FTS5 antes do cálculo de produto escalar em memória.
- [x] **Commit 6: Quantização Escalar de Vetores (f32 -> int8)**
  - Implementar quantização de vetores de 384 dimensões para `int8`, reduzindo uso de memória e acelerando comparações SIMD.
- [x] **Commit 7: Reranker local no RAG (`bge-reranker-base`)**
  - Adicionar suporte a Reranker no FastEmbed para reordenar os top 20 hits das ferramentas MCP (`search_semantic`, `search_code`).

---

## 🧩 Bloco 3: AST Real de Código & Análise Topológica

Objetivo: Evoluir de expressões regulares (AST-lite) para parsing sintático preciso e grafos de chamadas.

- [x] **Commit 8: Migração de Regex para AST em TypeScript/JavaScript**
  - Parser sintático exato para TS/JS, suportando imports multiline, tipos dinâmicos e re-exports.
- [x] **Commit 9: Migração de Regex para AST em Rust e Python**
  - Extração precisa de módulos, `mod`, `use` e `from ... import` para Rust e Python com aliases.
- [ ] **Commit 10: Call Graph inicial a nível de Funções/Métodos**
  - Mapear chamadas entre funções e métodos internos no grafo de código.
- [ ] **Commit 11: Detecção de Acoplamento Temporal via Git (`co-change`)**
  - Identificar arquivos que sofrem commit juntos com frequência para revelar dependências ocultas.

---

## 🎨 Bloco 4: Visualização e UX do Grafo

Objetivo: Melhorar legibilidade de grafos complexos e fornecer ferramentas de auditoria arquitetural.

- [ ] **Commit 12: Filtro de Ego Network (N-Hop Focus) no `GraphView`**
  - Slider dinâmico de profundidade para isolar vizinhanças imediatas de nós selecionados.
- [ ] **Commit 13: Agrupamento visual por Comunidade LPA**
  - Bounding hulls e clusters coloridos no Canvas ao redor de nós da mesma comunidade Louvain/LPA.
- [ ] **Commit 14: Comando CLI de Auditoria Arquitetural para CI/CD**
  - Flag `--fail-on-cycles` e `--max-god-nodes` no `graphfinder audit` para validação em GitHub Actions.

---

## 🔌 Bloco 5: Extensão Oficial do VS Code

Objetivo: Integrar o visualizador de grafos e a inteligência do GraphFinder diretamente no VS Code.

- [ ] **Commit 15: Scaffold do projeto da Extensão VS Code**
  - Criação de `vscode-extension/` com TypeScript, manifesto `package.json` e scripts de build.
- [ ] **Commit 16: Conexão IPC / Cliente do GraphFinder no VS Code**
  - Cliente stdio/HTTP para comunicar com o binário `graphfinder`.
- [ ] **Commit 17: Comandos de Busca Semântica e Pathfinding na Command Palette**
  - Comandos rápidos nativos no VS Code (`QuickPick`).
- [ ] **Commit 18: Webview do Grafo na Sidebar do VS Code**
  - Painel com o Force-Graph ativo e sincronização com o arquivo aberto no editor.
- [ ] **Commit 19: Autocomplete de `[[Wikilinks]]` no Editor Markdown**
  - `CompletionItemProvider` para links e cabeçalhos `[[Nota#Seção]]`.
