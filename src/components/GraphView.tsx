import React, { useEffect, useRef, useState } from 'react';
import ForceGraph from 'force-graph';
import {
  MagnifyingGlassPlus,
  MagnifyingGlassMinus,
  ArrowsOut,
  ArrowClockwise,
  Crosshair,
  Compass,
} from '@phosphor-icons/react';
import type { GraphData, GraphNode } from '../types';

interface GraphViewProps {
  graphData: GraphData;
  selectedNodeId: string | null;
  onSelectNode: (nodeId: string) => void;
  onRefresh: () => void;
}

const TAG_COLORS: Record<string, string> = {
  inicio: '#94a3b8',
  boasvindas: '#cbd5e1',
  arquitetura: '#64748b',
  rust: '#d97706',
  sqlite: '#64748b',
  mcp: '#6366f1',
  guia: '#10b981',
  tutorial: '#059669',
  ia: '#4f46e5',
  rag: '#6366f1',
  embeddings: '#0d9488',
  inexistente: '#71717a',
};

function getNodeColor(node: GraphNode, isSelected: boolean, isHovered: boolean): string {
  if (isSelected) return '#ffffff';
  if (isHovered) return '#f1f5f9';
  for (const tag of node.tags) {
    const key = tag.toLowerCase();
    if (TAG_COLORS[key]) {
      return TAG_COLORS[key];
    }
  }
  return '#94a3b8';
}

