export default `
<section id="code">
          <h2><span class="idx">07</span>Indexação de código<a class="anchor" href="#code" aria-label="Link para Indexação de código">#</a></h2>
          <p>Aponte o GraphFinder para um repositório e ele constrói um índice de símbolos e um grafo de dependências sem precisar de compilador ou language server.</p>
          <h3>Tipos de arquivo suportados</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Linguagem</th><th>Extensões</th><th>Imports analisados</th></tr></thead>
            <tbody>
              <tr><td>TypeScript / TSX</td><td><code>.ts .mts .cts .tsx</code></td><td><code>import</code>, <code>require</code></td></tr>
              <tr><td>JavaScript / JSX</td><td><code>.js .mjs .cjs .jsx</code></td><td><code>import</code>, <code>require</code></td></tr>
              <tr><td>Python</td><td><code>.py .pyi</code></td><td><code>import</code>, <code>from … import</code></td></tr>
              <tr><td>Rust</td><td><code>.rs</code></td><td><code>use</code>, <code>mod</code></td></tr>
              <tr><td>Go</td><td><code>.go</code></td><td><code>import</code></td></tr>
              <tr><td>C / C++</td><td><code>.c .h .cpp .hpp .cc .cxx .hh</code></td><td><code>#include</code></td></tr>
              <tr><td>Java / Kotlin</td><td><code>.java .kt .kts</code></td><td><code>import</code></td></tr>
              <tr><td>C# / F#</td><td><code>.cs .fs</code></td><td><code>using</code></td></tr>
              <tr><td>PHP · Ruby · Swift</td><td><code>.php .rb .swift</code></td><td><code>use</code>/<code>require</code>/<code>import</code></td></tr>
              <tr><td>Web e configuração</td><td><code>.html .css .scss .sass .less .json .yaml .toml .sql</code></td><td>indexados como arquivos</td></tr>
              <tr><td>Shell</td><td><code>.sh .bash .zsh</code></td><td><code>source</code></td></tr>
            </tbody>
          </table></div>
          <h3>O que é extraído</h3>
          <ul>
            <li><strong>Símbolos</strong> — funções, classes, structs, interfaces e enums de nível superior, com intervalos de linha e estimativa de tokens. Cada símbolo é um chunk pesquisável.</li>
            <li><strong>Imports locais</strong> — resolvidos para arquivos do projeto e salvos como arestas bidirecionais (<code>code_links</code>): <em>imports</em> e <em>dependentes</em>.</li>
            <li><strong>Pacotes externos</strong> — módulos de terceiros (ex.: <code>rusqlite</code>, <code>react</code>) listados por arquivo.</li>
          </ul>
          <h3>O que é ignorado</h3>
          <p>Tudo que está no <code>.gitignore</code>, além de <code>node_modules</code>, <code>target</code>, <code>dist</code>, <code>.git</code>, arquivos binários e arquivos maiores que <strong>1 MB</strong>.</p>
        </section>
`;
