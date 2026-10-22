import React, { useState, useEffect } from 'react';
import {
  X,
  Code,
  FileCode,
  BracketsCurly,
  ArrowSquareOut,
  Copy,
  Check,
  Cpu,
  GitBranch,
  ArrowUpRight,
  ArrowDownLeft,
  Package,
  ArrowsOut,
  ArrowsIn,
} from '@phosphor-icons/react';
import { getFileContext, readCodeChunk } from '../services/api';
import type { FileContext, CodeChunkHit, SymbolInfo } from '../types';

export const LANGUAGE_COLORS: Record<string, { bg: string; text: string; border: string; hex: string }> = {
  typescript: { bg: 'bg-sky-500/10', text: 'text-sky-400', border: 'border-sky-500/30', hex: '#38bdf8' },
  javascript: { bg: 'bg-amber-500/10', text: 'text-amber-400', border: 'border-amber-500/30', hex: '#facc15' },
  rust: { bg: 'bg-orange-500/10', text: 'text-orange-400', border: 'border-orange-500/30', hex: '#f97316' },
  python: { bg: 'bg-emerald-500/10', text: 'text-emerald-400', border: 'border-emerald-500/30', hex: '#10b981' },
  go: { bg: 'bg-cyan-500/10', text: 'text-cyan-400', border: 'border-cyan-500/30', hex: '#06b6d4' },
  c: { bg: 'bg-blue-500/10', text: 'text-blue-400', border: 'border-blue-500/30', hex: '#3b82f6' },
  cpp: { bg: 'bg-blue-600/10', text: 'text-blue-400', border: 'border-blue-600/30', hex: '#2563eb' },
  csharp: { bg: 'bg-purple-500/10', text: 'text-purple-400', border: 'border-purple-500/30', hex: '#a855f7' },
  java: { bg: 'bg-amber-600/10', text: 'text-amber-400', border: 'border-amber-600/30', hex: '#ea580c' },
  kotlin: { bg: 'bg-violet-500/10', text: 'text-violet-400', border: 'border-violet-500/30', hex: '#8b5cf6' },
  php: { bg: 'bg-indigo-500/10', text: 'text-indigo-400', border: 'border-indigo-500/30', hex: '#6366f1' },
  ruby: { bg: 'bg-rose-500/10', text: 'text-rose-400', border: 'border-rose-500/30', hex: '#e11d48' },
  swift: { bg: 'bg-pink-500/10', text: 'text-pink-400', border: 'border-pink-500/30', hex: '#f43f5e' },
  css: { bg: 'bg-pink-600/10', text: 'text-pink-400', border: 'border-pink-600/30', hex: '#ec4899' },
  shell: { bg: 'bg-teal-500/10', text: 'text-teal-400', border: 'border-teal-500/30', hex: '#14b8a6' },
  generic: { bg: 'bg-slate-500/10', text: 'text-slate-400', border: 'border-slate-500/30', hex: '#94a3b8' },
};

export function getLanguageStyle(lang: string) {
  const key = lang.toLowerCase();
  return LANGUAGE_COLORS[key] || LANGUAGE_COLORS.generic;
}

interface CodeInspectorProps {
  fileId: string | null;
  onClose: () => void;
  onNavigateFile: (fileId: string) => void;
  initialSymbol?: string | null;
  theme?: 'dark' | 'light';
}

