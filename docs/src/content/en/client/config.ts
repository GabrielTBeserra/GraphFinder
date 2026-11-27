export default `
<section id="config">
          <h2><span class="idx">14</span>Configuration<a class="anchor" href="#config" aria-label="Link to Configuration">#</a></h2>
          <p>GraphFinder is configured through environment variables — no config file required.</p>
          <div class="table-wrap"><table>
            <thead><tr><th>Variable</th><th>Default</th><th>Effect</th></tr></thead>
            <tbody>
              <tr><td><code>GRAPHFINDER_DATA_DIR</code></td><td>see above</td><td>Base directory for vaults, registry and databases.</td></tr>
              <tr><td><code>GRAPHFINDER_VAULT</code></td><td><code>&lt;data&gt;/vault</code></td><td>Overrides the default vault folder.</td></tr>
              <tr><td><code>GRAPHFINDER_DB</code></td><td><code>&lt;data&gt;/index.db</code></td><td>Overrides the default SQLite database path.</td></tr>
              <tr><td><code>GRAPHFINDER_MODEL</code> / <code>GRAPHFINDER_EMBEDDING_MODEL</code></td><td><code>auto</code></td><td>Defaults to <code>multilingual-e5-small</code> (E5 prefixes handled automatically). <code>minilm</code> loads legacy all-MiniLM-L6-v2.</td></tr>
              <tr><td><code>GRAPHFINDER_MODELS</code></td><td>—</td><td>Folder containing a local <code>model.onnx</code> + <code>tokenizer.json</code> for fully offline use (also auto-detected in <code>models/multilingual-e5-small</code> or <code>models/all-MiniLM-L6-v2</code>).</td></tr>
              <tr><td><code>GRAPHFINDER_DEVICE</code></td><td><code>auto</code></td><td>ONNX Runtime execution-provider preference: <code>auto</code> (DirectML on Windows, CoreML on macOS, CPU elsewhere — silent per-operator fallback), or force <code>cpu</code>, <code>dml</code>, <code>coreml</code>, <code>cuda</code>, <code>rocm</code> (GPU EPs degrade gracefully to CPU).</td></tr>
              <tr><td><code>GRAPHFINDER_BATCH</code></td><td><code>16</code></td><td>Embedding batch size per ONNX call during indexing. Lower values keep the UI responsive on machines where ORT saturates all physical cores.</td></tr>
              <tr><td><code>GRAPHFINDER_MCP_PORT</code></td><td><code>45123</code></td><td>Port of the embedded HTTP/SSE MCP server.</td></tr>
              <tr><td><code>GRAPHFINDER_AUTO_APPEND_LINKS</code></td><td><code>false</code></td><td><code>1</code>/<code>true</code> automatically appends suggested wikilinks when storing notes.</td></tr>
            </tbody>
          </table></div>
          <div class="code"><div class="code-bar">bash</div>
<pre data-lang="bash"># Portuguese / multilingual vault on a custom port
export GRAPHFINDER_MODEL=multilingual
export GRAPHFINDER_MCP_PORT=47000
graphfinder</pre></div>
          <div class="callout warn">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"/><path d="M12 9v4M12 17h.01"/></svg>
            <p>Model changes are detected automatically: the engine stamps <code>embedding_model</code> in the DB and re-embeds every note and code chunk on the next sync. No manual rebuild needed.</p>
          </div>
        </section>
`;
