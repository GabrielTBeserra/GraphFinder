export default `
<section id="search">
          <h2><span class="idx">06</span>Search engine<a class="anchor" href="#search" aria-label="Link to Search engine">#</a></h2>
          <p>Every query runs two retrievers in parallel and merges them:</p>
          <div class="table-wrap"><table>
            <thead><tr><th>Stage</th><th>Details</th></tr></thead>
            <tbody>
              <tr><td><strong>Dense vectors</strong></td><td>FastEmbed on ONNX Runtime. Default model <code>multilingual-e5-small</code> (384 dims, multilingual PT/EN). Legacy <code>all-MiniLM-L6-v2</code> via <code>GRAPHFINDER_MODEL=minilm</code>. Cosine similarity over <code>note_chunks</code> / <code>code_chunks</code>.</td></tr>
              <tr><td><strong>Keyword (BM25)</strong></td><td>SQLite FTS5 full-text index — catches exact identifiers, error codes and rare terms that embeddings miss.</td></tr>
              <tr><td><strong>Fusion</strong></td><td>Reciprocal Rank Fusion: <code>score = Σ 1 / (60 + rank)</code>. Robust without score normalisation.</td></tr>
              <tr><td><strong>Output</strong></td><td>Top-N chunks with title, heading, deep link, affinity score and token estimate.</td></tr>
            </tbody>
          </table></div>
          <p>Storage is a single SQLite database per project running in <strong>WAL mode</strong> for concurrent indexing and querying. Re-indexing is incremental: files are compared by timestamp and SHA-256 hash, and only changed files are re-parsed and re-embedded.</p>
        </section>
`;
