export default `
<section class="hero" aria-labelledby="hero-title">
  <canvas id="graph-canvas" aria-hidden="true"></canvas>
  <div class="container hero-content">
    <span class="eyebrow"><span class="pill" data-version>{{VERSION}}</span> Local-first · Plain Markdown · Native MCP</span>
    <h1 id="hero-title">Your knowledge,<br /><span class="grad">wired as a graph.</span></h1>
    <p class="lead">GraphFinder turns Markdown vaults and entire codebases into a semantically indexed knowledge graph — giving humans a beautiful place to think and giving AI agents surgical, ~50-token retrieval through a native MCP server.</p>
    <div class="hero-ctas">
      <a class="btn btn-primary" id="hero-download" href="https://github.com/GabrielTBeserra/GraphFinder/releases/latest" target="_blank" rel="noopener">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3v12m0 0-5-5m5 5 5-5M4 19h16"/></svg>
        Download <span data-version>{{VERSION}}</span>
      </a>
      <a class="btn btn-ghost" id="hero-docs" href="#docs">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2V5Z"/><path d="M4 19a2 2 0 0 1 2-2h13"/></svg>
        Read the docs
      </a>
    </div>

    <div class="terminal hero-terminal">
      <div class="terminal-bar"><span class="dots"><i></i><i></i><i></i></span>terminal</div>
<pre data-lang="bash"># Index any folder (notes + code) and export graph artifacts
$ graphfinder sync ./my-repo --export
# Register GraphFinder in every AI editor on this machine
$ graphfinder config --install
# Ask the graph
$ graphfinder search "how is authentication handled?"</pre>
    </div>

    <div class="stats">
      <div class="stat"><b>19+</b><span>MCP tools</span></div>
      <div class="stat"><b>20+</b><span>languages indexed</span></div>
      <div class="stat"><b>~50</b><span>tokens per query</span></div>
      <div class="stat"><b>0</b><span>cloud calls / API keys</span></div>
    </div>
  </div>
</section>
`;
