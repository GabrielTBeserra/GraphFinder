import React, { useEffect, useState } from 'react';
import {
  X,
  ChartBar,
  Crown,
  Lightning,
  Export,
  CheckCircle,
  SpinnerGap,
  SquaresFour,
  ArrowsClockwise,
} from '@phosphor-icons/react';
import { analyzeGraph, exportArtifacts } from '../services/api';
import type { ExportSummary, GraphAnalysisReport } from '../types';

interface ArchitecturalReportModalProps {
  isOpen: boolean;
  onClose: () => void;
  onSelectNode?: (nodeId: string) => void;
  theme?: 'dark' | 'light';
}

const COMMUNITY_COLORS = [
  '#6366f1', '#10b981', '#38bdf8', '#f59e0b', '#ec4899',
  '#8b5cf6', '#06b6d4', '#f97316', '#14b8a6', '#e11d48',
  '#84cc16', '#a855f7', '#0ea5e9', '#eab308'
];

export const ArchitecturalReportModal: React.FC<ArchitecturalReportModalProps> = ({
  isOpen,
  onClose,
  onSelectNode,
  theme = 'dark',
}) => {
  const [report, setReport] = useState<GraphAnalysisReport | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [activeTab, setActiveTab] = useState<'communities' | 'gods' | 'surprises'>('communities');
  const [exporting, setExporting] = useState<boolean>(false);
  const [exportResult, setExportResult] = useState<ExportSummary | null>(null);

  const isLight = theme === 'light';

  useEffect(() => {
    if (isOpen) {
      loadReport();
    } else {
      setExportResult(null);
    }
  }, [isOpen]);

  const loadReport = async () => {
    setLoading(true);
    try {
      const data = await analyzeGraph();
      setReport(data);
    } catch (err) {
      console.error('Failed to load architectural report:', err);
    } finally {
      setLoading(false);
    }
  };

  const handleExport = async () => {
    setExporting(true);
    try {
      const res = await exportArtifacts();
      setExportResult(res);
    } catch (err) {
      console.error('Failed to export graph artifacts:', err);
    } finally {
      setExporting(false);
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/60 backdrop-blur-sm animate-fade-in">
      <div
        className={`relative w-full max-w-4xl max-h-[88vh] flex flex-col rounded-xl shadow-2xl border transition-all overflow-hidden ${
          isLight
            ? 'bg-white border-slate-200 text-slate-800'
            : 'bg-[#0c0f17] border-[#1e2538] text-slate-200'
        }`}
      >
        {/* Header */}
        <div
          className={`flex items-center justify-between px-6 py-4 border-b ${
            isLight ? 'border-slate-200 bg-slate-50' : 'border-[#1e2538] bg-[#0f1422]'
          }`}
        >
          <div className="flex items-center gap-3">
            <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-indigo-500/10 text-indigo-400 border border-indigo-500/20">
              <ChartBar size={18} weight="bold" />
            </div>
            <div>
              <h2 className="text-base font-bold tracking-tight">
                Architectural Topology & Community Audit
              </h2>
              <p className={`text-xs ${isLight ? 'text-slate-500' : 'text-slate-400'}`}>
                Topological modularity (Louvain/LPA), God Nodes, and cross-subsystem dependencies
              </p>
            </div>
          </div>

          <div className="flex items-center gap-2">
            <button
              onClick={handleExport}
              disabled={exporting}
              className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition-all cursor-pointer ${
                exporting
                  ? 'opacity-60 cursor-not-allowed'
                  : 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-sm'
              }`}
              title="Generate graph.json, GRAPH_REPORT.md, and standalone graph.html"
            >
              {exporting ? (
                <SpinnerGap size={14} className="animate-spin" />
              ) : (
                <Export size={14} weight="bold" />
              )}
              <span>{exporting ? 'Exporting...' : 'Export Artifacts'}</span>
            </button>

            <button
              onClick={onClose}
              className={`p-1.5 rounded-lg transition-colors cursor-pointer ${
                isLight ? 'hover:bg-slate-200 text-slate-600' : 'hover:bg-[#1e2538] text-slate-400'
              }`}
            >
              <X size={18} />
            </button>
          </div>
        </div>

        {/* Export Success Banner */}
        {exportResult && (
          <div
            className={`mx-6 mt-4 p-3 rounded-lg border text-xs flex items-center justify-between animate-fade-in ${
              isLight
                ? 'bg-emerald-50 border-emerald-200 text-emerald-800'
                : 'bg-emerald-950/40 border-emerald-800/40 text-emerald-300'
            }`}
          >
            <div className="flex items-center gap-2">
              <CheckCircle size={16} weight="fill" className="text-emerald-500 shrink-0" />
              <span>
                Artifacts generated in <strong>{exportResult.output_dir}</strong> (
                <code>graph.json</code>, <code>GRAPH_REPORT.md</code>, <code>graph.html</code>)
              </span>
            </div>
            <button
              onClick={() => setExportResult(null)}
              className="text-[10px] underline ml-2 cursor-pointer"
            >
              Dismiss
            </button>
          </div>
        )}

        {/* Content Body */}
        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          {loading ? (
            <div className="flex flex-col items-center justify-center py-16 gap-3">
              <SpinnerGap size={32} className="animate-spin text-indigo-500" />
              <p className="text-xs text-slate-400">Computing Louvain communities & graph topology...</p>
            </div>
          ) : !report ? (
            <div className="text-center py-12 text-sm text-slate-400">
              Failed to load topological report. Try re-syncing the vault.
            </div>
          ) : (
            <>
              {/* Metric Highlights */}
              <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                <div
                  className={`p-3.5 rounded-xl border ${
                    isLight ? 'bg-slate-50 border-slate-200' : 'bg-[#121726] border-[#1e2538]'
                  }`}
                >
                  <div className="text-[11px] text-slate-400">Nodes / Entities</div>
                  <div className="text-xl font-bold mt-0.5">{report.total_nodes}</div>
                  <div className="text-[10px] text-slate-500 mt-1">{report.total_edges} connections</div>
                </div>

                <div
                  className={`p-3.5 rounded-xl border ${
                    isLight ? 'bg-slate-50 border-slate-200' : 'bg-[#121726] border-[#1e2538]'
                  }`}
                >
                  <div className="text-[11px] text-slate-400">Modular Communities</div>
                  <div className="text-xl font-bold mt-0.5 text-indigo-400">
                    {report.communities.length}
                  </div>
                  <div className="text-[10px] text-slate-500 mt-1">LPA Modularity Clusters</div>
                </div>

                <div
                  className={`p-3.5 rounded-xl border ${
                    isLight ? 'bg-slate-50 border-slate-200' : 'bg-[#121726] border-[#1e2538]'
                  }`}
                >
                  <div className="text-[11px] text-slate-400">God Nodes (Hubs)</div>
                  <div className="text-xl font-bold mt-0.5 text-amber-400">
                    {report.god_nodes.length}
                  </div>
                  <div className="text-[10px] text-slate-500 mt-1">High-degree central hubs</div>
                </div>

                <div
                  className={`p-3.5 rounded-xl border ${
                    isLight ? 'bg-slate-50 border-slate-200' : 'bg-[#121726] border-[#1e2538]'
                  }`}
                >
                  <div className="text-[11px] text-slate-400">Cross Surprises</div>
                  <div className="text-xl font-bold mt-0.5 text-sky-400">
                    {report.surprises.length}
                  </div>
                  <div className="text-[10px] text-slate-500 mt-1">Cross-community edges</div>
                </div>
              </div>

              {/* Navigation Tabs */}
              <div className="flex border-b border-slate-700/50 gap-2">
                <button
                  onClick={() => setActiveTab('communities')}
                  className={`flex items-center gap-1.5 pb-2.5 text-xs font-semibold border-b-2 transition-all cursor-pointer ${
                    activeTab === 'communities'
                      ? 'border-indigo-500 text-indigo-400'
                      : 'border-transparent text-slate-400 hover:text-slate-200'
                  }`}
                >
                  <SquaresFour size={14} weight="bold" />
                  <span>Subsystems & Communities ({report.communities.length})</span>
                </button>

                <button
                  onClick={() => setActiveTab('gods')}
                  className={`flex items-center gap-1.5 pb-2.5 text-xs font-semibold border-b-2 transition-all cursor-pointer ${
                    activeTab === 'gods'
                      ? 'border-amber-500 text-amber-400'
                      : 'border-transparent text-slate-400 hover:text-slate-200'
                  }`}
                >
                  <Crown size={14} weight="bold" />
                  <span>God Nodes ({report.god_nodes.length})</span>
                </button>

                <button
                  onClick={() => setActiveTab('surprises')}
                  className={`flex items-center gap-1.5 pb-2.5 text-xs font-semibold border-b-2 transition-all cursor-pointer ${
                    activeTab === 'surprises'
                      ? 'border-sky-500 text-sky-400'
                      : 'border-transparent text-slate-400 hover:text-slate-200'
                  }`}
                >
                  <Lightning size={14} weight="bold" />
                  <span>Surprises & Cycles ({report.surprises.length + report.cycles.length})</span>
                </button>
              </div>

              {/* Tab: Communities */}
              {activeTab === 'communities' && (
                <div className="space-y-3">
                  <div className="text-xs text-slate-400 mb-2">
                    Communities discovered via topological clustering. Each group represents a cohesive subsystem:
                  </div>

                  <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                    {report.communities.map((c) => {
                      const color = COMMUNITY_COLORS[c.id % COMMUNITY_COLORS.length];
                      return (
                        <div
                          key={c.id}
                          className={`p-4 rounded-xl border transition-all ${
                            isLight
                              ? 'bg-slate-50/60 border-slate-200'
                              : 'bg-[#101422] border-[#1c2236]'
                          }`}
                        >
                          <div className="flex items-center justify-between mb-2">
                            <div className="flex items-center gap-2">
                              <span
                                className="w-3 h-3 rounded-full shrink-0"
                                style={{ backgroundColor: color }}
                              />
                              <h3 className="font-semibold text-sm">
                                C{c.id}: {c.name}
                              </h3>
                            </div>
                            <span className="text-[11px] px-2 py-0.5 rounded-full bg-slate-700/40 text-slate-300 font-mono">
                              {c.node_count} nodes
                            </span>
                          </div>

                          <div className="text-[11px] text-slate-400 mb-2">
                            Dominant Stack:{' '}
                            <code className="text-indigo-300">{c.dominant_type}</code>
                          </div>

                          <div className="text-[11px] text-slate-400">Key Nodes:</div>
                          <div className="flex flex-wrap gap-1.5 mt-1">
                            {c.key_nodes.map((kn) => (
                              <button
                                key={kn}
                                onClick={() => {
                                  onSelectNode?.(kn);
                                  onClose();
                                }}
                                className="text-[11px] px-2 py-0.5 rounded bg-slate-800/80 hover:bg-indigo-600 text-slate-300 hover:text-white transition-colors cursor-pointer"
                              >
                                {kn}
                              </button>
                            ))}
                          </div>
                        </div>
                      );
                    })}
                  </div>
                </div>
              )}

              {/* Tab: God Nodes */}
              {activeTab === 'gods' && (
                <div className="space-y-3">
                  <div className="text-xs text-slate-400 mb-2">
                    Components with the highest architectural degree centrality. Everything flows through or depends on these hubs:
                  </div>

                  <div className="overflow-x-auto rounded-xl border border-slate-700/50">
                    <table className="w-full text-left text-xs">
                      <thead
                        className={`border-b font-semibold uppercase text-[10px] tracking-wider ${
                          isLight
                            ? 'bg-slate-100 border-slate-200 text-slate-600'
                            : 'bg-[#14192a] border-[#1e2538] text-slate-400'
                        }`}
                      >
                        <tr>
                          <th className="p-3">Rank</th>
                          <th className="p-3">Node / Symbol</th>
                          <th className="p-3">Type</th>
                          <th className="p-3">Degree</th>
                          <th className="p-3">In / Out</th>
                          <th className="p-3">Community</th>
                          <th className="p-3">Architectural Role</th>
                        </tr>
                      </thead>
                      <tbody className="divide-y divide-slate-800/50 font-mono">
                        {report.god_nodes.map((g, idx) => (
                          <tr
                            key={g.id}
                            className={`hover:bg-slate-800/30 transition-colors ${
                              isLight ? 'text-slate-800' : 'text-slate-200'
                            }`}
                          >
                            <td className="p-3 text-amber-400 font-bold">#{idx + 1}</td>
                            <td className="p-3">
                              <button
                                onClick={() => {
                                  onSelectNode?.(g.id);
                                  onClose();
                                }}
                                className="font-semibold text-indigo-400 hover:underline cursor-pointer"
                              >
                                {g.title}
                              </button>
                            </td>
                            <td className="p-3 text-slate-400">{g.node_type}</td>
                            <td className="p-3 font-bold text-amber-400">{g.degree}</td>
                            <td className="p-3 text-slate-400">
                              {g.in_degree} / {g.out_degree}
                            </td>
                            <td className="p-3">
                              <span className="px-1.5 py-0.5 rounded text-[10px] bg-slate-800 text-slate-300">
                                C{g.community_id}
                              </span>
                            </td>
                            <td className="p-3 font-sans text-slate-300">{g.role}</td>
                          </tr>
                        ))}
                      </tbody>
                    </table>
                  </div>
                </div>
              )}

              {/* Tab: Surprises & Cycles */}
              {activeTab === 'surprises' && (
                <div className="space-y-4">
                  {report.cycles.length > 0 && (
                    <div>
                      <h4 className="text-xs font-bold text-rose-400 uppercase tracking-wider mb-2">
                        ⚠️ Mutual Dependency Cycles ({report.cycles.length})
                      </h4>
                      <div className="space-y-1.5">
                        {report.cycles.map((cyc, idx) => (
                          <div
                            key={idx}
                            className="p-2.5 rounded-lg border border-rose-900/40 bg-rose-950/20 text-xs font-mono text-rose-300"
                          >
                            {cyc.nodes.join(' <======> ')}
                          </div>
                        ))}
                      </div>
                    </div>
                  )}

                  <div>
                    <h4 className="text-xs font-bold text-slate-300 uppercase tracking-wider mb-2">
                      Cross-Community Couplings ({report.surprises.length})
                    </h4>
                    <p className="text-xs text-slate-400 mb-3">
                      Edges that cross distinct functional clusters. These represent critical interfaces or potential architectural leaks:
                    </p>

                    <div className="space-y-2">
                      {report.surprises.map((s, idx) => (
                        <div
                          key={idx}
                          className={`p-3 rounded-lg border text-xs flex flex-col gap-1 ${
                            isLight
                              ? 'bg-slate-50 border-slate-200 text-slate-700'
                              : 'bg-[#101422] border-[#1c2236] text-slate-300'
                          }`}
                        >
                          <div className="flex items-center gap-2 font-mono">
                            <span className="font-semibold text-indigo-400">
                              {s.source_title} (C{s.source_community})
                            </span>
                            <span className="text-slate-500">──[{s.label}]──►</span>
                            <span className="font-semibold text-emerald-400">
                              {s.target_title} (C{s.target_community})
                            </span>
                          </div>
                          <p className="text-[11px] text-slate-400 italic">{s.reason}</p>
                        </div>
                      ))}
                    </div>
                  </div>
                </div>
              )}
            </>
          )}
        </div>

        {/* Footer */}
        <div
          className={`flex items-center justify-between px-6 py-3 border-t text-xs ${
            isLight
              ? 'border-slate-200 bg-slate-50 text-slate-500'
              : 'border-[#1e2538] bg-[#0c101b] text-slate-400'
          }`}
        >
          <span>
            Computed in pure Rust with 0 external dependencies • Full parity with Graphify
          </span>
          <button
            onClick={loadReport}
            className="flex items-center gap-1 hover:text-indigo-400 transition-colors cursor-pointer"
          >
            <ArrowsClockwise size={13} />
            <span>Recalculate</span>
          </button>
        </div>
      </div>
    </div>
  );
};
