export default `
<section id="quickstart">
          <h2><span class="idx">03</span>Início rápido<a class="anchor" href="#quickstart" aria-label="Link para Início rápido">#</a></h2>
          <ol class="steps">
            <li><strong>Abra o app</strong>Inicie o GraphFinder (ou rode <code>graphfinder</code> sem argumentos). Um vault padrão é criado e o grafo aparece.</li>
            <li><strong>Crie ou abra um projeto</strong>Use o seletor de projetos na barra superior → <em>Novo projeto</em>, dê um nome e, opcionalmente, escolha uma pasta existente (de notas ou um repositório de código).</li>
            <li><strong>Escreva uma nota</strong>Crie uma nota começando com <code># Título</code>, um resumo curto e depois seções <code>##</code>. Conecte conceitos com <code>[[Outra Nota]]</code> e termine com tags como <code>#arquitetura</code>. Salve com <kbd>Ctrl</kbd>+<kbd>S</kbd>.</li>
            <li><strong>Pesquise</strong>Pressione <kbd>Ctrl</kbd>/<kbd>⌘</kbd>+<kbd>K</kbd> para abrir o Omnisearch — busca semântica híbrida em notas e símbolos de código.</li>
            <li><strong>Conecte seu editor de IA</strong>Rode <code>graphfinder config --install</code> ou copie um snippet do modal MCP no app. Seu agente passa a ter as ferramentas MCP do GraphFinder disponíveis naquele modo.</li>
          </ol>
          <p>Prefere o terminal? O mesmo fluxo, sem interface:</p>
          <div class="code"><div class="code-bar">bash</div>
<pre data-lang="bash">cd ~/code/meu-projeto
graphfinder sync            # registra a pasta como projeto e a indexa
graphfinder status          # notas, arquivos de código, arestas, tamanho do banco
graphfinder search "middleware de rate limiting"
graphfinder report          # auditoria de arquitetura + artefatos do grafo</pre></div>
        </section>
`;
