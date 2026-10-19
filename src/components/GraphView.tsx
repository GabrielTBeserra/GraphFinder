import React, { useEffect, useRef, useState, useCallback } from 'react';
import ForceGraph from 'force-graph';
import {
  MagnifyingGlassPlus,
  MagnifyingGlassMinus,
  ArrowsOut,
  ArrowClockwise,
  Crosshair,
  Compass,
} from '@phosphor-icons/react';
import type { GraphData } from '../types';

interface GraphViewProps {
  graphData: GraphData;
  selectedNodeId: string | null;
  onSelectNode: (nodeId: string) => void;
  onRefresh: () => void;
}

export const GraphView: React.FC<GraphViewProps> = ({
  graphData,
  selectedNodeId,
  onSelectNode,
  onRefresh,
}) => {
  const containerRef = useRef<HTMLDivElement>(null);
  const graphInstanceRef = useRef<any>(null);
  const [localOnly, setLocalOnly] = useState<boolean>(false);

  // Interaction refs: keep rendering independent of React state to prevent canvas flickers
  const selectedNodeIdRef = useRef<string | null>(selectedNodeId);
  const hoverNodeIdRef = useRef<string | null>(null);
  const connectedNodeIdsRef = useRef<Set<string>>(new Set());
  const onSelectNodeRef = useRef(onSelectNode);

  useEffect(() => {
    onSelectNodeRef.current = onSelectNode;
  }, [onSelectNode]);

  // Recompute connected node IDs based on active target (hover or selected)
  const recomputeConnected = useCallback(
    (targetId: string | null) => {
      if (!targetId) {
        connectedNodeIdsRef.current = new Set();
        return;
      }
      const set = new Set<string>([targetId]);
      for (const link of graphData.links) {
        const sourceId = typeof link.source === 'object' ? (link.source as any).id : link.source;
        const targetLink = typeof link.target === 'object' ? (link.target as any).id : link.target;
        if (sourceId === targetId) set.add(targetLink);
        if (targetLink === targetId) set.add(sourceId);
      }
      connectedNodeIdsRef.current = set;
    },
    [graphData.links]
  );

  // Initialize ForceGraph once
  useEffect(() => {
    if (!containerRef.current) return;

    const elem = containerRef.current;
    const width = elem.clientWidth || 800;
    const height = elem.clientHeight || 600;

    const graph = new (ForceGraph as any)(elem)
      .width(width)
      .height(height)
      .backgroundColor('#090a0f')
      .nodeRelSize(4)
      .nodeVal((node: any) => Math.max(4, Math.min(12, 4 + (node.degree || 0) * 1.2)))
      .linkColor((link: any) => {
        const s = typeof link.source === 'object' ? link.source.id : link.source;
        const t = typeof link.target === 'object' ? link.target.id : link.target;
        const connected = connectedNodeIdsRef.current;
        if (connected.size > 0) {
          if (connected.has(s) && connected.has(t)) {
            return '#4f46e5';
          }
          return 'rgba(44, 51, 68, 0.15)';
        }
        return '#2c3344';
      })
      .linkWidth((link: any) => {
        const s = typeof link.source === 'object' ? link.source.id : link.source;
        const t = typeof link.target === 'object' ? link.target.id : link.target;
        const connected = connectedNodeIdsRef.current;
        if (connected.has(s) && connected.has(t)) return 2;
        return 1.4;
      })
      .nodeCanvasObject((node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
        const isSelected = node.id === selectedNodeIdRef.current;
        const isHovered = node.id === hoverNodeIdRef.current;
        const connected = connectedNodeIdsRef.current;
        const isConnected = connected.has(node.id);
        const isDimmed = connected.size > 0 && !isConnected;

        if (isSelected || isHovered) {
          // Focal node highlight
          ctx.beginPath();
          ctx.arc(node.x, node.y, 14, 0, 2 * Math.PI, false);
          ctx.fillStyle = '#1e1b4b';
          ctx.fill();
          ctx.lineWidth = 2.5;
          ctx.strokeStyle = '#6366f1';
          ctx.stroke();

          // Center solid dot
          ctx.beginPath();
          ctx.arc(node.x, node.y, 6, 0, 2 * Math.PI, false);
          ctx.fillStyle = '#ffffff';
          ctx.fill();
        } else {
          // Standard node representation
          const r = Math.max(4.5, Math.min(8, 4.5 + (node.degree || 0) * 1.1));
          ctx.beginPath();
          ctx.arc(node.x, node.y, r, 0, 2 * Math.PI, false);
          ctx.fillStyle = isDimmed ? 'rgba(71, 85, 105, 0.2)' : isConnected ? '#f8fafc' : '#cbd5e1';
          ctx.fill();

          if (!isDimmed) {
            ctx.lineWidth = 1;
            ctx.strokeStyle = '#475569';
            ctx.stroke();
          }
        }

        // Title and tag capsule pill
        if (globalScale > 0.75 || isSelected || isHovered) {
          const rawLabel = node.title || node.id;
          const label = rawLabel.endsWith('.md') ? rawLabel.replace(/\.md$/, '') : rawLabel;
          const tagText = node.tags && node.tags.length > 0 ? `#${node.tags[0]}` : label;

          const fontSize = Math.max(9 / globalScale, 9);
          ctx.font = `${isSelected ? '600 ' : '500 '}${fontSize}px 'Plus Jakarta Sans', system-ui, sans-serif`;
          ctx.textAlign = 'center';
          ctx.textBaseline = 'middle';

          const textWidth = ctx.measureText(tagText).width;
          const textY = node.y + (isSelected ? 20 : 12) + fontSize * 0.5;

          const padX = 5;
          const padY = 2.5;
          ctx.beginPath();
          ctx.roundRect(
            node.x - textWidth / 2 - padX,
            textY - fontSize / 2 - padY,
            textWidth + padX * 2,
            fontSize + padY * 2,
            4
          );
          ctx.fillStyle = isDimmed ? 'rgba(24, 27, 36, 0.4)' : '#181b24';
          ctx.fill();
          ctx.lineWidth = 1;
          ctx.strokeStyle = isDimmed ? 'rgba(43, 49, 64, 0.3)' : '#2b3140';
          ctx.stroke();

          ctx.fillStyle = isDimmed ? 'rgba(100, 116, 139, 0.4)' : isSelected ? '#ffffff' : '#e2e8f0';
          ctx.fillText(tagText, node.x, textY);
        }
      })
      .onNodeClick((node: any) => {
        if (node && node.id) {
          onSelectNodeRef.current(node.id);
        }
      })
      .onNodeHover((node: any) => {
        const nextId = node ? node.id : null;
        if (hoverNodeIdRef.current === nextId) return;
        hoverNodeIdRef.current = nextId;
        recomputeConnected(nextId || selectedNodeIdRef.current);
        // Reheat slightly to redraw without resetting the simulation or positions
        graph.d3ReheatSimulation();
      })
      .showPointerCursor((node: any) => !!node);

    // Forces configuration
    graph.d3Force('charge')?.strength(-160);
    graph.d3Force('link')?.distance(65);

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
      if (graphInstanceRef.current) {
        graphInstanceRef.current._destructor?.();
        graphInstanceRef.current = null;
      }
    };
  }, [recomputeConnected]);

  // Update selected node highlight without reloading graph data
  useEffect(() => {
    selectedNodeIdRef.current = selectedNodeId;
    recomputeConnected(hoverNodeIdRef.current || selectedNodeId);
    if (graphInstanceRef.current) {
      graphInstanceRef.current.d3ReheatSimulation();
    }
  }, [selectedNodeId, recomputeConnected]);

  // Update data ONLY when nodes/links change or focus mode is toggled
  useEffect(() => {
    if (!graphInstanceRef.current) return;

    let displayData = graphData;
    if (localOnly && selectedNodeId) {
      recomputeConnected(selectedNodeId);
      const connected = connectedNodeIdsRef.current;
      const subNodes = graphData.nodes.filter((n) => connected.has(n.id));
      const subLinks = graphData.links.filter((l) => {
        const s = typeof l.source === 'object' ? (l.source as any).id : l.source;
        const t = typeof l.target === 'object' ? (l.target as any).id : l.target;
        return connected.has(s) && connected.has(t);
      });
      displayData = { nodes: subNodes, links: subLinks };
    }

    graphInstanceRef.current.graphData({
      nodes: displayData.nodes.map((n) => ({ ...n })),
      links: displayData.links.map((l) => ({ ...l })),
    });
  }, [graphData, localOnly, selectedNodeId, recomputeConnected]);

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
        className="absolute top-4 right-4 z-20 flex items-center gap-1 p-1 rounded-md bg-[#12151c] border border-[#1e222d] shadow-2xl backdrop-blur-md"
        role="toolbar"
        aria-label="Graph Controls"
      >
        <button
          type="button"
          className={`flex items-center justify-center w-7 h-7 rounded transition-colors cursor-pointer ${
            localOnly
              ? 'bg-indigo-600/30 text-indigo-300 border border-indigo-500/40'
              : 'text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#1a1e28]'
          }`}
          title={localOnly ? 'Disable Focus (Show Entire Graph)' : 'Enable Focus Mode'}
          onClick={() => setLocalOnly(!localOnly)}
        >
          {localOnly ? <Crosshair size={14} weight="bold" /> : <Compass size={14} />}
        </button>

        <div className="w-[1px] h-3.5 bg-[#1f232d] my-auto" />

        <button
          type="button"
          className="flex items-center justify-center w-7 h-7 rounded text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#1a1e28] transition-colors cursor-pointer"
          title="Zoom In (+)"
          onClick={handleZoomIn}
        >
          <MagnifyingGlassPlus size={14} />
        </button>

        <button
          type="button"
          className="flex items-center justify-center w-7 h-7 rounded text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#1a1e28] transition-colors cursor-pointer"
          title="Zoom Out (-)"
          onClick={handleZoomOut}
        >
          <MagnifyingGlassMinus size={14} />
        </button>

        <button
          type="button"
          className="flex items-center justify-center w-7 h-7 rounded text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#1a1e28] transition-colors cursor-pointer"
          title="Fit All Nodes"
          onClick={handleFit}
        >
          <ArrowsOut size={14} />
        </button>

        <div className="w-[1px] h-3.5 bg-[#1f232d] my-auto" />

        <button
          type="button"
          className="flex items-center justify-center w-7 h-7 rounded text-[#64748b] hover:text-[#f1f5f9] hover:bg-[#1a1e28] transition-colors cursor-pointer"
          title="Reheat Force Simulation"
          onClick={onRefresh}
        >
          <ArrowClockwise size={14} />
        </button>
      </div>

      {/* Floating HUD Telemetry Capsule */}
      <div className="absolute bottom-4 left-4 z-20 flex items-center gap-2 px-3 py-1 rounded-md bg-[#12151c] border border-[#1e222d] shadow-xl backdrop-blur-md text-[11px] font-mono text-[#64748b]">
        <span className="w-2 h-2 rounded-full bg-[#10b981] shrink-0" />
        <span>
          Nodes: <strong className="font-semibold text-[#f8fafc]">{graphData.nodes.length}</strong>
        </span>
        <span className="text-[#2c3344]">•</span>
        <span>
          Edges: <strong className="font-semibold text-[#f8fafc]">{graphData.links.length}</strong>
        </span>
        {localOnly && (
          <>
            <span className="text-[#2c3344]">•</span>
            <span className="font-medium text-indigo-400">Focus Mode</span>
          </>
        )}
      </div>
    </div>
  );
};
