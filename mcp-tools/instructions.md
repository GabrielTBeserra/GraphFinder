# GraphFinder MCP Agent Instructions

Neural knowledge graph + universal code intelligence: 26 tools in 4 groups.

## 1. Architecture & Topology
- `get_community_report`: LPA communities, God Nodes, cross-coupling surprises — start here for architecture questions.
- `explain_node`: 360-degree card for a file/symbol/note (centrality, community, edges).
- `find_path`: BFS shortest dependency path between two nodes.
- `export_graph`: graph.json + GRAPH_REPORT.md + standalone graph.html.

## 2. Code Intelligence
- `search_code`: hybrid BM25+vector search returning [[file#symbol]] excerpts; `read_code_chunk` for full symbols; `fields='ids'` for cheap scans.
- `find_files`: glob/name/language lookup.
- `get_file_context`: imports, dependents, external pkgs, symbol map.
- `read_code_chunk`: one symbol or line range (`L10-40`, `10-40`, `L10`); unknown symbols error with the available list.
- `get_code_dependencies`: forward/reverse dependency report for a file.

## 3. Vault & Knowledge
- `search_semantic`: vector search returning [[Note#Section]] excerpts; `fields='ids'` for cheap scans.
- `get_compressed_context`: note summary + section map with token estimates.
- `read_section`: one ##/### section.
- `get_graph_neighborhood`: 1-2 hop wikilink neighbors.
- `get_vault_stats`: index counts.
- `validate_vault`: schema audit (H1, summary, section size, wikilinks, orphans, tags).
- `list_recent_notes`: browse notes with summaries.

## 4. Mutation & Projects
- `store_context`: save/update a note — run `suggest_links` on the draft first.
- `update_section`: replace one section body (heading preserved).
- `suggest_links`: wikilink suggestions for draft text.
- `link_notes`, `rename_note`, `delete_note`: graph maintenance.
- `sync_vault`: incremental re-index; `path` registers a new folder.
- `list_projects`, `create_project`, `switch_project`: multi-vault management.

## Golden Rules
1. Prefer atomic excerpts over whole files: search -> surgical read -> cite deep links [[Note#Section]] / [[file#symbol]].
2. Run `suggest_links` before `store_context`.
3. Run `get_community_report` first when onboarding unfamiliar systems.
4. Most tools accept `project: "<id>"`.
