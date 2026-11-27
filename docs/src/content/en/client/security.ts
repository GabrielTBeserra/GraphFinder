export default `
<section id="security">
          <h2><span class="idx">16</span>Security<a class="anchor" href="#security" aria-label="Link to Security">#</a></h2>
          <ul>
            <li><strong>Desktop loopback only</strong> — the desktop MCP HTTP server binds to <code>127.0.0.1</code>.</li>
            <li><strong>Private-server authorization</strong> — remote MCP access requires per-user Bearer tokens with project and scope restrictions.</li>
            <li><strong>DNS-rebinding defense</strong> — the <code>Host</code> header is validated on every request.</li>
            <li><strong>Origin validation</strong> — strict middleware instead of permissive CORS.</li>
            <li><strong>Content Security Policy</strong> — enforced in <code>tauri.conf.json</code>.</li>
            <li><strong>XSS protection</strong> — Markdown previews are sanitised with DOMPurify.</li>
            <li><strong>Path traversal protection</strong> — project ids are sanitised before touching the filesystem.</li>
          </ul>
          <p>Report vulnerabilities privately as described in <a href="https://github.com/GabrielTBeserra/GraphFinder/blob/main/SECURITY.md" target="_blank" rel="noopener">SECURITY.md</a>.</p>
        </section>
`;
