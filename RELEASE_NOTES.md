# Release Notes — GraphFinder v0.4.0

**Release Date:** October 2026  
**Target Milestone:** Multi-Language Codebase Indexing, AST Dependency Graph, 20-Tool MCP Suite & Code Inspector

---

## Executive Summary

GraphFinder **v0.4.0** expands beyond Markdown notes to index **entire codebases across any programming language**, building a bidirectional module dependency graph and arming AI agents with surgical code retrieval tools.

Key highlights include:
1. **Multi-Language AST-Lite Engine**: Scans 15+ languages (Rust, TypeScript, TSX, JavaScript, Python, Go, C/C++, Java, Kotlin, C#, PHP, Ruby, Swift, CSS, Shell). Extracts module imports (`import`, `require`, `use`, `include`, `mod`) and maps dependencies into SQLite `code_links`.
2. **20-Tool Unified MCP Suite**: Added 5 dedicated code retrieval tools (`search_code`, `find_files`, `get_file_context`, `read_code_chunk`, `get_code_dependencies`) preserving the **<80 token rule** for structural discovery.
3. **Cybernetic Visual Graph Canvas**: Renders code files with language-specific palette rings (TypeScript `#38bdf8`, Rust `#f97316`, Python `#10b981`, Go `#06b6d4`, etc.) and a 1-click filter bar: `[All]`, `[Notes]`, `[Code]`.
4. **Interactive Code Inspector Drawer (`CodeInspector.tsx`)**: Slide-out drawer with tabs for declared symbols (functions, structs, classes) with line numbers, interactive dependency jumps, and surgical code viewer.
5. **Incremental SHA-256 Code Syncing**: Filesystem scanning skips unchanged files and strictly respects `.gitignore` (ignoring `node_modules`, `target`, `dist`, `.git`, binaries, and files >1MB).
6. **Unified RRF Hybrid Search**: FastEmbed ONNX vectors combined with SQLite FTS5 BM25 search over code symbols, comments, and signatures.

---

# Release Notes — GraphFinder v0.3.0

**Release Date:** October 2026  
**Target Milestone:** Security Hardening, Hybrid Search, AST CommonMark Engine, Graph Stability & 15-Tool MCP Suite

---

## Executive Summary

GraphFinder **v0.3.0** represents a comprehensive architectural upgrade focused on **security hardening**, **retrieval precision**, **UI/canvas rendering stability**, and **expanded AI agent autonomy**.

Key highlights include:
1. **Zero-Flicker Canvas & UI Performance**: Solved the canvas teardown issue where the screen flashed blank during note editing, note selection, or vault sync.
2. **Hybrid Retrieval (Vectors + BM25 FTS5 via RRF)**: Combines dense semantic embeddings with SQLite FTS5 keyword indexing for superior code, symbol, and multilingual search.
3. **CommonMark AST Parser (`pulldown-cmark`)**: Replaced regex heuristics with full CommonMark AST parsing, section windowing for large texts, and exclusion of code blocks.
4. **Security & Sandboxing**: Implemented DNS rebinding mitigation, strict Origin filtering, CSP enforcement, and XSS sanitization (`DOMPurify`).
5. **Background File Watcher & Instant Sync**: Sub-second debounce synchronization with external editors (Obsidian, VS Code).
6. **Vault-Wide Wikilink Refactoring**: Renaming a note automatically rewrites all incoming `[[Old Title]]` references across every document in the vault.
7. **Expanded 15-Tool MCP Suite**: Official `rmcp` dual-mode support (Embedded Streamable HTTP/SSE + Headless Stdio CLI `--mcp`).

---

## 1. UI Rendering Performance & Flicker Elimination

### Root Cause
Previously, whenever a note was selected, saved, or updated in the background, `GraphView` suffered from canvas teardowns:
* The ForceGraph canvas initialization hook had dynamic dependencies on link topologies, invoking `_destructor()` and tearing down the HTML5 `<canvas>` element from the DOM on every state change.
* When re-rendering, nodes were passed as bare object literals without retaining `(x, y, vx, vy)` coordinates, causing the physics engine to reset and nodes to collapse to the center before exploding outwards.
* In `App.tsx`, a dynamic `key` on `NoteEditor` unmounted and remounted the entire editor panel upon saving drafts.

### Resolved Behavior
* **Permanent Canvas Instance**: `ForceGraph` is now strictly initialized once on component mount (`[]`). The canvas is never destroyed or detached from the DOM during runtime.
* **Physics Coordinate Preservation**: Existing node coordinates `(x, y, vx, vy)` and momentum are merged during graph data updates, preventing canvas blinks and simulation resets.
* **Structural Topological Hashing**: Graph topology changes are checked before touching the simulation. If nodes and links remain identical, data re-submission is bypassed entirely.
* **Smooth 60 FPS Node Selection**: Selecting nodes in the sidebar or clicking links updates node focus highlights inside the continuous animation loop without reloading graph data.
* **Stable Editor Component**: Removed the unmounting `key` from `NoteEditor`, preventing UI flashes while typing and saving.

---

## 2. Security Hardening & Sandboxing (P0)

* **DNS Rebinding & Host Validation**: Implemented strict security middleware in [`src-tauri/src/mcp/server.rs`](file:///Z:/GraphFinder/src-tauri/src/mcp/server.rs). Validates that incoming HTTP `Host` headers strictly resolve to local loopback addresses (`localhost`, `127.0.0.1`, `[::1]`).
* **Origin Filtering**: Restricts cross-origin HTTP and SSE requests exclusively to authorized desktop/local origins (`localhost`, `127.0.0.1`, `tauri://`, `vscode-webview://`), mitigating unauthorized browser-based intranet attacks.
* **Security Headers**: Injected `X-Content-Type-Options: nosniff` and `X-Frame-Options: DENY` on all HTTP responses.
* **Content Security Policy (CSP)**: Replaced `"csp": null` in `tauri.conf.json` with a strict CSP policy (`default-src 'self' ...`).
* **DOMPurify Markdown Preview Sanitization**: Installed and configured `DOMPurify` in [`src/components/NoteEditor.tsx`](file:///Z:/GraphFinder/src/components/NoteEditor.tsx), allowing only designated wikilink span tags (`<span class="wikilink-btn" data-target="...">`) and blocking arbitrary inline scripts/iframes.
* **Path Traversal Protection**: Added `sanitize_project_id` to enforce slugified alphanumeric project identifiers and block `../` directory traversal attacks.
* **Panic Removal**: Replaced runtime panics (`.expect()`) in the engine with graceful `Result` and fallback logic.

---

## 3. Parser & Retrieval Architecture (P1)

* **CommonMark AST Engine (`pulldown-cmark`)**: Rewrote [`src-tauri/src/engine/parser.rs`](file:///Z:/GraphFinder/src-tauri/src/engine/parser.rs) using an authentic CommonMark AST parser.
  - Code blocks (` ``` `) and inline code are strictly ignored when extracting headers, tags, and wikilinks.
  - `#tag` lines and code comments (`# comment`) are no longer misidentified as `# Title` H1 headings.
* **Oversized Section Windowing**: Sections exceeding `MAX_CHUNK_TOKENS` (320 tokens) are split into consecutive sub-chunks with a 40-token semantic overlap, preserving vector resolution without context boundary clipping.
* **Hybrid Search with Reciprocal Rank Fusion (RRF)**:
  - Created SQLite virtual table `chunks_fts` with FTS5 and `unicode61 remove_diacritics 2` tokenizer.
  - Blends dense vector similarity and BM25 full-text keyword ranking using Reciprocal Rank Fusion ($k=60$):
    $$RRF = \frac{1.0}{60 + r_{vector}} + \frac{1.0}{60 + r_{fts}}$$
  - Queries with exact programming identifiers, technical terms, or symbols now match with high precision.
* **Multilingual Embedding Support**: Added support for `MultilingualE5Small` alongside offline `AllMiniLML6V2`, configurable via the `GRAPHFINDER_MODEL` environment variable.
* **Parser Toolset**: Added `rewrite_wikilinks`, `replace_title`, `find_unlinked_mention`, `link_first_mention`, `compose_note`, and `content_hash`.

---

## 4. Vault Integrity & File Watcher (P1)

* **Incremental Hash Indexing ($O(1)$)**: Added `content_hash` (SHA-256) and filesystem timestamps to the `notes` table. Files that have not changed skip ONNX vector re-computation on vault sync.
* **Background Directory Watcher**: Integrated `notify-debouncer-mini` (500ms debounce) in [`src-tauri/src/lib.rs`](file:///Z:/GraphFinder/src-tauri/src/lib.rs). External modifications from Obsidian, VS Code, or Git trigger automatic reindexing and emit `vault-sync-complete`.
* **Duplicate Event Filtering**: The file watcher checks if any note was actually modified (`count > 0`) before emitting `vault-sync-complete`, preventing redundant UI refresh loops.
* **Opt-In Auto-Linking**: Disabled automated note file mutations by default. Appending semantic link sections to Markdown files is now opt-in via `GRAPHFINDER_AUTO_APPEND_LINKS=1`.
* **Vault-Wide Wikilink Refactoring (`rename_note`)**:
  - Renames the physical file on disk.
  - Updates SQLite primary keys, chunk records, and links.
  - Scans and rewrites all incoming `[[Old Title]]` references across all `.md` files in the vault to prevent broken links.
* **Atomic Deletion (`delete_note`)**: Removes the `.md` file from disk and deletes corresponding entries from `notes`, `note_chunks`, `embeddings`, `links`, and `tags`.

---

## 5. Model Context Protocol (MCP) Suite — 15 Tools

The native MCP server ([`src-tauri/src/mcp/mod.rs`](file:///Z:/GraphFinder/src-tauri/src/mcp/mod.rs)) now implements **15 tools** with dual-mode support:

### Execution Modes
1. **Embedded Streamable HTTP/SSE**: Active at `http://127.0.0.1:45123/mcp` and `http://127.0.0.1:45123/sse` while the desktop app is open.
2. **Headless Stdio CLI Mode**: `GraphFinder.exe --mcp` running headless standard IO for terminal agents, Claude Desktop, Cursor, and Antigravity.

### Complete Tool Reference

| # | Tool | Purpose | Key Parameters |
|---|---|---|---|
| 1 | `search_semantic` | Hybrid Dense Vector + BM25 FTS5 search with RRF | `query`, `limit`, `format`, `project` |
| 2 | `read_section` | Surgical retrieval of an exact H2/H3 chunk by heading or deep link | `target`, `section`, `project` |
| 3 | `get_compressed_context` | Executive summary, backlinks, and section token map (< 80 tokens) | `target`, `project` |
| 4 | `suggest_links` | Recommends conceptual wikilinks for draft paragraphs | `text`, `threshold`, `project` |
| 5 | `store_context` | Creates/updates Markdown note with AST vectorization | `title`, `content`, `tags`, `links`, `project` |
| 6 | `rename_note` | Renames note and refactors all incoming wikilinks across vault | `old_title`, `new_title`, `project` |
| 7 | `delete_note` | Deletes note file and cleans SQLite vectors | `target`, `project` |
| 8 | `get_vault_stats` | Inspects note count, chunks, links, tags, and tokens | `project` |
| 9 | `get_graph_neighborhood` | Topological exploration of adjacent nodes (1 or 2 hops) | `target`, `depth`, `project` |
| 10 | `link_notes` | Creates bidirectional wikilinks between notes | `from_note`, `to_note`, `project` |
| 11 | `list_recent_notes` | Rapid inventory of notes with single-line summaries | `limit`, `project` |
| 12 | `sync_vault` | Hash-based incremental sync between filesystem and SQLite | `project` |
| 13 | `list_projects` | Lists all configured isolated knowledge vaults | None |
| 14 | `create_project` | Provisions a new isolated vault at any path on disk | `name`, `path` |
| 15 | `switch_project` | Switches the active default vault | `project_id` |

---

## 6. Engineering, CI & Quality Assurance

* **Unit Testing Suite**: 15 unit tests covering:
  - FTS5 query sanitization and escape logic.
  - FastEmbed cosine similarity math.
  - Stable SHA-256 content hashing.
  - Title replacement variants.
  - Wikilink preservation with section fragments and aliases.
  - In-memory SQLite CRUD, foreign keys, and FTS5 search.
  - YAML frontmatter extraction.
  - Hex numbers vs hashtag distinction.
  - Code block isolation from metadata parser.
  - Section windowing on large texts.
  - Unlinked mention detection.
* **CI Automation**: Updated `.github/workflows/ci.yml` to include `cargo test --verbose`.
* **Zero Regressions**:
  - `cargo test --manifest-path src-tauri/Cargo.toml`: **15 passed; 0 failed**
  - `npm run build`: TypeScript `tsc -b` and Vite production build completed with **0 errors**.
