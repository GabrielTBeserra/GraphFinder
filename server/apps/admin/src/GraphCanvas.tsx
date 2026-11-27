import { useEffect, useRef, useState } from 'react';
import type { ProjectGraphEdge, ProjectGraphNode } from './api';

type SimNode = {
  node: ProjectGraphNode;
  x: number;
  y: number;
  vx: number;
  vy: number;
  radius: number;
  pinned: boolean;
};

type Props = {
  nodes: ProjectGraphNode[];
  edges: ProjectGraphEdge[];
  selectedId?: string | null;
  onSelectNode?: (node: ProjectGraphNode | null) => void;
};

// Community colors: golden-angle hue rotation keeps adjacent community ids
// maximally distinct; community 0 (the largest) always gets the brand green.
function communityColor(community: number): string {
  if (community === 0) return '#8fffbd';
  const hue = (community * 137.508) % 360;
  return `hsl(${hue.toFixed(0)}, 72%, 68%)`;
}

function nodeColor(node: ProjectGraphNode): string {
  return communityColor(node.community);
}

function shortLabel(node: ProjectGraphNode): string {
  if (node.title) return node.title;
  const name = node.path.split('/').pop() ?? node.path;
  return name.replace(/\.(md|rs|ts|tsx|js|jsx|py|go|java|c|cpp|h|css)$/, '');
}

/**
 * Dependency-free force-directed canvas graph, tuned for the control-room
 * aesthetic: hairline edges, glowing degree-weighted nodes, sparse labels.
 * The simulation cools down and stops consuming CPU once stable.
 */
