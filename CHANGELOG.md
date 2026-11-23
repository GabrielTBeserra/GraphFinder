# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.22.0] - 2026-11-21

### Changed
* **Compact MCP output (~26% fewer response tokens)**: all 26 tools now emit a uniform compact grammar — `N. [[link]] 73% ~60t` search hits, `L42-80` line ranges, comma-joined lists, `+more: offset=N` pagination, one-line confirmations for mutations, and `errors(N):`/`warnings(N):` audit reports. Deep links, `offset=`, `fields='ids'` and `read_code_chunk` hints are preserved as stable anchors.
* **Shorter tool/field descriptions**: descriptions rewritten in a "returns X; use when Y; prefer Z" pattern (field schema descriptions -37%), and server `instructions` became a compact decision pipeline. `mcp-tools/` artifacts regenerated — now including `update_section` and `validate_vault`, which were missing from the exported JSONs.
* **Compact output grammar documented + golden tests**: the format is now spec'd at the top of `mcp/mod.rs` and covered by 8 unit tests (`fmt_semantic_hit`, `fmt_code_hit` snapshots) so future refactors cannot silently break agents that parse it.

## [0.21.0] - 2026-11-21

### Fixed
* **Window revealed only after ~15s on every launch**: `boot.js` calls `window.show()` right after the boot splash paints, but `core:window:allow-show` was never granted in capabilities — the call was rejected and the window stayed hidden until the fallback timer. The window now appears immediately after first paint.
* **Neural model re-downloaded (~470MB) per launch directory**: fastembed resolved its `.fastembed_cache` relative to the current working directory, scattering cache folders and re-downloading `multilingual-e5-small` on every new cwd — failing outright on network drives (`Failed to retrieve onnx/model.onnx`). The cache is now pinned to the resolved base dir for the embedder (including the CPU-retry path) and the reranker.

## [0.20.0] - 2026-11-21

### Fixed
* **Inverted graph canvas on theme switch**: `data-theme` is now applied in `useLayoutEffect` — GraphView's passive effect no longer reads a stale `--gf-canvas` (also fixes wrong canvas color on boot when light theme was saved).

## [0.19.0] - 2026-11-21

### Added
* **Single running instance across GUI + CLI**: exclusive `graphfinder.lock` (fs2) + `instance.json` ownership metadata. `tauri-plugin-single-instance` focuses the existing window on a second GUI launch; CLI commands delegate to the running GUI via `POST /api/cli` (`sync`, `status`, `search`, `projects`, `path`, `explain`, `export`, `report`) so delegated syncs emit `vault-sync-complete`/`projects-changed` and the UI refreshes instantly.
* **System tray + Windows autostart**: native tray menu (Open / Sync now / Quit), close-to-tray, `tauri-plugin-autostart` Run-key registration, start-minimized flag — all configurable in the new Settings modal persisted to `settings.json`.
* **Graph UX**: directional arrows on `imports` edges, link curvature, particles on focused/path edges, drag-to-pin nodes, minimap with live viewport.
* **MCP stdio→HTTP bridge**: `graphfinder --mcp` delegates to the running instance's Streamable HTTP endpoint (`127.0.0.1:45123/mcp`) — no duplicated ONNX model, single DB writer, unified telemetry. Standalone fallback preserved.
* **MCP retrieval economy**: `offset` pagination with "more results" markers, `max_chars` output budget with explicit truncation marker, `fields=ids` two-stage retrieval, `queries[]` batch mode, `diversify()` (max 2 hits per note/file + Jaccard near-dup removal).
* **MCP operations**: `GRAPHFINDER_MCP_PROFILE=core` trims the tool schema 24→10 at session init; `/api/mcp-activity` reports per-tool count/avg/p50/p95 latency and `latest_mutation_ms` for cheap change polling; `graphfinder config` emits a native streamable-HTTP `mcpServers` entry.

### Changed
* Dedicated SQLite read connection — queries never block behind sync writes (WAL).
* Bounded 512-entry query-embedding cache — repeated agent queries skip ONNX entirely.
* Recency boost on hybrid scores (+15%, ~21-day half-life); FTS5 `token*` prefix-expansion fallback when literal MATCH yields zero hits.
* `VaultManager::reload_meta()` + merge-on-save for `projects.json` — concurrent CLI/GUI writers no longer clobber each other.
* Release builds now ship Windows + Linux only (macOS target dropped).
* Panel polish: inspector, sidebar, tabs and modal cleanup.

## [0.18.0] - 2026-11-19

### Added
* **One-click agent kit installer** + hamburger vault menu.
* **Animated boot splash** in `index.html`; window stays hidden until first paint (no blank frames).
* **Studio layout**: activity bar, document tabs, bottom panel, status bar — chrome-free canvas like VS Code.
* **Paper & Oxide theme**: terracotta accent, warm neutrals; editorial/engine-room redesign with token deslop, lazy graph and a11y pass.

### Changed
* Masthead removed; update banner strip dropped — update badge now lives on the activity-bar gear.

## [0.17.0] - 2026-11-09

### Changed
* **Sync throughput**: Markdown and code files parsed in parallel with rayon; `vault-structure-ready` event lets the UI render the graph before embeddings finish; embed depth scales down automatically on huge syncs.
* **DB batch queries**: call graph and code relations resolved in batch instead of per-row queries.
* **Graph rendering**: LOD rendering, physics tuning and density caps for huge graphs; community hulls cached; per-frame string churn eliminated.

## [0.16.0] - 2026-11-05

### Added
* **Automatic GPU/CPU execution providers**: ONNX Runtime now registers platform GPU EPs — DirectML on Windows (Intel iGPU, NVIDIA, AMD), CoreML on macOS — with silent per-operator fallback to CPU. No device selection needed on any hardware. `GRAPHFINDER_DEVICE=auto|cpu|dml|coreml|cuda|rocm` forces a backend; `gpu-cuda`/`gpu-rocm` remain opt-in cargo features (they swap the downloaded ORT binary flavor). Same chain applied to the bge-reranker session.
* **`GRAPHFINDER_BATCH`**: embedding batch size per ONNX call (default 16) with `yield_now()` between spawn_blocking chunks — keeps the UI responsive while fastembed saturates all physical cores during indexing.
* **`provider` field in `engine-status` events**: reports the requested chain (e.g. `auto(dml->cpu)`) for support/debugging.
* **MCP code file counts** exposed in file listings; code retrieval hardened.

### Fixed
* `tauri.localhost` origin accepted by the local MCP HTTP security middleware (Windows webviews).

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
