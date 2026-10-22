---
name: graphfinder
description: "Knowledge and codebase graph engine with vector RAG, AST code indexing, and pure-Rust topological graph analytics (communities, God Nodes, shortest pathfinding, and standalone HTML/JSON/Markdown artifact exports matching graphify)."
---

# GraphFinder Agent Skill

GraphFinder turns codebases and markdown vaults into a high-connectivity, semantically indexed knowledge and dependency graph. It provides:
1. **Hybrid Vector + BM25 RAG** via Reciprocal Rank Fusion ($k=60$) over atomic note chunks and code symbols.
2. **Multi-Language AST-Lite Indexing** across 14 programming languages with import and dependent tracking.
3. **Pure-Rust Topological Graph Analytics** (Louvain/LPA community clustering, degree centrality, God Nodes discovery, cross-community surprises, cycles).
4. **Graphify Parity Suite**: Instant generation of `graph.json`, `GRAPH_REPORT.md`, and zero-dependency standalone `graph.html`.

---

## When to Use This Skill

- Auditing a repository's architecture, modularity, and structural hubs ("God Nodes").
- Tracing dependency or conceptual connection paths between two files/modules (`find_path`).
- Explaining the architectural role and importance of an unfamiliar file or note (`explain_node`).
- Finding functions, classes, or code concepts surgically without loading entire files into LLM context.
- Exporting offline interactive graphs or markdown audit reports for teams and documentation.

---

## CLI Quick Reference

```bash
# Index a codebase and export standalone graph artifacts (graph.json, GRAPH_REPORT.md, graph.html)
GraphFinder sync /path/to/repo --export

# Generate an architectural audit report
GraphFinder report [vault_or_repo_dir]

# Find the shortest connection path between two nodes
GraphFinder path <source_id_or_file> <target_id_or_file> [dir]

# Get a complete architectural briefing for a node/file
GraphFinder explain <node_id_or_file> [dir]

# Export standalone artifacts (graph.json, GRAPH_REPORT.md, graph.html)
GraphFinder export [out_dir] [vault_dir]

# Run headless MCP server for AI clients (Cursor, Claude Desktop, Antigravity)
GraphFinder --mcp
```

---

## MCP Decision Matrix for AI Agents

| Task | Primary Tool | Fallback / Next Step |
|---|---|---|
| Search notes or documentation | `search_semantic(query)` | `read_section(target, section)` |
| Search functions / classes in code | `search_code(query, language?)` | `read_code_chunk(file, symbol)` |
| Inspect file structure / imports | `get_file_context(file)` | `get_code_dependencies(file)` |
| Trace connection between 2 components | `find_path(source, target)` | `get_graph_neighborhood(node)` |
| Understand a file's role in architecture | `explain_node(target)` | `get_file_context(file)` |
| Full system modularity & hub audit | `get_community_report()` | `export_graph(out_dir)` |
| Export offline interactive HTML visualizer | `export_graph(out_dir)` | CLI `GraphFinder export` |

---

## Output Artifacts Specification

When `export_artifacts` or `GraphFinder export` is triggered, it writes to `./graphfinder-out/`:
- **`graph.json`**: Nodes (with community, degree, centrality, god-node flag) and edges (with source, target, label, weight).
- **`GRAPH_REPORT.md`**: Executive summary, metrics badges, top God Nodes table, modular subsystems list, coupling surprises, and dependency cycles.
- **`graph.html`**: Zero-dependency offline HTML5 canvas application with real-time ForceGraph, dynamic search, community color coding, and hub halos.