export const CodeInspector: React.FC<CodeInspectorProps> = ({
  fileId,
  onClose,
  onNavigateFile,
  initialSymbol,
  theme = 'dark',
}) => {
  const isLight = theme === 'light';
  const [context, setContext] = useState<FileContext | null>(null);
  const [loading, setLoading] = useState<boolean>(true);
  const [selectedSymbol, setSelectedSymbol] = useState<SymbolInfo | null>(null);
  const [activeChunk, setActiveChunk] = useState<CodeChunkHit | null>(null);
  const [loadingChunk, setLoadingChunk] = useState<boolean>(false);
  const [copiedLink, setCopiedLink] = useState<boolean>(false);
  const [copiedCode, setCopiedCode] = useState<boolean>(false);
  const [isFullscreen, setIsFullscreen] = useState<boolean>(false);
  const [activeTab, setActiveTab] = useState<'symbols' | 'deps' | 'code'>('symbols');

  useEffect(() => {
    if (!fileId) {
      setContext(null);
      setSelectedSymbol(null);
      setActiveChunk(null);
      return;
    }

    setLoading(true);
    getFileContext(fileId)
      .then(async (ctx) => {
        setContext(ctx);
        if (ctx && ctx.symbols.length > 0) {
          const targetSym = initialSymbol
            ? ctx.symbols.find((s) => s.name.toLowerCase() === initialSymbol.toLowerCase()) || ctx.symbols[0]
            : ctx.symbols[0];
          setSelectedSymbol(targetSym);
          if (targetSym) {
            setLoadingChunk(true);
            const chunk = await readCodeChunk(fileId, targetSym.name);
            setActiveChunk(chunk);
            setLoadingChunk(false);
          }
        } else {
          setSelectedSymbol(null);
          setActiveChunk(null);
        }
      })
      .catch((err) => {
        console.error('Error fetching file context:', err);
      })
      .finally(() => {
        setLoading(false);
      });
  }, [fileId, initialSymbol]);

  const handleSelectSymbol = async (sym: SymbolInfo) => {
    if (!fileId) return;
    setSelectedSymbol(sym);
    setLoadingChunk(true);
    setActiveTab('code');
    try {
      const chunk = await readCodeChunk(fileId, sym.name);
      setActiveChunk(chunk);
    } catch (err) {
      console.error('Error loading chunk:', err);
    } finally {
      setLoadingChunk(false);
    }
  };

  const copyDeepLink = () => {
    if (!context) return;
    const link = selectedSymbol
      ? `[[${context.name}#${selectedSymbol.name}]]`
      : `[[${context.name}]]`;
    navigator.clipboard.writeText(link);
    setCopiedLink(true);
    setTimeout(() => setCopiedLink(false), 2000);
  };

  const copyChunkCode = () => {
    if (!activeChunk) return;
    navigator.clipboard.writeText(activeChunk.content);
    setCopiedCode(true);
    setTimeout(() => setCopiedCode(false), 2000);
  };

  if (!fileId) return null;

  const style = context ? getLanguageStyle(context.language) : LANGUAGE_COLORS.generic;

  return (
    <div
      className={`absolute top-0 bottom-0 z-30 flex flex-col border-l shadow-2xl backdrop-blur-md transition-all duration-200 ${
        isFullscreen
          ? 'left-0 right-0 w-full'
          : 'right-0 w-full md:w-[480px] lg:w-[540px]'
      } ${isLight ? 'bg-white border-slate-200 shadow-slate-200/50' : 'bg-[#0b0d13] border-[#181c26]'}`}
    >
      {/* Header */}
      <div className={`p-3.5 border-b flex items-start justify-between gap-3 transition-colors ${
        isLight ? 'bg-slate-50/70 border-slate-200' : 'bg-[#0e1118] border-[#181c26]'
      }`}>
        <div className="flex items-start gap-2.5 min-w-0">
          <div className={`p-2 rounded-lg ${style.bg} border ${style.border} ${style.text} shrink-0 mt-0.5`}>
            <FileCode size={18} weight="duotone" />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <h2 className={`text-sm font-semibold truncate ${isLight ? 'text-slate-900' : 'text-[#f8fafc]'}`} title={context?.name || fileId}>
                {context?.name || fileId}
              </h2>
              {context && (
                <span
                  className={`px-2 py-0.5 text-[10px] font-mono font-medium rounded-full ${style.bg} ${style.text} border ${style.border}`}
                >
                  {context.language}
                </span>
              )}
            </div>
            <p className="text-[11px] font-mono text-[#64748b] truncate mt-0.5" title={fileId}>
              {fileId}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-1.5 shrink-0">
          <button
            type="button"
            onClick={copyDeepLink}
            className={`flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs font-medium border transition-all cursor-pointer ${
              isLight
                ? 'bg-white hover:bg-slate-100 border-slate-200 text-slate-700'
                : 'bg-[#131722] hover:bg-[#181d2a] border-[#202636] text-[#cbd5e1]'
            }`}
            title="Copy Deep Link wikilink for this file/symbol"
          >
            {copiedLink ? (
              <>
                <Check size={12} weight="bold" className="text-emerald-500" />
                <span className="text-emerald-500 text-[11px]">Copied!</span>
              </>
            ) : (
              <>
                <Copy size={12} />
                <span className="text-[11px]">Deep Link</span>
              </>
            )}
          </button>
          <button
            type="button"
            onClick={() => setIsFullscreen(!isFullscreen)}
            className={`flex items-center justify-center w-7 h-7 rounded-lg transition-colors cursor-pointer ${
              isLight
                ? 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'
                : 'text-[#64748b] hover:text-[#f8fafc] hover:bg-[#161a24]'
            }`}
            title={isFullscreen ? 'Exit Fullscreen' : 'Focus Mode (Fullscreen)'}
          >
            {isFullscreen ? <ArrowsIn size={14} /> : <ArrowsOut size={14} />}
          </button>
          <button
            type="button"
            onClick={onClose}
            className={`flex items-center justify-center w-7 h-7 rounded-lg transition-colors cursor-pointer ${
              isLight
                ? 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'
                : 'text-[#64748b] hover:text-[#f8fafc] hover:bg-[#161a24]'
            }`}
            title="Close"
          >
            <X size={15} />
          </button>
        </div>
      </div>

      {loading ? (
        <div
          className={`flex-1 flex flex-col items-center justify-center p-8 text-center ${
            isLight ? 'text-slate-400' : 'text-[#64748b]'
          }`}
        >
          <Cpu size={28} className="animate-spin text-indigo-500 mb-3" />
          <span
            className={`text-xs font-medium ${
              isLight ? 'text-slate-700' : 'text-[#cbd5e1]'
            }`}
          >
            Analyzing AST & Dependencies...
          </span>
        </div>
      ) : !context ? (
        <div
          className={`flex-1 flex flex-col items-center justify-center p-8 text-center ${
            isLight ? 'text-slate-400' : 'text-[#64748b]'
          }`}
        >
          <span className="text-xs">File not found in index.</span>
        </div>
      ) : (
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Metadata telemetry pill bar */}
          <div
            className={`px-4 py-2 border-b flex items-center justify-between text-[11px] font-mono ${
              isLight
                ? 'border-slate-200 bg-slate-50 text-slate-500'
                : 'border-[#1b202c] bg-[#0e1118] text-[#64748b]'
            }`}
          >
            <div className="flex items-center gap-3">
              <span>{(context.size_bytes / 1024).toFixed(1)} KB</span>
              <span>•</span>
              <span>~{context.token_estimate} tokens</span>
            </div>
            <div className="flex items-center gap-2">
              <span className={isLight ? 'text-emerald-600 font-medium' : 'text-emerald-400'}>
                AST Indexed
              </span>
            </div>
          </div>

          {/* Navigation Tabs */}
          <div
            className={`flex items-center border-b px-3 ${
              isLight
                ? 'border-slate-200 bg-white'
                : 'border-[#1f2433] bg-[#11141c]'
            }`}
            role="tablist"
          >
            <button
              type="button"
              role="tab"
              aria-selected={activeTab === 'symbols'}
              onClick={() => setActiveTab('symbols')}
              className={`flex items-center gap-1.5 px-3 py-2 text-xs font-medium border-b-2 transition-all cursor-pointer ${
                activeTab === 'symbols'
                  ? isLight
                    ? 'border-indigo-600 text-indigo-600 font-semibold'
                    : 'border-indigo-500 text-[#f8fafc]'
                  : isLight
                  ? 'border-transparent text-slate-500 hover:text-slate-800'
                  : 'border-transparent text-[#64748b] hover:text-[#cbd5e1]'
              }`}
            >
              <BracketsCurly size={14} />
              <span>Symbols ({context.symbols.length})</span>
            </button>

            <button
              type="button"
              role="tab"
              aria-selected={activeTab === 'deps'}
              onClick={() => setActiveTab('deps')}
              className={`flex items-center gap-1.5 px-3 py-2 text-xs font-medium border-b-2 transition-all cursor-pointer ${
                activeTab === 'deps'
                  ? isLight
                    ? 'border-indigo-600 text-indigo-600 font-semibold'
                    : 'border-indigo-500 text-[#f8fafc]'
                  : isLight
                  ? 'border-transparent text-slate-500 hover:text-slate-800'
                  : 'border-transparent text-[#64748b] hover:text-[#cbd5e1]'
              }`}
            >
              <GitBranch size={14} />
              <span>
                Graph ({context.imported_files.length + context.dependent_files.length})
              </span>
            </button>

            <button
              type="button"
              role="tab"
              aria-selected={activeTab === 'code'}
              onClick={() => setActiveTab('code')}
              className={`flex items-center gap-1.5 px-3 py-2 text-xs font-medium border-b-2 transition-all cursor-pointer ${
                activeTab === 'code'
                  ? isLight
                    ? 'border-indigo-600 text-indigo-600 font-semibold'
                    : 'border-indigo-500 text-[#f8fafc]'
                  : isLight
                  ? 'border-transparent text-slate-500 hover:text-slate-800'
                  : 'border-transparent text-[#64748b] hover:text-[#cbd5e1]'
              }`}
            >
              <Code size={14} />
              <span>Surgical Viewer</span>
            </button>
          </div>

          {/* Tab Content */}
          <div className="flex-1 overflow-y-auto p-4 space-y-4">
            {activeTab === 'symbols' && (
              <div className="space-y-3">
                <div
                  className={`text-[11px] font-medium uppercase tracking-wider ${
                    isLight ? 'text-slate-500' : 'text-[#64748b]'
                  }`}
                >
                  Declared Symbols (Functions, Classes, Structs)
                </div>
                {context.symbols.length === 0 ? (
                  <p className={`text-xs italic ${isLight ? 'text-slate-400' : 'text-[#64748b]'}`}>
                    No isolated top-level symbols detected in this file.
                  </p>
                ) : (
                  <div className="grid grid-cols-1 gap-1.5">
                    {context.symbols.map((sym) => {
                      const isCurrent = selectedSymbol?.name === sym.name;
                      return (
                        <div
                          key={sym.name}
                          onClick={() => handleSelectSymbol(sym)}
                          className={`p-2.5 rounded-lg border transition-all cursor-pointer flex items-center justify-between ${
                            isCurrent
                              ? isLight
                                ? 'bg-indigo-50 border-indigo-300 text-indigo-950 font-medium shadow-xs'
                                : 'bg-indigo-600/10 border-indigo-500/50 text-[#f8fafc]'
                              : isLight
                              ? 'bg-slate-50/70 border-slate-200 hover:bg-slate-100 text-slate-800'
                              : 'bg-[#12151f] border-[#1f2434] hover:bg-[#161a26] text-[#cbd5e1]'
                          }`}
                        >
                          <div className="flex items-center gap-2 min-w-0">
                            <span
                              className={`w-1.5 h-1.5 rounded-full shrink-0 ${
                                isLight ? 'bg-indigo-600' : 'bg-indigo-400'
                              }`}
                            />
                            <span className="text-xs font-mono font-medium truncate">
                              {sym.name}
                            </span>
                          </div>
                          <div
                            className={`flex items-center gap-2 shrink-0 text-[10px] font-mono ${
                              isLight ? 'text-slate-500' : 'text-[#64748b]'
                            }`}
                          >
                            <span>
                              L{sym.start_line}–{sym.end_line}
                            </span>
                            <span
                              className={`px-1.5 py-0.5 rounded border ${
                                isLight
                                  ? 'bg-white border-slate-200 text-slate-600 shadow-2xs'
                                  : 'bg-[#1a1f2c] border-[#262c3e] text-[#64748b]'
                              }`}
                            >
                              ~{sym.token_estimate} tok
                            </span>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                )}
              </div>
            )}

            {activeTab === 'deps' && (
              <div className="space-y-5">
                {/* Internal Imports */}
                <div>
                  <div
                    className={`flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider mb-2 ${
                      isLight ? 'text-sky-600' : 'text-[#38bdf8]'
                    }`}
                  >
                    <ArrowUpRight size={13} weight="bold" />
                    <span>Imported Files ({context.imported_files.length})</span>
                  </div>
                  {context.imported_files.length === 0 ? (
                    <p className={`text-xs italic ${isLight ? 'text-slate-400' : 'text-[#64748b]'}`}>
                      Does not import other local project files.
                    </p>
                  ) : (
                    <div className="space-y-1">
                      {context.imported_files.map((imp) => (
                        <div
                          key={imp}
                          onClick={() => onNavigateFile(imp)}
                          className={`flex items-center justify-between p-2 rounded-lg border text-xs font-mono transition-all cursor-pointer group ${
                            isLight
                              ? 'bg-slate-50 border-slate-200 hover:border-sky-400 hover:bg-sky-50/50 text-slate-800'
                              : 'bg-[#12151f] border-[#1f2434] hover:border-sky-500/40 hover:bg-[#161a26] text-[#cbd5e1]'
                          }`}
                        >
                          <span className="truncate">{imp}</span>
                          <ArrowSquareOut
                            size={12}
                            className={`shrink-0 ${
                              isLight
                                ? 'text-slate-400 group-hover:text-sky-600'
                                : 'text-[#64748b] group-hover:text-sky-400'
                            }`}
                          />
                        </div>
                      ))}
                    </div>
                  )}
                </div>

                {/* External Packages */}
                <div>
                  <div
                    className={`flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider mb-2 ${
                      isLight ? 'text-emerald-700' : 'text-emerald-400'
                    }`}
                  >
                    <Package size={13} weight="bold" />
                    <span>External Packages ({context.external_packages.length})</span>
                  </div>
                  {context.external_packages.length === 0 ? (
                    <p className={`text-xs italic ${isLight ? 'text-slate-400' : 'text-[#64748b]'}`}>
                      No external dependencies declared.
                    </p>
                  ) : (
                    <div className="flex flex-wrap gap-1.5">
                      {context.external_packages.map((pkg) => (
                        <span
                          key={pkg}
                          className={`px-2 py-1 rounded-md border text-[11px] font-mono ${
                            isLight
                              ? 'bg-slate-50 border-slate-200 text-slate-700'
                              : 'bg-[#12151f] border-[#1f2434] text-[#94a3b8]'
                          }`}
                        >
                          {pkg}
                        </span>
                      ))}
                    </div>
                  )}
                </div>

                {/* Imported By (Dependents) */}
                <div>
                  <div
                    className={`flex items-center gap-1.5 text-[11px] font-semibold uppercase tracking-wider mb-2 ${
                      isLight ? 'text-violet-700' : 'text-violet-400'
                    }`}
                  >
                    <ArrowDownLeft size={13} weight="bold" />
                    <span>Imported By ({context.dependent_files.length})</span>
                  </div>
                  {context.dependent_files.length === 0 ? (
                    <p className={`text-xs italic ${isLight ? 'text-slate-400' : 'text-[#64748b]'}`}>
                      No local files import this file directly.
                    </p>
                  ) : (
                    <div className="space-y-1">
                      {context.dependent_files.map((dep) => (
                        <div
                          key={dep}
                          onClick={() => onNavigateFile(dep)}
                          className={`flex items-center justify-between p-2 rounded-lg border text-xs font-mono transition-all cursor-pointer group ${
                            isLight
                              ? 'bg-slate-50 border-slate-200 hover:border-violet-400 hover:bg-violet-50/50 text-slate-800'
                              : 'bg-[#12151f] border-[#1f2434] hover:border-violet-500/40 hover:bg-[#161a26] text-[#cbd5e1]'
                          }`}
                        >
                          <span className="truncate">{dep}</span>
                          <ArrowSquareOut
                            size={12}
                            className={`shrink-0 ${
                              isLight
                                ? 'text-slate-400 group-hover:text-violet-600'
                                : 'text-[#64748b] group-hover:text-violet-400'
                            }`}
                          />
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              </div>
            )}

            {activeTab === 'code' && (
              <div className="space-y-3">
                <div className="flex items-center justify-between">
                  <div
                    className={`text-[11px] font-medium uppercase tracking-wider ${
                      isLight ? 'text-slate-500' : 'text-[#64748b]'
                    }`}
                  >
                    {activeChunk?.symbol_name
                      ? `Symbol: ${activeChunk.symbol_name} (L${activeChunk.start_line}-${activeChunk.end_line})`
                      : 'File Excerpt'}
                  </div>
                  {activeChunk && (
                    <button
                      type="button"
                      onClick={copyChunkCode}
                      className={`flex items-center gap-1 text-[11px] font-mono cursor-pointer transition-colors ${
                        isLight
                          ? 'text-slate-500 hover:text-slate-900'
                          : 'text-[#64748b] hover:text-[#f8fafc]'
                      }`}
                    >
                      {copiedCode ? (
                        <>
                          <Check size={11} className="text-emerald-500" />
                          <span className="text-emerald-600 font-medium">Copied</span>
                        </>
                      ) : (
                        <>
                          <Copy size={11} />
                          <span>Copy</span>
                        </>
                      )}
                    </button>
                  )}
                </div>

                {loadingChunk ? (
                  <div
                    className={`py-12 text-center text-xs ${
                      isLight ? 'text-slate-400' : 'text-[#64748b]'
                    }`}
                  >
                    <Cpu size={20} className="animate-spin text-indigo-500 mx-auto mb-2" />
                    Loading surgical excerpt...
                  </div>
                ) : activeChunk ? (
                  <div
                    className={`relative rounded-xl border overflow-hidden ${
                      isLight
                        ? 'bg-slate-900 border-slate-700 text-slate-100 shadow-sm'
                        : 'bg-[#090b10] border-[#1b202c] text-[#e2e8f0]'
                    }`}
                  >
                    <pre className="p-3 text-[11.5px] font-mono leading-relaxed overflow-x-auto whitespace-pre">
                      {activeChunk.content}
                    </pre>
                  </div>
                ) : (
                  <div
                    className={`p-4 rounded-lg border text-xs text-center ${
                      isLight
                        ? 'bg-slate-50 border-slate-200 text-slate-500'
                        : 'bg-[#12151f] border-[#1f2434] text-[#64748b]'
                    }`}
                  >
                    Select a symbol in the "Symbols" tab to inspect its isolated excerpt.
                  </div>
                )}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
};
