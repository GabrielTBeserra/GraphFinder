# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.2.0] - 2026-10-19

### Added
* Native multi-vault support with isolated SQLite databases and independent vector embeddings per project.
* Embedded MCP server compiled into the primary desktop binary via the `--mcp` CLI argument.
* 7 atomic MCP tools: `search_semantic`, `get_compressed_context`, `suggest_links`, `store_context`, `get_graph_neighborhood`, `link_notes`, and `list_recent_notes`.
* Optional `project` parameter across all MCP tools for zero-cross-contamination multi-vault operations.
* Interactive modal for exporting MCP configuration snippets, agent rules, and curator skills (`IntegrationModal`).
* Top bar project switcher (`ProjectSwitcher`) and new project creation modal (`NewProjectModal`).
* Custom file dialog for selecting arbitrary directories on the filesystem as vaults.
* Minimalist matte SVG visual identity without neon glows or fluorescent shadows.

### Changed
* Complete design system refactor adopting a Swiss minimal devtool aesthetic (monochrome OLED Charcoal, technical borders, Phosphor icons, zero emojis).
* 2D force-directed graph renderer optimized with subtle concentric orbital rings and low-saturation tag clusters.
* Rust vector engine utilizing SQLite in WAL mode for high-throughput concurrent indexing and retrieval.

---

## [0.1.0] - 2026-10-01

### Added
* Initial release with single-vault architecture targeting `./vault`.
* Atomic semantic chunking splitting Markdown documents strictly by `##` and `###` headers.
* Local FastEmbed vector engine powered by ONNX Runtime.
* Interactive 2D force-directed graph canvas for bidirectional wikilink visualization.
* Markdown editor supporting `[[Note]]` wikilinks and real-time preview.
