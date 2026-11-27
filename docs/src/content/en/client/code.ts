export default `
<section id="code">
          <h2><span class="idx">07</span>Code indexing<a class="anchor" href="#code" aria-label="Link to Code indexing">#</a></h2>
          <p>Point GraphFinder at a repository and it builds a symbol index and dependency graph without needing a compiler or language server.</p>
          <h3>Supported file types</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Language</th><th>Extensions</th><th>Imports parsed</th></tr></thead>
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
              <tr><td>Web &amp; config</td><td><code>.html .css .scss .sass .less .json .yaml .toml .sql</code></td><td>indexed as files</td></tr>
              <tr><td>Shell</td><td><code>.sh .bash .zsh</code></td><td><code>source</code></td></tr>
            </tbody>
          </table></div>
          <h3>What gets extracted</h3>
          <ul>
            <li><strong>Symbols</strong> — top-level functions, classes, structs, interfaces, enums, with line ranges and token estimates. Each symbol is a searchable chunk.</li>
            <li><strong>Local imports</strong> — resolved to project files and stored as bidirectional edges (<code>code_links</code>): <em>imports</em> and <em>dependents</em>.</li>
            <li><strong>External packages</strong> — third-party modules (e.g. <code>rusqlite</code>, <code>react</code>) listed per file.</li>
          </ul>
          <h3>What gets skipped</h3>
          <p>Anything in <code>.gitignore</code>, plus <code>node_modules</code>, <code>target</code>, <code>dist</code>, <code>.git</code>, binary files and files larger than <strong>1 MB</strong>.</p>
        </section>
`;
