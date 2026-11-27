export default `
<section id="mcp">
          <h2><span class="idx">10</span>MCP server setup<a class="anchor" href="#mcp" aria-label="Link to MCP server setup">#</a></h2>
          <p>GraphFinder implements the Model Context Protocol with the official Rust SDK (<code>rmcp</code>) in two modes:</p>
          <div class="table-wrap"><table>
            <thead><tr><th>Mode</th><th>Endpoint</th><th>Best for</th></tr></thead>
            <tbody>
              <tr><td><strong>A · Embedded HTTP/SSE</strong> (app running)</td><td><code>http://127.0.0.1:45123/mcp</code> (Streamable HTTP)<br /><code>http://127.0.0.1:45123/sse</code> (SSE)</td><td>Live updates: agent-created notes appear on the graph instantly.</td></tr>
              <tr><td><strong>B · Headless stdio</strong></td><td><code>graphfinder mcp</code> / <code>GraphFinder.exe --mcp</code></td><td>Terminals, servers, CI, or when you don't want the GUI open.</td></tr>
            </tbody>
          </table></div>
          <p>The fastest path is automatic registration:</p>
          <div class="code"><div class="code-bar">bash</div>
<pre data-lang="bash">graphfinder config --install   # Cursor, Claude Desktop, Windsurf, Cline, Roo Code…
graphfinder config --json      # just print the JSON snippet</pre></div>
          <p>Or configure manually:</p>
          <div class="tabs">
            <div class="tab-list" role="tablist" aria-label="MCP client">
              <button role="tab" aria-selected="true" id="tab-cursor">Cursor</button>
              <button role="tab" aria-selected="false" id="tab-claude">Claude Desktop</button>
              <button role="tab" aria-selected="false" id="tab-antigravity">Antigravity</button>
              <button role="tab" aria-selected="false" id="tab-windsurf">Windsurf / Cline / Roo</button>
              <button role="tab" aria-selected="false" id="tab-stdio">Generic stdio</button>
            </div>
            <div class="tab-panel active" role="tabpanel" aria-labelledby="tab-cursor">
              <p>File: <code>.cursor/mcp.json</code> (workspace) or <code>~/.cursor/mcp.json</code> (global).</p>
              <div class="code"><div class="code-bar">mcp.json</div>
<pre data-lang="json">{
  "mcpServers": {
    "graphfinder": {
      "url": "http://127.0.0.1:45123/sse"
    }
  }
}</pre></div>
            </div>
            <div class="tab-panel" role="tabpanel" aria-labelledby="tab-claude">
              <p>File: <code>%APPDATA%\\Claude\\claude_desktop_config.json</code> (Windows) or <code>~/Library/Application Support/Claude/claude_desktop_config.json</code> (macOS).</p>
              <div class="code"><div class="code-bar">claude_desktop_config.json</div>
<pre data-lang="json">{
  "mcpServers": {
    "graphfinder": {
      "command": "C:\\\\Program Files\\\\GraphFinder\\\\GraphFinder.exe",
      "args": ["--mcp"]
    }
  }
}</pre></div>
            </div>
            <div class="tab-panel" role="tabpanel" aria-labelledby="tab-antigravity">
              <p>File: <code>~/.gemini/antigravity-ide/mcp_config.json</code>.</p>
              <div class="code"><div class="code-bar">mcp_config.json</div>
<pre data-lang="json">{
  "mcpServers": {
    "graphfinder": {
      "type": "sse",
      "url": "http://127.0.0.1:45123/sse"
    }
  }
}</pre></div>
            </div>
            <div class="tab-panel" role="tabpanel" aria-labelledby="tab-windsurf">
              <p>Windsurf: <code>~/.codeium/windsurf/mcp_config.json</code>. Cline / Roo Code: <code>cline_mcp_settings.json</code> in the extension's global storage.</p>
              <div class="code"><div class="code-bar">mcp_config.json</div>
<pre data-lang="json">{
  "mcpServers": {
    "graphfinder": {
      "url": "http://127.0.0.1:45123/sse"
    }
  }
}</pre></div>
            </div>
            <div class="tab-panel" role="tabpanel" aria-labelledby="tab-stdio">
              <p>Any MCP client that spawns a process (no GUI required):</p>
              <div class="code"><div class="code-bar">json</div>
<pre data-lang="json">{
  "mcpServers": {
    "graphfinder": {
      "command": "graphfinder",
      "args": ["--mcp"]
    }
  }
}</pre></div>
            </div>
          </div>
          <div class="callout tip">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18h6M10 22h4M12 2a7 7 0 0 0-4 12.7V17h8v-2.3A7 7 0 0 0 12 2Z"/></svg>
            <p>In stdio mode the MCP handshake answers in under 50 ms — the embedding model loads and the vault syncs in the background.</p>
          </div>
        </section>
`;
