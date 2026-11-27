export default `
<section id="search">
          <h2><span class="idx">06</span>Motor de busca<a class="anchor" href="#search" aria-label="Link para Motor de busca">#</a></h2>
          <p>Cada consulta executa dois mecanismos de recuperação em paralelo e os combina:</p>
          <div class="table-wrap"><table>
            <thead><tr><th>Etapa</th><th>Detalhes</th></tr></thead>
            <tbody>
              <tr><td><strong>Vetores densos</strong></td><td>FastEmbed sobre ONNX Runtime. Modelo padrão <code>multilingual-e5-small</code> (384 dimensões, multilíngue PT/EN). Legado <code>all-MiniLM-L6-v2</code> via <code>GRAPHFINDER_MODEL=minilm</code>. Similaridade de cosseno sobre <code>note_chunks</code> / <code>code_chunks</code>.</td></tr>
              <tr><td><strong>Palavra-chave (BM25)</strong></td><td>Índice full-text SQLite FTS5 — encontra identificadores exatos, códigos de erro e termos raros que embeddings deixam passar.</td></tr>
              <tr><td><strong>Fusão</strong></td><td>Reciprocal Rank Fusion: <code>score = Σ 1 / (60 + rank)</code>. Robusto sem normalização de scores.</td></tr>
              <tr><td><strong>Saída</strong></td><td>Top-N chunks com título, seção, deep link, score de afinidade e estimativa de tokens.</td></tr>
            </tbody>
          </table></div>
          <p>O armazenamento é um único banco SQLite por projeto em <strong>modo WAL</strong>, permitindo indexação e consultas concorrentes. A reindexação é incremental: arquivos são comparados por timestamp e hash SHA-256, e apenas os alterados são reprocessados e revetorizados.</p>
        </section>
`;
