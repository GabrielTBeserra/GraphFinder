import { useEffect, useMemo, useState } from 'react';
import { enhanceStaticDom, startGraphCanvas } from './effects';
import { clientSections as enClientSections, serverSections as enServerSections } from './content/en';
import { clientSections as ptClientSections, serverSections as ptServerSections } from './content/pt';
import enHero from './content/en/hero';
import enFeatures from './content/en/features';
import enCta from './content/en/cta';
import ptHero from './content/pt/hero';
import ptFeatures from './content/pt/features';
import ptCta from './content/pt/cta';
import releasesJson from './generated/releases.json';

type Language = 'en' | 'pt';
type Track = 'client' | 'server' | 'releases';
type ReleaseSection = { heading: string; items: string[] };
type Release = { tag: string; version: string; date: string; sections: ReleaseSection[] };
type ReleaseData = { latestVersion: string; generatedAt: string; source: string; releases: Release[] };

const releaseData = releasesJson as ReleaseData;
const repositoryUrl = 'https://github.com/GabrielTBeserra/GraphFinder';

const copy = {
  en: {
    features: 'Features',
    docs: 'Docs',
    cli: 'CLI',
    tools: 'MCP Tools',
    changelog: 'Changelog',
    documentation: 'Documentation',
    docsTitle: 'Everything about GraphFinder.',
    docsIntro: 'From installation to the internals of the search engine. Current release:',
    onThisPage: 'On this page',
    tocLabel: 'Table of contents',
    client: 'Client / Desktop',
    server: 'Private server',
    releases: 'Releases',
    clientHint: 'local app · CLI · embedded MCP',
    serverHint: 'team server · workers · tokens',
    releasesHint: 'git tags · changelog',
    copy: 'Copy',
    copied: 'Copied',
    latest: 'current',
    generated: 'Generated at build time from git tags and CHANGELOG.md.',
    fullHistory: 'Full history in',
    releasesPage: 'Releases page',
    generatedAt: 'generated',
    pageTitle: 'GraphFinder — Local-First Knowledge Graph & MCP Server for AI Agents',
    pageDescription: 'GraphFinder turns Markdown vaults and codebases into a semantically indexed knowledge graph. Hybrid vector + BM25 search, AST code indexing, graph analytics and native MCP access.',
    trackLabel: 'Documentation track',
    footerLicense: 'MIT License',
    footerLanguage: 'Português',
    copyright: '© 2026 GraphFinder contributors',
    backToTop: 'Back to top',
    openMenu: 'Open menu',
    home: 'GraphFinder home',
    primary: 'Primary',
    footer: 'Footer',
  },
  pt: {
    features: 'Recursos',
    docs: 'Documentação',
    cli: 'CLI',
    tools: 'Ferramentas MCP',
    changelog: 'Changelog',
    documentation: 'Documentação',
    docsTitle: 'Tudo sobre o GraphFinder.',
    docsIntro: 'Da instalação aos detalhes internos do motor de busca. Versão atual:',
    onThisPage: 'Nesta página',
    tocLabel: 'Sumário',
    client: 'Cliente / Desktop',
    server: 'Servidor privado',
    releases: 'Versões',
    clientHint: 'app local · CLI · MCP embarcado',
    serverHint: 'servidor de equipe · workers · tokens',
    releasesHint: 'tags git · changelog',
    copy: 'Copiar',
    copied: 'Copiado',
    latest: 'atual',
    generated: 'Gerado durante o build a partir das tags do Git e do CHANGELOG.md.',
    fullHistory: 'Histórico completo em',
    releasesPage: 'página de releases',
    generatedAt: 'gerado',
    pageTitle: 'GraphFinder — Grafo de Conhecimento Local-First e Servidor MCP para Agentes de IA',
    pageDescription: 'GraphFinder transforma vaults Markdown e bases de código em um grafo de conhecimento com indexação semântica, busca híbrida, análise de grafos e acesso MCP nativo.',
    trackLabel: 'Trilha da documentação',
    footerLicense: 'Licença MIT',
    footerLanguage: 'English',
    copyright: '© 2026 Contribuidores do GraphFinder',
    backToTop: 'Voltar ao topo',
    openMenu: 'Abrir menu',
    home: 'Início do GraphFinder',
    primary: 'Principal',
    footer: 'Rodapé',
  },
} as const;