export function GraphCanvas({ nodes, edges, selectedId, onSelectNode }: Props) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const wrapRef = useRef<HTMLDivElement | null>(null);
  const [hovered, setHovered] = useState<ProjectGraphNode | null>(null);
  const stateRef = useRef<{
    simNodes: SimNode[];
    links: Array<[number, number]>;
    panX: number;
    panY: number;
    zoom: number;
    alpha: number;
    dragging: number | null;
    panning: boolean;
    panStart: { x: number; y: number; panX: number; panY: number } | null;
    hoverIdx: number | null;
    selectedId: string | null;
    fitted: boolean;
  }>({
    simNodes: [],
    links: [],
    panX: 0,
    panY: 0,
    zoom: 1,
    alpha: 1,
    dragging: null,
    panning: false,
    panStart: null,
    hoverIdx: null,
    selectedId: null,
    fitted: false,
  });

  stateRef.current.selectedId = selectedId ?? null;

  // Rebuild simulation state when data changes
  useEffect(() => {
    const indexById = new Map<string, number>();
    const simNodes: SimNode[] = nodes.map((node, i) => {
      indexById.set(node.id, i);
      const angle = (i / Math.max(1, nodes.length)) * Math.PI * 2;
      const ring = 180 + (i % 7) * 30;
      return {
        node,
        x: Math.cos(angle) * ring + (Math.random() - 0.5) * 40,
        y: Math.sin(angle) * ring + (Math.random() - 0.5) * 40,
        vx: 0,
        vy: 0,
        radius: Math.min(11, 4 + Math.sqrt(node.degree) * 1.7),
        pinned: false,
      };
    });
    const links: Array<[number, number]> = [];
    edges.forEach((edge) => {
      const a = indexById.get(edge.source);
      const b = indexById.get(edge.target);
      if (a !== undefined && b !== undefined && a !== b) links.push([a, b]);
    });
    const st = stateRef.current;
    st.simNodes = simNodes;
    st.links = links;
    st.alpha = 1;
    st.fitted = false;
    st.zoom = 1;
    st.panX = 0;
    st.panY = 0;
  }, [nodes, edges]);

  useEffect(() => {
    const canvas = canvasRef.current;
    const wrap = wrapRef.current;
    if (!canvas || !wrap) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let raf = 0;
    let width = 0;
    let height = 0;
    const dpr = Math.min(2, window.devicePixelRatio || 1);

    const resize = () => {
      const rect = wrap.getBoundingClientRect();
      width = rect.width;
      height = rect.height;
      canvas.width = Math.max(1, Math.round(width * dpr));
      canvas.height = Math.max(1, Math.round(height * dpr));
      canvas.style.width = `${width}px`;
      canvas.style.height = `${height}px`;
    };
    resize();
    const observer = new ResizeObserver(resize);
    observer.observe(wrap);

    const tick = () => {
      const st = stateRef.current;
      const sim = st.simNodes;
      const n = sim.length;

      // ---- physics (cools down to zero) ----
      if (st.alpha > 0.005 && n > 0) {
        const alpha = st.alpha;
        // repulsion — O(n²) but cheap arithmetic; fine up to ~1.5k nodes
        for (let i = 0; i < n; i++) {
          const a = sim[i];
          for (let j = i + 1; j < n; j++) {
            const b = sim[j];
            let dx = a.x - b.x;
            let dy = a.y - b.y;
            let distSq = dx * dx + dy * dy;
            if (distSq < 1) {
              dx = (Math.random() - 0.5) * 2;
              dy = (Math.random() - 0.5) * 2;
              distSq = 1;
            }
            const dist = Math.sqrt(distSq);
            const force = (900 * alpha) / distSq;
            const fx = (dx / dist) * force;
            const fy = (dy / dist) * force;
            a.vx += fx;
            a.vy += fy;
            b.vx -= fx;
            b.vy -= fy;
          }
        }
        // springs
        const ideal = 90;
        for (const [ai, bi] of st.links) {
          const a = sim[ai];
          const b = sim[bi];
          const dx = b.x - a.x;
          const dy = b.y - a.y;
          const dist = Math.max(1, Math.sqrt(dx * dx + dy * dy));
          const force = ((dist - ideal) / dist) * 0.02 * alpha;
          const fx = dx * force;
          const fy = dy * force;
          a.vx += fx;
          a.vy += fy;
          b.vx -= fx;
          b.vy -= fy;
        }
        // center gravity + integration
        for (const node of sim) {
          node.vx -= node.x * 0.012 * alpha;
          node.vy -= node.y * 0.012 * alpha;
          if (!node.pinned) {
            node.x += node.vx * 0.5;
            node.y += node.vy * 0.5;
          }
          node.vx *= 0.82;
          node.vy *= 0.82;
        }
        st.alpha *= 0.985;
      }

      // keep framing the graph until the user takes control (pan/zoom/drag)
      if (!st.fitted && n > 0 && width > 0) {
        let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
        for (const node of sim) {
          minX = Math.min(minX, node.x);
          maxX = Math.max(maxX, node.x);
          minY = Math.min(minY, node.y);
          maxY = Math.max(maxY, node.y);
        }
        const spanX = Math.max(100, maxX - minX);
        const spanY = Math.max(100, maxY - minY);
        st.zoom = Math.min(1.4, Math.max(0.08, Math.min((width - 80) / spanX, (height - 80) / spanY)));
        st.panX = width / 2 - ((minX + maxX) / 2) * st.zoom;
        st.panY = height / 2 - ((minY + maxY) / 2) * st.zoom;
      }

      // ---- render ----
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
      ctx.clearRect(0, 0, width, height);

      const toScreen = (x: number, y: number): [number, number] => [
        x * st.zoom + st.panX,
        y * st.zoom + st.panY,
      ];

      // edges
      ctx.lineWidth = 1;
      for (const [ai, bi] of st.links) {
        const a = sim[ai];
        const b = sim[bi];
        const [ax, ay] = toScreen(a.x, a.y);
        const [bx, by] = toScreen(b.x, b.y);
        const highlight =
          st.hoverIdx !== null && (ai === st.hoverIdx || bi === st.hoverIdx);
        ctx.strokeStyle = highlight ? 'rgba(143,255,189,.4)' : 'rgba(129,145,142,.14)';
        ctx.beginPath();
        ctx.moveTo(ax, ay);
        ctx.lineTo(bx, by);
        ctx.stroke();
      }

      // label budget: top-degree nodes plus hover/selection
      const labelCandidates = new Set<number>();
      if (st.zoom > 0.35) {
        const byDegree = sim
          .map((s, i) => [s.node.degree, i] as [number, number])
          .sort((a, b) => b[0] - a[0])
          .slice(0, 18);
        byDegree.forEach(([, i]) => labelCandidates.add(i));
      }

      for (let i = 0; i < n; i++) {
        const s = sim[i];
        const [sx, sy] = toScreen(s.x, s.y);
        if (sx < -30 || sy < -30 || sx > width + 30 || sy > height + 30) continue;
        const isHover = st.hoverIdx === i;
        const isSelected = st.selectedId === s.node.id;
        const r = s.radius * Math.max(0.55, Math.min(1.6, st.zoom));

        if (isHover || isSelected) {
          ctx.fillStyle = 'rgba(143,255,189,.12)';
          ctx.beginPath();
          ctx.arc(sx, sy, r + 7, 0, Math.PI * 2);
          ctx.fill();
        }
        ctx.fillStyle = nodeColor(s.node);
        ctx.beginPath();
        ctx.arc(sx, sy, r, 0, Math.PI * 2);
        ctx.fill();
        if (s.node.is_god_node) {
          // architectural hubs get a halo ring — visible even at low zoom
          ctx.strokeStyle = 'rgba(232,240,238,.75)';
          ctx.lineWidth = 1.2;
          ctx.beginPath();
          ctx.arc(sx, sy, r + 3, 0, Math.PI * 2);
          ctx.stroke();
        }
        if (isSelected) {
          ctx.strokeStyle = '#e8f0ee';
          ctx.lineWidth = 1.5;
          ctx.stroke();
        }

        if (isHover || isSelected || labelCandidates.has(i)) {
          const label = shortLabel(s.node);
          ctx.font = '10px "JetBrains Mono", monospace';
          ctx.fillStyle = isHover || isSelected ? '#e8f0ee' : 'rgba(232,240,238,.55)';
          ctx.fillText(label, sx + r + 4, sy + 3);
        }
      }

      raf = requestAnimationFrame(tick);
    };
    raf = requestAnimationFrame(tick);

    // ---- interaction ----
    const posToWorld = (clientX: number, clientY: number): [number, number] => {
      const rect = canvas.getBoundingClientRect();
      const st = stateRef.current;
      return [
        (clientX - rect.left - st.panX) / st.zoom,
        (clientY - rect.top - st.panY) / st.zoom,
      ];
    };

    const nearestNode = (clientX: number, clientY: number): number | null => {
      const st = stateRef.current;
      const [wx, wy] = posToWorld(clientX, clientY);
      let best: number | null = null;
      let bestDist = Infinity;
      st.simNodes.forEach((s, i) => {
        const dx = s.x - wx;
        const dy = s.y - wy;
        const dist = Math.sqrt(dx * dx + dy * dy);
        const hit = Math.max(10 / st.zoom, s.radius + 4);
        if (dist < hit && dist < bestDist) {
          best = i;
          bestDist = dist;
        }
      });
      return best;
    };

    const onMove = (event: MouseEvent) => {
      const st = stateRef.current;
      if (st.dragging !== null) {
        const [wx, wy] = posToWorld(event.clientX, event.clientY);
        const s = st.simNodes[st.dragging];
        s.x = wx;
        s.y = wy;
        s.vx = 0;
        s.vy = 0;
        s.pinned = true;
        return;
      }
      if (st.panning && st.panStart) {
        st.panX = st.panStart.panX + (event.clientX - st.panStart.x);
        st.panY = st.panStart.panY + (event.clientY - st.panStart.y);
        return;
      }
      const idx = nearestNode(event.clientX, event.clientY);
      st.hoverIdx = idx;
      setHovered(idx !== null ? st.simNodes[idx].node : null);
      canvas.style.cursor = idx !== null ? 'pointer' : 'grab';
    };

    const onDown = (event: MouseEvent) => {
      const st = stateRef.current;
      st.fitted = true;
      const idx = nearestNode(event.clientX, event.clientY);
      if (idx !== null) {
        st.dragging = idx;
      } else {
        st.panning = true;
        st.panStart = { x: event.clientX, y: event.clientY, panX: st.panX, panY: st.panY };
        canvas.style.cursor = 'grabbing';
      }
    };

    const onUp = (event: MouseEvent) => {
      const st = stateRef.current;
      if (st.dragging !== null) {
        const idx = st.dragging;
        const s = st.simNodes[idx];
        s.pinned = false;
        st.dragging = null;
        const movedIdx = nearestNode(event.clientX, event.clientY);
        if (movedIdx === idx && onSelectNode) onSelectNode(s.node);
      } else if (st.panning && st.panStart) {
        const moved =
          Math.abs(event.clientX - st.panStart.x) + Math.abs(event.clientY - st.panStart.y);
        if (moved < 4 && onSelectNode) onSelectNode(null);
      }
      st.panning = false;
      st.panStart = null;
      canvas.style.cursor = 'grab';
    };

    const onWheel = (event: WheelEvent) => {
      event.preventDefault();
      const st = stateRef.current;
      st.fitted = true;
      const rect = canvas.getBoundingClientRect();
      const mx = event.clientX - rect.left;
      const my = event.clientY - rect.top;
      const factor = event.deltaY < 0 ? 1.12 : 0.89;
      const next = Math.min(4, Math.max(0.06, st.zoom * factor));
      st.panX = mx - ((mx - st.panX) / st.zoom) * next;
      st.panY = my - ((my - st.panY) / st.zoom) * next;
      st.zoom = next;
    };

    canvas.addEventListener('mousemove', onMove);
    canvas.addEventListener('mousedown', onDown);
    window.addEventListener('mouseup', onUp);
    canvas.addEventListener('wheel', onWheel, { passive: false });

    return () => {
      cancelAnimationFrame(raf);
      observer.disconnect();
      canvas.removeEventListener('mousemove', onMove);
      canvas.removeEventListener('mousedown', onDown);
      window.removeEventListener('mouseup', onUp);
      canvas.removeEventListener('wheel', onWheel);
    };
  }, [onSelectNode]);

  return (
    <div className="graph-canvas-wrap" ref={wrapRef}>
      <canvas ref={canvasRef} />
      {hovered && (
        <div className="graph-tooltip">
          <strong>{shortLabel(hovered)}</strong>
          <span>{hovered.path}</span>
          <em>
            {hovered.document_type}
            {hovered.language ? ` · ${hovered.language}` : ''} · {hovered.degree} links
          </em>
        </div>
      )}
      <div className="graph-legend">
        <span><i style={{ background: '#8fffbd' }} /> colors = communities</span>
        <span>{nodes.length} nodes · drag to rearrange</span>
      </div>
    </div>
  );
}
