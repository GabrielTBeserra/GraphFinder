export default `
<section id="tools">
          <h2><span class="idx">11</span>MCP tools<a class="anchor" href="#tools" aria-label="Link to MCP tools">#</a></h2>
          <p>Every tool accepts an optional <code>project</code> parameter to target a specific vault without cross-contamination. Dashed parameters are optional.</p>

          <p class="tool-group">Knowledge retrieval</p>
          <div class="tools">
            <div class="tool"><h4>search_semantic</h4><p>Hybrid vector + BM25 search over note chunks with RRF. Returns deep links and token estimates.</p><div class="params"><span>query</span><span class="opt">limit=5</span><span class="opt">format: dense_chunks | summaries</span></div></div>
            <div class="tool"><h4>read_section</h4><p>Reads exactly one section by heading or deep-link fragment.</p><div class="params"><span>target</span><span>section</span></div></div>
            <div class="tool"><h4>get_compressed_context</h4><p>Summary, links, tags and a heading map with per-section token costs — under 80 tokens.</p><div class="params"><span>target</span></div></div>
            <div class="tool"><h4>list_recent_notes</h4><p>Recently modified notes with one-line summaries and tags.</p><div class="params"><span class="opt">limit=10</span></div></div>
          </div>

          <p class="tool-group">Authoring &amp; linking</p>
          <div class="tools">
            <div class="tool"><h4>suggest_links</h4><p>Embeds draft text and recommends existing notes/sections to link (prevents orphans).</p><div class="params"><span>text</span><span class="opt">current_note</span><span class="opt">threshold=0.65</span></div></div>
            <div class="tool"><h4>store_context</h4><p>Creates or updates a note, parses it and indexes it immediately.</p><div class="params"><span>title</span><span>content</span><span class="opt">tags[]</span><span class="opt">links[]</span></div></div>
            <div class="tool"><h4>update_section</h4><p>Surgically replaces the body of a <code>##</code>/<code>###</code> section (heading preserved) and re-indexes — cheaper than rewriting whole notes.</p><div class="params"><span>target</span><span>heading</span><span>content</span></div></div>
            <div class="tool"><h4>validate_vault</h4><p>Audits notes against the AGENTS.md schema: H1+summary, 40-150 token sections, broken wikilinks, orphans, missing tags.</p><div class="params"><span class="opt">target</span><span class="opt">project</span></div></div>
            <div class="tool"><h4>link_notes</h4><p>Appends a <code>[[wikilink]]</code> from one note to another and updates the graph.</p><div class="params"><span>from_note</span><span>to_note</span></div></div>
            <div class="tool"><h4>rename_note</h4><p>Renames a note and refactors all incoming wikilinks vault-wide.</p><div class="params"><span>old_title</span><span>new_title</span></div></div>
            <div class="tool"><h4>delete_note</h4><p>Deletes the file and cleans chunks, embeddings, links and tags.</p><div class="params"><span>target</span></div></div>
          </div>

          <p class="tool-group">Graph &amp; architecture</p>
          <div class="tools">
            <div class="tool"><h4>get_graph_neighborhood</h4><p>Nodes connected within 1 or 2 hops, without reading bodies.</p><div class="params"><span>target</span><span class="opt">depth=1</span></div></div>
            <div class="tool"><h4>find_path</h4><p>BFS shortest path across wikilinks and imports between two nodes.</p><div class="params"><span>source</span><span>target</span><span class="opt">directed=false</span></div></div>
            <div class="tool"><h4>explain_node</h4><p>Centrality, community, God Node status, dependents, dependencies and siblings.</p><div class="params"><span>target</span></div></div>
            <div class="tool"><h4>get_community_report</h4><p>System-wide audit: communities, God Nodes, surprises and cycles.</p><div class="params"><span class="opt">project</span></div></div>
            <div class="tool"><h4>export_graph</h4><p>Writes <code>graph.json</code>, <code>GRAPH_REPORT.md</code> and <code>graph.html</code>.</p><div class="params"><span class="opt">out_dir=./graphfinder-out</span></div></div>
            <div class="tool"><h4>get_vault_stats</h4><p>Notes, chunks, links, tags and total token estimates.</p><div class="params"><span class="opt">project</span></div></div>
          </div>

          <p class="tool-group">Codebase</p>
          <div class="tools">
            <div class="tool"><h4>search_code</h4><p>Hybrid search over code symbols, signatures and comments. Returns <code>[[file#symbol]]</code> with line ranges.</p><div class="params"><span>query</span><span class="opt">language</span><span class="opt">limit=5</span></div></div>
            <div class="tool"><h4>find_files</h4><p>Glob / substring matching over indexed file paths.</p><div class="params"><span>pattern</span><span class="opt">limit=20</span></div></div>
            <div class="tool"><h4>get_file_context</h4><p>Symbols with line ranges, local imports, dependents and packages — under 80 tokens.</p><div class="params"><span>file</span></div></div>
            <div class="tool"><h4>read_code_chunk</h4><p>Returns only the body of a named function, class or struct.</p><div class="params"><span>file</span><span>symbol</span></div></div>
            <div class="tool"><h4>get_code_dependencies</h4><p>What a file imports and who imports it.</p><div class="params"><span>file</span></div></div>
          </div>

          <p class="tool-group">Projects &amp; sync</p>
          <div class="tools">
            <div class="tool"><h4>list_projects</h4><p>All registered vaults with paths and note counts.</p><div class="params"><span class="opt">—</span></div></div>
            <div class="tool"><h4>create_project</h4><p>Creates an isolated vault (optionally at any path on disk).</p><div class="params"><span>name</span><span class="opt">path</span></div></div>
            <div class="tool"><h4>switch_project</h4><p>Changes the active default vault.</p><div class="params"><span>project_id</span></div></div>
            <div class="tool"><h4>sync_vault</h4><p>Hash-based incremental re-index; a <code>path</code> auto-maps to (or registers) a project.</p><div class="params"><span class="opt">path</span><span class="opt">project</span></div></div>
          </div>
          <p style="margin-top:20px">Run <code>graphfinder tools</code> to export the JSON schemas of all tools for use with any agent framework.</p>
        </section>
`;
