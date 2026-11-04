import React, { useEffect, useRef, useState, useCallback, useMemo } from 'react';
import Sigma from 'sigma';
import Graph from 'graphology';
import { forceSimulation, forceManyBody, forceLink, forceX, forceY } from 'd3-force';
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
import type { GraphData, PathResult } from '../types';
import { findPath } from '../services/api';
import { ArchitecturalReportModal } from './ArchitecturalReportModal';
import { getNodeColor, communityColor } from './graph/palettes';
import { computeConvexHull, drawSmoothHull, type Point2D } from './graph/hulls';

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
}

interface SimNode {
  id: string;
  x?: number;
  y?: number;
  vx?: number;
  vy?: number;
  [key: string]: any;
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
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const underlayRef = useRef<HTMLCanvasElement>(null);
  const overlayRef = useRef<HTMLCanvasElement>(null);
  const sigmaRef = useRef<Sigma | null>(null);
  const graphRef = useRef<Graph | null>(null);
  const simRef = useRef<any>(null);
  const simNodesRef = useRef<Map<string, SimNode>>(new Map());
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

  // Interaction refs
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
  const glowingStateRef = useRef<{ id: string; startTime: number; duration: number } | null>(null);
  const graphDataRef = useRef<GraphData>(graphData);
  const lastRenderedKeyRef = useRef<string>('');
  const glowRafRef = useRef<number>(0);

  const refreshGraph = useCallback(() => {
    sigmaRef.current?.refresh();
  }, []);

  useEffect(() => {
    onSelectNodeRef.current = onSelectNode;
    onSelectCodeRef.current = onSelectCode;
    onClearSelectionRef.current = onClearSelection;
  }, [onSelectNode, onSelectCode, onClearSelection]);

  useEffect(() => {
    graphDataRef.current = graphData;
  }, [graphData]);

  useEffect(() => {
    colorModeRef.current = colorMode;
    highlightGodNodesRef.current = highlightGodNodes;
    pathModeRef.current = pathMode;
    pathStartNodeRef.current = pathStartNode;
    pathResultRef.current = pathResult;
    themeRef.current = theme;
    egoDepthRef.current = egoDepth;
    refreshGraph();
  }, [colorMode, highlightGodNodes, pathMode, pathStartNode, pathResult, theme, egoDepth, refreshGraph]);

