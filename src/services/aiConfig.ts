export const MCP_CONFIG_SNIPPET = `{
  "mcpServers": {
    "graphfinder": {
      "type": "sse",
      "url": "http://127.0.0.1:45123/sse"
    }
  }
}`;

export const MCP_CURSOR_CONFIG_SNIPPET = `{
  "mcpServers": {
    "graphfinder": {
      "url": "http://127.0.0.1:45123/sse"
    }
  }
}`;

export const MCP_CLAUDE_CONFIG_SNIPPET = `{
  "mcpServers": {
    "graphfinder": {
      "command": "GraphFinder.exe",
      "args": ["--mcp"]
    }
  }
}`;

export const AGENT_RULE_CONTENT = `# GraphFinder: Knowledge, Writing, and MCP Guidelines

This document establishes the mandatory rules for AI agents, editors, and contributors interacting with or managing the **GraphFinder** knowledge vault. The primary goals are **maximal semantic density**, **rigorous token economy**, and **high graph connectivity**.

---

## 1. Core Architectural Principles

1. **Local-First & Plain Markdown**: All notes live in \`.md\` files within the \`./vault\` directory (or designated project directory). They must remain 100% human-readable, formatted in standard Markdown with bidirectional wikilinks [[Link]].
2. **Atomic Semantic Chunking**: The FastEmbed/SQLite engine indexes notes by splitting content strictly across headers (\`##\` and \`###\`). Every header generates an independent vector embedding in the \`note_chunks\` database table.
3. **Token Conservation**: AI agents must **NEVER** request or inject full Markdown files into an LLM context window unless explicitly instructed by the user. Always prefer atomic excerpts via deep links (\`[[Note#Section]]\`).

---

## 2. Note Writing and Structural Standards

Every note created or modified in the vault MUST strictly adhere to the following schema:

### A. Title and Executive Summary
* The note must begin with a single H1 header (\`# Note Title\`).
* The first paragraph immediately following the H1 must be a high-density executive summary (2 to 3 lines maximum), synthesizing the document's core purpose.

### B. Header Granularity (\`##\` and \`###\`)
* Divide the body into self-contained subsections of **40 to 150 tokens** each.
* Use semantic, descriptive titles for headers (e.g., \`## 2. SQLite WAL Configuration\` instead of simply \`## Configuration\`).
* The search engine uses these titles to construct precise deep links: \`[[Note#2. SQLite WAL Configuration]]\`.

### C. Mandatory Bidirectional Connections (Wikilinks)
* Connect terminology and concepts to related notes using \`[[Note Title]]\` or \`[[Note Title#Section]]\`.
* Every note should maintain at least **1 to 3 connections** to other nodes in the graph to prevent orphaned knowledge.
* Verify potential connections using the \`suggest_links\` MCP tool prior to saving.

### D. Tag Taxonomy
* Append context hashtags \`#tag\` at the end of sections or at the bottom of the document.
* Use lowercase terms (e.g., \`#architecture\`, \`#sqlite\`, \`#rag\`, \`#mcp\`).
* The GraphFinder frontend utilizes these tags to compute and render colored semantic graph clusters.

---

## 3. AI Agent MCP Operation Protocol

When operating as an AI agent connected to the GraphFinder MCP server, follow this decision pipeline to minimize context consumption:

\`\`\`
[User Question]
       │
       ▼
1. Execute \`search_semantic(query)\`
   - Returns top 5 relevant chunks (~50 tokens each).
       │
       ├─► Answer found? ──► Respond citing deep links [[Note#Section]].
       │
       └─► Need structural overview? ──► Execute \`get_compressed_context(note_id)\`
                                          (Summary + section map with token estimates).
\`\`\`

### Golden Rules for MCP Agents:
1. **Never read raw Markdown files unnecessarily**: Use \`search_semantic\` first. It extracts the exact subsection containing the required knowledge.
2. **When saving new knowledge (\`store_context\`)**:
   - Provide a clear, descriptive title.
   - Format content cleanly with \`##\` headers.
   - Include explicit wikilinks to notes consulted earlier in the session.
   - Supply a relevant list of lowercase \`tags\`.
3. **When evaluating links (\`suggest_links\`)**:
   - Pass draft paragraphs before saving to identify semantic affinity with existing vault notes.

---

## 4. Multi-Vault and Multi-Project Support

GraphFinder supports isolated workspaces (\`./vaults/<project_id>\`), each with its own Markdown directory and independent SQLite vector database:

* \`list_projects()\`: Lists all configured projects, file paths, and note counts.
* \`create_project(name)\`: Initializes a new isolated vault at \`./vaults/<slug>/\` with an introductory note.
* \`switch_project(project_id)\`: Toggles the active default vault.
* **Direct Vault Targeting**: All querying and mutation tools (\`search_semantic\`, \`store_context\`, \`get_compressed_context\`, \`get_graph_neighborhood\`, \`suggest_links\`, \`link_notes\`, \`search_code\`, \`find_files\`, \`get_file_context\`, \`read_code_chunk\`, \`get_code_dependencies\`) accept an optional \`project: "<id>"\` parameter, enabling multi-vault management without cross-project contamination.

---

## 5. Multi-Language Codebase Indexing & AST-Lite Graph

GraphFinder indexes entire software repositories across multiple programming languages (Rust, TypeScript, JavaScript, Python, Go, C/C++, Java, Kotlin, C#, PHP, Ruby, Swift, CSS, Shell). It parses import statements (\`import\`, \`require\`, \`use\`, \`include\`, \`mod\`, etc.), constructs an internal dependency graph, and creates atomic symbol embeddings for functions, classes, and structs.

### Code Retrieval Decision Pipeline for AI Agents:
\`\`\`
[Need to find or inspect code]
       │
       ├─► Looking for a concept/feature? ──► \`search_code(query, language?)\`
       │                                       (RRF hybrid search across symbols and comments).
       │
       ├─► Looking for a file path? ────────► \`find_files(pattern)\`
       │                                       (Fast pattern/glob matching).
       │
       ├─► Need structural summary of file? ─► \`get_file_context(file)\`
       │                                       (Imports, dependents, external pkgs, symbols).
       │
       ├─► Need a specific function/symbol? ─► \`read_code_chunk(file, symbol)\`
       │                                       (Surgical snippet without reading full file).
       │
       └─► Analyzing architectural impact? ──► \`get_code_dependencies(file)\`
                                               (Who imports this file & what does it import?).
\`\`\`

### Golden Rules for Code Context:
1. **Never read whole source code files into context** unless explicitly asked. Use \`get_file_context\` to see the symbol map, then \`read_code_chunk\` to fetch only the required function or method.
2. **Cite deep links**: Reference code locations using \`[[filepath#symbol]]\` or line ranges.`;

