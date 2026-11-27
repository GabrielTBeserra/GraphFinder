export default `
<section id="server">
          <h2><span class="idx">01</span>Servidor privado<a class="anchor" href="#server" aria-label="Link para Servidor privado">#</a></h2>
          <p class="lead-p">Além do aplicativo desktop local-first, o GraphFinder agora possui um modo de <strong>servidor privado</strong> para centralizar projetos de uma equipe. Ele mantém o mesmo princípio de recuperação cirúrgica para agentes, mas usa PostgreSQL/pgvector, filas de indexação e tokens individuais de acesso.</p>
          <p>O modelo mental muda de “cada pessoa indexa sua própria pasta” para:</p>
          <div class="code"><div class="code-bar">fluxo</div>
<pre data-lang="text">projetos centralizados
  └─ fontes Git / vault gerenciado
      └─ jobs de indexação
          └─ gerações publicadas atomicamente
              └─ busca + grafo + MCP autorizados pelo token do usuário</pre></div>

          <h3>Serviços do deployment</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Serviço</th><th>Responsabilidade</th></tr></thead>
            <tbody>
              <tr><td><strong>PostgreSQL + pgvector</strong></td><td>Metadados, usuários, sessões, tokens, jobs, chunks, embeddings e gerações de índice.</td></tr>
              <tr><td><strong>api</strong></td><td>REST administrativo, busca e endpoint MCP em <code>/mcp</code>.</td></tr>
              <tr><td><strong>worker</strong></td><td>Clona/atualiza repositórios, executa parsing, embeddings, chunks e publicação de gerações.</td></tr>
              <tr><td><strong>admin</strong></td><td>Painel web para projetos, fontes, jobs, busca, tokens e usuários.</td></tr>
              <tr><td><strong>proxy</strong></td><td>Caddy na borda, encaminhando painel e API. Em produção deve terminar TLS ou ficar atrás de outro proxy seguro.</td></tr>
            </tbody>
          </table></div>

          <h3>Fluxo diário no painel</h3>
          <ol>
            <li><strong>Entrar no painel</strong> — o login é apenas usuário e senha. Tenant não aparece na UX; ele continua apenas como conceito interno de isolamento e auditoria.</li>
            <li><strong>Criar um projeto</strong> — por exemplo <code>meu-produto</code>.</li>
            <li><strong>Cadastrar uma fonte Git</strong> — URL HTTPS e branch (<code>main</code>, <code>master</code> ou outra branch rastreada).</li>
            <li><strong>Disparar indexação</strong> — em <em>Index jobs</em>. A tela atualiza progresso aproximadamente a cada segundo.</li>
            <li><strong>Testar no Search lab</strong> — valide busca híbrida, lexical e vetorial antes de distribuir tokens.</li>
            <li><strong>Criar um token por pessoa</strong> — em <em>Access → API tokens</em>. Prefira readonly por padrão.</li>
            <li><strong>Configurar o cliente MCP</strong> — cada pessoa usa seu próprio Bearer token; permissões e projetos visíveis derivam dele.</li>
          </ol>

          <h3>Configuração MCP do servidor</h3>
          <p>O endpoint remoto usa HTTP Streamable em <code>/mcp</code> e exige Bearer token:</p>
          <div class="code"><div class="code-bar">mcp_config.json</div>
<pre data-lang="json">{
  "mcpServers": {
    "graphfinder": {
      "url": "http://SEU-SERVIDOR:18083/mcp",
      "headers": {
        "Authorization": "Bearer SEU_TOKEN"
      }
    }
  }
}</pre></div>
          <p>Se o servidor estiver atrás de HTTPS, use a origem HTTPS no lugar da porta HTTP interna. Não publique esse endpoint na internet sem TLS, proxy reverso e política de tokens.</p>

          <h3>Escopos de token</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Escopo</th><th>Permite</th></tr></thead>
            <tbody>
              <tr><td><code>search:read</code></td><td>Listar projetos autorizados, buscar notas/conteúdo, consultar grafo e status de jobs.</td></tr>
              <tr><td><code>code:read</code></td><td>Buscar código, ler símbolos, dependências e contexto de arquivos.</td></tr>
              <tr><td><code>index:trigger</code></td><td>Enfileirar reindexação de projetos/fontes autorizados.</td></tr>
              <tr><td><code>vault:write</code></td><td>Criar ou atualizar notas no vault gerenciado.</td></tr>
              <tr><td><code>vault:delete</code></td><td>Excluir notas do vault gerenciado.</td></tr>
            </tbody>
          </table></div>
          <p>Um token readonly típico usa <code>search:read,code:read</code>. Um agente operacional pode receber também <code>index:trigger</code>. Um token pode ser restrito a um projeto específico; revogar o token encerra o acesso imediatamente.</p>

          <h3>Ferramentas MCP atualmente expostas pelo servidor</h3>
          <p>O servidor privado expõe 19 ferramentas, incluindo <code>list_projects</code>, <code>search_semantic</code>, <code>search_code</code>, <code>find_files</code>, <code>get_file_context</code>, <code>read_code_chunk</code>, <code>get_compressed_context</code>, <code>get_code_dependencies</code>, <code>get_graph_neighborhood</code>, <code>find_path</code>, <code>explain_node</code>, <code>trigger_index</code>, <code>get_index_status</code> e ferramentas de vault gerenciado.</p>
          <div class="code"><div class="code-bar">pipeline recomendado</div>
<pre data-lang="text">list_projects
  → search_semantic ou search_code
  → get_file_context
  → read_code_chunk
  → get_code_dependencies / find_path quando analisar impacto</pre></div>

          <h3>Gerações e rollback</h3>
          <p>Cada indexação cria uma nova geração. A publicação é atômica: o projeto continua servindo a geração ativa anterior enquanto a nova é construída. Quando o job termina com sucesso, a nova geração entra no ar com progresso final <code>100%</code>. Em caso de regressão, uma geração anterior pode ser restaurada pelo painel/API com confirmação explícita.</p>

          <h3>Linguagens e limites do parser no servidor</h3>
          <p>O parser AST-lite do servidor atualmente extrai símbolos e imports melhor em <strong>Rust, TypeScript/JavaScript, Python e Go</strong>. Repositórios em outras linguagens ainda podem ser indexados como texto/documentos, mas o grafo de símbolos/dependências pode ficar incompleto. Para TypeScript moderno, imports relativos como <code>./modulo.js</code> são resolvidos para arquivos <code>.ts</code>/<code>.tsx</code> indexados.</p>

          <h3>Operação e segurança</h3>
          <ul>
            <li>Use PostgreSQL com volume persistente e backups; ele é o estado principal do servidor.</li>
            <li>A primeira indexação pode demorar porque o modelo ONNX de embeddings precisa ser baixado para o cache.</li>
            <li>Mantenha o painel em rede privada, VPN/Tailscale ou HTTPS com proxy reverso.</li>
            <li>Login por senha possui rate limiting persistido no PostgreSQL; OIDC/SSO pode ser configurado por variáveis de ambiente.</li>
            <li>Crie um token por pessoa/agente, com o menor escopo necessário e restrição de projeto quando aplicável.</li>
            <li>Não compartilhe tokens em repositórios, tickets ou documentos públicos.</li>
          </ul>
        </section>
`;
