export default `
<section id="analytics">
          <h2><span class="idx">08</span>Graph analytics<a class="anchor" href="#analytics" aria-label="Link to Graph analytics">#</a></h2>
          <p>A pure-Rust analytics engine treats notes and code files as one unified graph and answers architectural questions.</p>
          <div class="table-wrap"><table>
            <thead><tr><th>Analysis</th><th>Description</th></tr></thead>
            <tbody>
              <tr><td><strong>Communities</strong></td><td>Label-propagation / Louvain-style clustering reveals modular subsystems.</td></tr>
              <tr><td><strong>Centrality &amp; God Nodes</strong></td><td>Degree centrality ranks nodes; the most connected hubs are flagged as God Nodes — refactor or document them first.</td></tr>
              <tr><td><strong>Cross-community surprises</strong></td><td>Edges that couple otherwise distant subsystems — often hidden architectural debt.</td></tr>
              <tr><td><strong>Cycles</strong></td><td>Circular dependency chains between files.</td></tr>
              <tr><td><strong>Shortest path</strong></td><td>BFS across imports and wikilinks between any two nodes (<code>graphfinder path</code>, <code>find_path</code>).</td></tr>
              <tr><td><strong>Node explanation</strong></td><td>360° card: community, centrality rank, callers, callees, siblings (<code>graphfinder explain</code>, <code>explain_node</code>).</td></tr>
            </tbody>
          </table></div>
          <h3>Exported artifacts</h3>
          <p><code>graphfinder report</code>, <code>graphfinder export</code>, <code>sync --export</code> or the <code>export_graph</code> tool write to <code>./graphfinder-out/</code>:</p>
          <ul>
            <li><code>graph.json</code> — nodes (community, degree, centrality, god-node flag) and edges (source, target, label, weight). Ideal for agents and scripts.</li>
            <li><code>GRAPH_REPORT.md</code> — executive summary, metrics, God Nodes table, subsystem list, coupling surprises and cycles.</li>
            <li><code>graph.html</code> — zero-dependency offline visualiser with search, community colours and hub halos. Open it in any browser.</li>
          </ul>
        </section>
`;
