export default `
<section id="projects">
          <h2><span class="idx">13</span>Projects &amp; vaults<a class="anchor" href="#projects" aria-label="Link to Projects">#</a></h2>
          <p>Each project is an isolated workspace with its own Markdown directory and its own SQLite vector database, so searches never leak between projects.</p>
          <ul>
            <li><strong>Default vault</strong> — <code>&lt;data dir&gt;/vault</code> with index <code>&lt;data dir&gt;/index.db</code>.</li>
            <li><strong>Managed vaults</strong> — <code>&lt;data dir&gt;/vaults/&lt;project_id&gt;/</code>, created from the UI or <code>create_project</code>.</li>
            <li><strong>External folders</strong> — any path on disk (e.g. a code repo) registered via the UI, <code>graphfinder sync &lt;path&gt;</code> or <code>create_project(name, path)</code>.</li>
            <li><strong>Registry</strong> — <code>vaults/projects.json</code> stores the active project, custom names and custom paths.</li>
          </ul>
          <div class="code"><div class="code-bar">vaults/projects.json</div>
<pre data-lang="json">{
  "active_project": "default",
  "custom_names": { "my-api": "My API" },
  "custom_paths": { "my-api": "C:/Repos/MyAPI" }
}</pre></div>
          <h3>Where is the data directory?</h3>
          <ol>
            <li><code>GRAPHFINDER_DATA_DIR</code>, if set.</li>
            <li>The current directory, if it contains <code>package.json</code> (source checkout / dev mode).</li>
            <li>Otherwise the OS data folder: <code>%APPDATA%\\GraphFinder</code> (Windows), <code>~/Library/Application Support/GraphFinder</code> (macOS), <code>~/.local/share/GraphFinder</code> (Linux).</li>
          </ol>
        </section>
`;
