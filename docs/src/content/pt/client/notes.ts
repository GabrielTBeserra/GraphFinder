export default `
<section id="notes">
          <h2><span class="idx">05</span>Escrevendo notas<a class="anchor" href="#notes" aria-label="Link para Escrevendo notas">#</a></h2>
          <p>O GraphFinder funciona com qualquer Markdown, mas a recuperação fica mais precisa quando as notas seguem esta estrutura:</p>
          <ol>
            <li><strong>Um único título H1</strong> (<code># Título da Nota</code>) seguido de um resumo executivo de 2–3 linhas.</li>
            <li><strong>Seções <code>##</code>/<code>###</code> autocontidas</strong> de aproximadamente 40–150 tokens, com títulos descritivos (<code>## 2. Configuração do SQLite WAL</code> em vez de <code>## Config</code>). Cada título vira um embedding próprio e um alvo de deep link.</li>
            <li><strong>Wikilinks</strong> — <code>[[Nota]]</code> ou <code>[[Nota#Seção]]</code>. Mire em 1–3 links por nota para evitar órfãos.</li>
            <li><strong>Hashtags em minúsculas</strong> — <code>#arquitetura #sqlite #mcp</code> no fim das seções ou da nota. Elas geram os clusters coloridos no grafo.</li>
          </ol>
          <div class="code"><div class="code-bar">vault/Auth JWT.md</div>
<pre data-lang="markdown"># Auth JWT

Como a API emite e valida JSON Web Tokens. Cobre chaves de assinatura,
política de expiração e rotação de refresh.

## 1. Assinatura de Tokens
Os tokens são assinados com RS256. As chaves giram a cada 30 dias e são
carregadas do cofre de segredos na inicialização. Veja [[Arquitetura do Sistema#2. Segredos]].

## 2. Rotação de Refresh
Cada refresh invalida a família de tokens anterior para impedir ataques de replay.

#auth #seguranca #arquitetura</pre></div>
          <h3>Como as notas são processadas</h3>
          <ul>
            <li>O parsing usa uma AST CommonMark real (<code>pulldown-cmark</code>). O conteúdo dentro de blocos de código é isolado, então um <code>#</code> em código nunca é tratado como título ou tag.</li>
            <li>Seções com mais de <strong>320 tokens</strong> são divididas em janelas com <strong>sobreposição de 40 tokens</strong> para não perder contexto nas bordas.</li>
            <li>Renomear uma nota (pela interface ou <code>rename_note</code>) reescreve todos os <code>[[Título Antigo]]</code> no vault.</li>
            <li>Notas editadas fora do GraphFinder (VS Code, Cursor, Neovim…) são detectadas pelo file watcher (debounce de 500 ms).</li>
          </ul>
        </section>
`;
