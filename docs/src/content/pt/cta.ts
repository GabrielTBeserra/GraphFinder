export default `
<section class="block" style="padding-top:0" aria-labelledby="cta-title">
  <div class="container">
    <div class="cta-band reveal">
      <h2 id="cta-title">Dê aos seus agentes uma memória<br />que não desperdiça tokens.</h2>
      <p>Gratuito e open source sob a licença MIT. Roda inteiramente na sua máquina.</p>
      <div class="hero-ctas">
        <a class="btn btn-primary" id="cta-download" href="https://github.com/GabrielTBeserra/GraphFinder/releases/latest" target="_blank" rel="noopener">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 3v12m0 0-5-5m5 5 5-5M4 19h16"/></svg>
          Baixar <span data-version>{{VERSION}}</span>
        </a>
        <a class="btn btn-ghost" id="cta-star" href="https://github.com/GabrielTBeserra/GraphFinder" target="_blank" rel="noopener">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="m12 2 3.1 6.3 6.9 1-5 4.9 1.2 6.8L12 17.8 5.8 21l1.2-6.8-5-4.9 6.9-1L12 2Z"/></svg>
          Dar estrela no GitHub
        </a>
      </div>
    </div>
  </div>
</section>
`;
