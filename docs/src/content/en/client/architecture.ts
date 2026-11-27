export default `
<section id="architecture">
          <h2><span class="idx">15</span>Architecture<a class="anchor" href="#architecture" aria-label="Link to Architecture">#</a></h2>
          <div class="arch">
            <div class="arch-col fe"><h5>Frontend · React 19</h5><ul><li>GraphView (force-graph)</li><li>NoteEditor + preview</li><li>CodeInspector</li><li>Sidebar · Omnisearch</li><li>ProjectSwitcher · Modals</li></ul></div>
            <div class="arch-arrow"><span>Tauri IPC</span></div>
            <div class="arch-col be"><h5>Rust core · Tauri v2</h5><ul><li>IPC commands (lib.rs)</li><li>MCP server (rmcp, axum)</li><li>Parser · Chunker</li><li>Code engine (AST-lite)</li><li>FastEmbed · Graph analytics</li></ul></div>
            <div class="arch-arrow"><span>I/O</span></div>
            <div class="arch-col st"><h5>Local storage</h5><ul><li>Markdown vaults</li><li>Code repositories</li><li>index.db (SQLite WAL + FTS5)</li><li>projects.json</li><li>.fastembed_cache</li></ul></div>
          </div>
          <h3>Repository layout</h3>
          <div class="code"><div class="code-bar">tree</div>
<pre>GraphFinder/
├── src/                      # React 19 + TypeScript frontend
│   ├── components/           # GraphView, NoteEditor, CodeInspector, Sidebar,
│   │                         # OmnisearchModal, McpModal, ArchitecturalReportModal…
│   ├── services/             # Tauri IPC bindings
│   └── types.ts
├── src-tauri/                # Rust backend
│   ├── src/main.rs           # CLI entry point (GUI when no args)
│   ├── src/lib.rs            # Tauri setup, IPC commands, embedded MCP HTTP server
│   ├── src/engine/
│   │   ├── parser.rs         # CommonMark AST, chunking, wikilinks, tags
│   │   ├── embeddings.rs     # FastEmbed model loading & inference
│   │   ├── db.rs             # SQLite schema, FTS5, hybrid RRF search
│   │   ├── vault.rs          # Engine: sync, watcher, note lifecycle
│   │   ├── code.rs           # Language detection, symbols, imports
│   │   ├── graph_analytics.rs# Communities, centrality, paths, exports
│   │   └── project.rs        # Multi-vault manager & registry
│   └── src/mcp/              # MCP tool definitions (mod.rs) & transports (server.rs)
├── .agents/skills/           # graphfinder & graphfinder-curator skills
├── mcp-tools/                # Exported tool schemas
├── docs/                     # This website (GitHub Pages)
└── AGENTS.md · MCP_TOOLS.md · CHANGELOG.md</pre></div>
          <h3>Tech stack</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Layer</th><th>Technology</th></tr></thead>
            <tbody>
              <tr><td>Desktop runtime</td><td>Tauri 2.12</td></tr>
              <tr><td>Backend</td><td>Rust 2021 · Tokio · Axum 0.7 · rmcp 3.5</td></tr>
              <tr><td>Embeddings</td><td>fastembed 4 (ONNX Runtime)</td></tr>
              <tr><td>Storage</td><td>rusqlite 0.32 (bundled SQLite, WAL, FTS5)</td></tr>
              <tr><td>Parsing</td><td>pulldown-cmark 0.13 · regex · walkdir</td></tr>
              <tr><td>File watching</td><td>notify-debouncer-mini 0.6</td></tr>
              <tr><td>Frontend</td><td>React 19.2 · TypeScript 6 · Vite 8 · Tailwind CSS 4</td></tr>
              <tr><td>UI</td><td>Radix UI · Phosphor Icons · force-graph · marked · DOMPurify</td></tr>
            </tbody>
          </table></div>
        </section>
`;
