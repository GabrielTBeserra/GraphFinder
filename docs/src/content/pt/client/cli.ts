export default `
<section id="cli">
          <h2><span class="idx">09</span>Referência da CLI<a class="anchor" href="#cli" aria-label="Link para Referência da CLI">#</a></h2>
          <p>O mesmo binário é o app desktop e a CLI. Executado <strong>sem argumentos</strong>, abre a interface gráfica; com um comando, roda no terminal.</p>
          <div class="code"><div class="code-bar">uso</div>
<pre data-lang="bash">graphfinder [COMANDO] [OPÇÕES]</pre></div>
          <div class="table-wrap"><table>
            <thead><tr><th>Comando (aliases)</th><th>Opções</th><th>Descrição</th></tr></thead>
            <tbody>
              <tr><td><code>sync</code> (<code>syn</code>, <code>index</code>) <code>[caminho]</code></td><td><code>--json</code> <code>--export</code></td><td>Indexa notas e código em <code>caminho</code> (padrão: diretório atual). Registra um novo projeto se for desconhecido; caso contrário, atualiza incrementalmente via SHA-256. <code>--export</code> também gera os artefatos do grafo.</td></tr>
              <tr><td><code>status</code> (<code>info</code>) <code>[caminho]</code></td><td>—</td><td>Mostra notas, arquivos de código, arestas e estatísticas do banco.</td></tr>
              <tr><td><code>projects</code> (<code>list</code>)</td><td>—</td><td>Lista todos os projetos registrados e seus caminhos.</td></tr>
              <tr><td><code>search &lt;consulta&gt;</code></td><td><code>--path</code> <code>--limit &lt;n&gt;</code></td><td>Busca semântica híbrida (BM25 + embeddings) em um projeto.</td></tr>
              <tr><td><code>report</code> (<code>audit</code>) <code>[caminho]</code></td><td><code>--out-dir</code> <code>--json</code></td><td>Comunidades, God Nodes, surpresas e ciclos, além dos três artefatos.</td></tr>
              <tr><td><code>path &lt;de&gt; &lt;para&gt;</code></td><td><code>--path</code> <code>--json</code></td><td>Caminho mínimo multi-salto entre dois arquivos, símbolos ou notas.</td></tr>
              <tr><td><code>explain &lt;nó&gt;</code></td><td><code>--path</code> <code>--json</code></td><td>Cartão de contexto arquitetural de um nó.</td></tr>
              <tr><td><code>export [caminho]</code></td><td><code>--out-dir</code></td><td>Gera <code>graph.json</code>, <code>GRAPH_REPORT.md</code> e <code>graph.html</code> sem relatório no terminal.</td></tr>
              <tr><td><code>config</code> (<code>mcp-config</code>, <code>setup</code>)</td><td><code>--install</code> / <code>-i</code> <code>--json</code></td><td>Imprime a configuração MCP para editores de IA. <code>--install</code> detecta os editores instalados, registra o GraphFinder neles e adiciona a CLI ao PATH.</td></tr>
              <tr><td><code>tools</code> (<code>schemas</code>, <code>mcp-tools</code>, <code>export-schemas</code>) <code>[pasta]</code></td><td>—</td><td>Exporta os schemas JSON das 26 ferramentas, um manifesto <code>tools.json</code> e <code>instructions.md</code> para <code>./mcp-tools</code>.</td></tr>
              <tr><td><code>install-cli</code> (<code>install-path</code>)</td><td>—</td><td>Adiciona <code>graphfinder</code> ao PATH do sistema.</td></tr>
              <tr><td><code>mcp</code> (<code>--mcp</code>)</td><td>—</td><td>Inicia o servidor MCP stdio headless.</td></tr>
              <tr><td><code>help</code> (<code>--help</code>, <code>-h</code>)</td><td>—</td><td>Mostra a ajuda.</td></tr>
            </tbody>
          </table></div>
          <h3>Exemplos</h3>
          <div class="code"><div class="code-bar">bash</div>
<pre data-lang="bash"># Sync com saída legível por máquina para CI ou scripts
graphfinder sync ./services/api --json

# Como a interface chega até a camada de banco?
graphfinder path "GraphView.tsx" "db.rs"

# Por que este arquivo é importante?
graphfinder explain "src-tauri/src/engine/db.rs"

# Auditoria em uma pasta personalizada
graphfinder report --out-dir ./docs/arquitetura

# Apenas 3 resultados em um projeto específico
graphfinder search "debounce do file watcher" --path ./GraphFinder --limit 3</pre></div>
          <p>A partir do código-fonte, scripts npm encapsulam a CLI: <code>npm run sync</code>, <code>npm run status</code>, <code>npm run search</code>, <code>npm run projects</code>, <code>npm run mcp</code>.</p>
        </section>
`;
