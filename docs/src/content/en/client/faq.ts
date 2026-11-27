export default `
<section id="faq">
          <h2><span class="idx">18</span>Troubleshooting &amp; FAQ<a class="anchor" href="#faq" aria-label="Link to FAQ">#</a></h2>
          <h4>My agent can't connect to <code>127.0.0.1:45123</code>.</h4>
          <p>The HTTP/SSE endpoint only exists while the desktop app is running. Open the app, or switch your client to stdio mode (<code>"command": "graphfinder", "args": ["--mcp"]</code>). If you changed <code>GRAPHFINDER_MCP_PORT</code>, update the URL.</p>
          <h4>The server MCP returns <code>insufficient project scope</code>.</h4>
          <p>This is a tool-level authorization error, not a connection failure. The token is missing the required scope (for example <code>index:trigger</code>) or is restricted to another project. Create or update the user's token in <em>Access → API tokens</em>.</p>
          <h4><code>graphfinder</code> is not recognised in the terminal.</h4>
          <p>Run <code>GraphFinder.exe install-cli</code> (or <code>graphfinder config --install</code>) and open a new terminal so the updated PATH is loaded.</p>
          <h4>Search results are poor for Portuguese / other languages.</h4>
          <p>Set <code>GRAPHFINDER_MODEL=multilingual</code> and re-index.</p>
          <h4>The first start is slow.</h4>
          <p>The embedding model is downloaded once. Subsequent starts load it from <code>.fastembed_cache</code>; point <code>GRAPHFINDER_MODELS</code> to a local copy for air-gapped machines.</p>
          <h4>Some files are missing from the code graph.</h4>
          <p>Check whether they are ignored by <code>.gitignore</code>, are binary, exceed 1 MB, or have an unsupported extension. Run <code>graphfinder sync</code> to force an incremental update.</p>
          <h4>Does any data leave my machine?</h4>
          <p>In desktop mode, embeddings, search and analytics run locally; the only network access is the one-time model download. In private-server mode, indexing and embeddings run on your self-hosted server rather than on a third-party cloud service.</p>
        </section>
`;
