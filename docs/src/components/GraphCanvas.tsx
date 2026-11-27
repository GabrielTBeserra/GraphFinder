import { useEffect, useRef } from 'react';

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

export default function GraphCanvas() {
  const ref = useRef<HTMLCanvasElement>(null);

  useEffect(() => {
    const canvas = ref.current;
    const context = canvas?.getContext('2d');
    if (!canvas || !context) return undefined;
    const graphCanvas = canvas;
    const ctx = context;

    const reduceMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    const palette = ['#818cf8', '#38bdf8', '#f97316', '#10b981', '#f472b6', '#94a3b8', '#94a3b8', '#94a3b8'];
    let width = 0;
    let height = 0;
    let nodes: GraphNode[] = [];
    let edges: Array<[number, number]> = [];
    let frame = 0;
    let resizeTimer: number | undefined;
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
        nodes
          .map((other, otherIndex) => ({ otherIndex, distance: (other.x - node.x) ** 2 + (other.y - node.y) ** 2 }))
          .filter((other) => other.otherIndex !== index)
          .sort((a, b) => a.distance - b.distance)
          .slice(0, countEdges)
          .forEach((other) => edges.push([index, other.otherIndex]));
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

    build();
    if (reduceMotion) {
      step(0);
      cancelAnimationFrame(frame);
    } else {
      frame = requestAnimationFrame(step);
    }
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
  }, []);

  return <canvas id="graph-canvas" ref={ref} aria-hidden="true" />;
}