  const stats = useMemo(() => {
    let notes = 0;
    let code = 0;
    for (const n of graphData.nodes) {
      if (n.node_type === 'code') code++;
      else notes++;
    }
    return { notes, code, total: graphData.nodes.length, links: graphData.links.length };
  }, [graphData]);

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
    const adj = new Map<string, string[]>();
    for (const link of graphDataRef.current.links) {
      const s = typeof link.source === 'object' ? (link.source as any).id : link.source;
      const t = typeof link.target === 'object' ? (link.target as any).id : link.target;
      if (!adj.has(s)) adj.set(s, []);
      if (!adj.has(t)) adj.set(t, []);
      adj.get(s)!.push(t);
      adj.get(t)!.push(s);
    }
    const visited = new Set<string>([startId]);
    let frontier = [startId];
    for (let d = 0; d < hops && frontier.length > 0; d++) {
      const next: string[] = [];
      for (const curr of frontier) {
        for (const nb of adj.get(curr) || []) {
          if (!visited.has(nb)) {
            visited.add(nb);
            next.push(nb);
          }
        }
      }
      frontier = next;
    }
    connectedNodeIdsRef.current = visited;
  }, []);

  useEffect(() => {
    recomputeConnected(hoverNodeIdRef.current || selectedNodeIdRef.current, egoDepth);
    refreshGraph();
  }, [egoDepth, recomputeConnected, refreshGraph]);

  // ---- overlay + underlay painters -------------------------------------------
  const nodeViewPos = useCallback((id: string): { x: number; y: number; size: number } | null => {
    const sigma = sigmaRef.current;
    if (!sigma) return null;
    const dd = sigma.getNodeDisplayData(id);
    if (!dd) return null;
    const p = sigma.framedGraphToViewport({ x: dd.x, y: dd.y });
    return { x: p.x, y: p.y, size: dd.size };
  }, []);

  const pxPerUnit = useCallback((): number => {
    const sigma = sigmaRef.current;
    if (!sigma) return 1;
    const a = sigma.framedGraphToViewport({ x: 0, y: 0 });
    const b = sigma.framedGraphToViewport({ x: 0.01, y: 0 });
    const d = Math.hypot(b.x - a.x, b.y - a.y);
    return d / 0.01 || 1;
  }, []);

  const paintUnderlay = useCallback(() => {
    const canvas = underlayRef.current;
    const sigma = sigmaRef.current;
    const graph = graphRef.current;
    if (!canvas || !sigma || !graph) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    if (canvas.width !== w * dpr || canvas.height !== h * dpr) {
      canvas.width = w * dpr;
      canvas.height = h * dpr;
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);

    if (colorModeRef.current !== 'communities') return;

    const isLight = themeRef.current === 'light';
    const connected = connectedNodeIdsRef.current;
    const groups = new Map<number, { pts: Point2D[]; sumX: number; minY: number; anyConnected: boolean }>();

    graph.forEachNode((id, attrs) => {
      if (typeof attrs.community !== 'number') return;
      const pos = nodeViewPos(id);
      if (!pos) return;
      let g = groups.get(attrs.community);
      if (!g) {
        g = { pts: [], sumX: 0, minY: Infinity, anyConnected: false };
        groups.set(attrs.community, g);
      }
      g.pts.push({ x: pos.x, y: pos.y });
      g.sumX += pos.x;
      if (pos.y < g.minY) g.minY = pos.y;
      if (connected.size > 0 && connected.has(id)) g.anyConnected = true;
    });

    const pad = 24;
    groups.forEach((g, commId) => {
      if (g.pts.length < 2) return;
      if (connected.size > 0 && !g.anyConnected) return;

      const padded: Point2D[] = [];
      for (const p of g.pts) {
        for (let a = 0; a < 8; a++) {
          const ang = (a * Math.PI) / 4;
          padded.push({ x: p.x + Math.cos(ang) * pad, y: p.y + Math.sin(ang) * pad });
        }
      }
      const hull = computeConvexHull(padded);
      if (hull.length < 3) return;

      const color = communityColor(commId, isLight);
      ctx.save();
      drawSmoothHull(ctx, hull);
      ctx.fillStyle = `${color}14`;
      ctx.fill();
      ctx.lineWidth = 1;
      ctx.setLineDash([5, 4]);
      ctx.strokeStyle = `${color}50`;
      ctx.stroke();
      ctx.setLineDash([]);
      ctx.font = `600 9.5px 'JetBrains Mono', monospace`;
      ctx.textAlign = 'center';
      ctx.textBaseline = 'bottom';
      ctx.fillStyle = `${color}b0`;
      ctx.fillText(`#${commId} · ${g.pts.length}`, g.sumX / g.pts.length, g.minY - pad - 4);
      ctx.restore();
    });
  }, [nodeViewPos]);

  const paintOverlay = useCallback(() => {
    const canvas = overlayRef.current;
    const sigma = sigmaRef.current;
    const graph = graphRef.current;
    if (!canvas || !sigma || !graph) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    const dpr = window.devicePixelRatio || 1;
    const w = canvas.clientWidth;
    const h = canvas.clientHeight;
    if (canvas.width !== w * dpr || canvas.height !== h * dpr) {
      canvas.width = w * dpr;
      canvas.height = h * dpr;
    }
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, w, h);

    const isLight = themeRef.current === 'light';
    const connected = connectedNodeIdsRef.current;
    const sel = selectedNodeIdRef.current;
    const hov = hoverNodeIdRef.current;
    const scale = pxPerUnit();
    const showLabels = sigma.getCamera().ratio < 1.4;
    const pathNodes = new Set<string>();
    if (pathResultRef.current?.steps) {
      for (const s of pathResultRef.current.steps) pathNodes.add(s.node_id);
    }

    let labelCount = 0;
    const maxLabels = 400;

    graph.forEachNode((id, attrs) => {
      const pos = nodeViewPos(id);
      if (!pos) return;
      const { x, y } = pos;
      const r = Math.max(3, pos.size * scale * 0.5);

      const matchesSel =
        sel !== null &&
        (id === sel ||
          id.toLowerCase() === sel.toLowerCase() ||
          id.replace(/\.md$/i, '').toLowerCase() === sel.replace(/\.md$/i, '').toLowerCase() ||
          (attrs.title && attrs.title.toLowerCase() === sel.toLowerCase()) ||
          (attrs.title && attrs.title.replace(/\.md$/i, '').toLowerCase() === sel.replace(/\.md$/i, '').toLowerCase()) ||
          (attrs.path && (attrs.path === sel || attrs.path.toLowerCase() === sel.toLowerCase())));
      const isHovered = id === hov;
      const isConnected = connected.has(id);
      const isDimmed = connected.size > 0 && !isConnected;
      const isCode = attrs.node_type === 'code';
      const accent = getNodeColor(attrs, isLight, colorModeRef.current);

      const glowing = glowingStateRef.current;
      const isGlowing =
        glowing &&
        (id === glowing.id ||
          id.toLowerCase() === glowing.id.toLowerCase() ||
          attrs.title?.toLowerCase() === glowing.id.toLowerCase() ||
          id.replace(/\.md$/i, '').toLowerCase() === glowing.id.replace(/\.md$/, '').toLowerCase());

      // God node dashed ring
      const isGod = highlightGodNodesRef.current && (attrs.is_god_node || (attrs.degree || 0) >= 12);
      if (isGod && !isDimmed) {
        ctx.save();
        ctx.beginPath();
        ctx.arc(x, y, r + 7, 0, 2 * Math.PI);
        ctx.strokeStyle = 'rgba(217, 160, 95, 0.55)';
        ctx.lineWidth = 1;
        ctx.setLineDash([3, 3]);
        ctx.stroke();
        ctx.restore();
      }

      // Path rings
      const isPathStart = pathStartNodeRef.current && pathStartNodeRef.current.id === id;
      if (isPathStart || pathNodes.has(id)) {
        ctx.save();
        ctx.beginPath();
        ctx.arc(x, y, r + 6, 0, 2 * Math.PI);
        ctx.strokeStyle = isPathStart ? '#5fa8d9' : '#4fb8c8';
        ctx.lineWidth = 2;
        if (isPathStart) ctx.setLineDash([4, 2]);
        ctx.stroke();
        ctx.restore();
      }

      // Code node outer ring
      if (isCode && !isDimmed) {
        ctx.save();
        ctx.beginPath();
        ctx.arc(x, y, r + 3, 0, 2 * Math.PI);
        ctx.strokeStyle = accent;
        ctx.lineWidth = isConnected ? 1.6 : 1;
        ctx.stroke();
        ctx.restore();
      }

      // Selection / hover ring
      if (matchesSel || isHovered) {
        ctx.save();
        ctx.beginPath();
        ctx.arc(x, y, r + 8, 0, 2 * Math.PI);
        ctx.fillStyle = `${accent}22`;
        ctx.fill();
        ctx.beginPath();
        ctx.arc(x, y, r + 4, 0, 2 * Math.PI);
        ctx.strokeStyle = accent;
        ctx.lineWidth = 2;
        ctx.stroke();
        ctx.restore();
      }

      // Glow shockwave
      if (isGlowing && glowing) {
        const elapsed = Date.now() - glowing.startTime;
        const progress = Math.min(1, elapsed / glowing.duration);
        const alpha = Math.max(0, 1 - progress);
        ctx.save();
        ctx.beginPath();
        ctx.arc(x, y, 12 + progress * 40, 0, 2 * Math.PI);
        ctx.strokeStyle = `rgba(79, 176, 140, ${alpha * 0.9})`;
        ctx.lineWidth = Math.max(1, 3.5 * (1 - progress));
        ctx.stroke();
        ctx.beginPath();
        ctx.arc(x, y, 16, 0, 2 * Math.PI);
        ctx.fillStyle = `rgba(79, 176, 140, ${alpha * 0.3})`;
        ctx.fill();
        ctx.font = `600 9px 'JetBrains Mono', monospace`;
        ctx.textAlign = 'center';
        ctx.textBaseline = 'bottom';
        ctx.fillStyle = `rgba(79, 176, 140, ${alpha})`;
        ctx.fillText('MCP LIVE', x, y - 18);
        ctx.restore();
      }

      // Label capsule
      if ((showLabels || matchesSel || isHovered || isGlowing) && labelCount < maxLabels) {
        labelCount++;
        const rawLabel = attrs.title || id;
        const label = rawLabel.endsWith('.md') ? rawLabel.replace(/\.md$/, '') : rawLabel;
        const text =
          !isCode && attrs.tags && attrs.tags.length > 0 ? `#${attrs.tags[0]}` : label;

        ctx.font = `${matchesSel || isGlowing ? '600 ' : '500 '}10.5px 'Geist Sans', system-ui, sans-serif`;
        const tw = ctx.measureText(text).width;
        const ty = y + (matchesSel ? r + 14 : r + 10) + 5;
        const padX = 6;
        const padY = 2.5;

        ctx.save();
        ctx.beginPath();
        if (typeof (ctx as any).roundRect === 'function') {
          (ctx as any).roundRect(x - tw / 2 - padX, ty - 5.25 - padY, tw + padX * 2, 10.5 + padY * 2, 4);
        } else {
          ctx.rect(x - tw / 2 - padX, ty - 5.25 - padY, tw + padX * 2, 10.5 + padY * 2);
        }
        ctx.fillStyle = isDimmed
          ? isLight
            ? 'rgba(241, 240, 235, 0.7)'
            : 'rgba(22, 25, 34, 0.4)'
          : isLight
          ? 'rgba(255, 255, 255, 0.92)'
          : 'rgba(16, 18, 22, 0.88)';
        ctx.fill();
        ctx.lineWidth = 1;
        ctx.strokeStyle = isGlowing
          ? '#4fb08c'
          : matchesSel
          ? accent
          : isDimmed
          ? 'rgba(120, 125, 140, 0.25)'
          : `${accent}55`;
        ctx.stroke();
        ctx.fillStyle = isGlowing
          ? '#4fb08c'
          : isDimmed
          ? isLight
            ? '#a8a49c'
            : 'rgba(120, 125, 140, 0.45)'
          : isLight
          ? '#2f3437'
          : '#edeef0';
        ctx.textAlign = 'center';
        ctx.textBaseline = 'middle';
        ctx.fillText(text, x, ty);
        ctx.restore();
      }
    });
  }, [nodeViewPos, pxPerUnit]);

  // ---- sigma init -------------------------------------------------------------
  useEffect(() => {
    if (!containerRef.current) return;
    const elem = containerRef.current;

    const graph = new Graph({ type: 'undirected' });
    graphRef.current = graph;

    const sigma = new Sigma(graph, elem, {
      settings: {
        renderLabels: false,
        renderEdgeLabels: false,
        minCameraRatio: 0.04,
        maxCameraRatio: 20,
        stagePadding: 40,
        allowInvalidContainer: true,
      },
      nodeReducer: (node, data) => {
        const attrs = graph.getNodeAttributes(node);
        const isLight = themeRef.current === 'light';
        const connected = connectedNodeIdsRef.current;
        const accent = getNodeColor(attrs, isLight, colorModeRef.current);
        const isDimmed = connected.size > 0 && !connected.has(node);

        data.color = isDimmed
          ? isLight
            ? 'rgba(168, 164, 156, 0.35)'
            : 'rgba(86, 94, 107, 0.25)'
          : accent;
        data.size = Math.max(5, Math.min(11, 5 + (attrs.degree || 0) * 1.1));
        data.label = '';
        return data;
      },
      edgeReducer: (edge, data) => {
        const attrs = graph.getEdgeAttributes(edge) as any;
        const s = attrs.__s as string;
        const t = attrs.__t as string;
        const isLight = themeRef.current === 'light';
        const connected = connectedNodeIdsRef.current;

        if (pathResultRef.current && pathResultRef.current.steps.length > 1) {
          const steps = pathResultRef.current.steps;
          const onPath = steps.some((step, idx) => {
            if (idx === 0) return false;
            const prev = steps[idx - 1];
            return (
              (prev.node_id === s && step.node_id === t) ||
              (prev.node_id === t && step.node_id === s)
            );
          });
          if (onPath) {
            data.color = isLight ? '#1f6c9f' : '#5fa8d9';
            data.size = 2.5;
            return data;
          }
        }

        if (connected.size > 0) {
          if (connected.has(s) && connected.has(t)) {
            data.color = isLight ? 'rgba(79, 91, 192, 0.65)' : 'rgba(109, 122, 224, 0.6)';
            data.size = 1.4;
          } else {
            data.color = isLight ? 'rgba(168, 164, 156, 0.18)' : 'rgba(86, 94, 107, 0.1)';
            data.size = 0.8;
          }
          return data;
        }

        const isImports = attrs.label === 'imports';
        data.color = isImports
          ? isLight
            ? 'rgba(31, 108, 159, 0.3)'
            : 'rgba(95, 168, 217, 0.22)'
          : isLight
          ? 'rgba(79, 91, 192, 0.25)'
          : 'rgba(109, 122, 224, 0.18)';
        data.size = 0.9;
        return data;
      },
    });
    sigmaRef.current = sigma;

    // Simulation — same force profile as before
    const sim = forceSimulation([] as SimNode[])
      .force('charge', forceManyBody().strength(-160))
      .force(
        'link',
        forceLink<SimNode, any>([])
          .id((d) => d.id)
          .distance(65)
      )
      .force('x', forceX(0).strength(0.03))
      .force('y', forceY(0).strength(0.03))
      .force('cluster', (alpha: number) => {
        if (colorModeRef.current !== 'communities') return;
        const centroids = new Map<number, { x: number; y: number; count: number }>();
        for (const n of sim.nodes()) {
          if (typeof n.community !== 'number') continue;
          const c = centroids.get(n.community) || { x: 0, y: 0, count: 0 };
          c.x += n.x || 0;
          c.y += n.y || 0;
          c.count += 1;
          centroids.set(n.community, c);
        }
        centroids.forEach((c) => {
          c.x /= c.count;
          c.y /= c.count;
        });
        const strength = 0.12 * alpha;
        for (const n of sim.nodes()) {
          const c = n.community !== undefined ? centroids.get(n.community) : undefined;
          if (c && c.count > 1) {
            n.vx = (n.vx || 0) + (c.x - (n.x || 0)) * strength;
            n.vy = (n.vy || 0) + (c.y - (n.y || 0)) * strength;
          }
        }
      })
      .on('tick', () => {
        const g = graphRef.current;
        if (!g) return;
        g.forEachNode((id, attrs) => {
          const sn = simNodesRef.current.get(id);
          if (sn && sn.x !== undefined && sn.y !== undefined) {
            attrs.x = sn.x;
            attrs.y = sn.y;
          }
        });
        sigma.refresh();
      });
    sim.stop();
    simRef.current = sim;

    // Repaint canvases whenever sigma renders
    sigma.on('afterRender', () => {
      paintUnderlay();
      paintOverlay();
    });

    // Interactions
    sigma.on('clickNode', ({ node }) => {
      const attrs = graph.getNodeAttributes(node) as any;

      if (pathModeRef.current) {
        if (!pathStartNodeRef.current) {
          pathStartNodeRef.current = { id: node, title: attrs.title };
          setPathStartNode({ id: node, title: attrs.title });
          sigma.refresh();
          return;
        }
        if (pathStartNodeRef.current.id === node) {
          pathStartNodeRef.current = null;
          pathResultRef.current = null;
          setPathStartNode(null);
          setPathResult(null);
          sigma.refresh();
          return;
        }
        findPath(pathStartNodeRef.current.id, node)
          .then((res) => {
            pathResultRef.current = res;
            setPathResult(res);
            sigma.refresh();
          })
          .catch((err) => console.error('Failed to find path:', err));
        return;
      }

      selectedNodeIdRef.current = node;
      recomputeConnected(node, egoDepthRef.current);
      sigma.refresh();

      if (attrs.node_type === 'code' && onSelectCodeRef.current) {
        onSelectCodeRef.current(attrs.path || node);
      }
      onSelectNodeRef.current(node);
    });

    sigma.on('clickStage', () => {
      selectedNodeIdRef.current = null;
      hoverNodeIdRef.current = null;
      connectedNodeIdsRef.current = new Set();
      if (pathModeRef.current) {
        pathStartNodeRef.current = null;
        pathResultRef.current = null;
        setPathStartNode(null);
        setPathResult(null);
      }
      onClearSelectionRef.current?.();
      sigma.refresh();
    });

    sigma.on('enterNode', ({ node }) => {
      if (hoverNodeIdRef.current === node) return;
      hoverNodeIdRef.current = node;
      recomputeConnected(node, egoDepthRef.current);
      elem.style.cursor = 'pointer';
      sigma.refresh();
    });

    sigma.on('leaveNode', () => {
      hoverNodeIdRef.current = null;
      recomputeConnected(selectedNodeIdRef.current, egoDepthRef.current);
      elem.style.cursor = 'default';
      sigma.refresh();
    });

    return () => {
      cancelAnimationFrame(glowRafRef.current);
      sim.stop();
      sigma.kill();
      sigmaRef.current = null;
      graphRef.current = null;
      lastRenderedKeyRef.current = '';
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // ---- data sync ---------------------------------------------------------------
  useEffect(() => {
    selectedNodeIdRef.current = selectedNodeId;
    recomputeConnected(hoverNodeIdRef.current || selectedNodeId);
    refreshGraph();
  }, [selectedNodeId, recomputeConnected, refreshGraph]);

  useEffect(() => {
    if (prevProjectIdRef.current !== projectId) {
      prevProjectIdRef.current = projectId;
      setFilterMode('all');
      setLocalOnly(false);
      setPathMode(false);
      hoverNodeIdRef.current = null;
      connectedNodeIdsRef.current = new Set();
      lastRenderedKeyRef.current = '';
      const g = graphRef.current;
      if (g) g.clear();
      simNodesRef.current.clear();
      simRef.current?.nodes([]);
      sigmaRef.current?.refresh();
    }
  }, [projectId]);

  useEffect(() => {
    const sigma = sigmaRef.current;
    const graph = graphRef.current;
    const sim = simRef.current;
    if (!sigma || !graph || !sim) return;

    let displayData = graphData;

    if (filterMode === 'notes') {
      const sub = graphData.nodes.filter((n) => n.node_type !== 'code');
      const ids = new Set(sub.map((n) => n.id));
      displayData = {
        nodes: sub,
        links: graphData.links.filter((l) => {
          const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
          const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
          return ids.has(s) && ids.has(t);
        }),
      };
    } else if (filterMode === 'code') {
      const sub = graphData.nodes.filter((n) => n.node_type === 'code');
      const ids = new Set(sub.map((n) => n.id));
      displayData = {
        nodes: sub,
        links: graphData.links.filter((l) => {
          const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
          const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
          return ids.has(s) && ids.has(t);
        }),
      };
    }

    if (localOnly && selectedNodeId) {
      recomputeConnected(selectedNodeId, egoDepth);
      const connected = connectedNodeIdsRef.current;
      displayData = {
        nodes: displayData.nodes.filter((n) => connected.has(n.id)),
        links: displayData.links.filter((l) => {
          const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
          const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
          return connected.has(s) && connected.has(t);
        }),
      };
    }

    const key =
      `${projectId || 'default'}|${filterMode}|${localOnly ? `${selectedNodeId}:${egoDepth}` : 'all'}|` +
      displayData.nodes.map((n) => `${n.id}:${n.title}:${n.degree || 0}`).sort().join(';') +
      '|' +
      displayData.links
        .map((l) => {
          const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
          const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
          return `${s}->${t}`;
        })
        .sort()
        .join(';');

    if (lastRenderedKeyRef.current === key) return;
    lastRenderedKeyRef.current = key;

    const hadNodes = simNodesRef.current.size > 0;
    const wanted = new Set(displayData.nodes.map((n) => n.id));

    // Drop removed nodes
    const toDrop: string[] = [];
    graph.forEachNode((id) => {
      if (!wanted.has(id)) toDrop.push(id);
    });
    toDrop.forEach((id) => {
      graph.dropNode(id);
      simNodesRef.current.delete(id);
    });

    // Upsert nodes
    const centroid = (() => {
      let cx = 0, cy = 0, c = 0;
      simNodesRef.current.forEach((n) => {
        if (n.x !== undefined && n.y !== undefined) {
          cx += n.x;
          cy += n.y;
          c++;
        }
      });
      return c ? { x: cx / c, y: cy / c } : { x: 0, y: 0 };
    })();

    for (const n of displayData.nodes) {
      const attrs: any = {
        title: n.title,
        tags: n.tags,
        degree: n.degree,
        node_type: n.node_type,
        path: n.path,
        community: n.community,
        is_god_node: n.is_god_node,
        centrality: n.centrality,
        label: n.title || n.id,
      };
      if (graph.hasNode(n.id)) {
        graph.updateNodeAttributes(n.id, (a) => ({ ...a, ...attrs }));
        const sn = simNodesRef.current.get(n.id);
        if (sn) Object.assign(sn, attrs);
      } else {
        const angle = Math.random() * Math.PI * 2;
        const rad = 40 + Math.random() * 80;
        const x = centroid.x + Math.cos(angle) * rad;
        const y = centroid.y + Math.sin(angle) * rad;
        graph.addNode(n.id, { ...attrs, x, y });
        simNodesRef.current.set(n.id, { id: n.id, x, y, ...attrs });
      }
    }

    // Drop removed edges, add new
    const wantedEdges = new Set<string>();
    const toDropEdges: string[] = [];
    for (const l of displayData.links) {
      const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
      const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
      if (wanted.has(s) && wanted.has(t)) wantedEdges.add(`${s}|${t}`);
    }
    graph.forEachEdge((edge, attrs: any) => {
      const k = `${attrs.__s}|${attrs.__t}`;
      const kr = `${attrs.__t}|${attrs.__s}`;
      if (!wantedEdges.has(k) && !wantedEdges.has(kr)) toDropEdges.push(edge);
    });
    toDropEdges.forEach((e) => graph.dropEdge(e));

    const simLinks: { source: string; target: string; label?: string }[] = [];
    for (const l of displayData.links) {
      const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
      const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
      if (!wanted.has(s) || !wanted.has(t)) continue;
      if (!graph.hasEdge(s, t) && !graph.hasEdge(t, s)) {
        graph.addEdge(s, t, { __s: s, __t: t, label: l.label });
      }
      simLinks.push({ source: s, target: t, label: l.label });
    }

    sim.nodes(Array.from(simNodesRef.current.values()));
    const linkForce = sim.force('link') as any;
    if (linkForce && linkForce.links) linkForce.links(simLinks);
    sim.alpha(hadNodes ? 0.5 : 1).restart();

    if (!hadNodes && displayData.nodes.length > 0) {
      setTimeout(() => zoomToFit(), 350);
    }
    sigma.refresh();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [projectId, graphData, filterMode, localOnly, egoDepth, localOnly ? selectedNodeId : null, recomputeConnected]);

  // ---- glow + camera focus ------------------------------------------------------
  useEffect(() => {
    if (!glowingNode || !glowingNode.id) return;
    const targetId = glowingNode.id;
    glowingStateRef.current = { id: targetId, startTime: Date.now(), duration: 3500 };

    const sigma = sigmaRef.current;
    const graph = graphRef.current;
    if (sigma && graph) {
      const clean = targetId.replace(/\.md$/, '').toLowerCase();
      let found: string | null = null;
      graph.forEachNode((id, attrs) => {
        if (found) return;
        if (
          id === targetId ||
          id.toLowerCase() === clean ||
          attrs.title?.toLowerCase() === clean ||
          id.replace(/\.md$/i, '').toLowerCase() === clean
        ) {
          found = id;
        }
      });
      if (found) {
        const dd = sigma.getNodeDisplayData(found);
        if (dd) {
          sigma.getCamera().animate({ x: dd.x, y: dd.y, ratio: 0.3 }, { duration: 800 });
        }
      }
    }

    const loop = () => {
      if (!glowingStateRef.current) return;
      sigmaRef.current?.refresh();
      glowRafRef.current = requestAnimationFrame(loop);
    };
    glowRafRef.current = requestAnimationFrame(loop);

    const timer = setTimeout(() => {
      glowingStateRef.current = null;
      cancelAnimationFrame(glowRafRef.current);
      sigmaRef.current?.refresh();
    }, 3500);

    return () => {
      clearTimeout(timer);
      cancelAnimationFrame(glowRafRef.current);
    };
  }, [glowingNode]);

  // ---- camera controls ----------------------------------------------------------
  const zoomCamera = useCallback((factor: number) => {
    const sigma = sigmaRef.current;
    if (!sigma) return;
    const cam = sigma.getCamera();
    cam.animate({ ratio: cam.ratio * factor }, { duration: 250 });
  }, []);

  const zoomToFit = useCallback(() => {
    const sigma = sigmaRef.current;
    if (!sigma) return;
    let minX = Infinity, minY = Infinity, maxX = -Infinity, maxY = -Infinity, count = 0;
    graphRef.current?.forEachNode((id) => {
      const dd = sigma.getNodeDisplayData(id);
      if (!dd) return;
      count++;
      if (dd.x < minX) minX = dd.x;
      if (dd.y < minY) minY = dd.y;
      if (dd.x > maxX) maxX = dd.x;
      if (dd.y > maxY) maxY = dd.y;
    });
    if (!count) return;
    const dims = sigma.getDimensions();
    const dx = Math.max(0.05, maxX - minX);
    const dy = Math.max(0.05, maxY - minY);
    const ratio = Math.min(20, Math.max(dx / dims.width, dy / dims.height) * Math.max(dims.width, dims.height) * 1.15);
    sigma.getCamera().animate(
      { x: (minX + maxX) / 2, y: (minY + maxY) / 2, ratio },
      { duration: 400 }
    );
  }, []);

  const isLight = theme === 'light';

  const toolbarBtn = (active: boolean, activeClass: string) =>
    `flex items-center gap-1 px-2 py-1 rounded-md text-[11px] font-medium transition-colors cursor-pointer border ${
      active
        ? activeClass
        : `border-transparent ${isLight ? 'text-muted hover:text-ink hover:bg-raise' : 'text-muted hover:text-ink hover:bg-raise'}`
    }`;

  return (
    <div className="relative w-full h-full overflow-hidden bg-canvas transition-colors">
      {/* Underlay: community hulls (below WebGL) */}
      <canvas ref={underlayRef} className="absolute inset-0 w-full h-full pointer-events-none" />
      {/* Sigma WebGL stage */}
      <div ref={containerRef} className="absolute inset-0 w-full h-full" />
      {/* Overlay: rings, glow, labels */}
      <canvas ref={overlayRef} className="absolute inset-0 w-full h-full pointer-events-none" />

      {/* Toolbar */}
      <div
        className="absolute top-3 left-3 z-30 flex items-center gap-1.5 p-1 rounded-lg bg-surface border border-line"
        role="toolbar"
        aria-label="Graph controls"
      >
        <div className="flex items-center gap-0.5 p-0.5 rounded-md border border-line bg-canvas">
          <button
            type="button"
            onClick={() => setFilterMode('all')}
            className={`flex items-center gap-1 px-2 py-1 rounded text-[11px] font-medium transition-colors cursor-pointer ${
              filterMode === 'all' ? 'bg-accent text-on-accent' : 'text-muted hover:text-ink'
            }`}
            title="Show all nodes"
          >
            <CirclesThree size={12} weight="bold" />
            <span>All ({stats.total})</span>
          </button>
          <button
            type="button"
            onClick={() => setFilterMode('notes')}
            className={`flex items-center gap-1 px-2 py-1 rounded text-[11px] font-medium transition-colors cursor-pointer ${
              filterMode === 'notes' ? 'bg-accent text-on-accent' : 'text-muted hover:text-ink'
            }`}
            title="Filter notes only"
          >
            <FileText size={12} />
            <span>Notes ({stats.notes})</span>
          </button>
          <button
            type="button"
            onClick={() => setFilterMode('code')}
            className={`flex items-center gap-1 px-2 py-1 rounded text-[11px] font-medium transition-colors cursor-pointer ${
              filterMode === 'code' ? 'bg-accent text-on-accent' : 'text-muted hover:text-ink'
            }`}
            title="Filter code files only"
          >
            <Code size={12} weight="bold" />
            <span>Code ({stats.code})</span>
          </button>
        </div>

        <div className="w-px h-3.5 my-auto bg-line" />

        <button
          type="button"
          className={toolbarBtn(
            colorMode === 'communities',
            'bg-accent/15 text-accent border-accent/40'
          )}
          title={colorMode === 'communities' ? 'Color Mode: Communities (Active)' : 'Color by Communities'}
          onClick={() => setColorMode(colorMode === 'communities' ? 'default' : 'communities')}
        >
          <SquaresFour size={12} weight={colorMode === 'communities' ? 'bold' : 'regular'} />
          <span>Clusters</span>
        </button>

        <button
          type="button"
          className={toolbarBtn(highlightGodNodes, 'bg-warn/15 text-warn border-warn/40')}
          title={highlightGodNodes ? 'Highlight Architectural Hubs / God Nodes' : 'God Nodes Hidden'}
          onClick={() => setHighlightGodNodes(!highlightGodNodes)}
        >
          <Crown size={12} weight={highlightGodNodes ? 'bold' : 'regular'} />
          <span>Hubs</span>
        </button>

        <button
          type="button"
          className={toolbarBtn(pathMode, 'bg-accent/15 text-accent border-accent/40')}
          title="Path Inspector: Click any 2 nodes to find shortest path"
          onClick={() => setPathMode(!pathMode)}
        >
          <GitBranch size={12} weight={pathMode ? 'bold' : 'regular'} />
          <span>Path</span>
        </button>

        <button
          type="button"
          className={toolbarBtn(false, '')}
          title="Open Architectural Audit Report"
          onClick={() => setShowAuditModal(true)}
        >
          <ChartBar size={12} weight="bold" />
          <span>Audit</span>
        </button>

        <div className="w-px h-3.5 my-auto bg-line" />

        <div className="flex items-center gap-1.5">
          <button
            type="button"
            className={toolbarBtn(localOnly, 'bg-accent/15 text-accent border-accent/40')}
            title={localOnly ? 'Disable Ego Network Focus (Show Full Graph)' : 'Enable Ego Network (N-Hop Focus)'}
            onClick={() => setLocalOnly(!localOnly)}
          >
            {localOnly ? <Crosshair size={12} weight="bold" /> : <Compass size={12} />}
            <span>Ego {egoDepth}H</span>
          </button>

          {(localOnly || selectedNodeId) && (
            <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-md border border-line bg-canvas text-[10px] font-mono text-muted">
              <span className="text-[9px] uppercase tracking-wider">Hops</span>
              <input
                type="range"
                min={1}
                max={5}
                step={1}
                value={egoDepth}
                onChange={(e) => setEgoDepth(Number(e.target.value))}
                className="w-14 h-1 accent-[#6d7ae0] cursor-pointer"
                aria-label="Ego Network depth"
              />
              <span className="font-semibold text-accent">{egoDepth}</span>
            </div>
          )}
        </div>

        <button
          type="button"
          className="flex items-center justify-center w-6 h-6 rounded-md transition-colors cursor-pointer text-muted hover:text-ink hover:bg-raise"
          title="Zoom In"
          onClick={() => zoomCamera(0.75)}
        >
          <MagnifyingGlassPlus size={13} />
        </button>
        <button
          type="button"
          className="flex items-center justify-center w-6 h-6 rounded-md transition-colors cursor-pointer text-muted hover:text-ink hover:bg-raise"
          title="Zoom Out"
          onClick={() => zoomCamera(1.35)}
        >
          <MagnifyingGlassMinus size={13} />
        </button>
        <button
          type="button"
          className="flex items-center justify-center w-6 h-6 rounded-md transition-colors cursor-pointer text-muted hover:text-ink hover:bg-raise"
          title="Fit to Screen"
          onClick={zoomToFit}
        >
          <ArrowsOut size={13} />
        </button>
        <button
          type="button"
          className="flex items-center justify-center w-6 h-6 rounded-md transition-colors cursor-pointer text-muted hover:text-ink hover:bg-raise"
          title="Reorganize Graph"
          onClick={onRefresh}
        >
          <ArrowClockwise size={13} />
        </button>
      </div>

      {/* Path inspector banner */}
      {pathMode && (
        <div className="absolute top-14 left-3 z-30 flex items-center gap-2.5 px-3 py-1.5 rounded-lg border border-line bg-surface text-[11px] font-mono text-muted">
          <GitBranch size={13} className="text-accent shrink-0" />
          {!pathStartNode ? (
            <span>
              Click the <strong className="text-accent">source node</strong>...
            </span>
          ) : !pathResult ? (
            <span>
              From: <strong className="text-accent">{pathStartNode.title || pathStartNode.id}</strong> → click the{' '}
              <strong className="text-good">target node</strong>...
            </span>
          ) : (
            <div className="flex items-center gap-2">
              <span>
                Path: <strong className="text-accent">{pathResult.length} nodes</strong> (
                {Math.max(0, pathResult.steps.length - 1)} hops)
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
          <button
            type="button"
            onClick={() => setPathMode(false)}
            className="p-1 text-faint hover:text-ink transition-colors cursor-pointer"
            title="Close Path Inspector"
          >
            <X size={12} />
          </button>
        </div>
      )}

      {/* HUD */}
      <div className="absolute bottom-3 left-3 z-30 flex items-center gap-2 px-2.5 py-1 rounded-md border border-line bg-surface text-[10.5px] font-mono text-muted">
        <span className="w-1.5 h-1.5 rounded-full bg-good shrink-0" />
        <span>
          Notes: <strong className="font-semibold text-ink">{stats.notes}</strong>
        </span>
        <span className="text-faint">·</span>
        <span>
          Code: <strong className="font-semibold text-accent">{stats.code}</strong>
        </span>
        <span className="text-faint">·</span>
        <span>
          Links: <strong className="font-semibold text-ink">{stats.links}</strong>
        </span>
        {localOnly && (
          <>
            <span className="text-faint">·</span>
            <span className="font-medium text-accent">Ego ({egoDepth}-Hop)</span>
          </>
        )}
      </div>

      <ArchitecturalReportModal
        isOpen={showAuditModal}
        onClose={() => setShowAuditModal(false)}
        onSelectNode={onSelectNode}
        theme={theme}
      />
    </div>
  );
};
