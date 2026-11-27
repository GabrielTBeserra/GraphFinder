export default `
<section id="faq">
          <h2><span class="idx">18</span>Problemas e FAQ<a class="anchor" href="#faq" aria-label="Link para FAQ">#</a></h2>
          <h4>Meu agente não conecta em <code>127.0.0.1:45123</code>.</h4>
          <p>O endpoint HTTP/SSE só existe enquanto o app desktop está aberto. Abra o app ou mude o cliente para o modo stdio (<code>"command": "graphfinder", "args": ["--mcp"]</code>). Se alterou <code>GRAPHFINDER_MCP_PORT</code>, atualize a URL.</p>
          <h4>O MCP do servidor retorna <code>insufficient project scope</code>.</h4>
          <p>Isso é um erro de autorização da ferramenta, não uma queda de conexão. O token não tem o escopo exigido (por exemplo <code>index:trigger</code>) ou está restrito a outro projeto. Crie ou atualize o token da pessoa em <em>Access → API tokens</em>.</p>
          <h4><code>graphfinder</code> não é reconhecido no terminal.</h4>
          <p>Rode <code>GraphFinder.exe install-cli</code> (ou <code>graphfinder config --install</code>) e abra um novo terminal para carregar o PATH atualizado.</p>
          <h4>Os resultados são ruins para português / outros idiomas.</h4>
          <p>Defina <code>GRAPHFINDER_MODEL=multilingual</code> e reindexe.</p>
          <h4>A primeira inicialização é lenta.</h4>
          <p>O modelo de embeddings é baixado uma vez. Nas próximas execuções ele é carregado de <code>.fastembed_cache</code>; aponte <code>GRAPHFINDER_MODELS</code> para uma cópia local em máquinas sem internet.</p>
          <h4>Alguns arquivos não aparecem no grafo de código.</h4>
          <p>Verifique se estão no <code>.gitignore</code>, se são binários, se passam de 1 MB ou se têm extensão não suportada. Rode <code>graphfinder sync</code> para forçar uma atualização incremental.</p>
          <h4>Algum dado sai da minha máquina?</h4>
          <p>No modo desktop, embeddings, busca e análises rodam localmente; o único acesso à rede é o download único do modelo. No modo servidor privado, indexação e embeddings rodam no seu servidor self-hosted, não em uma nuvem de terceiros.</p>
        </section>
`;
