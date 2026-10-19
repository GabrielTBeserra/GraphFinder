# Changelog

Todas as mudancas notaveis neste projeto serao documentadas neste arquivo.

O formato e baseado em [Keep a Changelog](https://keepachangelog.com/pt-BR/1.1.0/) e este projeto adere ao [Semantic Versioning](https://semver.org/lang/pt-BR/).

---

## [0.2.0] - 2026-10-19

### Adicionado
* Suporte nativo a Multi-Vaults isolados com bancos SQLite vetoriais independentes por projeto.
* Servidor MCP integrado no executavel principal via argumento CLI `--mcp`.
* Implementacao das 7 ferramentas MCP atômicas: `search_semantic`, `get_compressed_context`, `suggest_links`, `store_context`, `get_graph_neighborhood`, `link_notes` e `list_recent_notes`.
* Parametro opcional `project` em todas as ferramentas MCP para operacoes em vaults secundarios sem contaminacao cruzada.
* Modal interativo de exportacao de configuracao MCP, regras AGENTS.md e skill de curadoria.
* Componente de alternancia rapida de vaults na barra superior (`ProjectSwitcher`).
* Modal de criacao de novos projetos e vaults (`NewProjectModal`).
* Identidade visual minimalista em SVG vetorial sem efeitos neon.

### Modificado
* Refatoracao completa do design system para estetica minimalista fosca (paleta OLED Charcoal, remocao de halos de neon e sombras fluorescentes).
* Renderizacao do grafo 2D atualizada com aneis concentricos precisos e paleta sobria de tags.
* Motor vetorial em Rust com SQLite em modo WAL para consultas concorrentes de alta velocidade.

---

## [0.1.0] - 2026-10-01

### Adicionado
* Versao inicial com suporte a cofre unico em `./vault`.
* Chunking semantico atomico baseado em cabecalhos H2 e H3.
* Motor FastEmbed local via ONNX Runtime.
* Visualizacao interativa com física de forcas via Force-Graph Canvas.
* Editor de Markdown com suporte a wikilinks `[[Link]]` e preview em tempo real.
