# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-10-21

### Added
* Hybrid Search combining dense vector similarity and SQLite FTS5 (BM25) via Reciprocal Rank Fusion (RRF, $k=60$).
* CommonMark AST parser using `pulldown-cmark = 0.13` with code block isolation and tag line handling.
* Oversized section windowing (320 tokens with 40-token semantic overlap).
* Complete 15-tool MCP suite with `read_section`, `rename_note`, `delete_note`, `get_vault_stats`, `sync_vault`, `create_project`, and `switch_project`.
* Dual-mode MCP support: Embedded Streamable HTTP/SSE (`http://127.0.0.1:45123/mcp`) + Headless Stdio CLI (`GraphFinder.exe --mcp`).
* Automatic background file watcher with 500ms debounce (`notify-debouncer-mini`) for external editor synchronization.
* Vault-wide wikilink refactoring when renaming notes (`rename_note`).
* Multilingual embedding support (`GRAPHFINDER_MODEL=MultilingualE5Small` or offline `AllMiniLML6V2`).
* 15 comprehensive unit tests across parser, SQLite FTS5, and embeddings math.

### Changed
* Zero-Flicker Canvas: permanently anchored `ForceGraph` instance on mount, eliminating canvas teardowns and white/blank flashes.
* Preserved node simulation physics `(x, y, vx, vy)` and momentum across graph data reloads.
* Topologically guarded graph data updates: structural hashing bypasses simulation re-computation when nodes/links have not changed.
* Smooth 60 FPS node selection highlighting without graph data re-submission.
* Removed disruptive unmounting `key` on `NoteEditor` to maintain smooth note editing and saving.

### Security
* Replaced permissive CORS with strict local security middleware validating `Host` (DNS rebinding defense) and `Origin`.
* Enforced strict Content Security Policy (CSP) in `tauri.conf.json`.
* Markdown preview sanitization using `DOMPurify` to prevent XSS.
* Project path traversal protection via `sanitize_project_id`.

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
