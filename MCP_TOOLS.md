# GraphFinder MCP Server Tools Specification

All tools described below are **fully implemented and active** using the official Model Context Protocol Rust SDK (`rmcp`) within the GraphFinder embedded HTTP/SSE server (`http://127.0.0.1:45123/mcp` and `http://127.0.0.1:45123/sse`) and when started in headless mode with `GraphFinder.exe --mcp`.

---

## 1. `search_semantic` (Atomic Chunk Retrieval)
* **Purpose**: Surgical knowledge retrieval with minimal token overhead.
* **Mechanism**: Queries the `note_chunks` SQLite table to return the top 5 most relevant subsections (2 to 4 lines each), specifying the deep link `[[Note#Section]]` and token count estimate.
* **Parameters**:
  ```json
  {
    "query": "string (search query or natural language question)",
    "limit": "integer (optional, default: 5)",
    "format": "dense_chunks | summaries (optional, default: dense_chunks)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 2. `get_compressed_context` (Structural Compression)
* **Purpose**: Inspect the full layout and structure of a note while consuming under 100 tokens.
* **Mechanism**: Returns a 2-line executive summary, incoming and outgoing graph links, and a complete index of all H2/H3 headers with individual token estimates.
* **Parameters**:
  ```json
  {
    "target": "string (note title or note id)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 3. `suggest_links` (Semantic Auto-Linker)
* **Purpose**: Connect ideas and prevent orphaned notes within the graph.
* **Mechanism**: Analyzes draft text in real time, generates a vector embedding, and calculates cosine similarity against all vault notes and chunks, returning recommended wikilink targets with affinity scores.
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

## 4. `store_context` (Memory Persistence)
* **Purpose**: Create or update notes in the vault directly from an AI agent.
* **Mechanism**: Writes the `.md` file to the vault directory, splits text by `##` and `###` headers, computes offline ONNX embeddings, and indexes them into the `note_chunks` table.
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

## 5. `get_graph_neighborhood` (Graph Topology Expansion)
* **Purpose**: Explore relational graph structure without reading document bodies.
* **Mechanism**: Returns all adjacent nodes connected to the target within a 1-hop or 2-hop radius.
* **Parameters**:
  ```json
  {
    "target": "string (note title or note id)",
    "depth": "integer (1 or 2)",
    "project": "string (optional, target specific vault id)"
  }
  ```

---

## 6. `link_notes` (Edge Creation)
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

## 7. `list_recent_notes` (Rapid Inventory)
* **Purpose**: Initial vault reconnaissance for AI agents.
* **Mechanism**: Returns titles, tags, and compact summaries of the most recently modified notes.
* **Parameters**:
  ```json
  {
    "limit": "integer (optional, default: 10)",
    "project": "string (optional, target specific vault id)"
  }
  ```
