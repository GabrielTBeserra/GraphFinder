export default `
<section id="config">
          <h2><span class="idx">14</span>Configuração<a class="anchor" href="#config" aria-label="Link para Configuração">#</a></h2>
          <p>O GraphFinder é configurado por variáveis de ambiente — nenhum arquivo de configuração é necessário.</p>
          <div class="table-wrap"><table>
            <thead><tr><th>Variável</th><th>Padrão</th><th>Efeito</th></tr></thead>
            <tbody>
              <tr><td><code>GRAPHFINDER_DATA_DIR</code></td><td>ver acima</td><td>Diretório base para vaults, registro e bancos.</td></tr>
              <tr><td><code>GRAPHFINDER_VAULT</code></td><td><code>&lt;dados&gt;/vault</code></td><td>Substitui a pasta do vault padrão.</td></tr>
              <tr><td><code>GRAPHFINDER_DB</code></td><td><code>&lt;dados&gt;/index.db</code></td><td>Substitui o caminho do banco SQLite padrão.</td></tr>
              <tr><td><code>GRAPHFINDER_MODEL</code> / <code>GRAPHFINDER_EMBEDDING_MODEL</code></td><td><code>auto</code></td><td>Padrão é <code>multilingual-e5-small</code> (prefixos E5 aplicados automaticamente). <code>minilm</code> carrega o legado all-MiniLM-L6-v2.</td></tr>
              <tr><td><code>GRAPHFINDER_MODELS</code></td><td>—</td><td>Pasta com <code>model.onnx</code> + <code>tokenizer.json</code> locais para uso totalmente offline (também detectado em <code>models/multilingual-e5-small</code> ou <code>models/all-MiniLM-L6-v2</code>).</td></tr>
              <tr><td><code>GRAPHFINDER_DEVICE</code></td><td><code>auto</code></td><td>Preferência de execution provider do ONNX Runtime: <code>auto</code> (DirectML no Windows, CoreML no macOS, CPU nos demais — fallback silencioso por operador), ou force <code>cpu</code>, <code>dml</code>, <code>coreml</code>, <code>cuda</code>, <code>rocm</code> (EPs de GPU degradam graciosamente para CPU).</td></tr>
              <tr><td><code>GRAPHFINDER_BATCH</code></td><td><code>16</code></td><td>Tamanho do batch de embeddings por chamada ONNX durante a indexação. Valores menores mantêm a UI responsiva em máquinas onde o ORT satura todos os núcleos físicos.</td></tr>
              <tr><td><code>GRAPHFINDER_MCP_PORT</code></td><td><code>45123</code></td><td>Porta do servidor MCP HTTP/SSE embarcado.</td></tr>
              <tr><td><code>GRAPHFINDER_AUTO_APPEND_LINKS</code></td><td><code>false</code></td><td><code>1</code>/<code>true</code> adiciona automaticamente os wikilinks sugeridos ao salvar notas.</td></tr>
            </tbody>
          </table></div>
          <div class="code"><div class="code-bar">bash</div>
<pre data-lang="bash"># Vault em português / multilíngue em uma porta personalizada
export GRAPHFINDER_MODEL=multilingual
export GRAPHFINDER_MCP_PORT=47000
graphfinder</pre></div>
          <div class="code"><div class="code-bar">powershell</div>
<pre data-lang="powershell">$env:GRAPHFINDER_MODEL = "multilingual"
graphfinder sync</pre></div>
          <div class="callout warn">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.3 3.9 1.8 18a2 2 0 0 0 1.7 3h17a2 2 0 0 0 1.7-3L13.7 3.9a2 2 0 0 0-3.4 0Z"/><path d="M12 9v4M12 17h.01"/></svg>
            <p>Trocas de modelo são detectadas automaticamente: o engine grava <code>embedding_model</code> no DB e re-embeda todas as notas e chunks de código no próximo sync. Sem rebuild manual.</p>
          </div>
        </section>
`;
