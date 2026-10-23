/* GraphFinder Docs — interactions (vanilla JS, no dependencies) */
(() => {
  "use strict";

  const isPT = document.documentElement.lang.toLowerCase().startsWith("pt");
  const T = isPT
    ? { copy: "Copiar", copied: "Copiado" }
    : { copy: "Copy", copied: "Copied" };
  const REPO = "GabrielTBeserra/GraphFinder";
  const reduceMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

  /* ---------------- Language preference ---------------- */
  document.querySelectorAll("[data-lang-link]").forEach((a) => {
    a.addEventListener("click", () => {
      try { localStorage.setItem("gf-lang", a.dataset.langLink); } catch (_) {}
    });
  });

  /* ---------------- Nav state + back to top ---------------- */
  const nav = document.querySelector(".nav");
  const toTop = document.querySelector(".to-top");
  let scrollTicking = false;
  const updateScrollState = () => {
    const y = window.scrollY;
    nav && nav.classList.toggle("scrolled", y > 12);
    toTop && toTop.classList.toggle("show", y > 900);
    scrollTicking = false;
  };
  const onScroll = () => {
    if (!scrollTicking) {
      window.requestAnimationFrame(updateScrollState);
      scrollTicking = true;
    }
  };
  window.addEventListener("scroll", onScroll, { passive: true });
  updateScrollState();
  toTop && toTop.addEventListener("click", () => window.scrollTo({ top: 0, behavior: "smooth" }));

  /* ---------------- Mobile menu ---------------- */
  const menuBtn = document.querySelector(".menu-toggle");
  const links = document.querySelector(".nav-links");
  const closeMenu = () => {
    if (!links || !menuBtn) return;
    links.classList.remove("open");
    menuBtn.setAttribute("aria-expanded", "false");
    document.body.classList.remove("menu-open");
  };
  if (menuBtn && links) {
    menuBtn.addEventListener("click", () => {
      const open = links.classList.toggle("open");
      menuBtn.setAttribute("aria-expanded", String(open));
      document.body.classList.toggle("menu-open", open);
    });
    links.querySelectorAll("a").forEach((a) =>
      a.addEventListener("click", closeMenu)
    );
    window.addEventListener("keydown", (e) => {
      if (e.key === "Escape" && links.classList.contains("open")) {
        closeMenu();
      }
    });
  }

  /* ---------------- Code blocks: syntax tint + copy ---------------- */
  const esc = (s) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

  function highlight(pre) {
    const lang = pre.dataset.lang || "";
    const raw = pre.textContent;
    let html;
    if (lang === "bash" || lang === "powershell") {
      html = raw.split("\n").map((line) => {
        if (/^\s*#/.test(line)) return `<span class="c">${esc(line)}</span>`;
        const m = line.match(/^(\s*)(\$ )?(\S+)(.*)$/);
        if (!m) return esc(line);
        const rest = esc(m[4])
          .replace(/(&quot;|")(.*?)("|&quot;)/g, '<span class="s">"$2"</span>')
          .replace(/(\s)(--?[a-z][\w-]*)/gi, '$1<span class="o">$2</span>')
          .replace(/(\s)(#.*)$/, '$1<span class="c">$2</span>');
        return `${m[1]}${m[2] ? '<span class="p">$ </span>' : ""}<span class="k">${esc(m[3])}</span>${rest}`;
      }).join("\n");
    } else if (lang === "json") {
      html = esc(raw)
        .replace(/("(?:[^"\\]|\\.)*")(\s*:)/g, '<span class="k">$1</span>$2')
        .replace(/:\s*("(?:[^"\\]|\\.)*")/g, ': <span class="s">$1</span>')
        .replace(/(\[|,)\s*("(?:[^"\\]|\\.)*")(?=\s*[\],])/g, '$1 <span class="s">$2</span>')
        .replace(/\b(true|false|null|\d+(?:\.\d+)?)\b/g, '<span class="n">$1</span>');
    } else if (lang === "markdown") {
      html = raw.split("\n").map((line) => {
        if (/^#{1,6}\s/.test(line)) return `<span class="k">${esc(line)}</span>`;
        return esc(line)
          .replace(/(\[\[[^\]]+\]\])/g, '<span class="o">$1</span>')
          .replace(/(^|\s)(#[a-z][\w-]*)/g, '$1<span class="n">$2</span>');
      }).join("\n");
    } else {
      return;
    }
    pre.innerHTML = html;
  }

  const copyIcon = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>';
  const checkIcon = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4"><path d="M20 6 9 17l-5-5"/></svg>';

  document.querySelectorAll(".code, .terminal").forEach((block) => {
    const pre = block.querySelector("pre");
    if (!pre) return;
    const source = pre.textContent.replace(/^\$ /gm, "");
    highlight(pre);
    const bar = block.querySelector(".code-bar, .terminal-bar");
    if (!bar) return;
    const btn = document.createElement("button");
    btn.className = "copy-btn";
    btn.type = "button";
    btn.innerHTML = `${copyIcon}<span>${T.copy}</span>`;
    btn.setAttribute("aria-label", T.copy);
    btn.addEventListener("click", async () => {
      try {
        await navigator.clipboard.writeText(source.trim());
      } catch (_) {
        const ta = document.createElement("textarea");
        ta.value = source.trim();
        document.body.appendChild(ta);
        ta.select();
        document.execCommand("copy");
        ta.remove();
      }
      btn.classList.add("done");
      btn.innerHTML = `${checkIcon}<span>${T.copied}</span>`;
      setTimeout(() => {
        btn.classList.remove("done");
        btn.innerHTML = `${copyIcon}<span>${T.copy}</span>`;
      }, 1600);
    });
    bar.appendChild(btn);
  });

  /* ---------------- Tabs ---------------- */
  document.querySelectorAll(".tabs").forEach((tabs) => {
    const buttons = tabs.querySelectorAll('[role="tab"]');
    const panels = tabs.querySelectorAll('[role="tabpanel"]');
    buttons.forEach((b, i) => {
      b.addEventListener("click", () => {
        buttons.forEach((x) => x.setAttribute("aria-selected", "false"));
        panels.forEach((p) => p.classList.remove("active"));
        b.setAttribute("aria-selected", "true");
        panels[i] && panels[i].classList.add("active");
      });
    });
  });

  /* ---------------- Scrollspy for TOC ---------------- */
  const tocLinks = [...document.querySelectorAll(".toc a[href^='#']")];
  if (tocLinks.length && "IntersectionObserver" in window) {
    const map = new Map(tocLinks.map((a) => [a.getAttribute("href").slice(1), a]));
    const spy = new IntersectionObserver(
      (entries) => {
        entries.forEach((e) => {
          if (e.isIntersecting) {
            tocLinks.forEach((a) => a.classList.remove("active"));
            const a = map.get(e.target.id);
            a && a.classList.add("active");
          }
        });
      },
      { rootMargin: "-20% 0px -70% 0px" }
    );
    map.forEach((_, id) => {
      const el = document.getElementById(id);
      el && spy.observe(el);
    });
  }

  /* ---------------- Reveal on scroll ---------------- */
  const reveals = document.querySelectorAll(".reveal");
  if ("IntersectionObserver" in window && !reduceMotion) {
    const io = new IntersectionObserver(
      (entries) => entries.forEach((e) => {
        if (e.isIntersecting) { e.target.classList.add("in"); io.unobserve(e.target); }
      }),
      { threshold: 0.12 }
    );
    reveals.forEach((el) => io.observe(el));
  } else {
    reveals.forEach((el) => el.classList.add("in"));
  }

  /* ---------------- Card spotlight ---------------- */
  document.querySelectorAll(".card").forEach((card) => {
    card.addEventListener("pointermove", (e) => {
      if (e.pointerType !== "mouse") return;
      const r = card.getBoundingClientRect();
      card.style.setProperty("--mx", `${e.clientX - r.left}px`);
      card.style.setProperty("--my", `${e.clientY - r.top}px`);
    });
    card.addEventListener("pointerleave", (e) => {
      if (e.pointerType !== "mouse") return;
      card.style.removeProperty("--mx");
      card.style.removeProperty("--my");
    });
  });

  /* ---------------- Latest release (GitHub API, cached) ---------------- */
  (async () => {
    const targets = document.querySelectorAll("[data-version]");
    if (!targets.length) return;
    const apply = (tag) => targets.forEach((el) => { el.textContent = tag; });
    try {
      const cached = JSON.parse(sessionStorage.getItem("gf-release") || "null");
      if (cached && Date.now() - cached.t < 3600e3) return apply(cached.tag);
      const res = await fetch(`https://api.github.com/repos/${REPO}/releases/latest`, {
        headers: { Accept: "application/vnd.github+json" },
      });
      if (!res.ok) return;
      const data = await res.json();
      if (data && data.tag_name) {
        sessionStorage.setItem("gf-release", JSON.stringify({ tag: data.tag_name, t: Date.now() }));
        apply(data.tag_name);
      }
    } catch (_) { /* offline: keep static fallback */ }
  })();

  /* ---------------- Hero knowledge-graph canvas ---------------- */
  const canvas = document.getElementById("graph-canvas");
  if (!canvas) return;
  const ctx = canvas.getContext("2d");
  const palette = ["#818cf8", "#38bdf8", "#f97316", "#10b981", "#f472b6", "#94a3b8", "#94a3b8", "#94a3b8"];
  let W = 0, H = 0, DPR = 1, nodes = [], edges = [], raf = 0;
  const mouse = { x: -9999, y: -9999 };

  function build() {
    DPR = Math.min(window.devicePixelRatio || 1, 2);
    W = canvas.clientWidth; H = canvas.clientHeight;
    canvas.width = W * DPR; canvas.height = H * DPR;
    ctx.setTransform(DPR, 0, 0, DPR, 0, 0);
    const count = Math.round(Math.min(110, Math.max(40, (W * H) / 16000)));
    nodes = Array.from({ length: count }, (_, i) => {
      const hub = i < Math.max(4, count / 14);
      return {
        x: Math.random() * W, y: Math.random() * H,
        vx: (Math.random() - 0.5) * 0.22, vy: (Math.random() - 0.5) * 0.22,
        r: hub ? 3.2 + Math.random() * 2.2 : 1.3 + Math.random() * 1.4,
        c: hub ? palette[i % 5] : palette[Math.floor(Math.random() * palette.length)],
        hub, phase: Math.random() * Math.PI * 2,
      };
    });
    edges = [];
    nodes.forEach((n, i) => {
      const k = n.hub ? 5 : 1 + (Math.random() < 0.4 ? 1 : 0);
      const near = nodes
        .map((m, j) => ({ j, d: (m.x - n.x) ** 2 + (m.y - n.y) ** 2 }))
        .filter((o) => o.j !== i)
        .sort((a, b) => a.d - b.d)
        .slice(0, k);
      near.forEach((o) => edges.push([i, o.j]));
    });
  }

  function step(t) {
    ctx.clearRect(0, 0, W, H);
    for (const n of nodes) {
      n.x += n.vx; n.y += n.vy;
      if (n.x < -20) n.x = W + 20; else if (n.x > W + 20) n.x = -20;
      if (n.y < -20) n.y = H + 20; else if (n.y > H + 20) n.y = -20;
      const dx = n.x - mouse.x, dy = n.y - mouse.y, d2 = dx * dx + dy * dy;
      if (d2 < 22000) { n.x += dx * 0.004; n.y += dy * 0.004; }
    }
    ctx.lineWidth = 1;
    for (const [a, b] of edges) {
      const A = nodes[a], B = nodes[b];
      const dx = A.x - B.x, dy = A.y - B.y, d = Math.sqrt(dx * dx + dy * dy);
      if (d > 240) continue;
      const mx = (A.x + B.x) / 2 - mouse.x, my = (A.y + B.y) / 2 - mouse.y;
      const near = Math.max(0, 1 - Math.sqrt(mx * mx + my * my) / 220);
      const alpha = (1 - d / 240) * (0.16 + near * 0.5);
      ctx.strokeStyle = near > 0.05 ? `rgba(129,140,248,${alpha})` : `rgba(148,163,184,${alpha})`;
      ctx.beginPath(); ctx.moveTo(A.x, A.y); ctx.lineTo(B.x, B.y); ctx.stroke();
    }
    for (const n of nodes) {
      if (n.hub) {
        const pulse = 1 + Math.sin(t / 900 + n.phase) * 0.25;
        const g = ctx.createRadialGradient(n.x, n.y, 0, n.x, n.y, 26 * pulse);
        g.addColorStop(0, n.c + "55"); g.addColorStop(1, n.c + "00");
        ctx.fillStyle = g; ctx.beginPath(); ctx.arc(n.x, n.y, 26 * pulse, 0, Math.PI * 2); ctx.fill();
        ctx.strokeStyle = n.c + "66"; ctx.beginPath(); ctx.arc(n.x, n.y, n.r + 4, 0, Math.PI * 2); ctx.stroke();
      }
      ctx.fillStyle = n.c; ctx.globalAlpha = n.hub ? 1 : 0.75;
      ctx.beginPath(); ctx.arc(n.x, n.y, n.r, 0, Math.PI * 2); ctx.fill();
      ctx.globalAlpha = 1;
    }
    raf = requestAnimationFrame(step);
  }

  build();
  if (reduceMotion) { step(0); cancelAnimationFrame(raf); }
  else raf = requestAnimationFrame(step);

  let rt;
  window.addEventListener("resize", () => { clearTimeout(rt); rt = setTimeout(build, 150); });
  canvas.parentElement.addEventListener("pointermove", (e) => {
    if (e.pointerType !== "mouse") return;
    const r = canvas.getBoundingClientRect();
    mouse.x = e.clientX - r.left; mouse.y = e.clientY - r.top;
  });
  canvas.parentElement.addEventListener("pointerleave", () => { mouse.x = mouse.y = -9999; });
  document.addEventListener("visibilitychange", () => {
    if (document.hidden) cancelAnimationFrame(raf);
    else if (!reduceMotion) raf = requestAnimationFrame(step);
  });
})();
