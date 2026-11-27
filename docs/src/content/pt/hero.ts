export default `
<section class="hero" aria-labelledby="hero-title">
  <canvas id="graph-canvas" aria-hidden="true"></canvas>
  <div class="container hero-content">
    <span class="eyebrow"><span class="pill" data-version>{{VERSION}}</span> Local-first · Markdown puro · MCP nativo</span>
    <h1 id="hero-title">Seu conhecimento,<br /><span class="grad">conectado como grafo.</span></h1>
    <p class="lead">O GraphFinder transforma vaults Markdown e bases de código inteiras em um grafo de conhecimento com indexação semântica — um lugar bonito para humanos pensarem e recuperação cirúrgica de ~50 tokens para agentes de IA via servidor MCP nativo.</p>
    <div class="hero-ctas">
      <a class="btn btn-primary" id="hero-download" href="https://github.com/GabrielTBeserra/GraphFinder/releases/latest" target="_blank" rel="noopener">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3v12m0 0-5-5m5 5 5-5M4 19h16"/></svg>
        Baixar <span data-version>{{VERSION}}</span>
      </a>
      <a class="btn btn-ghost" id="hero-docs" href="#docs">
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M4 5a2 2 0 0 1 2-2h13v16H6a2 2 0 0 0-2 2V5Z"/><path d="M4 19a2 2 0 0 1 2-2h13"/></svg>
        Ler a documentação
      </a>
    </div>

    <div class="terminal hero-terminal">
      <div class="terminal-bar"><span class="dots"><i></i><i></i><i></i></span>terminal</div>
<pre data-lang="bash"># Indexa qualquer pasta (notas + código) e exporta artefatos do grafo
$ graphfinder sync ./meu-repo --export
# Registra o GraphFinder em todos os editores de IA desta máquina
$ graphfinder config --install
# Pergunte ao grafo
$ graphfinder search "como a autenticação é tratada?"</pre>
    </div>

    <div class="stats">
      <div class="stat"><b>19+</b><span>ferramentas MCP</span></div>
      <div class="stat"><b>20+</b><span>linguagens indexadas</span></div>
      <div class="stat"><b>~50</b><span>tokens por consulta</span></div>
      <div class="stat"><b>0</b><span>chamadas à nuvem / API keys</span></div>
    </div>
  </div>
</section>
`;
