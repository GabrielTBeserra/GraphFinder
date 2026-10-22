# GraphFinder MCP Server Tools Specification

All 24 tools described below are **fully implemented and active** using the official Model Context Protocol Rust SDK (`rmcp`).
They operate concurrently in two modes:
1. **Desktop App Embedded Server**: Streamable HTTP/SSE at `http://127.0.0.1:45123/mcp` and `http://127.0.0.1:45123/sse` (with DNS rebinding protection and origin validation).
2. **Headless CLI Stdio Mode**: `GraphFinder.exe --mcp` directly connecting to AI client standard IO (Claude Desktop, Cursor, Antigravity, etc.).

---

## 1. `search_semantic` (Hybrid Vector + BM25 Retrieval via RRF)
* **Purpose**: Surgical knowledge retrieval combining semantic dense embeddings and exact keyword matching with minimal token overhead.
* **Mechanism**: Runs vector cosine similarity and SQLite FTS5 BM25 search over `note_chunks` and blends them with Reciprocal Rank Fusion ($k=60$). Returns top matching subsections with deep links `[[Note#Section]]` and token estimates.
* **Parameters**:
  ```json
  {
    "query": "string (search query, concept, or exact code/symbol keyword)",
    "limit": "integer (optional, default: 5)",
    "format": "dense_chunks | summaries (optional, default: dense_chunks)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 2. `read_section` (Surgical Atomic Chunk Reader)
* **Purpose**: Read precisely the requested subsection without loading full files into LLM context, guaranteeing maximum token economy.
* **Mechanism**: Matches the target note and exact heading or deep link fragment (e.g. `[[Note#2. SQLite WAL]]` or `SQLite WAL`), returning only that chunk.
* **Parameters**:
  ```json
  {
    "target": "string (note title or note id)",
    "section": "string (header H2/H3 or deep link fragment)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 3. `get_compressed_context` (Structural Compression)
* **Purpose**: Inspect the layout, summary, and token map of a note while consuming under 80 tokens.
* **Mechanism**: Returns executive summary, outgoing/incoming links, tags, and an index of all headers with individual token estimates.
* **Parameters**:
  ```json
  {
    "target": "string (note title or note id)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 4. `suggest_links` (Semantic Auto-Linker)
* **Purpose**: Connect ideas and prevent orphaned notes in the knowledge graph.
* **Mechanism**: Analyzes draft text, generates vector embeddings, and calculates cosine similarity against existing notes and sections, returning recommended wikilinks.
* **Parameters**:
  ```json
  {
    "text": "string (draft paragraph or sentence)",
    "current_note": "string (optional, note id to exclude self-reference)",
    "threshold": "number (optional, default: 0.60)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 5. `store_context` (Memory Persistence)
* **Purpose**: Create or update notes in the vault with CommonMark AST validation and automatic vectorization.
* **Mechanism**: Writes the `.md` file, parses headers with `pulldown-cmark`, updates SQLite FTS5 and embeddings.
* **Parameters**:
  ```json
  {
    "title": "string (note title)",
    "content": "string (Markdown formatted body)",
    "tags": ["array of strings (optional)"],
    "links": ["array of note titles to connect via wikilinks (optional)"],
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 6. `rename_note` (Atomic Rename & Graph Refactoring)
* **Purpose**: Rename a note and automatically update all incoming `[[Old Title]]` wikilinks across every document in the vault to prevent broken links.
* **Mechanism**: Renames file on disk, updates SQLite records, and refactors wikilinks in all affected Markdown files.
* **Parameters**:
  ```json
  {
    "old_title": "string (current note title or path)",
    "new_title": "string (new note title)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 7. `delete_note` (Note Excision)
* **Purpose**: Permanently remove a note from disk and clean up all associated vectors and relational links in SQLite.
* **Mechanism**: Deletes the `.md` file and executes atomic deletion in SQLite `notes`, `note_chunks`, `embeddings`, `links`, and `tags`.
* **Parameters**:
  ```json
  {
    "target": "string (note title or note id to delete)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 8. `get_vault_stats` (Vault Integrity & Metrics)
* **Purpose**: Query vault health, total note count, vectorized chunk count, link graph connectivity, and estimated tokens.
* **Mechanism**: Computes aggregated metrics directly from SQLite database.
* **Parameters**:
  ```json
  {
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 9. `get_graph_neighborhood` (Graph Topology Expansion)
* **Purpose**: Explore relational graph structure without reading document bodies.
* **Mechanism**: Returns all adjacent nodes connected to the target within a 1-hop or 2-hop radius.
* **Parameters**:
  ```json
  {
    "target": "string (note title or note id)",
    "depth": "integer (1 or 2, default: 1)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 10. `link_notes` (Edge Creation)
* **Purpose**: Establish an explicit connection between two notes in the vault.
* **Mechanism**: Appends the `[[Target Note]]` wikilink to the origin Markdown file and updates the relational graph table.
* **Parameters**:
  ```json
  {
    "from_note": "string (source note title or id)",
    "to_note": "string (destination note title or id)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 11. `list_recent_notes` (Rapid Inventory)
* **Purpose**: Initial vault reconnaissance for AI agents.
* **Mechanism**: Returns titles, tags, and compact summaries of the most recently modified notes.
* **Parameters**:
  ```json
  {
    "limit": "integer (optional, default: 10)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 12. `sync_vault` (Hash-Based Incremental Indexing)
* **Purpose**: Resynchronize markdown notes and codebase files on disk with the SQLite vector database.
* **Mechanism**: $O(1)$ timestamp and SHA-256 hash comparison; only modified files undergo AST parsing and ONNX embedding re-inference. Automatically associates directory paths with projects.
* **Parameters**:
  ```json
  {
    "path": "string (optional, local folder path to index/update; automatically maps to existing project or registers new one)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 13. `list_projects` (Multi-Vault Discovery)
* **Purpose**: Discover all isolated knowledge projects/vaults configured in GraphFinder.
* **Parameters**: No parameters required.

---

## 14. `create_project` (Isolated Vault Creation)
* **Purpose**: Initialize a new project with its own isolated directory and vector database.
* **Parameters**:
  ```json
  {
    "name": "string (project name)",
    "path": "string (optional absolute directory path)"
  }
  ```

---

## 15. `switch_project` (Vault Switching)
* **Purpose**: Switch the active default vault for subsequent operations.
* **Parameters**:
  ```json
  {
    "project_id": "string (id or slug of target project)"
  }
  ```

---

## 16. `search_code` (Hybrid Vector + BM25 Codebase Retrieval via RRF)
* **Purpose**: Surgical code discovery across all indexed programming languages (Rust, TS, JS, Python, Go, C/C++, Java, Kotlin, C#, PHP, Ruby, Swift, CSS, Shell).
* **Mechanism**: Runs vector cosine similarity and SQLite FTS5 BM25 search over `code_chunks` and merges them via Reciprocal Rank Fusion ($k=60$). Returns top matching functions/structs with deep links (`[[filepath#symbol]]`), line ranges, and token estimates.
* **Parameters**:
  ```json
  {
    "query": "string (concept, feature, symbol name, or exact keyword)",
    "language": "string (optional, e.g. 'rust', 'typescript', 'python')",
    "limit": "integer (optional, default: 5)",
    "project": "string (optional, target specific vault/project id)"
  }
  ```

---

## 17. `find_files` (Fast Pattern / Glob Matching)
* **Purpose**: Rapid file discovery by path pattern or extension without reading file contents.
* **Mechanism**: Matches file paths in SQLite `code_files` using SQL LIKE / glob patterns with case-insensitive filtering.
* **Parameters**:
  ```json
  {
    "pattern": "string (glob pattern or filename substring, e.g. '*.rs', 'engine', 'Button')",
    "limit": "integer (optional, default: 20)",
    "project": "string (optional, target specific vault/project id)"
  }
  ```

---

## 18. `get_file_context` (Structural File Overview & AST Map)
* **Purpose**: Inspect a source file's architecture, declared symbols, and dependency footprint consuming <80 tokens.
* **Mechanism**: Returns file summary, top-level declared symbols with line ranges, local imported files, incoming dependents (who imports this file), and third-party package dependencies.
* **Parameters**:
  ```json
  {
    "file": "string (relative file path or filename)",
    "project": "string (optional, target specific vault/project id)"
  }
  ```

---

## 19. `read_code_chunk` (Surgical Symbol & Function Reader)
* **Purpose**: Read precisely a specific function, class, or struct body without injecting the entire file into the LLM context window.
* **Mechanism**: Retrieves the exact AST-lite chunk matching the target symbol name from `code_chunks` with line numbers and token estimate.
* **Parameters**:
  ```json
  {
    "file": "string (relative file path or filename)",
    "symbol": "string (function, class, struct, or interface name)",
    "project": "string (optional, target specific vault/project id)"
  }
  ```

---

## 20. `get_code_dependencies` (Relational Import & Dependent Graph)
* **Purpose**: Inspect the directional dependency graph around an individual source code file.
* **Mechanism**: Queries `code_links` to return all files imported by this file, external packages, and all project files that import this file (incoming dependents).
* **Parameters**:
  ```json
  {
    "file": "string (relative file path or filename)",
    "project": "string (optional, target specific vault/project id)"
  }
  ```

---

## 21. `find_path` (Multi-Hop Shortest Path Finding)
* **Purpose**: Trace the shortest connection path between any two nodes (notes, code files, or architectural concepts) across the unified knowledge and dependency graph.
* **Mechanism**: Executes a breadth-first search (BFS) over directed wikilinks and code imports/dependencies.
* **Parameters**:
  ```json
  {
    "source": "string (starting note title, file path, or node id)",
    "target": "string (destination note title, file path, or node id)",
    "directed": "boolean (optional, default: false)",
    "project": "string (optional, target specific vault/project id)"
  }
  ```
* **Output**: Returns an ordered sequence of hops (`steps`), edge relationships (`imports`, `wikilink`), direction, and node types.

---

## 22. `explain_node` (Deep Architectural & Topological Context)
* **Purpose**: Provide a complete architectural briefing for any file or note in the graph, explaining *why* it matters and how it fits into the broader system.
* **Mechanism**: Combines degree centrality, community/cluster membership, God Node status, incoming dependents, outgoing dependencies, and semantic siblings into a dense summary.
* **Parameters**:
  ```json
  {
    "target": "string (note title, file path, or node id)",
    "project": "string (optional, target specific vault/project id)"
  }
  ```

---

## 23. `get_community_report` (System-Wide Modular Subsystems & Hubs)
* **Purpose**: Deliver an architectural audit report of the entire repository or vault (similar to `graphify` audits).
* **Mechanism**: Runs Label Propagation (LPA) community detection, degree centrality ranking, God Nodes identification, cross-community architectural surprises, and dependency cycle detection.
* **Parameters**:
  ```json
  {
    "project": "string (optional, target specific vault/project id)"
  }
  ```

---

## 24. `export_graph` (Standalone Artifacts Generation Suite)
* **Purpose**: Export the unified knowledge and dependency graph into standalone, zero-dependency artifacts matching and exceeding `graphify-out/`.
* **Mechanism**: Generates three files on disk in `<out_dir>`:
  1. `graph.json` — Complete graph schema (nodes with communities, degrees, god-node flags, edges with labels and weights).
  2. `GRAPH_REPORT.md` — Full markdown audit report with executive stats, top God Nodes table, community breakdown, cross-coupling surprises, and cycles.
  3. `graph.html` — Zero-dependency interactive HTML5 canvas visualizer with ForceGraph, search filter, community color coding, and hub highlights (runs 100% offline).
* **Parameters**:
  ```json
  {
    "out_dir": "string (optional, default: './graphfinder-out')",
    "project": "string (optional, target specific vault/project id)"
  }
  ```


