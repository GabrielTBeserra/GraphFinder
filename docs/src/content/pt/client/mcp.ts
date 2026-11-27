export default `
<section id="mcp">
          <h2><span class="idx">10</span>Configurando o MCP<a class="anchor" href="#mcp" aria-label="Link para Configurando o MCP">#</a></h2>
          <p>O GraphFinder implementa o Model Context Protocol com o SDK oficial em Rust (<code>rmcp</code>) em dois modos:</p>
          <div class="table-wrap"><table>
            <thead><tr><th>Modo</th><th>Endpoint</th><th>Ideal para</th></tr></thead>
            <tbody>
              <tr><td><strong>A · HTTP/SSE embarcado</strong> (app aberto)</td><td><code>http://127.0.0.1:45123/mcp</code> (Streamable HTTP)<br /><code>http://127.0.0.1:45123/sse</code> (SSE)</td><td>Atualizações ao vivo: notas criadas pelo agente aparecem no grafo na hora.</td></tr>
              <tr><td><strong>B · stdio headless</strong></td><td><code>graphfinder mcp</code> / <code>GraphFinder.exe --mcp</code></td><td>Terminais, servidores, CI, ou quando você não quer a interface aberta.</td></tr>
            </tbody>
          </table></div>
          <p>O caminho mais rápido é o registro automático:</p>
          <div class="code"><div class="code-bar">bash</div>
<pre data-lang="bash">graphfinder config --install   # Cursor, Claude Desktop, Windsurf, Cline, Roo Code…
graphfinder config --json      # apenas imprime o snippet JSON</pre></div>
          <p>Ou configure manualmente:</p>
          <div class="tabs">
            <div class="tab-list" role="tablist" aria-label="Cliente MCP">
              <button role="tab" aria-selected="true" id="tab-cursor">Cursor</button>
              <button role="tab" aria-selected="false" id="tab-claude">Claude Desktop</button>
              <button role="tab" aria-selected="false" id="tab-antigravity">Antigravity</button>
              <button role="tab" aria-selected="false" id="tab-windsurf">Windsurf / Cline / Roo</button>
              <button role="tab" aria-selected="false" id="tab-stdio">stdio genérico</button>
            </div>
            <div class="tab-panel active" role="tabpanel" aria-labelledby="tab-cursor">
              <p>Arquivo: <code>.cursor/mcp.json</code> (workspace) ou <code>~/.cursor/mcp.json</code> (global).</p>
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
              <p>Arquivo: <code>%APPDATA%\\Claude\\claude_desktop_config.json</code> (Windows) ou <code>~/Library/Application Support/Claude/claude_desktop_config.json</code> (macOS).</p>
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
              <p>Arquivo: <code>~/.gemini/antigravity-ide/mcp_config.json</code>.</p>
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
              <p>Windsurf: <code>~/.codeium/windsurf/mcp_config.json</code>. Cline / Roo Code: <code>cline_mcp_settings.json</code> no armazenamento global da extensão.</p>
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
              <p>Qualquer cliente MCP que inicia um processo (sem interface gráfica):</p>
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
            <p>No modo stdio o handshake MCP responde em menos de 50 ms — o modelo de embeddings carrega e o vault sincroniza em segundo plano.</p>
          </div>
        </section>
`;
