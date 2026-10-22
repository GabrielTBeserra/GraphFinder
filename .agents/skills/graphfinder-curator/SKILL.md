---
name: graphfinder-curator
description: >-
  Guides authoring, knowledge curation, semantic linking, and high-performance MCP retrieval in GraphFinder.
  Use when creating or updating markdown notes in the vault, querying the knowledge graph, configuring MCP tools,
  or optimizing token efficiency for AI agents.
---

# GraphFinder Curator Skill

This skill guides AI agents and human editors in managing the **GraphFinder** knowledge vault, ensuring maximal token efficiency, relational graph coherence, and atomic semantic chunking across sections.

---

## When to Use This Skill
- When drafting or structuring new notes in `./vault` or isolated project vaults (`./vaults/<project>`).
- When toggling between workspaces with `list_projects`, `switch_project`, or creating new ones with `create_project`.
- When communicating with GraphFinder via the embedded MCP server (stdio).
- When executing semantic searches while preserving LLM context budget.
- When generating or applying bidirectional graph links.

---

## Read and Query Workflow (Cost-Effective Retrieval)

When requested to answer or retrieve context from stored knowledge:

1. **Step 1: Atomic Semantic Search**
   - Execute the MCP tool `search_semantic(query)`.
   - Review returned chunks, cosine affinity scores, and estimated token counts (~50–100 tokens per chunk).
   - Identify precise deep links formatted as `[[Note#Section]]`.

2. **Step 2: Neighborhood Exploration (Optional)**
   - If a chunk references related concepts or adjacent context is required, call `get_graph_neighborhood(note_id, depth: 1)`.
   - This exposes incoming backlinks and outgoing links without retrieving full document bodies.

3. **Step 3: Structural Compression (If Needed)**
   - To inspect a note's complete outline without loading full text, call `get_compressed_context(note_id)`.
   - Returns a 2-line executive summary and a list of all H2/H3 sections with respective token counts.

---

## Write and Curation Workflow (Persisting Knowledge)

When authoring or updating vault notes:

### 1. Structural Chunking by Headers
The semantic engine vectorizes documents by splitting on headers. Each `##` or `###` section must:
- Cover a single coherent concept.
- Span approximately 30 to 150 tokens.
- Feature a self-explanatory title that maintains clarity out of context (e.g., `## 2. SQLite WAL Configuration` instead of `## Configuration`).

### 2. Pre-Save Auto-Linker
Before committing a new note:
1. Pass draft paragraphs to `suggest_links(text)`.
2. The engine returns matching vault sections with high vector affinity (e.g., `Score: 84%`).
3. Embed these references into the text using wikilink syntax `[[Note Title#Section]]`.

### 3. Vault Persistence
- Via MCP: Call `store_context(title, content, tags)`.
- Via Direct Filesystem: Save as `./vault/<Title>.md` and trigger `sync_vault` or allow automatic watcher indexing.

---

## Codebase Retrieval & Dependency Workflow (Cost-Effective Code Inspection)

When inspecting or navigating codebase files:

1. **Step 1: Surgical Code Search**
   - Execute `search_code(query, language?)` to locate relevant functions, structs, or methods via hybrid semantic + BM25 search.
   - Or execute `find_files(pattern)` if looking for a specific path or file type.

2. **Step 2: File Context Inspection**
   - Execute `get_file_context(file)` to get a dense summary (<80 tokens), list of exported symbols with line numbers, internal dependencies, external packages, and dependents.

3. **Step 3: Atomic Function / Symbol Retrieval**
   - Execute `read_code_chunk(file, symbol)` to retrieve only the relevant function body without dumping the entire file into context.

4. **Step 4: Dependency Graph Navigation**
   - Execute `get_code_dependencies(file)` to see what files import this file and what local files are imported.

---

## Terminal CLI & Skill-Driven Synchronization

GraphFinder allows instant indexing and incremental updates directly from any terminal or programmatically from AI skills/MCP:

### 1. Terminal Command (`graphfinder sync [path]`)
Run from any project repository or directory:
```bash
# Sync and index the current directory
graphfinder sync

# Abbreviated shortcut
graphfinder syn

# Index a specific path on disk
graphfinder sync Z:\MyProject

# Structured JSON output (ideal for automation scripts and AI skills)
graphfinder sync --json
```

- **Smart Path Auto-Detection**: GraphFinder normalizes the directory path. If the project was previously indexed, it executes a lightning-fast **incremental update via SHA-256** (only modified files are reprocessed). If it is a new folder, it registers the project automatically and generates embeddings.
- **Other CLI Commands**:
  - `graphfinder status [path]`: Displays indexing statistics for notes, code files, and graph connections.
  - `graphfinder projects`: Lists all registered projects and paths.
  - `graphfinder search "query"`: Runs fast hybrid semantic search directly from the terminal.

### 2. Synchronization via MCP Tool in Skills
AI assistants and skills can synchronize any project by calling the `sync_vault` tool:
```json
{
  "name": "sync_vault",
  "arguments": {
    "path": "Z:/MyProject"
  }
}
```
This triggers the same smart path detection and incremental update workflow, returning complete telemetry on notes, code, and graph edges.

---

## Reference Documentation

For ready-to-use schemas and tool specifications:
- [Note Templates](./references/note-templates.md): Structured templates for Concepts, Architecture Decision Records (ADR), and MCP Sessions.
- [MCP Tools Specification](./references/mcp-spec.md): Parameter schemas, return types, and protocol payloads.

