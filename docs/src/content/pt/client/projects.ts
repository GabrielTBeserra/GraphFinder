export default `
<section id="projects">
          <h2><span class="idx">13</span>Projetos e vaults<a class="anchor" href="#projects" aria-label="Link para Projetos">#</a></h2>
          <p>Cada projeto é um workspace isolado com seu próprio diretório Markdown e seu próprio banco vetorial SQLite, então as buscas nunca vazam entre projetos.</p>
          <ul>
            <li><strong>Vault padrão</strong> — <code>&lt;pasta de dados&gt;/vault</code> com índice em <code>&lt;pasta de dados&gt;/index.db</code>.</li>
            <li><strong>Vaults gerenciados</strong> — <code>&lt;pasta de dados&gt;/vaults/&lt;project_id&gt;/</code>, criados pela interface ou por <code>create_project</code>.</li>
            <li><strong>Pastas externas</strong> — qualquer caminho do disco (ex.: um repositório) registrado pela interface, por <code>graphfinder sync &lt;caminho&gt;</code> ou <code>create_project(name, path)</code>.</li>
            <li><strong>Registro</strong> — <code>vaults/projects.json</code> guarda o projeto ativo, nomes e caminhos personalizados.</li>
          </ul>
          <div class="code"><div class="code-bar">vaults/projects.json</div>
<pre data-lang="json">{
  "active_project": "default",
  "custom_names": { "minha-api": "Minha API" },
  "custom_paths": { "minha-api": "C:/Repos/MinhaAPI" }
}</pre></div>
          <h3>Onde fica a pasta de dados?</h3>
          <ol>
            <li><code>GRAPHFINDER_DATA_DIR</code>, se definida.</li>
            <li>O diretório atual, se contiver <code>package.json</code> (código-fonte / modo dev).</li>
            <li>Caso contrário, a pasta de dados do sistema: <code>%APPDATA%\\GraphFinder</code> (Windows), <code>~/Library/Application Support/GraphFinder</code> (macOS), <code>~/.local/share/GraphFinder</code> (Linux).</li>
          </ol>
        </section>
`;
