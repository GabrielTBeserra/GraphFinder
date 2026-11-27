export default `
<section id="development">
          <h2><span class="idx">17</span>Development<a class="anchor" href="#development" aria-label="Link to Development">#</a></h2>
          <div class="table-wrap"><table>
            <thead><tr><th>Script</th><th>Purpose</th></tr></thead>
            <tbody>
              <tr><td><code>npm run dev</code></td><td>Vite dev server for the frontend only (port 1420).</td></tr>
              <tr><td><code>npm run app</code></td><td>Full desktop app in dev mode (<code>tauri dev</code>).</td></tr>
              <tr><td><code>npm run build</code></td><td>Type-check and build the frontend into <code>dist/</code>.</td></tr>
              <tr><td><code>npm run tauri build</code></td><td>Release binaries and installers.</td></tr>
              <tr><td><code>npm run lint</code></td><td>Lint with oxlint.</td></tr>
              <tr><td><code>cargo test --manifest-path src-tauri/Cargo.toml</code></td><td>Rust unit tests (parser, FTS5, embeddings math, language detection, imports, code search).</td></tr>
              <tr><td><code>npm run sync | status | search | projects | mcp</code></td><td>Run CLI commands via <code>cargo run</code>.</td></tr>
            </tbody>
          </table></div>
          <h3>Releases</h3>
          <p>Pushing a tag matching <code>v*</code> (or running the <em>Release</em> workflow manually) triggers <code>.github/workflows/release.yml</code>: <code>scripts/sync-version.js</code> stamps the tag version into <code>package.json</code>, <code>Cargo.toml</code> and <code>tauri.conf.json</code>, then <code>tauri-action</code> builds and publishes installers for Windows, macOS and Linux.</p>
          <div class="code"><div class="code-bar">bash</div>
<pre data-lang="bash">git tag v0.11.0
git push origin v0.11.0</pre></div>
          <h3>Contributing</h3>
          <p>Bug reports, features, docs and PRs are welcome. Read <a href="https://github.com/GabrielTBeserra/GraphFinder/blob/main/CONTRIBUTING.md" target="_blank" rel="noopener">CONTRIBUTING.md</a> and the <a href="https://github.com/GabrielTBeserra/GraphFinder/blob/main/CODE_OF_CONDUCT.md" target="_blank" rel="noopener">Code of Conduct</a>.</p>
        </section>
`;
