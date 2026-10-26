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

interface GraphViewProps {
  projectId?: string;
  graphData: GraphData;
  selectedNodeId: string | null;
  onSelectNode: (nodeId: string) => void;
  onSelectCode?: (fileId: string) => void;
  onRefresh: () => void;
  glowingNode?: { id: string; timestamp: number } | null;
  theme?: 'dark' | 'light';
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
  onRefresh,
  glowingNode,
  theme = 'dark',
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
        graphInstanceRef.current.backgroundColor(theme === 'light' ? '#f8fafc' : '#090a0f');
        requestCanvasRepaint();
      } catch (err) {
        console.warn('Error updating graph theme:', err);
      }
    }
  }, [theme, requestCanvasRepaint]);

  // Glowing animation ref
  const glowingStateRef = useRef<{ id: string; startTime: number; duration: number } | null>(null);

  // Graph data ref to avoid re-triggering effects when links change
  const graphDataRef = useRef<GraphData>(graphData);
  useEffect(() => {
    graphDataRef.current = graphData;
  }, [graphData]);

  const lastRenderedKeyRef = useRef<string>('');

  useEffect(() => {
    onSelectNodeRef.current = onSelectNode;
  }, [onSelectNode]);

  useEffect(() => {
    onSelectCodeRef.current = onSelectCode;
  }, [onSelectCode]);

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
    const hops = maxHops ?? egoDepthRef.current;
    const adj = new Map<string, string[]>();
    for (const link of graphDataRef.current.links) {
      const s = typeof link.source === 'object' ? (link.source as any).id : link.source;
      const t = typeof link.target === 'object' ? (link.target as any).id : link.target;
      if (!adj.has(s)) adj.set(s, []);
      if (!adj.has(t)) adj.set(t, []);
      adj.get(s)!.push(t);
      adj.get(t)!.push(s);
    }

    const visited = new Set<string>([targetId]);
    let frontier = [targetId];
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

    glowingStateRef.current = {
      id: targetId,
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

    const timer = setTimeout(() => {
      glowingStateRef.current = null;
    }, glowingStateRef.current.duration);

    return () => {
      clearTimeout(timer);
    };
  }, [glowingNode]);

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
      .backgroundColor(themeRef.current === 'light' ? '#f8fafc' : '#090a0f')
      .autoPauseRedraw(false)
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
      .onRenderFramePre((ctx: CanvasRenderingContext2D, globalScale: number) => {
        if (colorModeRef.current !== 'communities' || !graphInstanceRef.current) return;
        const currentData = graphInstanceRef.current.graphData();
        const nodes: any[] = currentData?.nodes || [];
        if (nodes.length === 0) return;

        const isLight = themeRef.current === 'light';
        const connected = connectedNodeIdsRef.current;
        const groups = new Map<number, any[]>();

        for (const n of nodes) {
          if (typeof n.community !== 'number' || !isFinite(n.x) || !isFinite(n.y)) continue;
          if (!groups.has(n.community)) groups.set(n.community, []);
          groups.get(n.community)!.push(n);
        }

        const padRadius = 26;
        const angles = [0, Math.PI / 4, Math.PI / 2, (3 * Math.PI) / 4, Math.PI, (5 * Math.PI) / 4, (3 * Math.PI) / 2, (7 * Math.PI) / 4];

        groups.forEach((commNodes, commId) => {
          if (commNodes.length < 2) return;

          const hasActiveFocus = connected.size > 0;
          const anyConnected = hasActiveFocus && commNodes.some((n) => connected.has(n.id));
          if (hasActiveFocus && !anyConnected) return;

          const color = COMMUNITY_PALETTE[commId % COMMUNITY_PALETTE.length];
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
          if (hull.length < 3) return;

          ctx.save();
          drawSmoothHull(ctx, hull);
          ctx.fillStyle = isLight ? `${color}14` : `${color}12`;
          ctx.fill();
          ctx.lineWidth = Math.max(1 / globalScale, 0.8);
          ctx.setLineDash([5 / globalScale, 4 / globalScale]);
          ctx.strokeStyle = isLight ? `${color}45` : `${color}38`;
          ctx.stroke();

          if (globalScale > 0.45) {
            const centerX = sumX / commNodes.length;
            const fontSize = Math.max(9.5 / globalScale, 8);
            ctx.font = `600 ${fontSize}px 'Plus Jakarta Sans', monospace, sans-serif`;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'bottom';
            ctx.fillStyle = isLight ? `${color}cc` : `${color}aa`;
            ctx.fillText(`Cluster #${commId} (${commNodes.length})`, centerX, minY - 4 / globalScale);
          }
          ctx.restore();
        });
      })
      .nodeCanvasObject((node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
        if (!node || !isFinite(node.x) || !isFinite(node.y)) return;

        const isLight = themeRef.current === 'light';
        const isSelected = node.id === selectedNodeIdRef.current;
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
            node.id?.toLowerCase() === glowing.id.toLowerCase() ||
            node.title?.toLowerCase() === glowing.id.toLowerCase() ||
            node.id?.replace(/\.md$/, '')?.toLowerCase() === glowing.id.replace(/\.md$/, '').toLowerCase());

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
        const isGod = highlightGodNodesRef.current && (node.is_god_node || (node.degree || 0) >= 12);
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
        if (!node || !isFinite(node.x) || !isFinite(node.y)) return;
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

        if (node.node_type === 'code' && onSelectCodeRef.current) {
          onSelectCodeRef.current(node.path || node.id);
        } else {
          onSelectNodeRef.current(node.id);
        }
      })
      .onNodeHover((node: any) => {
        const nextId = node ? node.id : null;
        if (hoverNodeIdRef.current === nextId) return;
        hoverNodeIdRef.current = nextId;
        recomputeConnected(nextId || selectedNodeIdRef.current);
      })
      .showPointerCursor((node: any) => !!node);

    // Forces configuration
    graph.d3Force('charge')?.strength(-160);
    graph.d3Force('link')?.distance(65);
    graph.d3Force('cluster', (alpha: number) => {
      if (colorModeRef.current !== 'communities' || !graphInstanceRef.current) return;
      const nodes: any[] = graphInstanceRef.current.graphData()?.nodes || [];
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
  }, [selectedNodeId, recomputeConnected]);

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

    const currentKey = `${projectId || 'default'}|${filterMode}|${localOnly ? `${selectedNodeId}:${egoDepth}` : 'all'}|` +
      displayData.nodes.map((n) => `${n.id}:${n.title}:${n.degree || 0}`).sort().join(';') + '|' +
      displayData.links.map((l) => {
        const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
        const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
        return `${s}->${t}`;
      }).sort().join(';');

    if (lastRenderedKeyRef.current === currentKey) {
      return;
    }
    lastRenderedKeyRef.current = currentKey;

    const currentData = graphInstanceRef.current.graphData();
    const existingNodesMap = new Map<string, any>((currentData.nodes || []).map((n: any) => [n.id, n]));

    const mergedNodes = displayData.nodes.map((n) => {
      const existing = existingNodesMap.get(n.id);
      if (existing && existing.x !== undefined && existing.y !== undefined) {
        existing.title = n.title;
        existing.tags = n.tags;
        existing.degree = n.degree;
        existing.node_type = n.node_type;
        existing.path = n.path;
        existing.community = n.community;
        existing.is_god_node = n.is_god_node;
        existing.centrality = n.centrality;
        return existing;
      }
      return { ...n };
    });

    const mergedLinks = displayData.links.map((l) => {
      const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
      const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
      return { ...l, source: s, target: t };
    });

    graphInstanceRef.current.graphData({
      nodes: mergedNodes,
      links: mergedLinks,
    });
    graphInstanceRef.current.d3ReheatSimulation?.();

    if (existingNodesMap.size === 0 && mergedNodes.length > 0) {
      setTimeout(() => {
        if (graphInstanceRef.current) {
          graphInstanceRef.current.zoomToFit(400, 60);
        }
      }, 250);
    }
  }, [projectId, graphData, filterMode, localOnly, egoDepth, localOnly ? selectedNodeId : null, recomputeConnected]);

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

  const isLight = theme === 'light';

  return (
    <div className={`relative w-full h-full overflow-hidden transition-colors duration-200 ${isLight ? 'bg-[#f8fafc]' : 'bg-[#090a0f]'}`}>
      {/* Canvas container specifically for ForceGraph */}
      <div ref={containerRef} className="absolute inset-0 w-full h-full" />

      {/* Floating Toolbar Deck */}
      <div
        className={`absolute top-3 left-3 z-30 flex items-center gap-1.5 p-1 rounded-xl shadow-xl backdrop-blur-md transition-colors ${
          isLight
            ? 'bg-white/90 border border-slate-200/90 shadow-slate-200/50'
            : 'bg-[#0e1119]/85 border border-[#1f2536]'
        }`}
        role="toolbar"
        aria-label="Controles do Grafo"
      >
        {/* Node Type Filter Pill Selector */}
        <div className={`flex items-center gap-0.5 p-0.5 rounded-lg border ${
          isLight ? 'bg-slate-100 border-slate-200' : 'bg-[#090b10] border-[#171b26]'
        }`}>
          <button
            type="button"
            onClick={() => setFilterMode('all')}
            className={`flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
              filterMode === 'all'
                ? 'bg-indigo-600 text-white shadow-xs'
                : isLight
                ? 'text-slate-600 hover:text-slate-900'
                : 'text-[#64748b] hover:text-[#f1f5f9]'
            }`}
            title="Show all nodes"
          >
            <CirclesThree size={12} weight="bold" />
            <span>All ({stats.total})</span>
          </button>

          <button
            type="button"
            onClick={() => setFilterMode('notes')}
            className={`flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
              filterMode === 'notes'
                ? 'bg-indigo-600 text-white shadow-xs'
                : isLight
                ? 'text-slate-600 hover:text-slate-900'
                : 'text-[#64748b] hover:text-[#f1f5f9]'
            }`}
            title="Filter notes only"
          >
            <FileText size={12} />
            <span>Notes ({stats.notes})</span>
          </button>

          <button
            type="button"
            onClick={() => setFilterMode('code')}
            className={`flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
              filterMode === 'code'
                ? 'bg-indigo-600 text-white shadow-xs'
                : isLight
                ? 'text-slate-600 hover:text-slate-900'
                : 'text-[#64748b] hover:text-[#f1f5f9]'
            }`}
            title="Filter code files only"
          >
            <Code size={12} weight="bold" />
            <span>Code ({stats.code})</span>
          </button>
        </div>

        <div className={`w-[1px] h-3.5 my-auto ${isLight ? 'bg-slate-200' : 'bg-[#1f2536]'}`} />

        {/* Communities / Clusters toggle */}
        <button
          type="button"
          className={`flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
            colorMode === 'communities'
              ? isLight
                ? 'bg-purple-100 text-purple-700 border border-purple-200'
                : 'bg-purple-600/30 text-purple-300 border border-purple-500/40'
              : isLight
              ? 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'
              : 'text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#151926]'
          }`}
          title={colorMode === 'communities' ? 'Color Mode: Communities (Active)' : 'Color by Communities'}
          onClick={() => setColorMode(colorMode === 'communities' ? 'default' : 'communities')}
        >
          <SquaresFour size={12} weight={colorMode === 'communities' ? 'bold' : 'regular'} />
          <span>Clusters</span>
        </button>

        {/* God Nodes / Hubs toggle */}
        <button
          type="button"
          className={`flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
            highlightGodNodes
              ? isLight
                ? 'bg-amber-100 text-amber-700 border border-amber-200'
                : 'bg-amber-500/25 text-amber-300 border border-amber-500/40'
              : isLight
              ? 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'
              : 'text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#151926]'
          }`}
          title={highlightGodNodes ? 'Highlight Architectural Hubs / God Nodes' : 'God Nodes Hidden'}
          onClick={() => setHighlightGodNodes(!highlightGodNodes)}
        >
          <Crown size={12} weight={highlightGodNodes ? 'bold' : 'regular'} />
          <span>Hubs</span>
        </button>

        {/* Path Inspector toggle */}
        <button
          type="button"
          className={`flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
            pathMode
              ? isLight
                ? 'bg-sky-100 text-sky-700 border border-sky-200'
                : 'bg-sky-500/25 text-sky-300 border border-sky-500/40'
              : isLight
              ? 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'
              : 'text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#151926]'
          }`}
          title="Path Inspector: Click any 2 nodes to find shortest path"
          onClick={() => setPathMode(!pathMode)}
        >
          <GitBranch size={12} weight={pathMode ? 'bold' : 'regular'} />
          <span>Path</span>
        </button>

        {/* Architectural Audit Modal */}
        <button
          type="button"
          className={`flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
            isLight
              ? 'text-slate-600 hover:text-slate-900 hover:bg-slate-100'
              : 'text-[#94a3b8] hover:text-white hover:bg-[#1a2030]'
          }`}
          title="Open Architectural Audit Report"
          onClick={() => setShowAuditModal(true)}
        >
          <ChartBar size={12} weight="bold" />
          <span>Audit</span>
        </button>

        <div className={`w-[1px] h-3.5 my-auto ${isLight ? 'bg-slate-200' : 'bg-[#1f2536]'}`} />

        {/* Ego Network (N-Hop Focus) toggle + Dynamic Depth Slider */}
        <div className="flex items-center gap-1.5">
          <button
            type="button"
            className={`flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-all cursor-pointer ${
              localOnly
                ? isLight
                  ? 'bg-indigo-100 text-indigo-700 border border-indigo-200'
                  : 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/40'
                : isLight
                ? 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'
                : 'text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#151926]'
            }`}
            title={localOnly ? 'Disable Ego Network Focus (Show Full Graph)' : 'Enable Ego Network (N-Hop Focus)'}
            onClick={() => setLocalOnly(!localOnly)}
          >
            {localOnly ? <Crosshair size={12} weight="bold" /> : <Compass size={12} />}
            <span>Ego {egoDepth}H</span>
          </button>

          {(localOnly || selectedNodeId) && (
            <div
              className={`flex items-center gap-1.5 px-2 py-0.5 rounded-lg border text-[10px] font-mono ${
                isLight
                  ? 'bg-slate-100/90 border-slate-200 text-slate-600'
                  : 'bg-[#090b10]/90 border-[#171b26] text-slate-300'
              }`}
              title={`Profundidade da Ego Network: ${egoDepth} salto(s)`}
            >
              <span className="text-[9.5px] uppercase tracking-wider opacity-75">Hops</span>
              <input
                type="range"
                min={1}
                max={5}
                step={1}
                value={egoDepth}
                onChange={(e) => setEgoDepth(Number(e.target.value))}
                className="w-14 h-1 accent-indigo-500 cursor-pointer"
                aria-label="Profundidade N-Hop da Ego Network"
              />
              <span className={`font-bold ${isLight ? 'text-indigo-600' : 'text-indigo-400'}`}>
                {egoDepth}
              </span>
            </div>
          )}
        </div>

        <button
          type="button"
          className={`flex items-center justify-center w-6.5 h-6.5 rounded-lg transition-colors cursor-pointer ${
            isLight
              ? 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'
              : 'text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#151926]'
          }`}
          title="Zoom In (+)"
          onClick={handleZoomIn}
        >
          <MagnifyingGlassPlus size={13} />
        </button>

        <button
          type="button"
          className={`flex items-center justify-center w-6.5 h-6.5 rounded-lg transition-colors cursor-pointer ${
            isLight
              ? 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'
              : 'text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#151926]'
          }`}
          title="Zoom Out (-)"
          onClick={handleZoomOut}
        >
          <MagnifyingGlassMinus size={13} />
        </button>

        <button
          type="button"
          className={`flex items-center justify-center w-6.5 h-6.5 rounded-lg transition-colors cursor-pointer ${
            isLight
              ? 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'
              : 'text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#151926]'
          }`}
          title="Fit to Screen"
          onClick={handleFit}
        >
          <ArrowsOut size={13} />
        </button>

        <button
          type="button"
          className={`flex items-center justify-center w-6.5 h-6.5 rounded-lg transition-colors cursor-pointer ${
            isLight
              ? 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'
              : 'text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#151926]'
          }`}
          title="Reorganize Graph"
          onClick={onRefresh}
        >
          <ArrowClockwise size={13} />
        </button>
      </div>

      {/* Floating Path Inspector Banner */}
      {pathMode && (
        <div className={`absolute top-16 left-3 z-30 flex items-center gap-2.5 px-3 py-1.5 rounded-xl shadow-lg backdrop-blur-md text-[11px] font-mono border transition-all ${
          isLight
            ? 'bg-white/95 border-sky-300 text-slate-700 shadow-sky-100'
            : 'bg-[#0b1220]/90 border-sky-500/40 text-slate-200 shadow-[0_4px_20px_rgba(56,189,248,0.15)]'
        }`}>
          <GitBranch size={14} className="text-sky-400 shrink-0 animate-pulse" />
          {!pathStartNode ? (
            <span>Clique no <strong className="text-sky-400">nó de origem</strong> no grafo...</span>
          ) : !pathResult ? (
            <span>
              Origem: <strong className="text-sky-400">{pathStartNode.title || pathStartNode.id}</strong> → Clique no <strong className="text-emerald-400">nó de destino</strong>...
            </span>
          ) : (
            <div className="flex items-center gap-2">
              <span>
                Caminho: <strong className="text-sky-400">{pathResult.length} nós</strong> ({pathResult.steps.length > 1 ? pathResult.steps.length - 1 : 0} saltos)
              </span>
              <button
                type="button"
                onClick={() => {
                  setPathStartNode(null);
                  setPathResult(null);
                }}
                className="px-1.5 py-0.5 rounded text-[10px] bg-slate-700 hover:bg-slate-600 text-white cursor-pointer"
              >
                Limpar
              </button>
            </div>
          )}
          <button
            type="button"
            onClick={() => setPathMode(false)}
            className="p-1 text-slate-400 hover:text-white transition-colors cursor-pointer"
            title="Fechar Inspetor de Caminhos"
          >
            <X size={12} />
          </button>
        </div>
      )}

      {/* Floating HUD Telemetry Capsule */}
      <div className={`absolute bottom-3 left-3 z-30 flex items-center gap-2 px-2.5 py-1 rounded-lg shadow-lg backdrop-blur-md text-[10.5px] font-mono transition-colors ${
        isLight
          ? 'bg-white/90 border border-slate-200/90 text-slate-600 shadow-slate-200/40'
          : 'bg-[#0e1119]/85 border border-[#1f2536] text-[#64748b]'
      }`}>
        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 shrink-0 shadow-[0_0_6px_rgba(16,185,129,0.5)]" />
        <span>
          Notes: <strong className={`font-semibold ${isLight ? 'text-slate-900' : 'text-[#f8fafc]'}`}>{stats.notes}</strong>
        </span>
        <span className={isLight ? 'text-slate-300' : 'text-[#202738]'}>•</span>
        <span>
          Code: <strong className={`font-semibold ${isLight ? 'text-sky-600' : 'text-[#38bdf8]'}`}>{stats.code}</strong>
        </span>
        <span className={isLight ? 'text-slate-300' : 'text-[#202738]'}>•</span>
        <span>
          Links: <strong className={`font-semibold ${isLight ? 'text-slate-900' : 'text-[#f8fafc]'}`}>{stats.links}</strong>
        </span>
        {localOnly && (
          <>
            <span className={isLight ? 'text-slate-300' : 'text-[#202738]'}>•</span>
            <span className={`font-medium ${isLight ? 'text-indigo-600' : 'text-indigo-400'}`}>
              Ego Network ({egoDepth}-Hop)
            </span>
          </>
        )}
      </div>

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
