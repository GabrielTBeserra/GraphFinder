export default `
<section class="block" id="features" aria-labelledby="features-title">
  <div class="container">
    <div class="section-head reveal">
      <span class="kicker">Features</span>
      <h2 id="features-title">One engine for notes, code and agents.</h2>
      <p>GraphFinder has two self-hosted shapes: a local-first desktop core with embedded ONNX and SQLite FTS5, and a private team server with PostgreSQL/pgvector, indexing workers and token-authenticated MCP.</p>
    </div>

    <div class="bento">
      <article class="card span-4 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></svg></div>
        <h3>Hybrid semantic search</h3>
        <p>Dense 384-dimension FastEmbed vectors fused with SQLite FTS5 BM25 through Reciprocal Rank Fusion (k = 60). Notes are chunked by <code>##</code>/<code>###</code> headings so every answer is an atomic, deep-linkable section like <code>[[Note#Section]]</code>.</p>
        <div class="tagrow"><span class="tag">FastEmbed ONNX</span><span class="tag">SQLite FTS5</span><span class="tag">RRF k=60</span><span class="tag">pulldown-cmark AST</span></div>
      </article>
      <article class="card span-2 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 2v4M12 18v4M4.9 4.9l2.8 2.8M16.3 16.3l2.8 2.8M2 12h4M18 12h4"/><circle cx="12" cy="12" r="3"/></svg></div>
        <h3>100% self-hosted</h3>
        <p>No API keys, no telemetry. Run it on your laptop or inside your own private server infrastructure.</p>
      </article>
      <article class="card span-2 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="6" cy="6" r="2.5"/><circle cx="18" cy="8" r="2.5"/><circle cx="9" cy="18" r="2.5"/><path d="M8.2 7.2 15.7 7.7M7 8.4l1.4 7.2M16.4 10l-5.6 6.4"/></svg></div>
        <h3>Interactive graph</h3>
        <p>Force-directed canvas with 1/2-hop highlighting, tag clusters and language-coded code rings.</p>
      </article>
      <article class="card span-4 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M9 7 4 12l5 5M15 7l5 5-5 5"/></svg></div>
        <h3>Codebase intelligence</h3>
        <p>AST-lite scanning of entire repositories: symbols (functions, classes, structs), local import graph and external packages — respecting <code>.gitignore</code> and syncing incrementally by SHA-256.</p>
        <div class="tagrow">
          <span class="tag"><i style="background:#38bdf8"></i>TypeScript</span>
          <span class="tag"><i style="background:#f97316"></i>Rust</span>
          <span class="tag"><i style="background:#10b981"></i>Python</span>
          <span class="tag"><i style="background:#06b6d4"></i>Go</span>
          <span class="tag"><i style="background:#facc15"></i>JavaScript</span>
          <span class="tag"><i style="background:#f472b6"></i>C / C++</span>
          <span class="tag"><i style="background:#a78bfa"></i>Java · Kotlin · C#</span>
          <span class="tag"><i style="background:#94a3b8"></i>+ more</span>
        </div>
      </article>
      <article class="card span-3 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 2 3 7v10l9 5 9-5V7l-9-5Z"/><path d="M3 7l9 5 9-5M12 12v10"/></svg></div>
        <h3>Native MCP server</h3>
        <p>MCP tools, prompts and resources over local Streamable HTTP/SSE or headless stdio on desktop — and over authenticated remote HTTP in private-server mode. Tools carry read-only/destructive annotations and work with Cursor, Claude Desktop, Antigravity, Windsurf, Cline, Roo Code and Zed.</p>
      </article>
      <article class="card span-3 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M3 3v18h18"/><path d="m7 14 4-4 3 3 5-6"/></svg></div>
        <h3>Graph analytics</h3>
        <p>Community detection, degree centrality, God Nodes, cross-community surprises, cycles and shortest paths — exported as <code>graph.json</code>, <code>GRAPH_REPORT.md</code> and an offline <code>graph.html</code>.</p>
      </article>
      <article class="card span-2 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M21 12a9 9 0 1 1-3-6.7L21 8"/><path d="M21 3v5h-5"/></svg></div>
        <h3>Live file watcher</h3>
        <p>Edit in VS Code, Cursor or Neovim — changes are re-indexed within 500 ms.</p>
      </article>
      <article class="card span-2 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="m12 2 10 5-10 5L2 7l10-5Z"/><path d="m2 12 10 5 10-5M2 17l10 5 10-5"/></svg></div>
        <h3>Multi-vault projects</h3>
        <p>Isolated vaults, each with its own index. Rename refactors every incoming wikilink.</p>
      </article>
      <article class="card span-2 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 2 4 5v6c0 5 3.4 9.4 8 11 4.6-1.6 8-6 8-11V5l-8-3Z"/></svg></div>
        <h3>Hardened by default</h3>
        <p>DNS-rebinding defense, Origin checks, strict CSP and DOMPurify sanitization.</p>
      </article>
    </div>

    <div class="flow reveal" style="margin-top:48px">
      <div class="flow-step"><span class="num">01 · INGEST</span><h4>Point it at a folder</h4><p>Markdown notes and source files are parsed, chunked and embedded locally.</p></div>
      <div class="flow-step"><span class="num">02 · CONNECT</span><h4>Watch the graph form</h4><p>Wikilinks, tags and code imports become edges you can explore and analyze.</p></div>
      <div class="flow-step"><span class="num">03 · RETRIEVE</span><h4>Let agents ask precisely</h4><p>MCP tools return only the section or symbol needed — not whole files.</p></div>
    </div>
  </div>
</section>
`;