function initialLanguage(): Language {
  const params = new URLSearchParams(window.location.search);
  if (params.get('lang') === 'pt') return 'pt';
  if (params.get('lang') === 'en') return 'en';
  if (window.location.pathname.startsWith('/pt')) return 'pt';
  try {
    return localStorage.getItem('gf-lang') === 'pt' ? 'pt' : 'en';
  } catch {
    return 'en';
  }
}

function trackFromHash(serverIds: string[]): Track {
  const hash = window.location.hash.replace(/^#/, '');
  if (hash === 'changelog' || hash === 'releases') return 'releases';
  if (serverIds.includes(hash)) return 'server';
  return 'client';
}

function withVersion(html: string, version: string) {
  return html.replaceAll('{{VERSION}}', version);
}

function ReleasesManual({ language }: { language: Language }) {
  const t = copy[language];
  const generatedDate = new Intl.DateTimeFormat(language === 'pt' ? 'pt-BR' : 'en', { dateStyle: 'medium', timeStyle: 'short' }).format(new Date(releaseData.generatedAt));
  return (
    <>
      <section id="changelog">
        <h2><span className="idx">01</span>Changelog<a className="anchor" href="#changelog" aria-label="Link to Changelog">#</a></h2>
        <p className="lead-p">{t.generated}</p>
        <p>
          {t.fullHistory} <a href={`${repositoryUrl}/blob/main/CHANGELOG.md`} target="_blank" rel="noopener">CHANGELOG.md</a> {language === 'pt' ? 'e na' : 'and on the'}{' '}
          <a href={`${repositoryUrl}/releases`} target="_blank" rel="noopener">{t.releasesPage}</a>.
        </p>
        <div className="release-source">
          <span>{releaseData.source}</span>
          <span>{t.generatedAt} {generatedDate}</span>
        </div>
        <div className="timeline">
          {releaseData.releases.map((release, releaseIndex) => (
            <div className={`release ${releaseIndex === 0 ? 'current' : ''}`} key={release.tag}>
              <div className="release-head">
                <b>{release.tag}</b>
                {releaseIndex === 0 && <span className="badge latest">{t.latest}</span>}
                <span className="date">{release.date}</span>
              </div>
              {release.sections.map((section, sectionIndex) => (
                <div className="release-section" key={`${release.tag}-${sectionIndex}`}>
                  <h4 dangerouslySetInnerHTML={{ __html: section.heading }} />
                  <ul>
                    {section.items.map((item, itemIndex) => (
                      <li key={`${release.tag}-${sectionIndex}-${itemIndex}`} dangerouslySetInnerHTML={{ __html: item }} />
                    ))}
                  </ul>
                </div>
              ))}
            </div>
          ))}
        </div>
      </section>
    </>
  );
}

function App() {
  const [language, setLanguage] = useState<Language>(initialLanguage);
  const content = language === 'pt'
    ? { client: ptClientSections, server: ptServerSections, hero: ptHero, features: ptFeatures, cta: ptCta }
    : { client: enClientSections, server: enServerSections, hero: enHero, features: enFeatures, cta: enCta };
  const [track, setTrack] = useState<Track>(() => trackFromHash(content.server.map((section) => section.id)));
  const t = copy[language];
  const latestVersion = releaseData.latestVersion;

  useEffect(() => {
    document.documentElement.lang = language === 'pt' ? 'pt-BR' : 'en';
    document.title = t.pageTitle;
    document.querySelector('meta[name="description"]')?.setAttribute('content', t.pageDescription);
    try {
      localStorage.setItem('gf-lang', language);
    } catch {
      void 0;
    }
  }, [language, t.pageTitle, t.pageDescription]);

  useEffect(() => {
    const onHashChange = () => setTrack(trackFromHash(content.server.map((section) => section.id)));
    window.addEventListener('hashchange', onHashChange);
    return () => window.removeEventListener('hashchange', onHashChange);
  }, [content.server]);

  useEffect(() => {
    const id = window.location.hash.replace(/^#/, '');
    if (!id) return undefined;
    const frame = window.requestAnimationFrame(() => {
      document.getElementById(id)?.scrollIntoView({ block: 'start' });
    });
    return () => window.cancelAnimationFrame(frame);
  }, [language, track]);

  useEffect(() => startGraphCanvas(), [language]);
  useEffect(() => enhanceStaticDom(t.copy, t.copied), [language, track, t.copy, t.copied]);

  const activeSections = track === 'server' ? content.server : content.client;
  const tocSections = track === 'releases'
    ? [{ id: 'changelog', label: t.changelog }]
    : activeSections.map(({ id, label }) => ({ id, label }));
  const manualHtml = useMemo(() => activeSections.map((section) => section.html).join('\n\n'), [activeSections]);
  const heroHtml = useMemo(() => withVersion(content.hero, latestVersion), [content.hero, latestVersion]);
  const featuresHtml = useMemo(() => withVersion(content.features, latestVersion), [content.features, latestVersion]);
  const ctaHtml = useMemo(() => withVersion(content.cta, latestVersion), [content.cta, latestVersion]);

  function changeLanguage(next: Language) {
    setLanguage(next);
    window.history.replaceState(null, '', `${next === 'pt' ? '/?lang=pt' : '/'}${window.location.hash}`);
  }

  function selectTrack(next: Track) {
    setTrack(next);
    const hash = next === 'server' ? 'server' : next === 'releases' ? 'changelog' : 'overview';
    window.history.replaceState(null, '', `${window.location.search}#${hash}`);
    document.getElementById('docs')?.scrollIntoView({ behavior: 'smooth', block: 'start' });
  }

  return (
    <>
      <header className="nav">
        <div className="container nav-inner">
          <a className="brand" href="#top" aria-label={t.home}>
            <img src="/assets/logo.svg" alt="" width="28" height="28" />
            GraphFinder <span className="ver" data-version>{latestVersion}</span>
          </a>
          <nav className="nav-links" id="nav-links" aria-label={t.primary}>
            <a href="#features">{t.features}</a>
            <a href="#docs">{t.docs}</a>
            <a href="#cli">{t.cli}</a>
            <a href="#tools">{t.tools}</a>
            <a href="#changelog">{t.changelog}</a>
          </nav>
          <div className="nav-actions">
            <div className="lang-switch" role="group" aria-label="Language">
              <a href="/?lang=en" className={language === 'en' ? 'active' : ''} aria-current={language === 'en' ? 'true' : undefined} onClick={(event) => { event.preventDefault(); changeLanguage('en'); }}>EN</a>
              <a href="/?lang=pt" className={language === 'pt' ? 'active' : ''} aria-current={language === 'pt' ? 'true' : undefined} hrefLang="pt-BR" onClick={(event) => { event.preventDefault(); changeLanguage('pt'); }}>PT</a>
            </div>
            <a className="icon-btn" id="nav-github" href={repositoryUrl} target="_blank" rel="noopener">
              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M12 .5a11.5 11.5 0 0 0-3.64 22.41c.58.1.79-.25.79-.56v-2c-3.2.7-3.88-1.37-3.88-1.37-.52-1.33-1.28-1.69-1.28-1.69-1.05-.72.08-.7.08-.7 1.16.08 1.77 1.19 1.77 1.19 1.03 1.77 2.7 1.26 3.36.96.1-.75.4-1.26.73-1.55-2.55-.29-5.24-1.28-5.24-5.68 0-1.26.45-2.28 1.18-3.09-.12-.29-.51-1.46.11-3.04 0 0 .97-.31 3.17 1.18a10.9 10.9 0 0 1 5.77 0c2.2-1.49 3.17-1.18 3.17-1.18.62 1.58.23 2.75.11 3.04.74.81 1.18 1.83 1.18 3.09 0 4.41-2.7 5.38-5.26 5.67.41.36.78 1.06.78 2.14v3.17c0 .31.21.67.8.56A11.5 11.5 0 0 0 12 .5Z"/></svg>
              <span className="gh-label">GitHub</span>
            </a>
            <button className="icon-btn menu-toggle" id="menu-toggle" aria-label={t.openMenu} aria-expanded="false" aria-controls="nav-links">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M4 7h16M4 12h16M4 17h16"/></svg>
            </button>
          </div>
        </div>
      </header>

      <main id="top">
        <div dangerouslySetInnerHTML={{ __html: heroHtml }} />
        <div dangerouslySetInnerHTML={{ __html: featuresHtml }} />

        <section className="block" id="docs" aria-labelledby="docs-title" style={{ paddingTop: 40 }}>
          <div className="container">
            <div className="section-head reveal">
              <span className="kicker">{t.documentation}</span>
              <h2 id="docs-title">{t.docsTitle}</h2>
              <p>{t.docsIntro} <strong data-version>{latestVersion}</strong>.</p>
            </div>

            <div className="docs-track-switch reveal" role="group" aria-label={t.trackLabel}>
              <button className={track === 'client' ? 'active' : ''} onClick={() => selectTrack('client')} aria-pressed={track === 'client'}>
                <strong>{t.client}</strong><span>{t.clientHint}</span>
              </button>
              <button className={track === 'server' ? 'active' : ''} onClick={() => selectTrack('server')} aria-pressed={track === 'server'}>
                <strong>{t.server}</strong><span>{t.serverHint}</span>
              </button>
              <button className={track === 'releases' ? 'active' : ''} onClick={() => selectTrack('releases')} aria-pressed={track === 'releases'}>
                <strong>{t.releases}</strong><span>{t.releasesHint}</span>
              </button>
            </div>

            <div className="docs-wrap">
              <aside className="toc" aria-label={t.tocLabel}>
                <p className="toc-title">{t.onThisPage}</p>
                <ol>
                  {tocSections.map((section) => <li key={section.id}><a href={`#${section.id}`}>{section.label}</a></li>)}
                </ol>
              </aside>

              {track === 'releases' ? (
                <article className="doc"><ReleasesManual language={language} /></article>
              ) : (
                <article className="doc" dangerouslySetInnerHTML={{ __html: manualHtml }} />
              )}
            </div>
          </div>
        </section>

        <div dangerouslySetInnerHTML={{ __html: ctaHtml }} />
      </main>

      <footer>
        <div className="container foot">
          <a className="brand" href="#top"><img src="/assets/logo.svg" alt="" width="24" height="24" />GraphFinder</a>
          <nav aria-label={t.footer}>
            <a href={repositoryUrl} target="_blank" rel="noopener">GitHub</a>
            <a href={`${repositoryUrl}/releases`} target="_blank" rel="noopener">Releases</a>
            <a href={`${repositoryUrl}/issues`} target="_blank" rel="noopener">Issues</a>
            <a href={`${repositoryUrl}/blob/main/LICENSE`} target="_blank" rel="noopener">{t.footerLicense}</a>
            <a href={language === 'pt' ? '/?lang=en' : '/?lang=pt'} hrefLang={language === 'pt' ? 'en' : 'pt-BR'} onClick={(event) => { event.preventDefault(); changeLanguage(language === 'pt' ? 'en' : 'pt'); }}>{t.footerLanguage}</a>
          </nav>
          <span>{t.copyright}</span>
        </div>
      </footer>

      <button className="to-top" aria-label={t.backToTop}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 19V5M5 12l7-7 7 7"/></svg>
      </button>
    </>
  );
}

export default App;
