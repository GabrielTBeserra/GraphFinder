import React, { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import ForceGraph from 'force-graph';
import {
  MagnifyingGlassPlus,
  MagnifyingGlassMinus,
  ArrowsOut,
  ArrowClockwise,
  Crosshair,
  Compass,
  Code,
  FileText,
  CirclesThree,
  Crown,
  GitBranch,
  ChartBar,
  SquaresFour,
  X,
} from '@phosphor-icons/react';
import type { GraphData, PathResult, PathStep } from '../types';
import { findPath } from '../services/api';
import { ArchitecturalReportModal } from './ArchitecturalReportModal';
import { Tip } from './Tip';

interface GraphViewProps {
  projectId?: string;
  graphData: GraphData;
  selectedNodeId: string | null;
  onSelectNode: (nodeId: string) => void;
  onSelectCode?: (fileId: string) => void;
  onClearSelection?: () => void;
  onRefresh: () => void;
  glowingNode?: { id: string; timestamp: number } | null;
  theme?: 'dark' | 'light';
  onCreateNote?: () => void;
  onSyncVault?: () => void;
}

const COMMUNITY_PALETTE = [
  '#6366f1', '#10b981', '#38bdf8', '#f59e0b', '#ec4899',
  '#8b5cf6', '#06b6d4', '#f97316', '#14b8a6', '#e11d48',
  '#84cc16', '#a855f7', '#0ea5e9', '#eab308'
];

const LANG_COLORS: Record<string, string> = {
  typescript: '#38bdf8',
  javascript: '#facc15',
  rust: '#f97316',
  python: '#10b981',
  go: '#06b6d4',
  c: '#3b82f6',
  cpp: '#2563eb',
  csharp: '#a855f7',
  java: '#ea580c',
  kotlin: '#8b5cf6',
  php: '#6366f1',
  ruby: '#e11d48',
  swift: '#f43f5e',
  css: '#ec4899',
  shell: '#14b8a6',
};

const TAG_PALETTES: Record<string, { dark: string; light: string }> = {
  architecture: { dark: '#6366f1', light: '#4f46e5' }, // Indigo
  system: { dark: '#6366f1', light: '#4f46e5' },
  design: { dark: '#6366f1', light: '#4f46e5' },
  rust: { dark: '#f97316', light: '#ea580c' },        // Orange
  embeddings: { dark: '#10b981', light: '#059669' },  // Emerald
  rag: { dark: '#06b6d4', light: '#0891b2' },         // Cyan
  ai: { dark: '#10b981', light: '#059669' },          // Emerald
  mcp: { dark: '#8b5cf6', light: '#7c3aed' },         // Violet
  neural: { dark: '#06b6d4', light: '#0891b2' },      // Cyan
  guide: { dark: '#f59e0b', light: '#d97706' },       // Amber
  tutorial: { dark: '#f59e0b', light: '#d97706' },    // Amber
  welcome: { dark: '#e11d48', light: '#be123c' },     // Rose
  onboarding: { dark: '#ec4899', light: '#db2777' },  // Pink
  sqlite: { dark: '#8b5cf6', light: '#7c3aed' },      // Purple
  database: { dark: '#8b5cf6', light: '#7c3aed' },    // Purple
  storage: { dark: '#a855f7', light: '#9333ea' },
  security: { dark: '#ef4444', light: '#dc2626' },    // Red
  auth: { dark: '#f43f5e', light: '#e11d48' },
  docs: { dark: '#38bdf8', light: '#0284c7' },
};

const LARGE_GRAPH_NODE_LIMIT = 2000;
// Above this node count, viewport culling drops offscreen nodes entirely
// (they are neither painted nor hit-tested).
const CULL_NODE_THRESHOLD = 20000;
// Below this zoom factor the viewport covers most of the graph anyway —
// render only the top-K degree hubs (deterministic sampling).
const FAR_ZOOM_K = 0.35;
const FAR_ZOOM_BUDGET = 15000;
// If this fraction of nodes arrives with precomputed positions, the force
// simulation stays frozen (Rust-side layout did the work already).
const PRECOMPUTED_LAYOUT_RATIO = 0.8;

const VIBRANT_ACCENTS = [
  { dark: '#6366f1', light: '#4f46e5' }, // Indigo
  { dark: '#10b981', light: '#059669' }, // Emerald
  { dark: '#38bdf8', light: '#0284c7' }, // Sky
  { dark: '#f59e0b', light: '#d97706' }, // Amber
  { dark: '#8b5cf6', light: '#7c3aed' }, // Violet
  { dark: '#ec4899', light: '#db2777' }, // Pink
  { dark: '#06b6d4', light: '#0891b2' }, // Cyan
  { dark: '#f97316', light: '#ea580c' }, // Orange
  { dark: '#14b8a6', light: '#0d9488' }, // Teal
  { dark: '#e11d48', light: '#be123c' }, // Rose
];

function hashStringToColor(str: string, isLight: boolean): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = (hash << 5) - hash + str.charCodeAt(i);
    hash |= 0;
  }
  const index = Math.abs(hash) % VIBRANT_ACCENTS.length;
  return isLight ? VIBRANT_ACCENTS[index].light : VIBRANT_ACCENTS[index].dark;
}

interface Point2D {
  x: number;
  y: number;
}

function computeConvexHull(points: Point2D[]): Point2D[] {
  if (points.length <= 2) return points.slice();
  const sorted = points
    .slice()
    .sort((a, b) => (a.x !== b.x ? a.x - b.x : a.y - b.y));

  const cross = (o: Point2D, a: Point2D, b: Point2D) =>
    (a.x - o.x) * (b.y - o.y) - (a.y - o.y) * (b.x - o.x);

  const lower: Point2D[] = [];
  for (const p of sorted) {
    while (lower.length >= 2 && cross(lower[lower.length - 2], lower[lower.length - 1], p) <= 0) {
      lower.pop();
    }
    lower.push(p);
  }

  const upper: Point2D[] = [];
  for (let i = sorted.length - 1; i >= 0; i--) {
    const p = sorted[i];
    while (upper.length >= 2 && cross(upper[upper.length - 2], upper[upper.length - 1], p) <= 0) {
      upper.pop();
    }
    upper.push(p);
  }

  upper.pop();
  lower.pop();
  return lower.concat(upper);
}

function drawSmoothHull(ctx: CanvasRenderingContext2D, hull: Point2D[]) {
  if (hull.length < 3) return;
  ctx.beginPath();
  const len = hull.length;
  const firstMidX = (hull[len - 1].x + hull[0].x) / 2;
  const firstMidY = (hull[len - 1].y + hull[0].y) / 2;
  ctx.moveTo(firstMidX, firstMidY);
  for (let i = 0; i < len; i++) {
    const p1 = hull[i];
    const p2 = hull[(i + 1) % len];
    const midX = (p1.x + p2.x) / 2;
    const midY = (p1.y + p2.y) / 2;
    ctx.quadraticCurveTo(p1.x, p1.y, midX, midY);
  }
  ctx.closePath();
}

