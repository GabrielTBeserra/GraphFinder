export default `
<section id="notes">
          <h2><span class="idx">05</span>Writing notes<a class="anchor" href="#notes" aria-label="Link to Writing notes">#</a></h2>
          <p>GraphFinder works with any Markdown, but retrieval is most precise when notes follow this structure:</p>
          <ol>
            <li><strong>One H1 title</strong> (<code># Note Title</code>) followed by a 2–3 line executive summary.</li>
            <li><strong>Self-contained <code>##</code>/<code>###</code> sections</strong> of roughly 40–150 tokens, with descriptive headings (<code>## 2. SQLite WAL Configuration</code> rather than <code>## Config</code>). Each heading becomes its own embedding and deep-link target.</li>
            <li><strong>Wikilinks</strong> — <code>[[Note]]</code> or <code>[[Note#Section]]</code>. Aim for 1–3 links per note to avoid orphans.</li>
            <li><strong>Lowercase hashtags</strong> — <code>#architecture #sqlite #mcp</code> at the end of sections or the note. They drive the coloured clusters in the graph.</li>
          </ol>
          <div class="code"><div class="code-bar">vault/Auth JWT.md</div>
<pre data-lang="markdown"># Auth JWT

How the API issues and validates JSON Web Tokens. Covers signing keys,
expiry policy and refresh rotation.

## 1. Token Signing
Tokens are signed with RS256. Keys rotate every 30 days and are loaded
from the secrets store at boot. See [[System Architecture#2. Secrets]].

## 2. Refresh Rotation
Each refresh invalidates the previous token family to stop replay attacks.

#auth #security #architecture</pre></div>
          <h3>How notes are parsed</h3>
          <ul>
            <li>Parsing uses a real CommonMark AST (<code>pulldown-cmark</code>). Content inside fenced code blocks is isolated, so a <code>#</code> in code is never treated as a heading or tag.</li>
            <li>Sections longer than <strong>320 tokens</strong> are windowed with a <strong>40-token overlap</strong> so no context is lost at boundaries.</li>
            <li>Renaming a note (UI or <code>rename_note</code>) rewrites every incoming <code>[[Old Title]]</code> across the vault.</li>
            <li>Notes edited outside GraphFinder (VS Code, Cursor, Neovim…) are picked up by the file watcher (500 ms debounce).</li>
          </ul>
        </section>
`;
