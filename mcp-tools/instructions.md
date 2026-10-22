# GraphFinder Universal MCP Agent Instructions

GraphFinder is a neural knowledge graph and universal codebase intelligence engine. It provides 24 atomic tools across 4 categories:

## 1. Architectural & Topological Audit
- `get_community_report`: Global map of modular subsystems, architectural God Nodes (central hubs), and unexpected cross-couplings.
- `explain_node`: 360-degree context card for a file/symbol (centrality, callers, callees, community).
- `find_path`: BFS shortest dependency path connecting any two files, symbols, or notes.
- `export_graph`: Exports standalone `graph.json`, `GRAPH_REPORT.md`, and visual `graph.html`.

## 2. Multi-Language Codebase Intelligence
- `search_code`: Hybrid semantic (BM25 + neural) search for symbols, functions, and logic across code repositories.
- `find_files`: Fast glob and regex pattern matching across project files.
- `get_file_context`: High-density structural overview of a file (imports, dependents, exported symbols) without reading the entire file.
- `read_code_chunk`: Surgical extraction of a specific function, class, or line range.
- `get_code_dependencies`: Forward and reverse dependency analysis for any file.

## 3. Semantic Vault & Knowledge Search
- `search_semantic`: Atomic vector search over knowledge notes with deep links (`[[Note#Section]]`).
- `get_compressed_context`: Compressed structural map and token estimates for a note.
- `read_section`: Targeted reading of a single H2/H3 section.
- `get_vault_stats`: Metrics on indexed notes, chunks, and embeddings.
- `get_graph_neighborhood`: 1-hop or 2-hop graph adjacency.

## 4. Vault Synchronization & Mutation
- `sync_vault`: Incrementally re-indexes notes and codebases via SHA-256 change detection.
- `store_context`: Saves a new markdown note with automatic semantic chunking and tag taxonomy.
- `suggest_links`: Evaluates draft text and suggests semantic wikilinks to existing vault notes.
- `link_notes`: Creates bidirectional relationships between notes.
- `rename_note`, `delete_note`: Knowledge maintenance operations.
- `list_projects`, `switch_project`, `create_project`: Multi-workspace management.

## Golden Rules for AI Agents:
1. Prefer atomic excerpts: Use `search_semantic` or `read_code_chunk` rather than reading entire files into context.
2. Verify connections: Use `suggest_links` before saving new knowledge with `store_context`.
3. Audit architecture first: Use `get_community_report()` when onboarding or inspecting unfamiliar systems.