export const SKILL_CONTENT = `---
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
- When drafting or structuring new notes in \`./vault\` or isolated project vaults (\`./vaults/<project>\`).
- When toggling between workspaces with \`list_projects\`, \`switch_project\`, or creating new ones with \`create_project\`.
- When communicating with GraphFinder via the embedded MCP server (HTTP/SSE or Stdio).
- When executing semantic searches while preserving LLM context budget.
- When generating or applying bidirectional graph links.
- When inspecting codebase files, symbols, and dependency import graphs.

---

## Read and Query Workflow (Cost-Effective Retrieval)

When requested to answer or retrieve context from stored knowledge:

1. **Step 1: Atomic Semantic Search**
   - Execute the MCP tool \`search_semantic(query)\`.
   - Review returned chunks, cosine affinity scores, and estimated token counts (~50–100 tokens per chunk).
   - Identify precise deep links formatted as \`[[Note#Section]]\`.

2. **Step 2: Neighborhood Exploration (Optional)**
   - If a chunk references related concepts or adjacent context is required, call \`get_graph_neighborhood(note_id, depth: 1)\`.
   - This exposes incoming backlinks and outgoing links without retrieving full document bodies.

3. **Step 3: Structural Compression (If Needed)**
   - To inspect a note's complete outline without loading full text, call \`get_compressed_context(note_id)\`.
   - Returns a 2-line executive summary and a list of all H2/H3 sections with respective token counts.

---

## Write and Curation Workflow (Persisting Knowledge)

When authoring or updating vault notes:

### 1. Structural Chunking by Headers
The semantic engine vectorizes documents by splitting on headers. Each \`##\` or \`###\` section must:
- Cover a single coherent concept.
- Span approximately 30 to 150 tokens.
- Feature a self-explanatory title that maintains clarity out of context (e.g., \`## 2. SQLite WAL Configuration\` instead of \`## Configuration\`).

### 2. Pre-Save Auto-Linker
Before committing a new note:
1. Pass draft paragraphs to \`suggest_links(text)\`.
2. The engine returns matching vault sections with high vector affinity (e.g., \`Score: 84%\`).
3. Embed these references into the text using wikilink syntax \`[[Note Title#Section]]\`.

### 3. Vault Persistence
- Via MCP: Call \`store_context(title, content, tags)\`.
- Via Direct Filesystem: Save as \`./vault/<Title>.md\` and trigger \`sync_vault\` or allow automatic watcher indexing.

---

## Codebase Retrieval & Dependency Workflow (Cost-Effective Code Inspection)

When inspecting or navigating codebase files:

1. **Step 1: Surgical Code Search**
   - Execute \`search_code(query, language?)\` to locate relevant functions, structs, or methods via hybrid semantic + BM25 search.
   - Or execute \`find_files(pattern)\` if looking for a specific path or file type.

2. **Step 2: File Context Inspection**
   - Execute \`get_file_context(file)\` to get a dense summary (<80 tokens), list of exported symbols with line numbers, internal dependencies, external packages, and dependents.

3. **Step 3: Atomic Function / Symbol Retrieval**
   - Execute \`read_code_chunk(file, symbol)\` to retrieve only the relevant function body without dumping the entire file into context.

4. **Step 4: Dependency Graph Navigation**
   - Execute \`get_code_dependencies(file)\` to see what files import this file and what local files are imported.

---

## Terminal CLI & Skill-Driven Synchronization

GraphFinder allows instant indexing and incremental updates directly from any terminal or programmatically from AI skills/MCP:

\`\`\`bash
# Sync and index the current directory
graphfinder sync

# Abbreviated shortcut
graphfinder syn

# Index a specific path on disk
graphfinder sync Z:\\MyProject

# Structured JSON output (ideal for automation scripts and AI skills)
graphfinder sync --json
\`\`\`

- **Smart Path Auto-Detection**: GraphFinder normalizes the directory path. If the project was previously indexed, it executes a lightning-fast **incremental update via SHA-256** (only modified files are reprocessed). If it is a new folder, it registers the project automatically and generates embeddings.
- **Other CLI Commands**:
  - \`graphfinder status [path]\`: Displays indexing statistics for notes, code files, and graph connections.
  - \`graphfinder projects\`: Lists all registered projects and paths.
  - \`graphfinder search "query"\`: Runs fast hybrid semantic search directly from the terminal.

### 2. Synchronization via MCP Tool in Skills
AI assistants and skills can synchronize any project by calling the \`sync_vault\` tool:
\`\`\`json
{
  "name": "sync_vault",
  "arguments": {
    "path": "Z:/MyProject"
  }
}
\`\`\`
This triggers the same smart path detection and incremental update workflow, returning complete telemetry on notes, code, and graph edges.

---

## Reference Documentation

For ready-to-use schemas and tool specifications:
- [Note Templates](./references/note-templates.md): Structured templates for Concepts, Architecture Decision Records (ADR), and MCP Sessions.
- [MCP Tools Specification](./references/mcp-spec.md): Parameter schemas, return types, and protocol payloads.`;
