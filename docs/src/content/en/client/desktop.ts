export default `
<section id="desktop">
          <h2><span class="idx">04</span>Desktop app<a class="anchor" href="#desktop" aria-label="Link to Desktop app">#</a></h2>
          <p>The interface is organised around a sidebar, the central graph canvas and contextual panels.</p>
          <div class="table-wrap"><table>
            <thead><tr><th>Area</th><th>What it does</th></tr></thead>
            <tbody>
              <tr><td><strong>Top bar</strong></td><td>Project switcher, new project, Omnisearch, architectural report, MCP integration modal, dark/light theme toggle (persisted in <code>localStorage</code>).</td></tr>
              <tr><td><strong>Sidebar</strong></td><td>Tabs for <em>Notes</em> and <em>Code</em> (with language filters), plus semantic search with a toggle between Markdown notes and code symbols. Toggle with <kbd>Ctrl</kbd>+<kbd>B</kbd>.</td></tr>
              <tr><td><strong>Graph view</strong></td><td>Force-directed canvas. Click a node to focus it and highlight its 1- and 2-hop neighbourhood. Filter with <code>[All]</code>, <code>[Notes]</code>, <code>[Code]</code>. Notes are coloured by tag cluster; code files get language-coded rings; hubs glow proportionally to their degree.</td></tr>
              <tr><td><strong>Note editor</strong></td><td>Markdown editor with live preview (sanitised by DOMPurify), wikilink autocomplete (type <code>[[</code>, navigate with <kbd>↑</kbd><kbd>↓</kbd>, accept with <kbd>Enter</kbd>/<kbd>Tab</kbd>, close with <kbd>Esc</kbd>) and <kbd>Ctrl</kbd>+<kbd>S</kbd> to save.</td></tr>
              <tr><td><strong>Code Inspector</strong></td><td>Slide-out drawer for a code node: declared symbols with line ranges, import graph with clickable jumps, external packages and a surgical code viewer.</td></tr>
              <tr><td><strong>Omnisearch</strong></td><td><kbd>Ctrl</kbd>/<kbd>⌘</kbd>+<kbd>K</kbd>. Hybrid search across the active project, keyboard navigable.</td></tr>
              <tr><td><strong>Architectural report</strong></td><td>Communities, God Nodes, coupling surprises and cycles for the active project, with one-click export.</td></tr>
              <tr><td><strong>MCP modal</strong></td><td>Copy-ready config snippets for each AI editor, plus the agent rules (<code>AGENTS.md</code>) and curator skill.</td></tr>
            </tbody>
          </table></div>
          <h3>Keyboard shortcuts</h3>
          <div class="table-wrap"><table>
            <thead><tr><th>Shortcut</th><th>Action</th></tr></thead>
            <tbody>
              <tr><td><kbd>Ctrl</kbd>/<kbd>⌘</kbd> + <kbd>K</kbd></td><td>Toggle Omnisearch</td></tr>
              <tr><td><kbd>Ctrl</kbd>/<kbd>⌘</kbd> + <kbd>B</kbd></td><td>Toggle sidebar</td></tr>
              <tr><td><kbd>Ctrl</kbd>/<kbd>⌘</kbd> + <kbd>S</kbd></td><td>Save current note</td></tr>
              <tr><td><kbd>↑</kbd> <kbd>↓</kbd> <kbd>Enter</kbd> <kbd>Esc</kbd></td><td>Navigate search results and wikilink suggestions</td></tr>
            </tbody>
          </table></div>
          <div class="callout info">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
            <p>While the desktop app is open it also hosts the MCP server on <code>http://127.0.0.1:45123</code>, so notes created by your agent appear on the graph in real time.</p>
          </div>
        </section>
`;
