export default `
<section id="agents">
          <h2><span class="idx">12</span>AI agents &amp; skills<a class="anchor" href="#agents" aria-label="Link to AI agents">#</a></h2>
          <p>GraphFinder ships three layers that work together: <strong>the agent</strong> (Cursor, Claude, Antigravity…) reasons and plans, <strong>the skill</strong> enforces token economy and note quality, and <strong>the MCP server</strong> executes retrieval and writes.</p>
          <h3>Bundled skills &amp; rules</h3>
          <ul>
            <li><code>AGENTS.md</code> — repository rules for agents (token economy, note schema, decision pipelines). Antigravity loads it automatically; in Cursor add it to Project Rules; in Claude paste it into the project prompt.</li>
            <li><code>.agents/skills/graphfinder-curator</code> — authoring and curation workflow (summaries, atomic sections, <code>suggest_links</code> before <code>store_context</code>, tags).</li>
            <li><code>.agents/skills/graphfinder</code> — architecture auditing workflow (reports, paths, explain, exports).</li>
          </ul>
          <h3>Recommended workflows</h3>
          <h4>Answering a question</h4>
          <div class="code"><div class="code-bar">flow</div>
<pre>search_semantic("sqlite wal mode")        → top 5 chunks (~50 tokens each)
  └─ need the outline?  get_compressed_context("System Architecture")
  └─ need one section?  read_section("System Architecture", "1. Local Storage")</pre></div>
          <h4>Saving knowledge</h4>
          <div class="code"><div class="code-bar">flow</div>
<pre>draft note (# Title + summary + ## sections)
  → suggest_links(draft)                   → e.g. "System Architecture: 85%"
  → insert [[System Architecture]] into draft
  → store_context(title, content, tags)    → file written, indexed, graph updated</pre></div>
          <h4>Inspecting code</h4>
          <div class="code"><div class="code-bar">flow</div>
<pre>search_code("hybrid_search", language="rust")
  → get_file_context("src-tauri/src/engine/db.rs")       # symbol map, &lt;80 tokens
  → read_code_chunk("src-tauri/src/engine/db.rs", "hybrid_search")
  → get_code_dependencies("src-tauri/src/engine/db.rs")  # impact analysis</pre></div>
          <h4>Auditing architecture</h4>
          <div class="code"><div class="code-bar">flow</div>
<pre>get_community_report()      → subsystems, God Nodes, surprises, cycles
  → explain_node("db.rs")   → why it matters
  → find_path("App.tsx", "db.rs")</pre></div>
          <h3>Prompt examples</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Goal</th><th>Prompt</th></tr></thead>
            <tbody>
              <tr><td>Surgical search</td><td><em>"Query the GraphFinder vault and explain how code indexing works without reading full files."</em></td></tr>
              <tr><td>Connected note</td><td><em>"Document our OAuth2 flow as a GraphFinder note. Call suggest_links first and add relevant tags."</em></td></tr>
              <tr><td>Code inspection</td><td><em>"Find where import parsing is implemented and read only that function with read_code_chunk."</em></td></tr>
              <tr><td>Impact analysis</td><td><em>"Which files import GraphView.tsx? Summarise the architectural impact of changing it."</em></td></tr>
              <tr><td>Architecture audit</td><td><em>"Run a GraphFinder community report and list the God Nodes we should document first."</em></td></tr>
            </tbody>
          </table></div>
        </section>
`;
