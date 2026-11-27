export default `
<section id="architecture">
          <h2><span class="idx">15</span>Arquitetura<a class="anchor" href="#architecture" aria-label="Link para Arquitetura">#</a></h2>
          <div class="arch">
            <div class="arch-col fe"><h5>Frontend · React 19</h5><ul><li>GraphView (force-graph)</li><li>NoteEditor + preview</li><li>CodeInspector</li><li>Sidebar · Omnisearch</li><li>ProjectSwitcher · Modais</li></ul></div>
            <div class="arch-arrow"><span>Tauri IPC</span></div>
            <div class="arch-col be"><h5>Núcleo Rust · Tauri v2</h5><ul><li>Comandos IPC (lib.rs)</li><li>Servidor MCP (rmcp, axum)</li><li>Parser · Chunker</li><li>Motor de código (AST-lite)</li><li>FastEmbed · Análise de grafos</li></ul></div>
            <div class="arch-arrow"><span>I/O</span></div>
            <div class="arch-col st"><h5>Armazenamento local</h5><ul><li>Vaults Markdown</li><li>Repositórios de código</li><li>index.db (SQLite WAL + FTS5)</li><li>projects.json</li><li>.fastembed_cache</li></ul></div>
          </div>
          <h3>Estrutura do repositório</h3>
          <div class="code"><div class="code-bar">árvore</div>
<pre>GraphFinder/
├── src/                      # Frontend React 19 + TypeScript
│   ├── components/           # GraphView, NoteEditor, CodeInspector, Sidebar,
│   │                         # OmnisearchModal, McpModal, ArchitecturalReportModal…
│   ├── services/             # Bindings IPC do Tauri
│   └── types.ts
├── src-tauri/                # Backend Rust
│   ├── src/main.rs           # Ponto de entrada da CLI (GUI sem argumentos)
│   ├── src/lib.rs            # Setup do Tauri, comandos IPC, servidor MCP HTTP
│   ├── src/engine/
│   │   ├── parser.rs         # AST CommonMark, chunking, wikilinks, tags
│   │   ├── embeddings.rs     # Carregamento e inferência do FastEmbed
│   │   ├── db.rs             # Schema SQLite, FTS5, busca híbrida RRF
│   │   ├── vault.rs          # Engine: sync, watcher, ciclo de vida das notas
│   │   ├── code.rs           # Detecção de linguagem, símbolos, imports
│   │   ├── graph_analytics.rs# Comunidades, centralidade, caminhos, exportação
│   │   └── project.rs        # Gerenciador multi-vault e registro
│   └── src/mcp/              # Definição das ferramentas (mod.rs) e transportes (server.rs)
├── .agents/skills/           # Skills graphfinder e graphfinder-curator
├── mcp-tools/                # Schemas exportados das ferramentas
├── docs/                     # Este site (GitHub Pages)
└── AGENTS.md · MCP_TOOLS.md · CHANGELOG.md</pre></div>
          <h3>Stack tecnológica</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Camada</th><th>Tecnologia</th></tr></thead>
            <tbody>
              <tr><td>Runtime desktop</td><td>Tauri 2.12</td></tr>
              <tr><td>Backend</td><td>Rust 2021 · Tokio · Axum 0.7 · rmcp 3.5</td></tr>
              <tr><td>Embeddings</td><td>fastembed 4 (ONNX Runtime)</td></tr>
              <tr><td>Armazenamento</td><td>rusqlite 0.32 (SQLite embutido, WAL, FTS5)</td></tr>
              <tr><td>Parsing</td><td>pulldown-cmark 0.13 · regex · walkdir</td></tr>
              <tr><td>Monitoramento</td><td>notify-debouncer-mini 0.6</td></tr>
              <tr><td>Frontend</td><td>React 19.2 · TypeScript 6 · Vite 8 · Tailwind CSS 4</td></tr>
              <tr><td>UI</td><td>Radix UI · Phosphor Icons · force-graph · marked · DOMPurify</td></tr>
            </tbody>
          </table></div>
        </section>
`;
