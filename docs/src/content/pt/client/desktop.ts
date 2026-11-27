export default `
<section id="desktop">
          <h2><span class="idx">04</span>Aplicativo desktop<a class="anchor" href="#desktop" aria-label="Link para Aplicativo desktop">#</a></h2>
          <p>A interface é organizada em uma barra lateral, o canvas central do grafo e painéis contextuais.</p>
          <div class="table-wrap"><table>
            <thead><tr><th>Área</th><th>O que faz</th></tr></thead>
            <tbody>
              <tr><td><strong>Barra superior</strong></td><td>Seletor de projetos, novo projeto, Omnisearch, relatório arquitetural, modal de integração MCP e alternância de tema claro/escuro (salva no <code>localStorage</code>).</td></tr>
              <tr><td><strong>Barra lateral</strong></td><td>Abas <em>Notes</em> e <em>Code</em> (com filtros de linguagem), além de busca semântica com alternância entre notas Markdown e símbolos de código. Mostrar/ocultar com <kbd>Ctrl</kbd>+<kbd>B</kbd>.</td></tr>
              <tr><td><strong>Visão de grafo</strong></td><td>Canvas force-directed. Clique em um nó para focá-lo e destacar sua vizinhança de 1 e 2 saltos. Filtre com <code>[All]</code>, <code>[Notes]</code>, <code>[Code]</code>. Notas são coloridas por cluster de tag; arquivos de código ganham anéis por linguagem; hubs brilham proporcionalmente ao grau.</td></tr>
              <tr><td><strong>Editor de notas</strong></td><td>Editor Markdown com preview ao vivo (sanitizado pelo DOMPurify), autocomplete de wikilinks (digite <code>[[</code>, navegue com <kbd>↑</kbd><kbd>↓</kbd>, aceite com <kbd>Enter</kbd>/<kbd>Tab</kbd>, feche com <kbd>Esc</kbd>) e <kbd>Ctrl</kbd>+<kbd>S</kbd> para salvar.</td></tr>
              <tr><td><strong>Code Inspector</strong></td><td>Painel deslizante para um nó de código: símbolos declarados com intervalos de linha, grafo de imports com navegação clicável, pacotes externos e visualizador de código cirúrgico.</td></tr>
              <tr><td><strong>Omnisearch</strong></td><td><kbd>Ctrl</kbd>/<kbd>⌘</kbd>+<kbd>K</kbd>. Busca híbrida no projeto ativo, navegável pelo teclado.</td></tr>
              <tr><td><strong>Relatório arquitetural</strong></td><td>Comunidades, God Nodes, acoplamentos surpresa e ciclos do projeto ativo, com exportação em um clique.</td></tr>
              <tr><td><strong>Modal MCP</strong></td><td>Snippets de configuração prontos para cada editor de IA, além das regras de agente (<code>AGENTS.md</code>) e da skill curadora.</td></tr>
            </tbody>
          </table></div>
          <h3>Atalhos de teclado</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Atalho</th><th>Ação</th></tr></thead>
            <tbody>
              <tr><td><kbd>Ctrl</kbd>/<kbd>⌘</kbd> + <kbd>K</kbd></td><td>Abrir/fechar Omnisearch</td></tr>
              <tr><td><kbd>Ctrl</kbd>/<kbd>⌘</kbd> + <kbd>B</kbd></td><td>Mostrar/ocultar barra lateral</td></tr>
              <tr><td><kbd>Ctrl</kbd>/<kbd>⌘</kbd> + <kbd>S</kbd></td><td>Salvar a nota atual</td></tr>
              <tr><td><kbd>↑</kbd> <kbd>↓</kbd> <kbd>Enter</kbd> <kbd>Esc</kbd></td><td>Navegar em resultados de busca e sugestões de wikilink</td></tr>
            </tbody>
          </table></div>
          <div class="callout info">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
            <p>Enquanto o app desktop está aberto ele também hospeda o servidor MCP em <code>http://127.0.0.1:45123</code>, então notas criadas pelo seu agente aparecem no grafo em tempo real.</p>
          </div>
        </section>
`;
