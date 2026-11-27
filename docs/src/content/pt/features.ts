export default `
<section class="block" id="features" aria-labelledby="features-title">
  <div class="container">
    <div class="section-head reveal">
      <span class="kicker">Recursos</span>
      <h2 id="features-title">Um motor para notas, código e agentes.</h2>
      <p>O GraphFinder tem dois formatos self-hosted: um núcleo desktop local-first com ONNX embarcado e SQLite FTS5, e um servidor privado de equipe com PostgreSQL/pgvector, workers de indexação e MCP autenticado por token.</p>
    </div>

    <div class="bento">
      <article class="card span-4 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="11" cy="11" r="7"/><path d="m20 20-3.5-3.5"/></svg></div>
        <h3>Busca semântica híbrida</h3>
        <p>Vetores densos FastEmbed de 384 dimensões combinados com BM25 do SQLite FTS5 via Reciprocal Rank Fusion (k = 60). As notas são divididas por títulos <code>##</code>/<code>###</code>, então cada resposta é uma seção atômica com deep link, como <code>[[Nota#Seção]]</code>.</p>
        <div class="tagrow"><span class="tag">FastEmbed ONNX</span><span class="tag">SQLite FTS5</span><span class="tag">RRF k=60</span><span class="tag">AST pulldown-cmark</span></div>
      </article>
      <article class="card span-2 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 2v4M12 18v4M4.9 4.9l2.8 2.8M16.3 16.3l2.8 2.8M2 12h4M18 12h4"/><circle cx="12" cy="12" r="3"/></svg></div>
        <h3>100% self-hosted</h3>
        <p>Sem API keys, sem telemetria. Rode no seu laptop ou dentro da infraestrutura do seu próprio servidor privado.</p>
      </article>
      <article class="card span-2 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><circle cx="6" cy="6" r="2.5"/><circle cx="18" cy="8" r="2.5"/><circle cx="9" cy="18" r="2.5"/><path d="M8.2 7.2 15.7 7.7M7 8.4l1.4 7.2M16.4 10l-5.6 6.4"/></svg></div>
        <h3>Grafo interativo</h3>
        <p>Canvas force-directed com destaque de vizinhança de 1/2 saltos, clusters por tag e anéis coloridos por linguagem.</p>
      </article>
      <article class="card span-4 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M9 7 4 12l5 5M15 7l5 5-5 5"/></svg></div>
        <h3>Inteligência de código</h3>
        <p>Varredura AST-lite de repositórios inteiros: símbolos (funções, classes, structs), grafo de imports locais e pacotes externos — respeitando o <code>.gitignore</code> e sincronizando de forma incremental por SHA-256.</p>
        <div class="tagrow">
          <span class="tag"><i style="background:#38bdf8"></i>TypeScript</span>
          <span class="tag"><i style="background:#f97316"></i>Rust</span>
          <span class="tag"><i style="background:#10b981"></i>Python</span>
          <span class="tag"><i style="background:#06b6d4"></i>Go</span>
          <span class="tag"><i style="background:#facc15"></i>JavaScript</span>
          <span class="tag"><i style="background:#f472b6"></i>C / C++</span>
          <span class="tag"><i style="background:#a78bfa"></i>Java · Kotlin · C#</span>
          <span class="tag"><i style="background:#94a3b8"></i>+ outras</span>
        </div>
      </article>
      <article class="card span-3 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 2 3 7v10l9 5 9-5V7l-9-5Z"/><path d="M3 7l9 5 9-5M12 12v10"/></svg></div>
        <h3>Servidor MCP nativo</h3>
        <p>Ferramentas, prompts e resources MCP via HTTP/SSE local ou stdio headless no desktop — e via HTTP remoto autenticado no modo servidor privado. Funciona com Cursor, Claude Desktop, Antigravity, Windsurf, Cline, Roo Code e Zed.</p>
      </article>
      <article class="card span-3 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M3 3v18h18"/><path d="m7 14 4-4 3 3 5-6"/></svg></div>
        <h3>Análise de grafos</h3>
        <p>Detecção de comunidades, centralidade de grau, God Nodes, acoplamentos surpresa, ciclos e caminhos mínimos — exportados como <code>graph.json</code>, <code>GRAPH_REPORT.md</code> e um <code>graph.html</code> offline.</p>
      </article>
      <article class="card span-2 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M21 12a9 9 0 1 1-3-6.7L21 8"/><path d="M21 3v5h-5"/></svg></div>
        <h3>Monitoramento ao vivo</h3>
        <p>Edite no VS Code, Cursor ou Neovim — as mudanças são reindexadas em 500 ms.</p>
      </article>
      <article class="card span-2 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="m12 2 10 5-10 5L2 7l10-5Z"/><path d="m2 12 10 5 10-5M2 17l10 5 10-5"/></svg></div>
        <h3>Múltiplos projetos</h3>
        <p>Vaults isolados, cada um com seu índice. Renomear atualiza todos os wikilinks de entrada.</p>
      </article>
      <article class="card span-2 reveal">
        <div class="ico"><svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8"><path d="M12 2 4 5v6c0 5 3.4 9.4 8 11 4.6-1.6 8-6 8-11V5l-8-3Z"/></svg></div>
        <h3>Seguro por padrão</h3>
        <p>Defesa contra DNS rebinding, validação de Origin, CSP estrita e sanitização com DOMPurify.</p>
      </article>
    </div>

    <div class="flow reveal" style="margin-top:48px">
      <div class="flow-step"><span class="num">01 · INGESTÃO</span><h4>Aponte para uma pasta</h4><p>Notas Markdown e arquivos de código são analisados, divididos e vetorizados localmente.</p></div>
      <div class="flow-step"><span class="num">02 · CONEXÃO</span><h4>Veja o grafo se formar</h4><p>Wikilinks, tags e imports viram arestas que você pode explorar e analisar.</p></div>
      <div class="flow-step"><span class="num">03 · RECUPERAÇÃO</span><h4>Agentes perguntam com precisão</h4><p>As ferramentas MCP retornam só a seção ou símbolo necessário — não arquivos inteiros.</p></div>
    </div>
  </div>
</section>
`;