export const GraphView: React.FC<GraphViewProps> = ({
  graphData,
  selectedNodeId,
  onSelectNode,
  onRefresh,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphInstanceRef = useRef<any>(null);
  const [hoverNodeId, setHoverNodeId] = useState<string | null>(null);
  const [localOnly, setLocalOnly] = useState<boolean>(false);

  // Compute connected nodes for hover & focus highlighting
  const connectedNodeIds = React.useMemo(() => {
    const targetId = hoverNodeId || selectedNodeId;
    if (!targetId) return new Set<string>();
    const set = new Set<string>([targetId]);
    for (const link of graphData.links) {
      const sourceId = typeof link.source === 'object' ? (link.source as any).id : link.source;
      const targetLink = typeof link.target === 'object' ? (link.target as any).id : link.target;
      if (sourceId === targetId) set.add(targetLink);
      if (targetLink === targetId) set.add(sourceId);
    }
    return set;
  }, [hoverNodeId, selectedNodeId, graphData.links]);

  useEffect(() => {
    if (!containerRef.current) return;

    const elem = containerRef.current;
    const width = elem.clientWidth || 800;
    const height = elem.clientHeight || 600;

    // Filter nodes if localOnly is active
    let displayData = graphData;
    if (localOnly && selectedNodeId) {
      const subNodes = graphData.nodes.filter((n) => connectedNodeIds.has(n.id));
      const subLinks = graphData.links.filter((l) => {
        const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
        const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
        return connectedNodeIds.has(s) && connectedNodeIds.has(t);
      });
      displayData = { nodes: subNodes, links: subLinks };
    }

    if (!graphInstanceRef.current) {
      const graph = new (ForceGraph as any)(elem)
        .width(width)
        .height(height)
        .backgroundColor('transparent')
        .nodeRelSize(4)
        .nodeVal((node: any) => Math.max(3.5, Math.min(14, 3.5 + (node.degree || 0) * 1.5)))
        .linkColor((link: any) => {
          const s = typeof link.source === 'object' ? link.source.id : link.source;
          const t = typeof link.target === 'object' ? link.target.id : link.target;
          if (connectedNodeIds.size > 0) {
            if (connectedNodeIds.has(s) && connectedNodeIds.has(t)) {
              return 'rgba(255, 255, 255, 0.45)';
            }
            return 'rgba(255, 255, 255, 0.03)';
          }
          return 'rgba(255, 255, 255, 0.1)';
        })
        .linkWidth((link: any) => {
          const s = typeof link.source === 'object' ? link.source.id : link.source;
          const t = typeof link.target === 'object' ? link.target.id : link.target;
          if (connectedNodeIds.has(s) && connectedNodeIds.has(t)) return 1.5;
          return 1;
        })
        .nodeCanvasObject((node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
          const isSelected = node.id === selectedNodeId;
          const isHovered = node.id === hoverNodeId;
          const isDimmed = connectedNodeIds.size > 0 && !connectedNodeIds.has(node.id);

          const r = Math.max(4, Math.min(13, 4 + (node.degree || 0) * 1.3));
          const color = getNodeColor(node, isSelected, isHovered);

          // Concentric target ring for active / hovered (No neon blooms)
          if (isSelected || isHovered) {
            ctx.beginPath();
            ctx.arc(node.x, node.y, r + 4.5, 0, 2 * Math.PI, false);
            ctx.lineWidth = 1;
            ctx.strokeStyle = isSelected ? 'rgba(255, 255, 255, 0.45)' : 'rgba(255, 255, 255, 0.2)';
            ctx.stroke();
          }

          // Main node circle
          ctx.beginPath();
          ctx.arc(node.x, node.y, r, 0, 2 * Math.PI, false);
          ctx.fillStyle = isDimmed ? 'rgba(71, 85, 105, 0.25)' : color;
          ctx.fill();

          // Concentric outer hairline border
          if (isSelected) {
            ctx.lineWidth = 2;
            ctx.strokeStyle = '#ffffff';
            ctx.stroke();
          } else if (!isDimmed) {
            ctx.lineWidth = 1;
            ctx.strokeStyle = 'rgba(255, 255, 255, 0.25)';
            ctx.stroke();
          }

          // Label text with contrast pill behind
          if (globalScale > 0.75 || isSelected || isHovered) {
            const label = node.title || node.id;
            const fontSize = Math.max(10 / globalScale, 9);
            ctx.font = `${isSelected ? '600 ' : '500 '}${fontSize}px 'Plus Jakarta Sans', system-ui, sans-serif`;
            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';

            const textWidth = ctx.measureText(label).width;
            const textY = node.y + r + fontSize * 0.9;

            // Background pill for contrast
            ctx.fillStyle = isDimmed ? 'rgba(9, 10, 15, 0.4)' : 'rgba(9, 10, 15, 0.8)';
            const paddingX = 5;
            const paddingY = 2;
            ctx.beginPath();
            ctx.roundRect(
              node.x - textWidth / 2 - paddingX,
              textY - fontSize / 2 - paddingY,
              textWidth + paddingX * 2,
              fontSize + paddingY * 2,
              3
            );
            ctx.fill();

            // Text
            ctx.fillStyle = isDimmed ? 'rgba(148, 163, 184, 0.35)' : isSelected ? '#ffffff' : '#e2e8f0';
            ctx.fillText(label, node.x, textY);
          }
        })
        .onNodeClick((node: any) => {
          onSelectNode(node.id);
        })
        .onNodeHover((node: any) => {
          setHoverNodeId(node ? node.id : null);
        });

      // Configure force simulation for smooth organic physics
      graph.d3Force('charge')?.strength(-160);
      graph.d3Force('link')?.distance(65);

      graphInstanceRef.current = graph;
    }

    graphInstanceRef.current.graphData(displayData);

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
    };
  }, [graphData, selectedNodeId, hoverNodeId, connectedNodeIds, localOnly, onSelectNode]);

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

  return (
    <div className="relative w-full h-full overflow-hidden bg-[#090a0f]" ref={containerRef}>
      {/* Floating Toolbar Deck */}
      <div
        className="absolute top-4 right-4 z-20 flex items-center gap-1 p-1 rounded-lg bg-zinc-900/90 border border-zinc-800 shadow-xl backdrop-blur-md"
        role="toolbar"
        aria-label="Controles do Grafo"
      >
        <button
          type="button"
          className={`flex items-center justify-center w-8 h-8 rounded-md transition-colors cursor-pointer ${
            localOnly
              ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/40'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/80'
          }`}
          title={localOnly ? 'Desativar Foco (Exibir Todo o Grafo)' : 'Ativar Modo Foco (Vizinhanca Imediata)'}
          onClick={() => setLocalOnly(!localOnly)}
        >
          {localOnly ? <Crosshair size={16} weight="bold" /> : <Compass size={16} />}
        </button>

        <div className="w-[1px] h-4 bg-zinc-800 my-auto" />

        <button
          type="button"
          className="flex items-center justify-center w-8 h-8 rounded-md text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/80 transition-colors cursor-pointer"
          title="Aproximar Zoom (+)"
          onClick={handleZoomIn}
        >
          <MagnifyingGlassPlus size={16} />
        </button>

        <button
          type="button"
          className="flex items-center justify-center w-8 h-8 rounded-md text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/80 transition-colors cursor-pointer"
          title="Afastar Zoom (-)"
          onClick={handleZoomOut}
        >
          <MagnifyingGlassMinus size={16} />
        </button>

        <button
          type="button"
          className="flex items-center justify-center w-8 h-8 rounded-md text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/80 transition-colors cursor-pointer"
          title="Enquadrar Todos os Nos"
          onClick={handleFit}
        >
          <ArrowsOut size={16} />
        </button>

        <div className="w-[1px] h-4 bg-zinc-800 my-auto" />

        <button
          type="button"
          className="flex items-center justify-center w-8 h-8 rounded-md text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/80 transition-colors cursor-pointer"
          title="Recalcular Simulacao de Forca"
          onClick={onRefresh}
        >
          <ArrowClockwise size={16} />
        </button>
      </div>

      {/* Floating HUD Telemetry Capsule */}
      <div className="absolute bottom-4 left-4 z-20 flex items-center gap-2.5 px-3 py-1.5 rounded-full bg-zinc-900/90 border border-zinc-800/90 shadow-lg backdrop-blur-md text-[11px] text-zinc-400">
        <span className="w-2 h-2 rounded-full bg-emerald-400 shrink-0" />
        <span>
          <strong className="font-semibold text-zinc-200">{graphData.nodes.length}</strong> nos
        </span>
        <span className="text-zinc-600">/</span>
        <span>
          <strong className="font-semibold text-zinc-200">{graphData.links.length}</strong> arestas
        </span>
        {localOnly && (
          <>
            <span className="text-zinc-600">/</span>
            <span className="font-medium text-indigo-400">Foco Local</span>
          </>
        )}
      </div>
    </div>
  );
};
