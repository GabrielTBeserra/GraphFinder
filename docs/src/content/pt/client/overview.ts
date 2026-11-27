export default `
<section id="overview">
          <h2><span class="idx">01</span>Visão geral<a class="anchor" href="#overview" aria-label="Link para Visão geral">#</a></h2>
          <p class="lead-p">O <strong>GraphFinder</strong> é um aplicativo desktop multiplataforma (Tauri v2 + Rust + React 19) e uma ferramenta de linha de comando que indexa vaults de conhecimento em Markdown e repositórios de código em um único grafo pesquisável semanticamente.</p>
          <p>Pipelines de RAG tradicionais cortam documentos em janelas fixas arbitrárias ou jogam arquivos inteiros no contexto do LLM, desperdiçando milhares de tokens. Já as ferramentas de notas raramente oferecem busca vetorial local ou uma API utilizável por agentes. O GraphFinder resolve os dois lados:</p>
          <ul>
            <li><strong>Para humanos</strong> — um ambiente local-first e sem distrações para escrever Markdown puro com <code>[[wikilinks]]</code>, explorar conexões em um grafo 2D interativo e manter total posse dos seus arquivos.</li>
            <li><strong>Para agentes de IA</strong> — um servidor Model Context Protocol (MCP) embarcado que retorna resultados no nível de subseção (<code>[[Nota#Seção]]</code>) e símbolos de código individuais, tipicamente com cerca de <strong>50 tokens por resultado</strong>.</li>
          </ul>
          <h3>Conceitos principais</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Conceito</th><th>Significado</th></tr></thead>
            <tbody>
              <tr><td><strong>Vault</strong></td><td>Uma pasta de arquivos <code>.md</code> (e opcionalmente código) indexada pelo GraphFinder.</td></tr>
              <tr><td><strong>Projeto</strong></td><td>Um vault registrado com seu próprio índice SQLite isolado. Identificado por um slug (ex.: <code>minha-api</code>).</td></tr>
              <tr><td><strong>Chunk</strong></td><td>Uma seção atômica de uma nota delimitada por títulos <code>##</code> / <code>###</code>, vetorizada de forma independente.</td></tr>
              <tr><td><strong>Deep link</strong></td><td>Referência a uma seção específica: <code>[[Título da Nota#Título da Seção]]</code>, ou a um símbolo: <code>[[caminho/arquivo.rs#funcao]]</code>.</td></tr>
              <tr><td><strong>Nó / Aresta</strong></td><td>Notas e arquivos de código são nós; wikilinks e imports são arestas.</td></tr>
              <tr><td><strong>God Node</strong></td><td>Um hub altamente central no grafo, detectado por centralidade de grau.</td></tr>
            </tbody>
          </table></div>
        </section>
`;
