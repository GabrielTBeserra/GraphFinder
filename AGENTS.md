# GraphFinder: Knowledge, Writing, and MCP Guidelines

This document establishes the mandatory rules for AI agents, editors, and contributors interacting with or managing the **GraphFinder** knowledge vault. The primary goals are **maximal semantic density**, **rigorous token economy**, and **high graph connectivity**.

---

## 1. Core Architectural Principles

1. **Local-First & Plain Markdown**: All notes live in `.md` files within the `./vault` directory (or designated project directory). They must remain 100% human-readable, formatted in standard Markdown with bidirectional wikilinks `[[Link]]`.
2. **Atomic Semantic Chunking**: The FastEmbed/SQLite engine indexes notes by splitting content strictly across headers (`##` and `###`). Every header generates an independent vector embedding in the `note_chunks` database table.
3. **Token Conservation**: AI agents must **NEVER** request or inject full Markdown files into an LLM context window unless explicitly instructed by the user. Always prefer atomic excerpts via deep links (`[[Note#Section]]`).

---

## 2. Note Writing and Structural Standards

Every note created or modified in the vault MUST strictly adhere to the following schema:

### A. Title and Executive Summary
* The note must begin with a single H1 header (`# Note Title`).
* The first paragraph immediately following the H1 must be a high-density executive summary (2 to 3 lines maximum), synthesizing the document's core purpose.

### B. Header Granularity (`##` and `###`)
* Divide the body into self-contained subsections of **40 to 150 tokens** each.
* Use semantic, descriptive titles for headers (e.g., `## 2. SQLite WAL Configuration` instead of simply `## Configuration`).
* The search engine uses these titles to construct precise deep links: `[[Note#2. SQLite WAL Configuration]]`.

### C. Mandatory Bidirectional Connections (Wikilinks)
* Connect terminology and concepts to related notes using `[[Note Title]]` or `[[Note Title#Section]]`.
* Every note should maintain at least **1 to 3 connections** to other nodes in the graph to prevent orphaned knowledge.
* Verify potential connections using the `suggest_links` MCP tool prior to saving.

### D. Tag Taxonomy
* Append context hashtags `#tag` at the end of sections or at the bottom of the document.
* Use lowercase terms (e.g., `#architecture`, `#sqlite`, `#rag`, `#mcp`).
* The GraphFinder frontend utilizes these tags to compute and render colored semantic graph clusters.

---

## 3. AI Agent MCP Operation Protocol

When operating as an AI agent connected to the GraphFinder MCP server, follow this decision pipeline to minimize context consumption:

```
[User Question]
       │
       ▼
1. Execute `search_semantic(query)`
   - Returns top 5 relevant chunks (~50 tokens each).
       │
       ├─► Answer found? ──► Respond citing deep links [[Note#Section]].
       │
       └─► Need structural overview? ──► Execute `get_compressed_context(note_id)`
                                          (Summary + section map with token estimates).
```

### Golden Rules for MCP Agents:
1. **Never read raw Markdown files unnecessarily**: Use `search_semantic` first. It extracts the exact subsection containing the required knowledge.
2. **When saving new knowledge (`store_context`)**:
   - Provide a clear, descriptive title.
   - Format content cleanly with `##` headers.
   - Include explicit wikilinks to notes consulted earlier in the session.
   - Supply a relevant list of lowercase `tags`.
3. **When evaluating links (`suggest_links`)**:
   - Pass draft paragraphs before saving to identify semantic affinity with existing vault notes.

---

## 4. Multi-Vault and Multi-Project Support

GraphFinder supports isolated workspaces (`./vaults/<project_id>`), each with its own Markdown directory and independent SQLite vector database:

* `list_projects()`: Lists all configured projects, file paths, and note counts.
* `create_project(name)`: Initializes a new isolated vault at `./vaults/<slug>/` with an introductory note.
* `switch_project(project_id)`: Toggles the active default vault.
* **Direct Vault Targeting**: All querying and mutation tools (`search_semantic`, `store_context`, `get_compressed_context`, `get_graph_neighborhood`, `suggest_links`, `link_notes`, `search_code`, `find_files`, `get_file_context`, `read_code_chunk`, `get_code_dependencies`) accept an optional `project: "<id>"` parameter, enabling multi-vault management without cross-project contamination.

---

## 5. Multi-Language Codebase Indexing & AST-Lite Graph

GraphFinder indexes entire software repositories across multiple programming languages (Rust, TypeScript, JavaScript, Python, Go, C/C++, Java, Kotlin, C#, PHP, Ruby, Swift, CSS, Shell). It parses import statements (`import`, `require`, `use`, `include`, `mod`, etc.), constructs an internal dependency graph, and creates atomic symbol embeddings for functions, classes, and structs.

### Code Retrieval Decision Pipeline for AI Agents:
```
[Need to find or inspect code]
       │
       ├─► Looking for a concept/feature? ──► `search_code(query, language?)`
       │                                       (RRF hybrid search across symbols and comments).
       │
       ├─► Looking for a file path? ────────► `find_files(pattern)`
       │                                       (Fast pattern/glob matching).
       │
       ├─► Need structural summary of file? ─► `get_file_context(file)`
       │                                       (Imports, dependents, external pkgs, symbols).
       │
       ├─► Need a specific function/symbol? ─► `read_code_chunk(file, symbol)`
       │                                       (Surgical snippet without reading full file).
       │
       └─► Analyzing architectural impact? ──► `get_code_dependencies(file)`
                                               (Who imports this file & what does it import?).
```

### Golden Rules for Code Context:
1. **Never read whole source code files into context** unless explicitly asked. Use `get_file_context` to see the symbol map, then `read_code_chunk` to fetch only the required function or method.
2. **Cite deep links**: Reference code locations using `[[filepath#symbol]]` or line ranges.

