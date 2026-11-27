export default `
<section id="install">
          <h2><span class="idx">02</span>Installation<a class="anchor" href="#install" aria-label="Link to Installation">#</a></h2>
          <h3>Option A — Pre-built installers (recommended)</h3>
          <p>Every tagged release is built by GitHub Actions for all three desktop platforms. Grab the latest from the <a href="https://github.com/GabrielTBeserra/GraphFinder/releases/latest" target="_blank" rel="noopener">Releases page</a>:</p>
          <div class="downloads">
            <a class="dl" id="dl-windows" href="https://github.com/GabrielTBeserra/GraphFinder/releases/latest" target="_blank" rel="noopener">
              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M3 5.5 10.5 4.5v7H3v-6Zm8.5-1.1L21 3v8.5h-9.5V4.4ZM3 12.5h7.5v7L3 18.5v-6Zm8.5 0H21V21l-9.5-1.4v-7.1Z"/></svg>
              <b>Windows</b><span>.exe (NSIS) · .msi (WiX)</span>
            </a>
            <a class="dl" id="dl-macos" href="https://github.com/GabrielTBeserra/GraphFinder/releases/latest" target="_blank" rel="noopener">
              <svg viewBox="0 0 24 24" fill="currentColor"><path d="M16.4 12.6c0-2.5 2-3.7 2.1-3.8a4.6 4.6 0 0 0-3.6-2c-1.5-.2-3 .9-3.7.9-.8 0-2-.9-3.2-.9a4.8 4.8 0 0 0-4 2.5c-1.7 3-.4 7.4 1.2 9.8.8 1.2 1.8 2.5 3 2.4 1.2 0 1.7-.8 3.1-.8 1.5 0 1.9.8 3.2.8 1.3 0 2.1-1.2 2.9-2.4.9-1.3 1.3-2.6 1.3-2.7 0 0-2.4-.9-2.3-3.8ZM14.1 5.3A4.4 4.4 0 0 0 15.1 2a4.5 4.5 0 0 0-2.9 1.5 4.2 4.2 0 0 0-1 3.1 3.7 3.7 0 0 0 2.9-1.3Z"/></svg>
              <b>macOS</b><span>.dmg (Apple Silicon)</span>
            </a>
            <a class="dl" id="dl-linux" href="https://github.com/GabrielTBeserra/GraphFinder/releases/latest" target="_blank" rel="noopener">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.6"><rect x="3" y="4" width="18" height="14" rx="2"/><path d="m7 9 3 2.5L7 14M12 14h5"/></svg>
              <b>Linux</b><span>.deb · .AppImage</span>
            </a>
          </div>
          <p>After installing, enable the terminal command so <code>graphfinder</code> works from any shell:</p>
          <div class="code"><div class="code-bar">bash</div>
<pre data-lang="bash"># Windows: adds the install folder to your User PATH + creates graphfinder.cmd shims
# macOS/Linux: creates a symlink in ~/.local/bin/graphfinder
graphfinder install-cli</pre></div>
          <div class="callout info">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4M12 8h.01"/></svg>
            <p>If <code>graphfinder</code> is not yet on your PATH, run the binary directly, e.g. <code>"C:\\Program Files\\GraphFinder\\GraphFinder.exe" install-cli</code>. Open a new terminal afterwards.</p>
          </div>

          <h3>Option B — Build from source</h3>
          <h4>Prerequisites</h4>
          <ul>
            <li><strong>Node.js 20+</strong> and npm</li>
            <li><strong>Rust 1.75+</strong> via <a href="https://rustup.rs" target="_blank" rel="noopener">rustup</a></li>
            <li>Windows: Visual Studio C++ Build Tools · Linux: <code>libwebkit2gtk-4.1-dev</code>, <code>build-essential</code>, <code>libssl-dev</code>, <code>libayatana-appindicator3-dev</code>, <code>librsvg2-dev</code></li>
          </ul>
          <div class="code"><div class="code-bar">bash</div>
<pre data-lang="bash">git clone https://github.com/GabrielTBeserra/GraphFinder.git
cd GraphFinder
npm install

# Run the desktop app in development mode (Vite on :1420 + Tauri)
npm run app

# Produce installers in src-tauri/target/release/bundle/
npm run tauri build</pre></div>
          <div class="callout tip">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M9 18h6M10 22h4M12 2a7 7 0 0 0-4 12.7V17h8v-2.3A7 7 0 0 0 12 2Z"/></svg>
            <p>On first launch the embedding model (multilingual-e5-small, ~118 MB, PT/EN) is downloaded once into <code>.fastembed_cache</code>. After that GraphFinder works fully offline.</p>
          </div>
        </section>
`;
