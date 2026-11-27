export default `
<section id="overview">
          <h2><span class="idx">01</span>Overview<a class="anchor" href="#overview" aria-label="Link to Overview">#</a></h2>
          <p class="lead-p"><strong>GraphFinder</strong> is a cross-platform desktop application (Tauri v2 + Rust + React 19) and command-line tool that indexes Markdown knowledge vaults and source-code repositories into a single, semantically searchable graph.</p>
          <p>Traditional RAG pipelines split documents into arbitrary fixed windows or push entire files into an LLM context, wasting thousands of tokens. Note-taking tools, in turn, rarely offer local vector search or an API agents can use. GraphFinder addresses both sides:</p>
          <ul>
            <li><strong>For humans</strong> — a distraction-free, local-first environment to write pure Markdown with <code>[[wikilinks]]</code>, explore connections in an interactive 2D graph and keep full ownership of your files.</li>
            <li><strong>For AI agents</strong> — an embedded Model Context Protocol (MCP) server that returns sub-section level results (<code>[[Note#Section]]</code>) and individual code symbols, typically around <strong>50 tokens per hit</strong>.</li>
          </ul>
          <h3>Key concepts</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Concept</th><th>Meaning</th></tr></thead>
            <tbody>
              <tr><td><strong>Vault</strong></td><td>A folder of <code>.md</code> files (and optionally code) indexed by GraphFinder.</td></tr>
              <tr><td><strong>Project</strong></td><td>A registered vault with its own isolated SQLite index. Identified by a slug id (e.g. <code>my-api</code>).</td></tr>
              <tr><td><strong>Chunk</strong></td><td>An atomic section of a note delimited by <code>##</code> / <code>###</code> headings, embedded independently.</td></tr>
              <tr><td><strong>Deep link</strong></td><td>A reference to a specific section: <code>[[Note Title#Section Heading]]</code>, or to a symbol: <code>[[path/file.rs#function]]</code>.</td></tr>
              <tr><td><strong>Node / Edge</strong></td><td>Notes and code files are nodes; wikilinks and imports are edges.</td></tr>
              <tr><td><strong>God Node</strong></td><td>A highly central hub in the graph, detected by degree centrality.</td></tr>
            </tbody>
          </table></div>
        </section>
`;
