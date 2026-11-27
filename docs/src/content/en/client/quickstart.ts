export default `
<section id="quickstart">
          <h2><span class="idx">03</span>Quick start<a class="anchor" href="#quickstart" aria-label="Link to Quick start">#</a></h2>
          <ol class="steps">
            <li><strong>Open the app</strong>Launch GraphFinder (or run <code>graphfinder</code> with no arguments). A default vault is created and the graph appears.</li>
            <li><strong>Create or open a project</strong>Use the project switcher in the top bar → <em>New project</em>, give it a name and optionally pick an existing folder (a notes folder or a code repository).</li>
            <li><strong>Write a note</strong>Create a note, start with <code># Title</code>, a short summary, then <code>##</code> sections. Link concepts with <code>[[Other Note]]</code> and finish with tags like <code>#architecture</code>. Save with <kbd>Ctrl</kbd>+<kbd>S</kbd>.</li>
            <li><strong>Search</strong>Press <kbd>Ctrl</kbd>/<kbd>⌘</kbd>+<kbd>K</kbd> for Omnisearch — hybrid semantic search across notes and code symbols.</li>
            <li><strong>Connect your AI editor</strong>Run <code>graphfinder config --install</code> or copy a snippet from the in-app MCP modal. Your agent now has the GraphFinder MCP tools available in that mode.</li>
          </ol>
          <p>Prefer the terminal? The same flow headless:</p>
          <div class="code"><div class="code-bar">bash</div>
<pre data-lang="bash">cd ~/code/my-project
graphfinder sync            # registers the folder as a project and indexes it
graphfinder status          # notes, code files, edges, database size
graphfinder search "rate limiting middleware"
graphfinder report          # architecture audit + graph artifacts</pre></div>
        </section>
`;
