# GraphFinder MCP Server Specification

GraphFinder exposes tools adhering to the **Model Context Protocol (JSON-RPC 2.0)** over stdio.

---

## 1. `search_semantic` (Atomic Chunk Retrieval)
Primary retrieval tool engineered for minimal token consumption.

* **Parameters**:
  - `query` (string, required): Search query or natural language question.
  - `limit` (number, optional): Maximum chunks returned (default: 5).
  - `project` (string, optional): Target specific vault project id.
* **Returns**:
  List of ranked sections with `[[Note#Section]]` deep links, cosine affinity scores, and chunk token estimates.

---

## 2. `get_compressed_context` (Structural Compression)
Returns an ultra-compact representation of a note without transferring full text.

* **Parameters**:
  - `target` (string, required): Note title or note identifier.
  - `project` (string, optional): Target specific vault project id.
* **Returns**:
  - Executive summary (~30 tokens).
  - Incoming backlinks and outgoing graph edges.
  - Comprehensive index of H2/H3 headers with token counts.

---

## 3. `suggest_links` (Semantic Auto-Linker)
Identifies notes and sections in the vault sharing high topical correlation with provided draft text.

* **Parameters**:
  - `text` (string, required): Draft paragraph or sentence.
  - `threshold` (number, optional): Minimum cosine similarity cutoff (e.g., 0.65).
  - `project` (string, optional): Target specific vault project id.
* **Returns**:
  List of recommended wikilinks (e.g., `[[System Architecture#1. SQLite]]` with 81% affinity).

---

## 4. `store_context` (Memory Persistence)
Creates a new note or updates an existing one with immediate semantic reindexing.

* **Parameters**:
  - `title` (string, required): Note title (maps to `.md` filename).
  - `content` (string, required): Markdown content structured with `##` headers.
  - `tags` (string array, optional): Categorical hashtags.
  - `project` (string, optional): Target specific vault project id.
* **Returns**:
  Write status and total count of indexed atomic chunks.

---

## 5. `get_graph_neighborhood` (Graph Expansion)
Explores relational topology around a target node.

* **Parameters**:
  - `target` (string, required): Center node identifier.
  - `depth` (number, optional): Traversal hop radius (default: 1).
  - `project` (string, optional): Target specific vault project id.
* **Returns**:
  Connected nodes, directional edges, and associated tags.

---

## 6. `link_notes` (Edge Creation)
Establishes a bidirectional relationship between two notes, inserting the wikilink into the source Markdown file.

* **Parameters**:
  - `from_note` (string, required): Source note title or id.
  - `to_note` (string, required): Target note title or id.
  - `project` (string, optional): Target specific vault project id.

---

## 7. `search_code` (Hybrid Code Retrieval)
Surgical code search across all repository programming languages using RRF (FastEmbed vectors + BM25).

* **Parameters**:
  - `query` (string, required): Search query, function name, or conceptual keyword.
  - `language` (string, optional): Filter by language (e.g. `rust`, `typescript`, `python`).
  - `limit` (number, optional): Maximum chunks returned (default: 5).
  - `project` (string, optional): Target specific project vault.
* **Returns**:
  List of matching code chunks with deep links `[[filepath#symbol]]`, line ranges, and token estimates.

---

## 8. `find_files` (Fast Pattern / Glob Matching)
Locates source code files quickly by name or glob pattern.

* **Parameters**:
  - `pattern` (string, required): Glob or substring (e.g., `*.rs`, `engine`, `App.tsx`).
  - `limit` (number, optional): Maximum files returned (default: 20).
  - `project` (string, optional): Target specific project vault.
* **Returns**:
  List of matching relative file paths.

---

## 9. `get_file_context` (Structural File Overview)
Returns a high-density summary (<80 tokens) of an individual source file.

* **Parameters**:
  - `file` (string, required): Relative file path or filename.
  - `project` (string, optional): Target specific project vault.
* **Returns**:
  Language, token estimate, exported symbols with line ranges, local imported files, dependents, and third-party packages.

---

## 10. `read_code_chunk` (Atomic Symbol Retrieval)
Retrieves the body of a specific function, class, or struct without reading full files.

* **Parameters**:
  - `file` (string, required): Relative file path.
  - `symbol` (string, required): Name of function, struct, or class.
  - `project` (string, optional): Target specific project vault.
* **Returns**:
  Surgical code chunk with start/end lines and token estimate.

---

## 11. `get_code_dependencies` (Relational Code Graph)
Navigates the directional dependency graph around an individual file.

* **Parameters**:
  - `file` (string, required): Relative file path.
  - `project` (string, optional): Target specific project vault.
* **Returns**:
  Internal imports, third-party packages, and incoming dependents (files that import this file).

