export default `
<section id="security">
          <h2><span class="idx">16</span>Segurança<a class="anchor" href="#security" aria-label="Link para Segurança">#</a></h2>
          <ul>
            <li><strong>Desktop somente loopback</strong> — o servidor MCP HTTP do desktop escuta apenas em <code>127.0.0.1</code>.</li>
            <li><strong>Autorização no servidor privado</strong> — o acesso MCP remoto exige Bearer tokens individuais com restrições de projeto e escopo.</li>
            <li><strong>Defesa contra DNS rebinding</strong> — o cabeçalho <code>Host</code> é validado em cada requisição.</li>
            <li><strong>Validação de Origin</strong> — middleware estrito em vez de CORS permissivo.</li>
            <li><strong>Content Security Policy</strong> — aplicada no <code>tauri.conf.json</code>.</li>
            <li><strong>Proteção contra XSS</strong> — previews Markdown são sanitizados com DOMPurify.</li>
            <li><strong>Proteção contra path traversal</strong> — ids de projeto são sanitizados antes de tocar o sistema de arquivos.</li>
          </ul>
          <p>Reporte vulnerabilidades de forma privada conforme o <a href="https://github.com/GabrielTBeserra/GraphFinder/blob/main/SECURITY.md" target="_blank" rel="noopener">SECURITY.md</a>.</p>
        </section>
`;
