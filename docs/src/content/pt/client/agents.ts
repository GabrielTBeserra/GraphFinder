export default `
<section id="agents">
          <h2><span class="idx">12</span>Agentes de IA e skills<a class="anchor" href="#agents" aria-label="Link para Agentes de IA">#</a></h2>
          <p>O GraphFinder traz três camadas que trabalham juntas: <strong>o agente</strong> (Cursor, Claude, Antigravity…) raciocina e planeja, <strong>a skill</strong> impõe economia de tokens e qualidade das notas, e <strong>o servidor MCP</strong> executa buscas e escritas.</p>
          <h3>Skills e regras incluídas</h3>
          <ul>
            <li><code>AGENTS.md</code> — regras do repositório para agentes (economia de tokens, schema de notas, pipelines de decisão). O Antigravity carrega automaticamente; no Cursor adicione às Project Rules; no Claude cole no prompt do projeto.</li>
            <li><code>.agents/skills/graphfinder-curator</code> — fluxo de autoria e curadoria (resumos, seções atômicas, <code>suggest_links</code> antes de <code>store_context</code>, tags).</li>
            <li><code>.agents/skills/graphfinder</code> — fluxo de auditoria de arquitetura (relatórios, caminhos, explain, exportações).</li>
          </ul>
          <h3>Fluxos recomendados</h3>
          <h4>Respondendo uma pergunta</h4>
          <div class="code"><div class="code-bar">fluxo</div>
<pre>search_semantic("sqlite wal mode")        → top 5 chunks (~50 tokens cada)
  └─ precisa da estrutura?  get_compressed_context("Arquitetura do Sistema")
  └─ precisa de uma seção?  read_section("Arquitetura do Sistema", "1. Armazenamento Local")</pre></div>
          <h4>Salvando conhecimento</h4>
          <div class="code"><div class="code-bar">fluxo</div>
<pre>rascunho (# Título + resumo + seções ##)
  → suggest_links(rascunho)                → ex.: "Arquitetura do Sistema: 85%"
  → inserir [[Arquitetura do Sistema]] no rascunho
  → store_context(title, content, tags)    → arquivo gravado, indexado, grafo atualizado</pre></div>
          <h4>Inspecionando código</h4>
          <div class="code"><div class="code-bar">fluxo</div>
<pre>search_code("hybrid_search", language="rust")
  → get_file_context("src-tauri/src/engine/db.rs")       # mapa de símbolos, &lt;80 tokens
  → read_code_chunk("src-tauri/src/engine/db.rs", "hybrid_search")
  → get_code_dependencies("src-tauri/src/engine/db.rs")  # análise de impacto</pre></div>
          <h4>Auditando a arquitetura</h4>
          <div class="code"><div class="code-bar">fluxo</div>
<pre>get_community_report()      → subsistemas, God Nodes, surpresas, ciclos
  → explain_node("db.rs")   → por que importa
  → find_path("App.tsx", "db.rs")</pre></div>
          <h3>Exemplos de prompts</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Objetivo</th><th>Prompt</th></tr></thead>
            <tbody>
              <tr><td>Busca cirúrgica</td><td><em>"Consulte o vault do GraphFinder e explique como funciona a indexação de código sem ler arquivos inteiros."</em></td></tr>
              <tr><td>Nota conectada</td><td><em>"Documente nosso fluxo OAuth2 como uma nota no GraphFinder. Chame suggest_links antes e adicione tags relevantes."</em></td></tr>
              <tr><td>Inspeção de código</td><td><em>"Encontre onde o parsing de imports é implementado e leia só essa função com read_code_chunk."</em></td></tr>
              <tr><td>Análise de impacto</td><td><em>"Quais arquivos importam GraphView.tsx? Resuma o impacto arquitetural de alterá-lo."</em></td></tr>
              <tr><td>Auditoria</td><td><em>"Gere um community report do GraphFinder e liste os God Nodes que devemos documentar primeiro."</em></td></tr>
            </tbody>
          </table></div>
        </section>
`;
