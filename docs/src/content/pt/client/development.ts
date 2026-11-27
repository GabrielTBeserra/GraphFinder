export default `
<section id="development">
          <h2><span class="idx">17</span>Desenvolvimento<a class="anchor" href="#development" aria-label="Link para Desenvolvimento">#</a></h2>
          <div class="table-wrap"><table>
            <thead><tr><th>Script</th><th>Finalidade</th></tr></thead>
            <tbody>
              <tr><td><code>npm run dev</code></td><td>Servidor Vite apenas do frontend (porta 1420).</td></tr>
              <tr><td><code>npm run app</code></td><td>App desktop completo em modo dev (<code>tauri dev</code>).</td></tr>
              <tr><td><code>npm run build</code></td><td>Checagem de tipos e build do frontend em <code>dist/</code>.</td></tr>
              <tr><td><code>npm run tauri build</code></td><td>Binários e instaladores de release.</td></tr>
              <tr><td><code>npm run lint</code></td><td>Lint com oxlint.</td></tr>
              <tr><td><code>cargo test --manifest-path src-tauri/Cargo.toml</code></td><td>Testes unitários Rust (parser, FTS5, matemática de embeddings, detecção de linguagem, imports, busca de código).</td></tr>
              <tr><td><code>npm run sync | status | search | projects | mcp</code></td><td>Executa comandos da CLI via <code>cargo run</code>.</td></tr>
            </tbody>
          </table></div>
          <h3>Releases</h3>
          <p>Enviar uma tag no formato <code>v*</code> (ou rodar o workflow <em>Release</em> manualmente) dispara o <code>.github/workflows/release.yml</code>: o <code>scripts/sync-version.js</code> aplica a versão da tag em <code>package.json</code>, <code>Cargo.toml</code> e <code>tauri.conf.json</code>, e então o <code>tauri-action</code> compila e publica instaladores para Windows, macOS e Linux.</p>
          <div class="code"><div class="code-bar">bash</div>
<pre data-lang="bash">git tag v0.11.0
git push origin v0.11.0</pre></div>
          <h3>Contribuindo</h3>
          <p>Relatos de bugs, sugestões, documentação e PRs são bem-vindos. Leia o <a href="https://github.com/GabrielTBeserra/GraphFinder/blob/main/CONTRIBUTING.md" target="_blank" rel="noopener">CONTRIBUTING.md</a> e o <a href="https://github.com/GabrielTBeserra/GraphFinder/blob/main/CODE_OF_CONDUCT.md" target="_blank" rel="noopener">Código de Conduta</a>.</p>
        </section>
`;
