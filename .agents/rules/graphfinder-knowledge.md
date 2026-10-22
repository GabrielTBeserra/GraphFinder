# Knowledge Curation and Semantic Graph Guidelines

This rule applies to all read, write, and query operations performed within the **GraphFinder** knowledge vault.

## 1. Note Structure (Markdown)
- **Single H1**: `# Note Title` on the first line.
- **Atomic Subsections**: `## Section` and `### Subsection` structuring concepts into concise blocks (50–150 tokens). This enables direct deep linking (`[[Note#Section]]`).
- **Top Summary**: The first block of text must be concise and exhibit high informational density.
- **Wikilinks**: Always reference other notes in the vault using `[[Note Title]]` or `[[Note Title#Section]]`.
- **Tags**: Include contextual `#tag` markers to inform color-coded semantic clusters in the graph.

## 2. Efficient MCP Consumption (Token Economy)
- Always prioritize `search_semantic` (atomic chunk retrieval).
- Use `get_compressed_context` to inspect section summaries without retrieving full document bodies.
- Use `suggest_links` when drafting new content to discover semantic affinity with existing vault nodes.
- Avoid duplicate concepts; connect new insights to existing notes.

## 3. Codebase Retrieval & Dependency Navigation
- **Never load whole source code files**: Use `get_file_context` to see the symbol map and dependency graph in <80 tokens.
- **Surgical Function Retrieval**: Use `read_code_chunk(file, symbol)` to inspect only the required function or method body.
- **Search Conceptually**: Use `search_code(query, language?)` for hybrid vector + BM25 keyword matching across all programming languages.
- **Deep Code Links**: Cite code locations using `[[filepath#symbol]]` or line ranges.

