export default `
<section id="tools">
          <h2><span class="idx">11</span>Ferramentas MCP<a class="anchor" href="#tools" aria-label="Link para Ferramentas MCP">#</a></h2>
          <p>Todas as ferramentas aceitam o parâmetro opcional <code>project</code> para mirar um vault específico sem contaminação entre projetos. Parâmetros tracejados são opcionais.</p>

          <p class="tool-group">Recuperação de conhecimento</p>
          <div class="tools">
            <div class="tool"><h4>search_semantic</h4><p>Busca híbrida vetor + BM25 sobre chunks de notas com RRF. Retorna deep links e estimativa de tokens.</p><div class="params"><span>query</span><span class="opt">limit=5</span><span class="opt">format: dense_chunks | summaries</span></div></div>
            <div class="tool"><h4>read_section</h4><p>Lê exatamente uma seção pelo título ou fragmento de deep link.</p><div class="params"><span>target</span><span>section</span></div></div>
            <div class="tool"><h4>get_compressed_context</h4><p>Resumo, links, tags e mapa de títulos com custo de tokens por seção — menos de 80 tokens.</p><div class="params"><span>target</span></div></div>
            <div class="tool"><h4>list_recent_notes</h4><p>Notas modificadas recentemente com resumos de uma linha e tags.</p><div class="params"><span class="opt">limit=10</span></div></div>
          </div>

          <p class="tool-group">Autoria e conexões</p>
          <div class="tools">
            <div class="tool"><h4>suggest_links</h4><p>Vetoriza um rascunho e recomenda notas/seções existentes para linkar (evita órfãos).</p><div class="params"><span>text</span><span class="opt">current_note</span><span class="opt">threshold=0.65</span></div></div>
            <div class="tool"><h4>store_context</h4><p>Cria ou atualiza uma nota, faz o parsing e a indexa imediatamente.</p><div class="params"><span>title</span><span>content</span><span class="opt">tags[]</span><span class="opt">links[]</span></div></div>
            <div class="tool"><h4>link_notes</h4><p>Adiciona um <code>[[wikilink]]</code> de uma nota para outra e atualiza o grafo.</p><div class="params"><span>from_note</span><span>to_note</span></div></div>
            <div class="tool"><h4>rename_note</h4><p>Renomeia uma nota e refatora todos os wikilinks de entrada no vault.</p><div class="params"><span>old_title</span><span>new_title</span></div></div>
            <div class="tool"><h4>delete_note</h4><p>Apaga o arquivo e limpa chunks, embeddings, links e tags.</p><div class="params"><span>target</span></div></div>
          </div>

          <p class="tool-group">Grafo e arquitetura</p>
          <div class="tools">
            <div class="tool"><h4>get_graph_neighborhood</h4><p>Nós conectados em 1 ou 2 saltos, sem ler o conteúdo.</p><div class="params"><span>target</span><span class="opt">depth=1</span></div></div>
            <div class="tool"><h4>find_path</h4><p>Caminho mínimo (BFS) por wikilinks e imports entre dois nós.</p><div class="params"><span>source</span><span>target</span><span class="opt">directed=false</span></div></div>
            <div class="tool"><h4>explain_node</h4><p>Centralidade, comunidade, status de God Node, dependentes, dependências e irmãos.</p><div class="params"><span>target</span></div></div>
            <div class="tool"><h4>get_community_report</h4><p>Auditoria do sistema: comunidades, God Nodes, surpresas e ciclos.</p><div class="params"><span class="opt">project</span></div></div>
            <div class="tool"><h4>export_graph</h4><p>Gera <code>graph.json</code>, <code>GRAPH_REPORT.md</code> e <code>graph.html</code>.</p><div class="params"><span class="opt">out_dir=./graphfinder-out</span></div></div>
            <div class="tool"><h4>get_vault_stats</h4><p>Notas, chunks, links, tags e estimativa total de tokens.</p><div class="params"><span class="opt">project</span></div></div>
          </div>

          <p class="tool-group">Base de código</p>
          <div class="tools">
            <div class="tool"><h4>search_code</h4><p>Busca híbrida em símbolos, assinaturas e comentários. Retorna <code>[[arquivo#simbolo]]</code> com intervalos de linha.</p><div class="params"><span>query</span><span class="opt">language</span><span class="opt">limit=5</span></div></div>
            <div class="tool"><h4>find_files</h4><p>Busca por glob / substring nos caminhos indexados.</p><div class="params"><span>pattern</span><span class="opt">limit=20</span></div></div>
            <div class="tool"><h4>get_file_context</h4><p>Símbolos com linhas, imports locais, dependentes e pacotes — menos de 80 tokens.</p><div class="params"><span>file</span></div></div>
            <div class="tool"><h4>read_code_chunk</h4><p>Retorna apenas o corpo de uma função, classe ou struct.</p><div class="params"><span>file</span><span>symbol</span></div></div>
            <div class="tool"><h4>get_code_dependencies</h4><p>O que um arquivo importa e quem o importa.</p><div class="params"><span>file</span></div></div>
          </div>

          <p class="tool-group">Projetos e sincronização</p>
          <div class="tools">
            <div class="tool"><h4>list_projects</h4><p>Todos os vaults registrados com caminhos e contagem de notas.</p><div class="params"><span class="opt">—</span></div></div>
            <div class="tool"><h4>create_project</h4><p>Cria um vault isolado (opcionalmente em qualquer caminho do disco).</p><div class="params"><span>name</span><span class="opt">path</span></div></div>
            <div class="tool"><h4>switch_project</h4><p>Troca o vault padrão ativo.</p><div class="params"><span>project_id</span></div></div>
            <div class="tool"><h4>sync_vault</h4><p>Reindexação incremental por hash; um <code>path</code> é mapeado (ou registrado) automaticamente como projeto.</p><div class="params"><span class="opt">path</span><span class="opt">project</span></div></div>
          </div>
          <p style="margin-top:20px">Rode <code>graphfinder tools</code> para exportar os schemas JSON de todas as ferramentas e usá-los com qualquer framework de agentes.</p>
        </section>
`;
