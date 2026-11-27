export default `
<section id="cli">
          <h2><span class="idx">09</span>CLI reference<a class="anchor" href="#cli" aria-label="Link to CLI reference">#</a></h2>
          <p>The same binary is both the desktop app and the CLI. Running it <strong>without arguments</strong> opens the GUI; with a command it runs in the terminal.</p>
          <div class="code"><div class="code-bar">usage</div>
<pre data-lang="bash">graphfinder [COMMAND] [OPTIONS]</pre></div>
          <div class="table-wrap"><table>
            <thead><tr><th>Command (aliases)</th><th>Options</th><th>Description</th></tr></thead>
            <tbody>
              <tr><td><code>sync</code> (<code>syn</code>, <code>index</code>) <code>[path]</code></td><td><code>--json</code> <code>--export</code></td><td>Indexes notes and code in <code>path</code> (default: current dir). Registers a new project if unknown, otherwise updates incrementally via SHA-256. <code>--export</code> also writes graph artifacts.</td></tr>
              <tr><td><code>status</code> (<code>info</code>) <code>[path]</code></td><td>—</td><td>Shows notes, code files, graph edges and database statistics.</td></tr>
              <tr><td><code>projects</code> (<code>list</code>)</td><td>—</td><td>Lists every registered project and its path.</td></tr>
              <tr><td><code>search &lt;query&gt;</code></td><td><code>--path</code> <code>--limit &lt;n&gt;</code></td><td>Hybrid semantic search (BM25 + embeddings) in a project.</td></tr>
              <tr><td><code>report</code> (<code>audit</code>) <code>[path]</code></td><td><code>--out-dir</code> <code>--json</code></td><td>Communities, God Nodes, surprises and cycles, plus all three artifacts.</td></tr>
              <tr><td><code>path &lt;from&gt; &lt;to&gt;</code></td><td><code>--path</code> <code>--json</code></td><td>Shortest multi-hop path between two files, symbols or notes.</td></tr>
              <tr><td><code>explain &lt;node&gt;</code></td><td><code>--path</code> <code>--json</code></td><td>Architectural context card for a node.</td></tr>
              <tr><td><code>export [path]</code></td><td><code>--out-dir</code></td><td>Writes <code>graph.json</code>, <code>GRAPH_REPORT.md</code>, <code>graph.html</code> silently.</td></tr>
              <tr><td><code>config</code> (<code>mcp-config</code>, <code>setup</code>)</td><td><code>--install</code> / <code>-i</code> <code>--json</code></td><td>Prints MCP config for AI editors. <code>--install</code> auto-detects installed editors, registers GraphFinder in them and adds the CLI to PATH.</td></tr>
              <tr><td><code>tools</code> (<code>schemas</code>, <code>mcp-tools</code>, <code>export-schemas</code>) <code>[dir]</code></td><td>—</td><td>Exports all 26 tool JSON schemas, a <code>tools.json</code> manifest and <code>instructions.md</code> to <code>./mcp-tools</code>.</td></tr>
              <tr><td><code>install-cli</code> (<code>install-path</code>)</td><td>—</td><td>Adds <code>graphfinder</code> to the OS PATH.</td></tr>
              <tr><td><code>mcp</code> (<code>--mcp</code>)</td><td>—</td><td>Starts the headless stdio MCP server.</td></tr>
              <tr><td><code>help</code> (<code>--help</code>, <code>-h</code>)</td><td>—</td><td>Prints help.</td></tr>
            </tbody>
          </table></div>
          <h3>Examples</h3>
          <div class="code"><div class="code-bar">bash</div>
<pre data-lang="bash"># Machine-readable sync for CI or scripts
graphfinder sync ./services/api --json

# How does the UI reach the database layer?
graphfinder path "GraphView.tsx" "db.rs"

# Why does this file matter?
graphfinder explain "src-tauri/src/engine/db.rs"

# Audit into a custom folder
graphfinder report --out-dir ./docs/architecture

# Search only 3 results in a specific project path
graphfinder search "file watcher debounce" --path ./GraphFinder --limit 3</pre></div>
          <p>From a source checkout, npm scripts wrap the CLI: <code>npm run sync</code>, <code>npm run status</code>, <code>npm run search</code>, <code>npm run projects</code>, <code>npm run mcp</code>.</p>
        </section>
`;
