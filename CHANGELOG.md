# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.15.0] - 2026-01-XX

### Added
* **Embedding migration to `multilingual-e5-small`** (384d, multilingual PT/EN): automatic vault re-embed driven by a model stamp in the DB meta table — no manual index.db rebuild. E5 `query:`/`passage:` prefixes applied at search vs index time. `GRAPHFINDER_MODEL=minilm` keeps the legacy model; local `models/multilingual-e5-small/` dirs auto-detected.
* **26 MCP tools** (was 24): `update_section` (surgical `##`/`###` section replacement, heading preserved) and `validate_vault` (AGENTS.md schema audit: H1+summary, 40-150 token sections, broken wikilinks, orphan notes, missing tags).
* **MCP tool annotations**: read-only/destructive/idempotent/open-world hints on all 26 tools for safer client auto-approval.
* **MCP prompts**: `onboard_project`, `answer_question`, `investigate_code`, `write_note`, `audit_vault` — encoding the AGENTS.md pipelines as slash commands.
* **MCP resources**: `graphfinder://instructions`, `//stats`, `//note/{id}`, `//code/{path}` + URI templates.
* **MCP telemetry**: JSON-RPC call log ring buffer (method, tool, status, latency) exposed via `GET /api/tools` and `GET /api/mcp-activity`; rebuilt agent hub modal with live tool registry, annotation badges and activity feed.
* **Fuzzy "did you mean"** suggestions on failed note/node lookups (bigram-Jaccard) + `read_section` lists available headings on misses.
* **Omnisearch (Ctrl+K)**: local fuzzy scorer + semantic chunks + code search (debounced), grouped results, recent items, actions (new note, sync, theme, filters, re-layout, Alt+Enter reveal-in-graph).
* **Redesigned sidebar**: pinned notes (localStorage), A-Z/recency sort, expandable tag/language chips, reveal-in-graph hover, semantic results strip.
* **Toolbar tooltips** (PT-BR) on every graph control; `Tip` CSS-only component.
* **Dark-first design tokens**, self-hosted Geist/JetBrains Mono/Instrument Serif fonts (removed Google Fonts CDN — fully offline).
* **Code-splitting**: modals + NoteEditor + CodeInspector lazy-loaded; `getLanguageStyle` extracted to shared module.

### Changed
* Graph renderer restored to force-graph with `autoPauseRedraw` on-demand repaints (idle CPU fix) and a bounded repaint loop for the MCP glow animation.
* Sidebar search input removed — global Ctrl+K omnisearch is the single search entry point.
* Auto-link threshold retuned 0.65 → 0.70; `suggest_links` default 0.60 → 0.65 (E5 cosine distribution).
* Code file list cap raised 200 → 5000 (sidebar count now matches the graph).

### Fixed
* Embedding-model detection stamps and migrates automatically on the next sync after any model swap.

---

## [0.4.0] - 2026-10-22

### Added
* Multi-Language Codebase Indexing: Scans and indexes entire software repositories across 15+ languages (Rust, TypeScript, TSX, JavaScript, Python, Go, C/C++, Java, Kotlin, C#, PHP, Ruby, Swift, CSS, Shell).
* AST-Lite Module Import Parser & Relational Graph: Extracts module imports (`import`, `require`, `use`, `include`, `mod`) and builds bidirectional dependency edges in SQLite `code_links`.
* 5 New Surgical MCP Code Tools (expanding suite to 20 tools):
  - `search_code`: Hybrid vector (FastEmbed ONNX) + BM25 keyword search across functions, structs, and symbols.
  - `find_files`: Fast glob pattern matching across indexed codebase files.
  - `get_file_context`: High-density file summary (<80 tokens) with symbol list, line numbers, local imports, dependents, and external packages.
  - `read_code_chunk`: Surgical retrieval of individual symbol/function bodies.
  - `get_code_dependencies`: Relational dependency graph navigation (imports & dependents).
* Interactive Code Inspector Drawer (`CodeInspector.tsx`): Slide-out drawer with tabs for declared symbols, import graph navigation with clickable jumps, and surgical code viewer.
* Cybernetic Graph Canvas Visualization: Code files rendered with language-coded concentric rings (TypeScript `#38bdf8`, Rust `#f97316`, Python `#10b981`, Go `#06b6d4`, etc.).
* Graph Filter Controls: Instant 1-click filtering between `[All]`, `[Notes]`, and `[Code]`.
* Sidebar Code Navigation: Dedicated `Code` tab with language filters and toggle in semantic search between Markdown notes and code symbols.
* Incremental SHA-256 Code Syncing: Filesystem scanning skips unchanged files and strictly respects `.gitignore` (ignoring `node_modules`, `target`, `dist`, `.git`, binaries, and files >1MB).
* 6 New Unit Tests covering language detection, AST-lite import extraction (Rust, JS/TS, Python), path resolution, and SQLite code search (21/21 tests passing).

---

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
