export default `
<section id="analytics">
          <h2><span class="idx">08</span>Análise de grafos<a class="anchor" href="#analytics" aria-label="Link para Análise de grafos">#</a></h2>
          <p>Um motor de análise escrito em Rust puro trata notas e arquivos de código como um grafo unificado e responde perguntas de arquitetura.</p>
          <div class="table-wrap"><table>
            <thead><tr><th>Análise</th><th>Descrição</th></tr></thead>
            <tbody>
              <tr><td><strong>Comunidades</strong></td><td>Clustering por propagação de rótulos / estilo Louvain revela subsistemas modulares.</td></tr>
              <tr><td><strong>Centralidade e God Nodes</strong></td><td>A centralidade de grau ranqueia os nós; os hubs mais conectados são marcados como God Nodes — refatore ou documente-os primeiro.</td></tr>
              <tr><td><strong>Acoplamentos surpresa</strong></td><td>Arestas que acoplam subsistemas distantes — frequentemente dívida arquitetural oculta.</td></tr>
              <tr><td><strong>Ciclos</strong></td><td>Cadeias de dependência circular entre arquivos.</td></tr>
              <tr><td><strong>Caminho mínimo</strong></td><td>BFS por imports e wikilinks entre dois nós quaisquer (<code>graphfinder path</code>, <code>find_path</code>).</td></tr>
              <tr><td><strong>Explicação de nó</strong></td><td>Cartão 360°: comunidade, ranking de centralidade, quem chama, quem é chamado, irmãos (<code>graphfinder explain</code>, <code>explain_node</code>).</td></tr>
            </tbody>
          </table></div>
          <h3>Artefatos exportados</h3>
          <p><code>graphfinder report</code>, <code>graphfinder export</code>, <code>sync --export</code> ou a ferramenta <code>export_graph</code> gravam em <code>./graphfinder-out/</code>:</p>
          <ul>
            <li><code>graph.json</code> — nós (comunidade, grau, centralidade, flag de god-node) e arestas (origem, destino, rótulo, peso). Ideal para agentes e scripts.</li>
            <li><code>GRAPH_REPORT.md</code> — resumo executivo, métricas, tabela de God Nodes, lista de subsistemas, acoplamentos surpresa e ciclos.</li>
            <li><code>graph.html</code> — visualizador offline sem dependências, com busca, cores por comunidade e halos nos hubs. Abra em qualquer navegador.</li>
          </ul>
        </section>
`;
