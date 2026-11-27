type Cleanup = () => void;

const copyIcon = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg>';
const checkIcon = '<svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.4"><path d="M20 6 9 17l-5-5"/></svg>';

function escapeHtml(value: string) {
  return value.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
}

function highlight(pre: HTMLElement) {
  const lang = pre.dataset.lang || '';
  const raw = pre.textContent ?? '';
  let html: string | undefined;
  if (lang === 'bash' || lang === 'powershell') {
    html = raw.split('\n').map((line) => {
      if (/^\s*#/.test(line)) return `<span class="c">${escapeHtml(line)}</span>`;
      const match = line.match(/^(\s*)(\$ )?(\S+)(.*)$/);
      if (!match) return escapeHtml(line);
      const rest = escapeHtml(match[4])
        .replace(/(&quot;|")(.*?)("|&quot;)/g, '<span class="s">"$2"</span>')
        .replace(/(\s)(--?[a-z][\w-]*)/gi, '$1<span class="o">$2</span>')
        .replace(/(\s)(#.*)$/, '$1<span class="c">$2</span>');
      return `${match[1]}${match[2] ? '<span class="p">$ </span>' : ''}<span class="k">${escapeHtml(match[3])}</span>${rest}`;
    }).join('\n');
  } else if (lang === 'json') {
    html = escapeHtml(raw)
      .replace(/("(?:[^"\\]|\\.)*")(\s*:)/g, '<span class="k">$1</span>$2')
      .replace(/:\s*("(?:[^"\\]|\\.)*")/g, ': <span class="s">$1</span>')
      .replace(/(\[|,)\s*("(?:[^"\\]|\\.)*")(?=\s*[\],])/g, '$1 <span class="s">$2</span>')
      .replace(/\b(true|false|null|\d+(?:\.\d+)?)\b/g, '<span class="n">$1</span>');
  } else if (lang === 'markdown') {
    html = raw.split('\n').map((line) => {
      if (/^#{1,6}\s/.test(line)) return `<span class="k">${escapeHtml(line)}</span>`;
      return escapeHtml(line)
        .replace(/(\[\[[^\]]+\]\])/g, '<span class="o">$1</span>')
        .replace(/(^|\s)(#[a-z][\w-]*)/g, '$1<span class="n">$2</span>');
    }).join('\n');
  }
  if (html !== undefined) pre.innerHTML = html;
}

export function enhanceStaticDom(copyLabel: string, copiedLabel: string): Cleanup {
  const cleanups: Cleanup[] = [];
  const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const nav = document.querySelector('.nav');
  const toTop = document.querySelector<HTMLButtonElement>('.to-top');
  let scrollTicking = false;

  const updateScrollState = () => {
    const y = window.scrollY;
    nav?.classList.toggle('scrolled', y > 12);
    toTop?.classList.toggle('show', y > 900);
    scrollTicking = false;
  };
  const onScroll = () => {
    if (!scrollTicking) {
      window.requestAnimationFrame(updateScrollState);
      scrollTicking = true;
    }
  };
  window.addEventListener('scroll', onScroll, { passive: true });
  cleanups.push(() => window.removeEventListener('scroll', onScroll));
  updateScrollState();

  if (toTop) {
    const scrollTop = () => window.scrollTo({ top: 0, behavior: 'smooth' });
    toTop.addEventListener('click', scrollTop);
    cleanups.push(() => toTop.removeEventListener('click', scrollTop));
  }

  const menuBtn = document.querySelector<HTMLButtonElement>('.menu-toggle');
  const links = document.querySelector('.nav-links');
  const closeMenu = () => {
    links?.classList.remove('open');
    menuBtn?.setAttribute('aria-expanded', 'false');
    document.body.classList.remove('menu-open');
  };
  if (menuBtn && links) {
    const toggleMenu = () => {
      const open = links.classList.toggle('open');
      menuBtn.setAttribute('aria-expanded', String(open));
      document.body.classList.toggle('menu-open', open);
    };
    const escapeMenu = (event: KeyboardEvent) => {
      if (event.key === 'Escape' && links.classList.contains('open')) closeMenu();
    };
    menuBtn.addEventListener('click', toggleMenu);
    window.addEventListener('keydown', escapeMenu);
    links.querySelectorAll('a').forEach((link) => link.addEventListener('click', closeMenu));
    cleanups.push(() => {
      menuBtn.removeEventListener('click', toggleMenu);
      window.removeEventListener('keydown', escapeMenu);
      links.querySelectorAll('a').forEach((link) => link.removeEventListener('click', closeMenu));
      closeMenu();
    });
  }

  document.querySelectorAll('.code, .terminal').forEach((block) => {
    const pre = block.querySelector('pre');
    if (!pre) return;
    const source = (pre.textContent ?? '').replace(/^\$ /gm, '');
    highlight(pre);
    const bar = block.querySelector('.code-bar, .terminal-bar');
    if (!bar || bar.querySelector('.copy-btn')) return;
    const button = document.createElement('button');
    button.className = 'copy-btn';
    button.type = 'button';
    button.innerHTML = `${copyIcon}<span>${copyLabel}</span>`;
    button.setAttribute('aria-label', copyLabel);
    const copy = async () => {
      try {
        await navigator.clipboard.writeText(source.trim());
      } catch {
        const textarea = document.createElement('textarea');
        textarea.value = source.trim();
        document.body.appendChild(textarea);
        textarea.select();
        document.execCommand('copy');
        textarea.remove();
      }
      button.classList.add('done');
      button.innerHTML = `${checkIcon}<span>${copiedLabel}</span>`;
      window.setTimeout(() => {
        button.classList.remove('done');
        button.innerHTML = `${copyIcon}<span>${copyLabel}</span>`;
      }, 1600);
    };
    button.addEventListener('click', copy);
    bar.appendChild(button);
    cleanups.push(() => {
      button.removeEventListener('click', copy);
      button.remove();
    });
  });

  document.querySelectorAll('.tabs').forEach((tabs) => {
    const buttons = [...tabs.querySelectorAll<HTMLButtonElement>('[role="tab"]')];
    const panels = [...tabs.querySelectorAll<HTMLElement>('[role="tabpanel"]')];
    const listeners: Array<[HTMLButtonElement, () => void]> = [];
    buttons.forEach((button, index) => {
      const activate = () => {
        buttons.forEach((item) => item.setAttribute('aria-selected', 'false'));
        panels.forEach((panel) => panel.classList.remove('active'));
        button.setAttribute('aria-selected', 'true');
        panels[index]?.classList.add('active');
      };
      button.addEventListener('click', activate);
      listeners.push([button, activate]);
    });
    cleanups.push(() => listeners.forEach(([button, activate]) => button.removeEventListener('click', activate)));
  });

  const tocLinks = [...document.querySelectorAll<HTMLAnchorElement>(".toc a[href^='#']")];
  let spy: IntersectionObserver | undefined;
  if (tocLinks.length && 'IntersectionObserver' in window) {
    const map = new Map(tocLinks.map((link) => [link.getAttribute('href')?.slice(1) ?? '', link]));
    spy = new IntersectionObserver((entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          tocLinks.forEach((link) => link.classList.remove('active'));
          map.get(entry.target.id)?.classList.add('active');
        }
      });
    }, { rootMargin: '-20% 0px -70% 0px' });
    map.forEach((_, id) => {
      const element = document.getElementById(id);
      if (element) spy?.observe(element);
    });
    cleanups.push(() => spy?.disconnect());
  }

  const reveals = [...document.querySelectorAll('.reveal')];
  let revealObserver: IntersectionObserver | undefined;
  if ('IntersectionObserver' in window && !reduceMotion) {
    revealObserver = new IntersectionObserver((entries) => entries.forEach((entry) => {
      if (entry.isIntersecting) {
        entry.target.classList.add('in');
        revealObserver?.unobserve(entry.target);
      }
    }), { threshold: 0.12 });
    reveals.forEach((element) => revealObserver?.observe(element));
  } else {
    reveals.forEach((element) => element.classList.add('in'));
  }
  cleanups.push(() => revealObserver?.disconnect());

  document.querySelectorAll<HTMLElement>('.card').forEach((card) => {
    const move = (event: PointerEvent) => {
      if (event.pointerType !== 'mouse') return;
      const rect = card.getBoundingClientRect();
      card.style.setProperty('--mx', `${event.clientX - rect.left}px`);
      card.style.setProperty('--my', `${event.clientY - rect.top}px`);
    };
    const leave = (event: PointerEvent) => {
      if (event.pointerType !== 'mouse') return;
      card.style.removeProperty('--mx');
      card.style.removeProperty('--my');
    };
    card.addEventListener('pointermove', move);
    card.addEventListener('pointerleave', leave);
    cleanups.push(() => {
      card.removeEventListener('pointermove', move);
      card.removeEventListener('pointerleave', leave);
    });
  });

  return () => cleanups.forEach((cleanup) => cleanup());
}

type GraphNode = {
  x: number;
  y: number;
  vx: number;
  vy: number;
  r: number;
  c: string;
  hub: boolean;
  phase: number;
};

export function startGraphCanvas(): Cleanup {
  const canvas = document.getElementById('graph-canvas') as HTMLCanvasElement | null;
  const context = canvas?.getContext('2d');
  if (!canvas || !context) return () => undefined;
  const graphCanvas = canvas;
  const ctx = context;

  const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
  const palette = ['#818cf8', '#38bdf8', '#f97316', '#10b981', '#f472b6', '#94a3b8', '#94a3b8', '#94a3b8'];
  let width = 0;
  let height = 0;
  let nodes: GraphNode[] = [];
  let edges: Array<[number, number]> = [];
  let frame = 0;
  const mouse = { x: -9999, y: -9999 };

  function build() {
    const ratio = Math.min(window.devicePixelRatio || 1, 2);
    width = graphCanvas.clientWidth;
    height = graphCanvas.clientHeight;
    graphCanvas.width = width * ratio;
    graphCanvas.height = height * ratio;
    ctx.setTransform(ratio, 0, 0, ratio, 0, 0);
    const count = Math.round(Math.min(110, Math.max(40, (width * height) / 16000)));
    nodes = Array.from({ length: count }, (_, index) => {
      const hub = index < Math.max(4, count / 14);
      return {
        x: Math.random() * width,
        y: Math.random() * height,
        vx: (Math.random() - 0.5) * 0.22,
        vy: (Math.random() - 0.5) * 0.22,
        r: hub ? 3.2 + Math.random() * 2.2 : 1.3 + Math.random() * 1.4,
        c: hub ? palette[index % 5] : palette[Math.floor(Math.random() * palette.length)],
        hub,
        phase: Math.random() * Math.PI * 2,
      };
    });
    edges = [];
    nodes.forEach((node, index) => {
      const countEdges = node.hub ? 5 : 1 + (Math.random() < 0.4 ? 1 : 0);
      const near = nodes
        .map((other, otherIndex) => ({ otherIndex, distance: (other.x - node.x) ** 2 + (other.y - node.y) ** 2 }))
        .filter((other) => other.otherIndex !== index)
        .sort((a, b) => a.distance - b.distance)
        .slice(0, countEdges);
      near.forEach((other) => edges.push([index, other.otherIndex]));
    });
  }

  function step(time: number) {
    ctx.clearRect(0, 0, width, height);
    for (const node of nodes) {
      node.x += node.vx;
      node.y += node.vy;
      if (node.x < -20) node.x = width + 20;
      else if (node.x > width + 20) node.x = -20;
      if (node.y < -20) node.y = height + 20;
      else if (node.y > height + 20) node.y = -20;
      const dx = node.x - mouse.x;
      const dy = node.y - mouse.y;
      if (dx * dx + dy * dy < 22000) {
        node.x += dx * 0.004;
        node.y += dy * 0.004;
      }
    }
    ctx.lineWidth = 1;
    for (const [a, b] of edges) {
      const first = nodes[a];
      const second = nodes[b];
      const dx = first.x - second.x;
      const dy = first.y - second.y;
      const distance = Math.sqrt(dx * dx + dy * dy);
      if (distance > 240) continue;
      const mx = (first.x + second.x) / 2 - mouse.x;
      const my = (first.y + second.y) / 2 - mouse.y;
      const near = Math.max(0, 1 - Math.sqrt(mx * mx + my * my) / 220);
      const alpha = (1 - distance / 240) * (0.16 + near * 0.5);
      ctx.strokeStyle = near > 0.05 ? `rgba(129,140,248,${alpha})` : `rgba(148,163,184,${alpha})`;
      ctx.beginPath();
      ctx.moveTo(first.x, first.y);
      ctx.lineTo(second.x, second.y);
      ctx.stroke();
    }
    for (const node of nodes) {
      if (node.hub) {
        const pulse = 1 + Math.sin(time / 900 + node.phase) * 0.25;
        const gradient = ctx.createRadialGradient(node.x, node.y, 0, node.x, node.y, 26 * pulse);
        gradient.addColorStop(0, `${node.c}55`);
        gradient.addColorStop(1, `${node.c}00`);
        ctx.fillStyle = gradient;
        ctx.beginPath();
        ctx.arc(node.x, node.y, 26 * pulse, 0, Math.PI * 2);
        ctx.fill();
        ctx.strokeStyle = `${node.c}66`;
        ctx.beginPath();
        ctx.arc(node.x, node.y, node.r + 4, 0, Math.PI * 2);
        ctx.stroke();
      }
      ctx.fillStyle = node.c;
      ctx.globalAlpha = node.hub ? 1 : 0.75;
      ctx.beginPath();
      ctx.arc(node.x, node.y, node.r, 0, Math.PI * 2);
      ctx.fill();
      ctx.globalAlpha = 1;
    }
    frame = requestAnimationFrame(step);
  }

  build();
  if (reduceMotion) {
    step(0);
    cancelAnimationFrame(frame);
  } else {
    frame = requestAnimationFrame(step);
  }

  let resizeTimer: number | undefined;
  const resize = () => {
    window.clearTimeout(resizeTimer);
    resizeTimer = window.setTimeout(build, 150);
  };
  const parent = graphCanvas.parentElement;
  const pointerMove = (event: PointerEvent) => {
    if (event.pointerType !== 'mouse') return;
    const rect = graphCanvas.getBoundingClientRect();
    mouse.x = event.clientX - rect.left;
    mouse.y = event.clientY - rect.top;
  };
  const pointerLeave = () => {
    mouse.x = -9999;
    mouse.y = -9999;
  };
  window.addEventListener('resize', resize);
  parent?.addEventListener('pointermove', pointerMove);
  parent?.addEventListener('pointerleave', pointerLeave);

  return () => {
    cancelAnimationFrame(frame);
    window.clearTimeout(resizeTimer);
    window.removeEventListener('resize', resize);
    parent?.removeEventListener('pointermove', pointerMove);
    parent?.removeEventListener('pointerleave', pointerLeave);
  };
}