function getNodeColor(node: any, isLight: boolean = false, colorMode: 'communities' | 'default' = 'communities'): string {
  if (colorMode === 'communities' && typeof node.community === 'number') {
    return COMMUNITY_PALETTE[node.community % COMMUNITY_PALETTE.length];
  }
  if (node.node_type === 'code') {
    const lang = (node.tags && node.tags[0]) || '';
    return LANG_COLORS[lang.toLowerCase()] || (isLight ? '#0284c7' : '#38bdf8');
  }
  if (node.node_type === 'unresolved') {
    return isLight ? '#94a3b8' : '#64748b';
  }
  // Check tags
  if (node.tags && Array.isArray(node.tags)) {
    for (const t of node.tags) {
      const cleanTag = t.toLowerCase().replace(/^#/, '');
      if (TAG_PALETTES[cleanTag]) {
        return isLight ? TAG_PALETTES[cleanTag].light : TAG_PALETTES[cleanTag].dark;
      }
    }
  }
  // Check title
  const title = (node.title || node.id || '').replace(/\.md$/, '').toLowerCase();
  for (const [key, palette] of Object.entries(TAG_PALETTES)) {
    if (title.includes(key)) {
      return isLight ? palette.light : palette.dark;
    }
  }
  return hashStringToColor(node.id || 'note', isLight);
}

export const GraphView: React.FC<GraphViewProps> = ({
  projectId,
  graphData,
  selectedNodeId,
  onSelectNode,
  onSelectCode,
  onClearSelection,
  onRefresh,
  glowingNode,
  theme = 'dark',
  onCreateNote,
  onSyncVault,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphInstanceRef = useRef<any>(null);
  const prevProjectIdRef = useRef<string | undefined>(projectId);
  const [localOnly, setLocalOnly] = useState<boolean>(false);
  const [egoDepth, setEgoDepth] = useState<number>(1);
  const [filterMode, setFilterMode] = useState<'all' | 'notes' | 'code'>('all');
  const [colorMode, setColorMode] = useState<'communities' | 'default'>('communities');
  const [highlightGodNodes, setHighlightGodNodes] = useState<boolean>(true);
  const [pathMode, setPathMode] = useState<boolean>(false);
  const [pathStartNode, setPathStartNode] = useState<any | null>(null);
  const [pathResult, setPathResult] = useState<PathResult | null>(null);
  const [showAuditModal, setShowAuditModal] = useState<boolean>(false);

  // Interaction refs: keep rendering independent of React state to prevent canvas flickers
  const selectedNodeIdRef = useRef<string | null>(selectedNodeId);
  const hoverNodeIdRef = useRef<string | null>(null);
  const connectedNodeIdsRef = useRef<Set<string>>(new Set());
  const egoDepthRef = useRef<number>(egoDepth);
  const onSelectNodeRef = useRef(onSelectNode);
  const onSelectCodeRef = useRef(onSelectCode);
  const onClearSelectionRef = useRef(onClearSelection);
  const themeRef = useRef<'dark' | 'light'>(theme);
  const colorModeRef = useRef<'communities' | 'default'>('communities');
  const highlightGodNodesRef = useRef<boolean>(true);
  const pathModeRef = useRef<boolean>(false);
  const pathStartNodeRef = useRef<any | null>(null);
  const pathResultRef = useRef<PathResult | null>(null);

  const requestCanvasRepaint = useCallback(() => {
    const fg = graphInstanceRef.current;
    if (!fg) return;
    try {
      if (typeof fg.linkColor === 'function') {
        fg.linkColor(fg.linkColor());
      }
    } catch {
      // ignore repaint errors
    }
  }, []);

  useEffect(() => {
    colorModeRef.current = colorMode;
    requestCanvasRepaint();
  }, [colorMode, requestCanvasRepaint]);

  useEffect(() => {
    highlightGodNodesRef.current = highlightGodNodes;
    requestCanvasRepaint();
  }, [highlightGodNodes, requestCanvasRepaint]);

  useEffect(() => {
    pathModeRef.current = pathMode;
    if (!pathMode) {
      setPathStartNode(null);
      setPathResult(null);
      pathStartNodeRef.current = null;
      pathResultRef.current = null;
      requestCanvasRepaint();
    }
  }, [pathMode, requestCanvasRepaint]);

  useEffect(() => {
    pathStartNodeRef.current = pathStartNode;
    requestCanvasRepaint();
  }, [pathStartNode, requestCanvasRepaint]);

  useEffect(() => {
    pathResultRef.current = pathResult;
    requestCanvasRepaint();
  }, [pathResult, requestCanvasRepaint]);

  useEffect(() => {
    themeRef.current = theme;
    if (graphInstanceRef.current) {
      try {
        graphInstanceRef.current.backgroundColor(
          getComputedStyle(document.documentElement).getPropertyValue('--gf-canvas').trim() ||
            (theme === 'light' ? '#faf6ef' : '#120f0d')
        );
        requestCanvasRepaint();
      } catch (err) {
        console.warn('Error updating graph theme:', err);
      }
    }
  }, [theme, requestCanvasRepaint]);

  // Glowing animation ref
  const glowingStateRef = useRef<{ id: string; lc: string; stemLc: string; startTime: number; duration: number } | null>(null);
  // Memoized lowercase/stem of the selected node id for per-frame comparisons
  const selNormCacheRef = useRef<{ sel: string | null; lc: string | null; stemLc: string | null }>({
    sel: null,
    lc: null,
    stemLc: null,
  });

  // Cached community hulls — recomputed at most every 400ms, never per frame
  const cachedHullsRef = useRef<Map<number, { hull: Point2D[]; minY: number; centerX: number; count: number }>>(new Map());
  const hullsComputedAtRef = useRef<number>(0);
  // id -> community lookup, rebuilt when graph data changes (single O(N) pass)
  const nodeCommunityByIdRef = useRef<Map<string, number>>(new Map());

  // Graph data ref to avoid re-triggering effects when links change
  const graphDataRef = useRef<GraphData>(graphData);
  useEffect(() => {
    graphDataRef.current = graphData;
  }, [graphData]);

  const lastRenderedKeyRef = useRef<string>('');

  // Viewport culling / zoom transform state (mutable, never in React state)
  const transformRef = useRef({ k: 1, x: 0, y: 0 });
  const cullingEnabledRef = useRef(false);
  const visibilityRafRef = useRef(0);
  const degreeRankRef = useRef<Set<string>>(new Set());
  // Lazily-built adjacency map so hover/selection BFS never rescans all links
  const adjacencyCacheRef = useRef<{ source: GraphData | null; map: Map<string, string[]> }>({
    source: null,
    map: new Map(),
  });

  useEffect(() => {
    onSelectNodeRef.current = onSelectNode;
  }, [onSelectNode]);

  useEffect(() => {
    onSelectCodeRef.current = onSelectCode;
  }, [onSelectCode]);

  useEffect(() => {
    onClearSelectionRef.current = onClearSelection;
  }, [onClearSelection]);

  // Telemetry stats
  const stats = useMemo(() => {
    let notes = 0;
    let code = 0;
    for (const n of graphData.nodes) {
      if (n.node_type === 'code') code++;
      else notes++;
    }
    return { notes, code, total: graphData.nodes.length, links: graphData.links.length };
  }, [graphData]);

  // Recompute N-hop ego network connected node IDs based on active target (hover or selected)
  const recomputeConnected = useCallback((targetId: string | null, maxHops?: number) => {
    if (!targetId) {
      connectedNodeIdsRef.current = new Set();
      return;
    }

    const nodes = graphDataRef.current.nodes;
    const resolvedTarget = nodes.find(
      (n) =>
        n.id === targetId ||
        n.id.toLowerCase() === targetId.toLowerCase() ||
        n.id.replace(/\.md$/i, '').toLowerCase() === targetId.replace(/\.md$/i, '').toLowerCase() ||
        (n.title && n.title.toLowerCase() === targetId.toLowerCase()) ||
        (n.title && n.title.replace(/\.md$/i, '').toLowerCase() === targetId.replace(/\.md$/i, '').toLowerCase()) ||
        (n.path && (n.path === targetId || n.path.toLowerCase() === targetId.toLowerCase()))
    );
    const startId = resolvedTarget ? resolvedTarget.id : targetId;

    const hops = maxHops ?? egoDepthRef.current;
    if (adjacencyCacheRef.current.source !== graphDataRef.current) {
      const map = new Map<string, string[]>();
      for (const link of graphDataRef.current.links) {
        const s = typeof link.source === 'object' ? (link.source as any).id : link.source;
        const t = typeof link.target === 'object' ? (link.target as any).id : link.target;
        if (!map.has(s)) map.set(s, []);
        if (!map.has(t)) map.set(t, []);
        map.get(s)!.push(t);
        map.get(t)!.push(s);
      }
      adjacencyCacheRef.current = { source: graphDataRef.current, map };
    }
    const adj = adjacencyCacheRef.current.map;

    const visited = new Set<string>([startId]);
    let frontier = [startId];
    for (let d = 0; d < hops && frontier.length > 0; d++) {
      const nextFrontier: string[] = [];
      for (const curr of frontier) {
        const neighbors = adj.get(curr) || [];
        for (const nb of neighbors) {
          if (!visited.has(nb)) {
            visited.add(nb);
            nextFrontier.push(nb);
          }
        }
      }
      frontier = nextFrontier;
    }
    connectedNodeIdsRef.current = visited;
  }, []);

  // Viewport culling: flags node.__vis so offscreen/sampled-out nodes are
  // neither rendered nor hit-tested. Runs on zoom/pan, rAF-throttled.
  const recomputeVisibility = useCallback(() => {
    const g = graphInstanceRef.current;
    if (!g || !cullingEnabledRef.current) return;
    const nodes: any[] = g.graphData()?.nodes || [];
    const w = g.width?.() || 0;
    const h = g.height?.() || 0;
    if (!w || !h) return;

    const keep = new Set<string>();
    if (selectedNodeIdRef.current) keep.add(selectedNodeIdRef.current);
    if (hoverNodeIdRef.current) keep.add(hoverNodeIdRef.current);
    if (glowingStateRef.current) keep.add(glowingStateRef.current.id);
    for (const id of connectedNodeIdsRef.current) keep.add(id);
    const steps = pathResultRef.current?.steps;
    if (steps) {
      for (const s of steps) keep.add(s.node_id);
    }

    const glowing = glowingStateRef.current;
    const isGlowMatch = (n: any) =>
      !!glowing &&
      (n.id === glowing.id || n._idLc === glowing.lc || n._idStemLc === glowing.stemLc);

    const { k, x, y } = transformRef.current;
    if (k < FAR_ZOOM_K) {
      const rank = degreeRankRef.current;
      for (const n of nodes) {
        n.__vis = keep.has(n.id) || n.is_god_node || rank.has(n.id) || isGlowMatch(n);
      }
    } else {
      const padX = (w / k) * 0.12;
      const padY = (h / k) * 0.12;
      const left = -x / k - padX;
      const right = (w - x) / k + padX;
      const top = -y / k - padY;
      const bottom = (h - y) / k + padY;
      for (const n of nodes) {
        n.__vis =
          keep.has(n.id) ||
          isGlowMatch(n) ||
          (n.x >= left && n.x <= right && n.y >= top && n.y <= bottom);
      }
    }
    requestCanvasRepaint();
  }, [requestCanvasRepaint]);

  const scheduleVisibilityRecompute = useCallback(() => {
    if (!cullingEnabledRef.current || visibilityRafRef.current) return;
    visibilityRafRef.current = requestAnimationFrame(() => {
      visibilityRafRef.current = 0;
      recomputeVisibility();
    });
  }, [recomputeVisibility]);

  // ── Minimap ──────────────────────────────────────────────────────
  const minimapRef = useRef<HTMLCanvasElement>(null);
  const minimapBoundsRef = useRef({ minX: 0, minY: 0, maxX: 1, maxY: 1 });
  const MINIMAP_W = 168;
  const MINIMAP_H = 112;

  const paintMinimap = useCallback(() => {
    const canvas = minimapRef.current;
    const g = graphInstanceRef.current;
    if (!canvas || !g) return;
    const data = g.graphData?.() || { nodes: [], links: [] };
    const nodes: any[] = data.nodes || [];
    if (nodes.length === 0) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let minX = Infinity, maxX = -Infinity, minY = Infinity, maxY = -Infinity;
    for (const n of nodes) {
      if (!isFinite(n.x) || !isFinite(n.y)) continue;
      if (n.x < minX) minX = n.x;
      if (n.x > maxX) maxX = n.x;
      if (n.y < minY) minY = n.y;
      if (n.y > maxY) maxY = n.y;
    }
    if (!isFinite(minX)) return;
    const pad = Math.max(20, (maxX - minX) * 0.06);
    minX -= pad; maxX += pad; minY -= pad; maxY += pad;
    minimapBoundsRef.current = { minX, minY, maxX, maxY };

    const w = canvas.width, h = canvas.height;
    const sx = w / (maxX - minX), sy = h / (maxY - minY);
    const isLight = themeRef.current === 'light';
    ctx.clearRect(0, 0, w, h);

    const idToNode = new Map<string, any>();
    for (const n of nodes) idToNode.set(n.id, n);

    // Edges — cheap straight segments
    ctx.strokeStyle = isLight ? 'rgba(100,116,139,0.22)' : 'rgba(148,163,184,0.14)';
    ctx.lineWidth = 0.5;
    ctx.beginPath();
    for (const l of data.links || []) {
      const s = typeof l.source === 'object' ? l.source : idToNode.get(l.source);
      const t = typeof l.target === 'object' ? l.target : idToNode.get(l.target);
      if (!s || !t || !isFinite(s.x) || !isFinite(t.x)) continue;
      ctx.moveTo((s.x - minX) * sx, (s.y - minY) * sy);
      ctx.lineTo((t.x - minX) * sx, (t.y - minY) * sy);
    }
    ctx.stroke();

    // Nodes — 2px color dots
    for (const n of nodes) {
      if (!isFinite(n.x) || !isFinite(n.y)) continue;
      ctx.fillStyle = getNodeColor(n, isLight, colorModeRef.current);
      ctx.fillRect((n.x - minX) * sx - 1, (n.y - minY) * sy - 1, 2.2, 2.2);
    }

    // Viewport rectangle from the current zoom transform
    const { k, x, y } = transformRef.current;
    const gw = g.width?.() || 0, gh = g.height?.() || 0;
    if (gw && gh && k > 0) {
      const left = -x / k, top = -y / k, vw = gw / k, vh = gh / k;
      ctx.strokeStyle = isLight ? 'rgba(79,70,229,0.85)' : 'rgba(129,140,248,0.9)';
      ctx.lineWidth = 1.2;
      ctx.strokeRect((left - minX) * sx, (top - minY) * sy, vw * sx, vh * sy);
    }
  }, []);

  useEffect(() => {
    const id = setInterval(paintMinimap, 350);
    return () => clearInterval(id);
  }, [paintMinimap]);

  const handleMinimapClick = useCallback((e: React.MouseEvent<HTMLCanvasElement>) => {
    const g = graphInstanceRef.current;
    const canvas = minimapRef.current;
    if (!g || !canvas) return;
    const b = minimapBoundsRef.current;
    const rect = canvas.getBoundingClientRect();
    const px = ((e.clientX - rect.left) / rect.width) * canvas.width;
    const py = ((e.clientY - rect.top) / rect.height) * canvas.height;
    const gx = b.minX + px / (canvas.width / (b.maxX - b.minX));
    const gy = b.minY + py / (canvas.height / (b.maxY - b.minY));
    g.centerAt(gx, gy, 400);
  }, []);

  useEffect(() => {
    egoDepthRef.current = egoDepth;
    recomputeConnected(hoverNodeIdRef.current || selectedNodeIdRef.current, egoDepth);
    if (localOnly) {
      graphInstanceRef.current?.d3ReheatSimulation();
    } else {
      requestCanvasRepaint();
    }
  }, [egoDepth, localOnly, recomputeConnected, requestCanvasRepaint]);

  // Handle Glowing Node and Camera Auto-Centering
  useEffect(() => {
    if (!glowingNode || !glowingNode.id) return;
    const targetId = glowingNode.id;

    const glowLc = targetId.toLowerCase();
    glowingStateRef.current = {
      id: targetId,
      lc: glowLc,
      stemLc: glowLc.replace(/\.md$/i, ''),
      startTime: Date.now(),
      duration: 3500, // 3.5 seconds neon animation
    };

    // Auto-center camera onto the newly updated node
    const attemptCenter = () => {
      if (graphInstanceRef.current) {
        const currentData = graphInstanceRef.current.graphData();
        const nodes = currentData?.nodes || [];
        const cleanTarget = targetId.replace(/\.md$/, '').toLowerCase();

        const match = nodes.find(
          (n: any) =>
            n.id === targetId ||
            n.id?.toLowerCase() === cleanTarget ||
            n.title?.toLowerCase() === cleanTarget ||
            n.id?.replace(/\.md$/, '')?.toLowerCase() === cleanTarget
        );

        if (match && match.x !== undefined && match.y !== undefined) {
          graphInstanceRef.current.centerAt(match.x, match.y, 800);
          graphInstanceRef.current.zoom(2.4, 800);
        }
      }
    };

    setTimeout(attemptCenter, 80);

    // Keep repainting while the neon glow animates (autoPauseRedraw is on-demand)
    const repaintInterval = setInterval(() => {
      requestCanvasRepaint();
    }, 50);

    const timer = setTimeout(() => {
      glowingStateRef.current = null;
      clearInterval(repaintInterval);
      requestCanvasRepaint();
    }, glowingStateRef.current.duration);

    return () => {
      clearTimeout(timer);
      clearInterval(repaintInterval);
    };
  }, [glowingNode, requestCanvasRepaint]);

  // Initialize ForceGraph once
  useEffect(() => {
    if (!containerRef.current) return;

    const elem = containerRef.current;
    elem.innerHTML = '';
    const width = elem.clientWidth || 800;
    const height = elem.clientHeight || 600;

    const fgFactory = (ForceGraph as any).default || ForceGraph;
    const graph = fgFactory()(elem);

    graph
      .width(width)
      .height(height)
      .backgroundColor(
        getComputedStyle(document.documentElement).getPropertyValue('--gf-canvas').trim() ||
          (themeRef.current === 'light' ? '#faf6ef' : '#120f0d')
      )
      .autoPauseRedraw(true)
      .minZoom(0.05)
      .maxZoom(8)
      .nodeRelSize(4)
      .nodeVal((node: any) => Math.max(4, Math.min(12, 4 + (node.degree || 0) * 1.2)))
      .linkColor((link: any) => {
        const s = typeof link.source === 'object' ? link.source.id : link.source;
        const t = typeof link.target === 'object' ? link.target.id : link.target;
        const connected = connectedNodeIdsRef.current;
        const isImports = link.label === 'imports';
        const isLight = themeRef.current === 'light';

        // Path finding priority highlight
        if (pathResultRef.current && pathResultRef.current.steps.length > 1) {
          const steps = pathResultRef.current.steps;
          const isOnPath = steps.some((step, idx) => {
            if (idx === 0) return false;
            const prev = steps[idx - 1];
            return (prev.node_id === s && step.node_id === t) || (prev.node_id === t && step.node_id === s);
          });
          if (isOnPath) {
            return isLight ? '#0284c7' : '#38bdf8';
          }
        }

        if (connected.size > 0) {
          if (connected.has(s) && connected.has(t)) {
            return isImports
              ? (isLight ? '#0284c7' : '#38bdf8')
              : (isLight ? '#4f46e5' : '#818cf8');
          }
          return isLight ? 'rgba(203, 213, 225, 0.25)' : 'rgba(44, 51, 68, 0.12)';
        }
        return isImports
          ? (isLight ? 'rgba(2, 132, 199, 0.35)' : 'rgba(56, 189, 248, 0.25)')
          : (isLight ? 'rgba(99, 102, 241, 0.35)' : 'rgba(99, 102, 241, 0.22)');
      })
      .linkWidth((link: any) => {
        const s = typeof link.source === 'object' ? link.source.id : link.source;
        const t = typeof link.target === 'object' ? link.target.id : link.target;

        if (pathResultRef.current && pathResultRef.current.steps.length > 1) {
          const steps = pathResultRef.current.steps;
          const isOnPath = steps.some((step, idx) => {
            if (idx === 0) return false;
            const prev = steps[idx - 1];
            return (prev.node_id === s && step.node_id === t) || (prev.node_id === t && step.node_id === s);
          });
          if (isOnPath) return 3.5;
        }

        const connected = connectedNodeIdsRef.current;
        if (connected.has(s) && connected.has(t)) return 2.2;
        return 1.2;
      })
      // Slight curvature disambiguates parallel/bidirectional edges.
      .linkCurvature(0.12)
      // Directional arrows on dependency ('imports') edges only — wikilinks
      // stay undirected so the semantics remain honest.
      .linkDirectionalArrowLength((link: any) => (link.label === 'imports' ? 5 : 0))
      .linkDirectionalArrowRelPos(1)
      .linkDirectionalArrowColor(() => {
        const isLight = themeRef.current === 'light';
        return isLight ? 'rgba(2, 132, 199, 0.55)' : 'rgba(56, 189, 248, 0.45)';
      })
      // Flowing particles on the edges of the focused ego-network (or path)
      // make the active neighborhood visually "alive" without noise elsewhere.
      .linkDirectionalParticles((link: any) => {
        const s = typeof link.source === 'object' ? link.source.id : link.source;
        const t = typeof link.target === 'object' ? link.target.id : link.target;
        if (pathResultRef.current && pathResultRef.current.steps.length > 1) {
          const steps = pathResultRef.current.steps;
          const onPath = steps.some((step, idx) => {
            if (idx === 0) return false;
            const prev = steps[idx - 1];
            return (prev.node_id === s && step.node_id === t) || (prev.node_id === t && step.node_id === s);
          });
          if (onPath) return 3;
        }
        const connected = connectedNodeIdsRef.current;
        return connected.size > 0 && connected.has(s) && connected.has(t) ? 2 : 0;
      })
      .linkDirectionalParticleSpeed(0.005)
      .linkDirectionalParticleWidth(2.2)
      .linkDirectionalParticleColor((link: any) =>
        link.label === 'imports' ? '#38bdf8' : '#a5b4fc'
      )
      // Dragging pins a node in place; right-click releases it back to the sim.
      .onNodeDragEnd((node: any) => {
        node.fx = node.x;
        node.fy = node.y;
      })
      .onNodeRightClick((node: any) => {
        if (!node) return;
        node.fx = undefined;
        node.fy = undefined;
        graphInstanceRef.current?.d3ReheatSimulation();
      })
      .onRenderFramePre((ctx: CanvasRenderingContext2D, globalScale: number) => {
        if (colorModeRef.current !== 'communities' || !graphInstanceRef.current) return;
        const currentData = graphInstanceRef.current.graphData();
        const nodes: any[] = currentData?.nodes || [];
        // Hull computation is O(N log N) — skip entirely on very large graphs
        if (nodes.length === 0 || nodes.length > LARGE_GRAPH_NODE_LIMIT) return;

        const isLight = themeRef.current === 'light';
        const connected = connectedNodeIdsRef.current;

        // Recompute hulls at most every 400ms — positions barely change per frame
        const now = Date.now();
        if (now - hullsComputedAtRef.current > 400) {
          hullsComputedAtRef.current = now;
          const groups = new Map<number, any[]>();
          for (const n of nodes) {
            if (typeof n.community !== 'number' || !isFinite(n.x) || !isFinite(n.y)) continue;
            if (!groups.has(n.community)) groups.set(n.community, []);
            groups.get(n.community)!.push(n);
          }

          const padRadius = 26;
          const angles = [0, Math.PI / 4, Math.PI / 2, (3 * Math.PI) / 4, Math.PI, (5 * Math.PI) / 4, (3 * Math.PI) / 2, (7 * Math.PI) / 4];
          const newHulls = new Map<number, { hull: Point2D[]; minY: number; centerX: number; count: number }>();

          groups.forEach((commNodes, commId) => {
            if (commNodes.length < 2) return;
            const paddedPoints: Point2D[] = [];
            let minY = Infinity;
            let sumX = 0;
            for (const n of commNodes) {
              sumX += n.x;
              if (n.y - padRadius < minY) minY = n.y - padRadius;
              for (const a of angles) {
                paddedPoints.push({
                  x: n.x + Math.cos(a) * padRadius,
                  y: n.y + Math.sin(a) * padRadius,
                });
              }
            }
            const hull = computeConvexHull(paddedPoints);
            if (hull.length >= 3) {
              newHulls.set(commId, {
                hull,
                minY,
                centerX: sumX / commNodes.length,
                count: commNodes.length,
              });
            }
          });
          cachedHullsRef.current = newHulls;
        }

        const connectedCommunities = new Set<number>();
        if (connected.size > 0) {
          for (const id of connected) {
            const c = nodeCommunityByIdRef.current.get(id);
            if (c !== undefined) connectedCommunities.add(c);
          }
        }

        cachedHullsRef.current.forEach((entry, commId) => {
          const hasActiveFocus = connected.size > 0;
          if (hasActiveFocus && !connectedCommunities.has(commId)) return;

          const color = COMMUNITY_PALETTE[commId % COMMUNITY_PALETTE.length];
          ctx.save();
          drawSmoothHull(ctx, entry.hull);
          ctx.fillStyle = isLight ? `${color}14` : `${color}12`;
          ctx.fill();
          ctx.lineWidth = Math.max(1 / globalScale, 0.8);
          ctx.setLineDash([5 / globalScale, 4 / globalScale]);
          ctx.strokeStyle = isLight ? `${color}45` : `${color}38`;
          ctx.stroke();

          if (globalScale > 0.45) {
            const fontSize = Math.max(9.5 / globalScale, 8);
            ctx.font = `600 ${fontSize}px 'Plus Jakarta Sans', monospace, sans-serif`;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'bottom';
            ctx.fillStyle = isLight ? `${color}cc` : `${color}aa`;
            ctx.fillText(`Cluster #${commId} (${entry.count})`, entry.centerX, entry.minY - 4 / globalScale);
          }
          ctx.restore();
        });
      })
      .nodeCanvasObject((node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
        if (!node || node.__vis === false || !isFinite(node.x) || !isFinite(node.y)) return;

        const isLight = themeRef.current === 'light';
        const currentSel = selectedNodeIdRef.current;
        // Normalize selection once per distinct value (memoized) instead of
        // running toLowerCase/regex on every node, every frame.
        const selCache = selNormCacheRef.current;
        if (selCache.sel !== currentSel) {
          const lc = currentSel?.toLowerCase() ?? null;
          selNormCacheRef.current = {
            sel: currentSel,
            lc,
            stemLc: lc ? lc.replace(/\.md$/i, '') : null,
          };
        }
        const { lc: selLc, stemLc: selStemLc } = selNormCacheRef.current;
        const isSelected =
          currentSel !== null &&
          (node.id === currentSel ||
            node._idLc === selLc ||
            node._idStemLc === selStemLc ||
            (selLc !== null && node._titleLc === selLc) ||
            (selStemLc !== null && node._titleStemLc === selStemLc) ||
            (node.path === currentSel || (selLc !== null && node._pathLc === selLc)));
        const isHovered = node.id === hoverNodeIdRef.current;
        const connected = connectedNodeIdsRef.current;
        const isConnected = connected.has(node.id);
        const isDimmed = connected.size > 0 && !isConnected;
        const isCode = node.node_type === 'code';
        const accentColor = getNodeColor(node, isLight, colorModeRef.current);

        // Check Glowing MCP State
        const glowing = glowingStateRef.current;
        const isGlowing =
          glowing &&
          (node.id === glowing.id ||
            node._idLc === glowing.lc ||
            node._idStemLc === glowing.stemLc ||
            node._titleLc === glowing.lc);

        // Level-of-detail: far zoom or huge graphs get a single cheap circle.
        // Selected/hovered/glowing nodes, path steps and God Nodes keep full detail.
        const hasPath = !!pathResultRef.current && pathResultRef.current.steps.length > 1;
        const nodeCount = (graphInstanceRef.current?.graphData().nodes || []).length;
        const isGod = highlightGodNodesRef.current && (node.is_god_node || (node.degree || 0) >= 12);
        const liteMode =
          (globalScale < 0.6 || nodeCount > LARGE_GRAPH_NODE_LIMIT) &&
          !isSelected &&
          !isHovered &&
          !isGlowing &&
          !isGod &&
          (!hasPath || !pathResultRef.current!.steps.some((s) => s.node_id === node.id));

        if (liteMode) {
          ctx.beginPath();
          ctx.arc(node.x, node.y, 4, 0, 2 * Math.PI, false);
          ctx.fillStyle = isDimmed
            ? (isLight ? 'rgba(203, 213, 225, 0.35)' : 'rgba(71, 85, 105, 0.25)')
            : accentColor;
          ctx.fill();
          return;
        }

        if (isGlowing && glowing) {
          const elapsed = Date.now() - glowing.startTime;
          const progress = Math.min(1, elapsed / glowing.duration);
          const alpha = Math.max(0, 1 - progress);

          ctx.save();
          const shockRadius1 = 12 + progress * 40;
          ctx.beginPath();
          ctx.arc(node.x, node.y, shockRadius1, 0, 2 * Math.PI, false);
          ctx.strokeStyle = `rgba(16, 185, 129, ${alpha * 0.9})`;
          ctx.lineWidth = Math.max(1, 3.5 * (1 - progress));
          ctx.shadowColor = '#10b981';
          ctx.shadowBlur = 18;
          ctx.stroke();

          ctx.beginPath();
          ctx.arc(node.x, node.y, 16, 0, 2 * Math.PI, false);
          ctx.fillStyle = `rgba(16, 185, 129, ${alpha * 0.35})`;
          ctx.fill();

          const badgeFontSize = Math.max(8.5 / globalScale, 8.5);
          ctx.font = `700 ${badgeFontSize}px 'Plus Jakarta Sans', monospace, sans-serif`;
          ctx.textAlign = 'center';
          ctx.textBaseline = 'bottom';
          ctx.fillStyle = `rgba(52, 211, 153, ${alpha})`;
          ctx.shadowColor = '#10b981';
          ctx.shadowBlur = 8;
          ctx.fillText('MCP LIVE', node.x, node.y - 18);
          ctx.restore();
        }

        // God Node / Hub halo effect
        if (isGod && !isDimmed) {
          ctx.save();
          ctx.beginPath();
          ctx.arc(node.x, node.y, Math.max(14, (node.degree || 0) * 1.5 + 6), 0, 2 * Math.PI, false);
          ctx.strokeStyle = 'rgba(245, 158, 11, 0.65)';
          ctx.lineWidth = 1.6;
          ctx.setLineDash([3, 3]);
          ctx.shadowColor = '#f59e0b';
          ctx.shadowBlur = 8;
          ctx.stroke();
          ctx.restore();
        }

        // Path finding node highlight
        const isPathStart = pathStartNodeRef.current && pathStartNodeRef.current.id === node.id;
        const isPathStep = pathResultRef.current?.steps?.some((s: PathStep) => s.node_id === node.id);
        if (isPathStart || isPathStep) {
          ctx.save();
          ctx.beginPath();
          ctx.arc(node.x, node.y, 14, 0, 2 * Math.PI, false);
          ctx.strokeStyle = isPathStart ? '#38bdf8' : '#06b6d4';
          ctx.lineWidth = 2.4;
          if (isPathStart) ctx.setLineDash([4, 2]);
          ctx.shadowColor = '#38bdf8';
          ctx.shadowBlur = 10;
          ctx.stroke();
          ctx.restore();
        }

        if (isSelected || isHovered) {
          // Focal node highlight with outer glow aura
          const pulseRadius = 16;
          ctx.beginPath();
          ctx.arc(node.x, node.y, pulseRadius, 0, 2 * Math.PI, false);
          ctx.fillStyle = isLight ? `${accentColor}22` : `${accentColor}33`;
          ctx.fill();

          ctx.beginPath();
          ctx.arc(node.x, node.y, 11, 0, 2 * Math.PI, false);
          ctx.fillStyle = isLight ? '#ffffff' : (isCode ? 'rgba(15, 23, 42, 0.9)' : '#1e1b4b');
          ctx.fill();
          ctx.lineWidth = 2.5;
          ctx.strokeStyle = accentColor;
          ctx.shadowColor = accentColor;
          ctx.shadowBlur = 12;
          ctx.stroke();
          ctx.shadowBlur = 0;

          // Center solid dot
          ctx.beginPath();
          ctx.arc(node.x, node.y, 5, 0, 2 * Math.PI, false);
          ctx.fillStyle = accentColor;
          ctx.fill();
        } else {
          // Standard node representation
          const r = Math.max(5, Math.min(10, 5 + (node.degree || 0) * 1.2));

          if (isCode) {
            // Code node: distinct cybernetic double-ring representation
            ctx.beginPath();
            ctx.arc(node.x, node.y, r, 0, 2 * Math.PI, false);
            ctx.fillStyle = isDimmed
              ? (isLight ? 'rgba(241, 245, 249, 0.5)' : 'rgba(30, 41, 59, 0.2)')
              : (isLight ? '#ffffff' : 'rgba(15, 23, 42, 0.85)');
            ctx.fill();

            ctx.lineWidth = isConnected ? 2 : 1.3;
            ctx.strokeStyle = isDimmed ? (isLight ? '#cbd5e1' : 'rgba(71, 85, 105, 0.3)') : accentColor;
            ctx.stroke();

            // Inner core dot
            ctx.beginPath();
            ctx.arc(node.x, node.y, Math.max(2, r * 0.45), 0, 2 * Math.PI, false);
            ctx.fillStyle = isDimmed ? (isLight ? '#cbd5e1' : 'rgba(71, 85, 105, 0.3)') : accentColor;
            ctx.fill();
          } else {
            // Note node: soft celestial halo for connected nodes with degree >= 2
            if ((node.degree || 0) >= 2 && !isDimmed) {
              const haloGrad = ctx.createRadialGradient(node.x, node.y, r * 0.5, node.x, node.y, r * 2.8);
              haloGrad.addColorStop(0, `${accentColor}${isLight ? '35' : '40'}`);
              haloGrad.addColorStop(1, 'transparent');
              ctx.beginPath();
              ctx.arc(node.x, node.y, r * 2.8, 0, 2 * Math.PI, false);
              ctx.fillStyle = haloGrad;
              ctx.fill();
            }

            // Solid colored circle
            ctx.beginPath();
            ctx.arc(node.x, node.y, r, 0, 2 * Math.PI, false);
            ctx.fillStyle = isDimmed
              ? (isLight ? 'rgba(203, 213, 225, 0.35)' : 'rgba(71, 85, 105, 0.25)')
              : accentColor;
            ctx.fill();

            // Crisp rim
            ctx.lineWidth = isConnected ? 2 : 1;
            ctx.strokeStyle = isLight ? '#ffffff' : (isDimmed ? 'transparent' : 'rgba(255, 255, 255, 0.35)');
            ctx.stroke();

            // Specular core dot
            if (!isDimmed) {
              ctx.beginPath();
              ctx.arc(node.x, node.y, Math.max(1.5, r * 0.32), 0, 2 * Math.PI, false);
              ctx.fillStyle = isLight ? 'rgba(255, 255, 255, 0.9)' : 'rgba(255, 255, 255, 0.85)';
              ctx.fill();
            }
          }
        }

        // Title and tag capsule pill
        if (globalScale > 0.75 || isSelected || isHovered || isGlowing) {
          const rawLabel = node.title || node.id;
          const label = rawLabel.endsWith('.md') ? rawLabel.replace(/\.md$/, '') : rawLabel;
          const tagText = isCode
            ? label
            : node.tags && node.tags.length > 0
            ? `#${node.tags[0]}`
            : label;

          const fontSize = 11;
          ctx.font = `${isSelected || isGlowing ? '600 ' : '500 '}${fontSize}px 'Plus Jakarta Sans', system-ui, sans-serif`;
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';

          const textWidth = ctx.measureText(tagText).width;
          const textY = node.y + (isSelected ? 20 : 13) + fontSize * 0.5;

          const padX = isCode ? 7 : 6;
          const padY = 2.5;

          ctx.save();
          ctx.beginPath();
          if (typeof (ctx as any).roundRect === 'function') {
            (ctx as any).roundRect(
              node.x - textWidth / 2 - padX,
              textY - fontSize / 2 - padY,
              textWidth + padX * 2,
              fontSize + padY * 2,
              4
            );
          } else {
            ctx.rect(
              node.x - textWidth / 2 - padX,
              textY - fontSize / 2 - padY,
              textWidth + padX * 2,
              fontSize + padY * 2
            );
          }

          if (isLight) {
            ctx.fillStyle = isDimmed ? 'rgba(241, 245, 249, 0.7)' : 'rgba(255, 255, 255, 0.95)';
            ctx.shadowColor = 'rgba(0, 0, 0, 0.08)';
            ctx.shadowBlur = 4;
          } else {
            ctx.fillStyle = isDimmed ? 'rgba(24, 27, 36, 0.4)' : 'rgba(15, 20, 32, 0.92)';
            ctx.shadowColor = 'rgba(0, 0, 0, 0.4)';
            ctx.shadowBlur = 6;
          }
          ctx.fill();

          ctx.lineWidth = 1;
          ctx.strokeStyle = isGlowing
            ? '#10b981'
            : isSelected
            ? accentColor
            : isDimmed
            ? (isLight ? '#e2e8f0' : 'rgba(43, 49, 64, 0.3)')
            : (isLight ? `${accentColor}55` : `${accentColor}44`);
          ctx.stroke();

          // Text
          ctx.shadowBlur = 0;
          ctx.fillStyle = isGlowing
            ? '#10b981'
            : isDimmed
            ? (isLight ? '#94a3b8' : 'rgba(100, 116, 139, 0.4)')
            : isSelected
            ? (isLight ? '#0f172a' : '#ffffff')
            : (isLight ? '#1e293b' : '#f1f5f9');
          ctx.fillText(tagText, node.x, textY);
          ctx.restore();
        }
      })
      .nodePointerAreaPaint((node: any, color: string, ctx: CanvasRenderingContext2D) => {
        if (!node || node.__vis === false || !isFinite(node.x) || !isFinite(node.y)) return;
        ctx.fillStyle = color;
        ctx.beginPath();
        ctx.arc(node.x, node.y, 14, 0, 2 * Math.PI, false);
        ctx.fill();

        const rawLabel = node.title || node.id;
        const label = rawLabel.endsWith('.md') ? rawLabel.replace(/\.md$/, '') : rawLabel;
        const tagText = node.tags && node.tags.length > 0 ? `#${node.tags[0]}` : label;
        ctx.font = '500 10px sans-serif';
        const textWidth = ctx.measureText(tagText).width;
        ctx.fillRect(node.x - textWidth / 2 - 8, node.y + 10, textWidth + 16, 18);
      })
      .onNodeClick(async (node: any) => {
        if (!node || !node.id) return;

        // Path Inspector mode handling
        if (pathModeRef.current) {
          if (!pathStartNodeRef.current) {
            setPathStartNode(node);
            pathStartNodeRef.current = node;
            graphInstanceRef.current?.d3ReheatSimulation();
            return;
          }
          if (pathStartNodeRef.current.id === node.id) {
            setPathStartNode(null);
            pathStartNodeRef.current = null;
            setPathResult(null);
            pathResultRef.current = null;
            graphInstanceRef.current?.d3ReheatSimulation();
            return;
          }
          try {
            const res = await findPath(pathStartNodeRef.current.id, node.id);
            setPathResult(res);
            pathResultRef.current = res;
            graphInstanceRef.current?.d3ReheatSimulation();
          } catch (err) {
            console.error('Failed to find path:', err);
          }
          return;
        }

        // Immediately mark the clicked node and recompute neighbors
        selectedNodeIdRef.current = node.id;
        recomputeConnected(node.id, egoDepthRef.current);
        requestCanvasRepaint();

        if (node.node_type === 'code' && onSelectCodeRef.current) {
          onSelectCodeRef.current(node.path || node.id);
        }
        onSelectNodeRef.current(node.id);
      })
      .onBackgroundClick(() => {
        selectedNodeIdRef.current = null;
        hoverNodeIdRef.current = null;
        connectedNodeIdsRef.current = new Set();
        if (pathModeRef.current) {
          setPathStartNode(null);
          pathStartNodeRef.current = null;
          setPathResult(null);
          pathResultRef.current = null;
        }
        onClearSelectionRef.current?.();
        requestCanvasRepaint();
      })
      .onNodeHover((node: any) => {
        const nextId = node ? node.id : null;
        if (hoverNodeIdRef.current === nextId) return;
        hoverNodeIdRef.current = nextId;
        recomputeConnected(nextId || selectedNodeIdRef.current, egoDepthRef.current);
        scheduleVisibilityRecompute();
        requestCanvasRepaint();
      })
      .showPointerCursor((node: any) => !!node)
      .onZoom((transform: any) => {
        transformRef.current = { k: transform.k, x: transform.x, y: transform.y };
        scheduleVisibilityRecompute();
      })
      .nodeVisibility((node: any) => node.__vis !== false)
      .linkVisibility((link: any) => {
        const s = link.source;
        const t = link.target;
        return s.__vis !== false && t.__vis !== false;
      });

    // Forces configuration
    graph.d3Force('charge')?.strength(-160);
    graph.d3Force('link')?.distance(65);
    graph.d3Force('cluster', (alpha: number) => {
      if (colorModeRef.current !== 'communities' || !graphInstanceRef.current) return;
      const nodes: any[] = graphInstanceRef.current.graphData()?.nodes || [];
      // Skip centroid clustering on huge graphs — the per-tick O(2N) cost is not worth it
      if (nodes.length > LARGE_GRAPH_NODE_LIMIT) return;
      const centroids = new Map<number, { x: number; y: number; count: number }>();
      for (const n of nodes) {
        if (typeof n.community !== 'number' || !isFinite(n.x) || !isFinite(n.y)) continue;
        const c = centroids.get(n.community) || { x: 0, y: 0, count: 0 };
        c.x += n.x;
        c.y += n.y;
        c.count += 1;
        centroids.set(n.community, c);
      }
      centroids.forEach((c) => {
        c.x /= c.count;
        c.y /= c.count;
      });
      const strength = 0.12 * alpha;
      for (const n of nodes) {
        if (typeof n.community !== 'number' || !isFinite(n.x) || !isFinite(n.y)) continue;
        const c = centroids.get(n.community);
        if (c && c.count > 1) {
          n.vx = (n.vx || 0) + (c.x - n.x) * strength;
          n.vy = (n.vy || 0) + (c.y - n.y) * strength;
        }
      }
    });

    graphInstanceRef.current = graph;

    const handleResize = () => {
      if (elem && graphInstanceRef.current) {
        graphInstanceRef.current.width(elem.clientWidth);
        graphInstanceRef.current.height(elem.clientHeight);
      }
    };

    const resizeObserver = new ResizeObserver(() => {
      handleResize();
    });
    resizeObserver.observe(elem);

    window.addEventListener('resize', handleResize);

    return () => {
      resizeObserver.disconnect();
      window.removeEventListener('resize', handleResize);
      if (visibilityRafRef.current) {
        cancelAnimationFrame(visibilityRafRef.current);
        visibilityRafRef.current = 0;
      }
      lastRenderedKeyRef.current = '';
      if (graphInstanceRef.current) {
        try {
          graphInstanceRef.current._destructor?.();
        } catch {}
        graphInstanceRef.current = null;
      }
      if (elem) {
        elem.innerHTML = '';
      }
    };
  }, []);

  // Update selected node highlight without reloading graph data
  useEffect(() => {
    selectedNodeIdRef.current = selectedNodeId;
    recomputeConnected(hoverNodeIdRef.current || selectedNodeId);
    scheduleVisibilityRecompute();
    requestCanvasRepaint();
  }, [selectedNodeId, recomputeConnected, requestCanvasRepaint, scheduleVisibilityRecompute]);

  // Reset filters and camera state when switching projects
  useEffect(() => {
    if (prevProjectIdRef.current !== projectId) {
      prevProjectIdRef.current = projectId;
      setFilterMode('all');
      setLocalOnly(false);
      setPathMode(false);
      hoverNodeIdRef.current = null;
      connectedNodeIdsRef.current = new Set();
      lastRenderedKeyRef.current = '';
      cullingEnabledRef.current = false;
      degreeRankRef.current = new Set();
      adjacencyCacheRef.current = { source: null, map: new Map() };
      if (graphInstanceRef.current) {
        graphInstanceRef.current.graphData({ nodes: [], links: [] });
      }
    }
  }, [projectId]);

  // Update data ONLY when nodes/links change or focus/filter mode is toggled
  useEffect(() => {
    if (!graphInstanceRef.current) return;

    let displayData = graphData;

    // Apply filter mode (all vs notes vs code)
    if (filterMode === 'notes') {
      const subNodes = graphData.nodes.filter((n) => n.node_type === 'note' || !n.node_type);
      const nodeIds = new Set(subNodes.map((n) => n.id));
      const subLinks = graphData.links.filter((l) => {
        const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
        const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
        return nodeIds.has(s) && nodeIds.has(t);
      });
      displayData = { nodes: subNodes, links: subLinks };
    } else if (filterMode === 'code') {
      const subNodes = graphData.nodes.filter((n) => n.node_type === 'code');
      const nodeIds = new Set(subNodes.map((n) => n.id));
      const subLinks = graphData.links.filter((l) => {
        const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
        const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
        return nodeIds.has(s) && nodeIds.has(t);
      });
      displayData = { nodes: subNodes, links: subLinks };
    }

    if (localOnly && selectedNodeId) {
      recomputeConnected(selectedNodeId, egoDepth);
      const connected = connectedNodeIdsRef.current;
      const subNodes = displayData.nodes.filter((n) => connected.has(n.id));
      const subLinks = displayData.links.filter((l) => {
        const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
        const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
        return connected.has(s) && connected.has(t);
      });
      displayData = { nodes: subNodes, links: subLinks };
    }

    // Precomputed-layout detection: when the backend ships positions for the
    // vast majority of nodes, we pin them (fx/fy) and freeze the simulation.
    let positionedCount = 0;
    for (const n of displayData.nodes) {
      if (typeof n.x === 'number') positionedCount++;
    }
    const mostlyPrecomputed =
      displayData.nodes.length > 0 &&
      positionedCount / displayData.nodes.length >= PRECOMPUTED_LAYOUT_RATIO;

    // Density cap: beyond the safe limit, render only structural hubs
    // (degree >= 2, god nodes, current selection) so d3-force stays interactive.
    // Skipped when positions are precomputed — the sim is frozen and viewport
    // culling does the reduction more faithfully (degree-1 nodes stay visible).
    if (displayData.nodes.length > LARGE_GRAPH_NODE_LIMIT && !localOnly && !mostlyPrecomputed) {
      const keep = new Set<string>();
      for (const n of displayData.nodes) {
        if ((n.degree || 0) >= 2 || n.is_god_node || n.id === selectedNodeId) {
          keep.add(n.id);
        }
      }
      displayData = {
        nodes: displayData.nodes.filter((n) => keep.has(n.id)),
        links: displayData.links.filter((l) => {
          const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
          const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
          return keep.has(s) && keep.has(t);
        }),
      };
    }

    // Lightweight change signature: FNV-1a over node/link identity in iteration
    // order — single O(N+E) pass, no sorting, no megabyte-sized join strings.
    let h = 0x811c9dc5;
    const mix = (v: number | string) => {
      const s = String(v);
      for (let i = 0; i < s.length; i++) {
        h ^= s.charCodeAt(i);
        h = Math.imul(h, 0x01000193);
      }
      h ^= 0x2f;
      h = Math.imul(h, 0x01000193);
    };
    for (const n of displayData.nodes) {
      mix(n.id);
      mix(n.title);
      mix(n.degree || 0);
      mix(n.community ?? -1);
    }
    for (const l of displayData.links) {
      mix(typeof l.source === 'object' ? (l.source as any).id : l.source);
      mix(typeof l.target === 'object' ? (l.target as any).id : l.target);
    }
    const currentKey = `${projectId || 'default'}|${filterMode}|${localOnly ? `${selectedNodeId}:${egoDepth}` : 'all'}|${h >>> 0}`;

    if (lastRenderedKeyRef.current === currentKey) {
      return;
    }
    lastRenderedKeyRef.current = currentKey;

    const currentData = graphInstanceRef.current.graphData();
    const existingNodesMap = new Map<string, any>((currentData.nodes || []).map((n: any) => [n.id, n]));

    const communityById = new Map<string, number>();
    const mergedNodes = displayData.nodes.map((n) => {
      const existing = existingNodesMap.get(n.id);
      const target = existing && existing.x !== undefined && existing.y !== undefined ? existing : { ...n };
      target.id = n.id;
      target.title = n.title;
      target.tags = n.tags;
      target.degree = n.degree;
      target.node_type = n.node_type;
      target.path = n.path;
      target.community = n.community;
      target.is_god_node = n.is_god_node;
      target.centrality = n.centrality;
      // Precomputed layout positions pin the node (d3 fx/fy) so the
      // simulation never moves it — the whole freeze stays cheap.
      if (typeof n.x === 'number' && typeof n.y === 'number' && isFinite(n.x) && isFinite(n.y)) {
        target.x = n.x;
        target.y = n.y;
        target.fx = n.x;
        target.fy = n.y;
        target.__pinned = true;
      } else {
        target.fx = undefined;
        target.fy = undefined;
        target.__pinned = false;
      }
      // Pre-normalized lookup keys so the per-frame paint loop never
      // re-runs toLowerCase/regex on hot paths.
      target._idLc = String(n.id).toLowerCase();
      target._idStemLc = target._idLc.replace(/\.md$/i, '');
      target._titleLc = n.title ? String(n.title).toLowerCase() : '';
      target._titleStemLc = target._titleLc.replace(/\.md$/i, '');
      target._pathLc = n.path ? String(n.path).toLowerCase() : '';
      if (typeof n.community === 'number') communityById.set(n.id, n.community);
      return target;
    });
    nodeCommunityByIdRef.current = communityById;
    cachedHullsRef.current = new Map();
    hullsComputedAtRef.current = 0;

    const mergedLinks = displayData.links.map((l) => {
      const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
      const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
      return { ...l, source: s, target: t };
    });

    // Physics tuning scaled to graph size: large graphs warm up synchronously,
    // decay faster and cool down quickly instead of simmering forever.
    // With precomputed positions the sim is frozen outright (0 ticks).
    const isLargeGraph = mergedNodes.length > LARGE_GRAPH_NODE_LIMIT;
    const t0 = performance.now();
    try {
      graphInstanceRef.current.warmupTicks?.(mostlyPrecomputed ? 0 : isLargeGraph ? 50 : 0);
      graphInstanceRef.current.cooldownTicks?.(mostlyPrecomputed ? 0 : isLargeGraph ? 160 : Infinity);
      graphInstanceRef.current.d3AlphaMin?.(mostlyPrecomputed ? 0.9 : isLargeGraph ? 0.02 : 0.001);
      graphInstanceRef.current.d3AlphaDecay?.(mostlyPrecomputed ? 0.95 : isLargeGraph ? 0.05 : 0.0228);
      graphInstanceRef.current.d3VelocityDecay?.(isLargeGraph ? 0.55 : 0.4);
    } catch {
      // ignore config errors on older force-graph versions
    }

    graphInstanceRef.current.graphData({
      nodes: mergedNodes,
      links: mergedLinks,
    });
    if (!mostlyPrecomputed) {
      graphInstanceRef.current.d3ReheatSimulation?.();
    }

    // Enable viewport culling above the threshold and precompute the
    // degree-ranked hub set used for far-zoom deterministic sampling.
    cullingEnabledRef.current = mergedNodes.length > CULL_NODE_THRESHOLD;
    if (cullingEnabledRef.current) {
      const byDegree = mergedNodes
        .slice()
        .sort((a, b) => (b.degree || 0) - (a.degree || 0))
        .slice(0, FAR_ZOOM_BUDGET);
      degreeRankRef.current = new Set(byDegree.map((n) => n.id));
      recomputeVisibility();
    } else {
      degreeRankRef.current = new Set();
      for (const n of mergedNodes) n.__vis = true;
    }
    console.info(
      `[GraphView] graphData applied: ${(performance.now() - t0).toFixed(0)}ms ` +
        `nodes=${mergedNodes.length} links=${mergedLinks.length} precomputed=${mostlyPrecomputed} culling=${cullingEnabledRef.current}`
    );

    if (existingNodesMap.size === 0 && mergedNodes.length > 0) {
      setTimeout(() => {
        if (graphInstanceRef.current) {
          graphInstanceRef.current.zoomToFit(400, 60);
        }
      }, 250);
    }
  }, [projectId, graphData, filterMode, localOnly, egoDepth, localOnly ? selectedNodeId : null, recomputeConnected, recomputeVisibility]);

  const handleZoomIn = () => {
    if (graphInstanceRef.current) {
      graphInstanceRef.current.zoom(graphInstanceRef.current.zoom() * 1.35, 300);
    }
  };

  const handleZoomOut = () => {
    if (graphInstanceRef.current) {
      graphInstanceRef.current.zoom(graphInstanceRef.current.zoom() * 0.75, 300);
    }
  };

  const handleFit = () => {
    if (graphInstanceRef.current) {
      graphInstanceRef.current.zoomToFit(400, 48);
    }
  };

  const pillBtn = (active: boolean) =>
    `flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
      active ? 'bg-accent text-on-accent shadow-xs' : 'text-muted hover:text-ink'
    }`;

  const toggleBtn = (active: boolean, activeCls: string) =>
    `flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
      active ? activeCls : 'text-muted hover:text-ink hover:bg-raise'
    }`;

  const iconBtn =
    'flex items-center justify-center w-6.5 h-6.5 rounded-lg transition-colors cursor-pointer text-muted hover:text-ink hover:bg-raise';

  return (
    <div className="relative w-full h-full overflow-hidden transition-colors duration-200 bg-canvas">
      {/* Canvas container specifically for ForceGraph */}
      <div ref={containerRef} className="absolute inset-0 w-full h-full" />

      {/* Empty state — vault with no indexed content */}
      {stats.total === 0 && (
        <div className="absolute inset-0 z-20 flex items-center justify-center pointer-events-none">
          <div className="flex flex-col items-center gap-3 text-center max-w-xs px-6 py-8 rounded-2xl border border-line bg-surface/90 backdrop-blur-md shadow-xl pointer-events-auto">
            <CirclesThree size={28} className="text-accent" weight="duotone" />
            <div>
              <p className="text-sm font-semibold text-ink">Nothing here yet</p>
              <p className="text-[11px] text-muted mt-1 leading-relaxed">
                The graph is empty. Create your first note or sync the vault to index files.
              </p>
            </div>
            <div className="flex items-center gap-2">
              {onCreateNote && (
                <button
                  type="button"
                  onClick={onCreateNote}
                  className="px-3 h-7 rounded-md text-[11px] font-semibold bg-accent text-on-accent hover:bg-accent-strong transition-colors cursor-pointer"
                >
                  New note
                </button>
              )}
              {onSyncVault && (
                <button
                  type="button"
                  onClick={onSyncVault}
                  className="px-3 h-7 rounded-md text-[11px] font-semibold text-muted border border-line hover:text-ink hover:border-line-strong transition-colors cursor-pointer"
                >
                  Sync vault
                </button>
              )}
            </div>
          </div>
        </div>
      )}

      {/* Floating Toolbar Deck */}
      <div
        className="absolute top-3 left-3 z-30 flex items-center gap-1.5 p-1 rounded-xl shadow-xl backdrop-blur-md transition-colors bg-surface/85 border border-line"
        role="toolbar"
        aria-label="Graph controls"
      >
        {/* Node Type Filter Pill Selector */}
        <div className="flex items-center gap-0.5 p-0.5 rounded-lg border bg-canvas border-line">
          <Tip label="Show all nodes — notes and code files together in the graph.">
            <button type="button" onClick={() => setFilterMode('all')} className={pillBtn(filterMode === 'all')}>
              <CirclesThree size={12} weight="bold" />
              <span>All ({stats.total})</span>
            </button>
          </Tip>
          <Tip label="Show only Markdown notes and their wikilinks.">
            <button type="button" onClick={() => setFilterMode('notes')} className={pillBtn(filterMode === 'notes')}>
              <FileText size={12} />
              <span>Notes ({stats.notes})</span>
            </button>
          </Tip>
          <Tip label="Show only code files and their import dependencies.">
            <button type="button" onClick={() => setFilterMode('code')} className={pillBtn(filterMode === 'code')}>
              <Code size={12} weight="bold" />
              <span>Code ({stats.code})</span>
            </button>
          </Tip>
        </div>

        <div className="w-[1px] h-3.5 my-auto bg-line-strong" />

        <Tip label="Clusters: color nodes by detected community (LPA) — reveals architectural boundaries and natural groupings.">
          <button
            type="button"
            className={toggleBtn(colorMode === 'communities', 'bg-accent-soft text-accent-strong border border-accent-line')}
            onClick={() => setColorMode(colorMode === 'communities' ? 'default' : 'communities')}
          >
            <SquaresFour size={12} weight={colorMode === 'communities' ? 'bold' : 'regular'} />
            <span>Clusters</span>
          </button>
        </Tip>

        <Tip label="Hubs: highlight God Nodes (degree ≥ 12) with a golden ring — the most connected points in the system.">
          <button
            type="button"
            className={toggleBtn(highlightGodNodes, 'bg-warn/20 text-warn border border-warn/40')}
            onClick={() => setHighlightGodNodes(!highlightGodNodes)}
          >
            <Crown size={12} weight={highlightGodNodes ? 'bold' : 'regular'} />
            <span>Hubs</span>
          </button>
        </Tip>

        <Tip label="Path Inspector: click 2 nodes to find the shortest path between them (wikilinks + imports)."><button
          type="button"
          className={toggleBtn(pathMode, 'bg-accent-soft text-accent-strong border border-accent-line')}
          onClick={() => setPathMode(!pathMode)}
        >
          <GitBranch size={12} weight={pathMode ? 'bold' : 'regular'} />
          <span>Path</span>
        </button></Tip>

        <Tip label="Open the architectural audit report — communities, god nodes and suspicious couplings."><button
          type="button"
          className={toggleBtn(false, '')}
          onClick={() => setShowAuditModal(true)}
        >
          <ChartBar size={12} weight="bold" />
          <span>Audit</span>
        </button></Tip>

        <div className="w-[1px] h-3.5 my-auto bg-line-strong" />

        <div className="flex items-center gap-1.5">
          <Tip label={localOnly ? 'Ego Network active — click to show the full graph again.' : 'Ego Network: focus on the selected node and its neighbors up to N hops — ideal for exploring specific regions.'}>
            <button
              type="button"
              className={toggleBtn(localOnly, 'bg-accent/30 text-accent-strong border border-accent/40')}
              onClick={() => setLocalOnly(!localOnly)}
            >
              {localOnly ? <Crosshair size={12} weight="bold" /> : <Compass size={12} />}
              <span>Ego {egoDepth}H</span>
            </button>
          </Tip>

          {localOnly && (
            <div
              className="flex items-center gap-1.5 px-2 py-0.5 rounded-lg border text-[10px] font-mono bg-canvas/90 border-line text-muted"
              title="Ego Network depth: how many neighbor hops enter the focus (1-5)"
            >
              <span className="text-[10px] uppercase tracking-wider opacity-75">Hops</span>
              <input
                type="range"
                min={1}
                max={5}
                step={1}
                value={egoDepth}
                onChange={(e) => setEgoDepth(Number(e.target.value))}
                className="w-14 h-1 accent-[var(--gf-accent)] cursor-pointer"
                aria-label="Ego Network N-hop depth"
              />
              <span className="font-bold text-accent-strong">{egoDepth}</span>
            </div>
          )}
        </div>

        <Tip label="Zoom in."><button type="button" className={iconBtn} onClick={handleZoomIn}>
          <MagnifyingGlassPlus size={13} />
        </button></Tip>

        <Tip label="Zoom out."><button type="button" className={iconBtn} onClick={handleZoomOut}>
          <MagnifyingGlassMinus size={13} />
        </button></Tip>

        <Tip label="Fit all nodes into view."><button type="button" className={iconBtn} onClick={handleFit}>
          <ArrowsOut size={13} />
        </button></Tip>

        <Tip label="Re-run the force-directed layout — reorganizes node positions."><button type="button" className={iconBtn} onClick={onRefresh}>
          <ArrowClockwise size={13} />
        </button></Tip>
      </div>

      {/* Floating Path Inspector Banner */}
      {pathMode && (
        <div className="absolute top-16 left-3 z-30 flex items-center gap-2.5 px-3 py-1.5 rounded-xl shadow-lg backdrop-blur-md text-[11px] font-mono border transition-all bg-canvas/90 border-accent/40 text-ink">
          <GitBranch size={14} className="text-accent shrink-0" />
          {!pathStartNode ? (
            <span>Click the <strong className="text-accent">source node</strong> in the graph…</span>
          ) : !pathResult ? (
            <span>
              Source: <strong className="text-accent">{pathStartNode.title || pathStartNode.id}</strong> → click the <strong className="text-good">target node</strong>…
            </span>
          ) : (
            <div className="flex items-center gap-2">
              <span>
                Path: <strong className="text-accent">{pathResult.length} nodes</strong> ({pathResult.steps.length > 1 ? pathResult.steps.length - 1 : 0} hops)
              </span>
              <button
                type="button"
                onClick={() => {
                  setPathStartNode(null);
                  setPathResult(null);
                }}
                className="px-1.5 py-0.5 rounded text-[10px] bg-raise hover:bg-line text-ink cursor-pointer"
              >
                Clear
              </button>
            </div>
          )}
          <Tip label="Close Path Inspector.">
            <button
              type="button"
              onClick={() => setPathMode(false)}
              className="p-1 text-faint hover:text-ink transition-colors cursor-pointer"
            >
              <X size={12} />
            </button>
          </Tip>
        </div>
      )}

      {/* Floating HUD Telemetry Capsule */}
      <div className="absolute bottom-3 left-3 z-30 flex items-center gap-2 px-2.5 py-1 rounded-lg shadow-lg backdrop-blur-md text-[11px] font-mono transition-colors bg-surface/85 border border-line text-muted">
        <span className="w-1.5 h-1.5 rounded-full bg-good shrink-0 shadow-[0_0_6px_rgba(16,185,129,0.5)]" />
        <span>Notes: <strong className="font-semibold text-ink">{stats.notes}</strong></span>
        <span className="text-line-strong">•</span>
        <span>Code: <strong className="font-semibold text-sky-500">{stats.code}</strong></span>
        <span className="text-line-strong">•</span>
        <span>Links: <strong className="font-semibold text-ink">{stats.links}</strong></span>
        {localOnly && (
          <>
            <span className="text-line-strong">•</span>
            <span className="font-medium text-accent-strong">Ego Network ({egoDepth}-Hop)</span>
          </>
        )}
      </div>

      {/* Minimap — live overview + click-to-navigate */}
      {stats.total > 0 && (
        <canvas
          ref={minimapRef}
          width={MINIMAP_W}
          height={MINIMAP_H}
          onClick={handleMinimapClick}
          title="Minimap — click to navigate"
          className="absolute bottom-12 right-3 z-30 rounded-lg shadow-lg backdrop-blur-md border border-line bg-surface/70 cursor-crosshair"
        />
      )}

      {/* Legend */}
      {stats.total > 0 && (
        <div className="absolute bottom-3 right-3 z-30 flex items-center gap-2.5 px-2.5 py-1 rounded-lg shadow-lg backdrop-blur-md text-[10px] font-mono transition-colors bg-surface/85 border border-line text-muted">
          {colorMode === 'communities' ? (
            <span title="Nodes are colored by detected community">Colors = communities</span>
          ) : (
            <>
              <span className="flex items-center gap-1">
                <span className="w-2 h-2 rounded-full bg-accent" /> note
              </span>
              <span className="flex items-center gap-1">
                <span className="w-2 h-2 rounded-full bg-sky-500" /> code
              </span>
              <span className="flex items-center gap-1">
                <span className="w-2 h-2 rounded-full border border-faint" /> unresolved
              </span>
            </>
          )}
          {highlightGodNodes && (
            <span className="flex items-center gap-1" title="God Nodes are highlighted with a golden ring">
              <span className="w-2 h-2 rounded-full border-2 border-warn" /> hub
            </span>
          )}
        </div>
      )}

      {/* Architectural Audit Modal */}
      <ArchitecturalReportModal
        isOpen={showAuditModal}
        onClose={() => setShowAuditModal(false)}
        onSelectNode={onSelectNode}
        theme={theme}
      />
    </div>
  );
};
