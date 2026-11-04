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

import { getLanguageStyle, LANGUAGE_COLORS } from './languageStyle';

export { getLanguageStyle };

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
      } ${isLight ? 'bg-white border-slate-200 shadow-slate-200/50' : 'bg-[#0d0f14] border-[#1e222c]'}`}
    >
      {/* Header */}
      <div className={`p-3.5 border-b flex items-start justify-between gap-3 transition-colors ${
        isLight ? 'bg-slate-50/70 border-slate-200' : 'bg-[#0d0f14] border-[#1e222c]'
      }`}>
        <div className="flex items-start gap-2.5 min-w-0">
          <div className={`p-2 rounded-lg ${style.bg} border ${style.border} ${style.text} shrink-0 mt-0.5`}>
            <FileCode size={18} weight="duotone" />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <h2 className={`text-sm font-semibold truncate ${isLight ? 'text-slate-900' : 'text-[#edeef0]'}`} title={context?.name || fileId}>
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
            <p className="text-[11px] font-mono text-[#8a919e] truncate mt-0.5" title={fileId}>
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
                : 'bg-[#161922] hover:bg-[#1a1f2c] border-[#1e222c] text-[#d8dae0]'
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
                : 'text-[#8a919e] hover:text-[#edeef0] hover:bg-[#161922]'
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
                : 'text-[#8a919e] hover:text-[#edeef0] hover:bg-[#161922]'
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
            isLight ? 'text-slate-400' : 'text-[#8a919e]'
          }`}
        >
          <Cpu size={28} className="animate-spin text-indigo-500 mb-3" />
          <span
            className={`text-xs font-medium ${
              isLight ? 'text-slate-700' : 'text-[#d8dae0]'
            }`}
          >
            Analyzing AST & Dependencies...
          </span>
        </div>
      ) : !context ? (
        <div
          className={`flex-1 flex flex-col items-center justify-center p-8 text-center ${
            isLight ? 'text-slate-400' : 'text-[#8a919e]'
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
                : 'border-[#1b202c] bg-[#0d0f14] text-[#8a919e]'
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
                : 'border-[#1e222c] bg-[#11141c]'
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
                    : 'border-indigo-500 text-[#edeef0]'
                  : isLight
                  ? 'border-transparent text-slate-500 hover:text-slate-800'
                  : 'border-transparent text-[#8a919e] hover:text-[#d8dae0]'
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
                    : 'border-indigo-500 text-[#edeef0]'
                  : isLight
                  ? 'border-transparent text-slate-500 hover:text-slate-800'
                  : 'border-transparent text-[#8a919e] hover:text-[#d8dae0]'
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
                    : 'border-indigo-500 text-[#edeef0]'
                  : isLight
                  ? 'border-transparent text-slate-500 hover:text-slate-800'
                  : 'border-transparent text-[#8a919e] hover:text-[#d8dae0]'
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
                    isLight ? 'text-slate-500' : 'text-[#8a919e]'
                  }`}
                >
                  Declared Symbols (Functions, Classes, Structs)
                </div>
                {context.symbols.length === 0 ? (
                  <p className={`text-xs italic ${isLight ? 'text-slate-400' : 'text-[#8a919e]'}`}>
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
                          className={`p-2.5 rounded-lg border transition-all cursor-pointer flex flex-col gap-1.5 ${
                            isCurrent
                              ? isLight
                                ? 'bg-indigo-50 border-indigo-300 text-indigo-950 font-medium shadow-xs'
                                : 'bg-indigo-600/10 border-indigo-500/50 text-[#edeef0]'
                              : isLight
                              ? 'bg-slate-50/70 border-slate-200 hover:bg-slate-100 text-slate-800'
                              : 'bg-[#161922] border-[#1e222c] hover:bg-[#161922] text-[#d8dae0]'
                          }`}
                        >
                          <div className="flex items-center justify-between">
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
                                isLight ? 'text-slate-500' : 'text-[#8a919e]'
                              }`}
                            >
                              <span>
                                L{sym.start_line}–{sym.end_line}
                              </span>
                              <span
                                className={`px-1.5 py-0.5 rounded border ${
                                  isLight
                                    ? 'bg-white border-slate-200 text-slate-600 shadow-2xs'
                                    : 'bg-[#1a1f2c] border-[#262c3e] text-[#8a919e]'
                                }`}
                              >
                                ~{sym.token_estimate} tok
                              </span>
                            </div>
                          </div>
                          {((sym.calls && sym.calls.length > 0) ||
                            (sym.called_by && sym.called_by.length > 0)) && (
                            <div className="flex flex-wrap items-center gap-1.5 pt-0.5 text-[10px] font-mono">
                              {sym.calls && sym.calls.length > 0 && (
                                <span
                                  className={`px-1.5 py-0.5 rounded ${
                                    isLight
                                      ? 'bg-sky-100/80 text-sky-800'
                                      : 'bg-sky-500/10 text-sky-400'
                                  }`}
                                  title={`Calls: ${sym.calls.join(', ')}`}
                                >
                                  → calls {sym.calls.slice(0, 3).join(', ')}
                                  {sym.calls.length > 3 ? ` +${sym.calls.length - 3}` : ''}
                                </span>
                              )}
                              {sym.called_by && sym.called_by.length > 0 && (
                                <span
                                  className={`px-1.5 py-0.5 rounded ${
                                    isLight
                                      ? 'bg-violet-100/80 text-violet-800'
                                      : 'bg-violet-500/10 text-violet-400'
                                  }`}
                                  title={`Called by: ${sym.called_by.join(', ')}`}
                                >
                                  ← called by {sym.called_by.slice(0, 2).join(', ')}
                                  {sym.called_by.length > 2 ? ` +${sym.called_by.length - 2}` : ''}
                                </span>
                              )}
                            </div>
                          )}
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
                      isLight ? 'text-sky-600' : 'text-[#5fa8d9]'
                    }`}
                  >
                    <ArrowUpRight size={13} weight="bold" />
                    <span>Imported Files ({context.imported_files.length})</span>
                  </div>
                  {context.imported_files.length === 0 ? (
                    <p className={`text-xs italic ${isLight ? 'text-slate-400' : 'text-[#8a919e]'}`}>
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
                              : 'bg-[#161922] border-[#1e222c] hover:border-sky-500/40 hover:bg-[#161922] text-[#d8dae0]'
                          }`}
                        >
                          <span className="truncate">{imp}</span>
                          <ArrowSquareOut
                            size={12}
                            className={`shrink-0 ${
                              isLight
                                ? 'text-slate-400 group-hover:text-sky-600'
                                : 'text-[#8a919e] group-hover:text-sky-400'
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
                    <p className={`text-xs italic ${isLight ? 'text-slate-400' : 'text-[#8a919e]'}`}>
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
                              : 'bg-[#161922] border-[#1e222c] text-[#8a919e]'
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
                    <p className={`text-xs italic ${isLight ? 'text-slate-400' : 'text-[#8a919e]'}`}>
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
                              : 'bg-[#161922] border-[#1e222c] hover:border-violet-500/40 hover:bg-[#161922] text-[#d8dae0]'
                          }`}
                        >
                          <span className="truncate">{dep}</span>
                          <ArrowSquareOut
                            size={12}
                            className={`shrink-0 ${
                              isLight
                                ? 'text-slate-400 group-hover:text-violet-600'
                                : 'text-[#8a919e] group-hover:text-violet-400'
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
                      isLight ? 'text-slate-500' : 'text-[#8a919e]'
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
                          : 'text-[#8a919e] hover:text-[#edeef0]'
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
                      isLight ? 'text-slate-400' : 'text-[#8a919e]'
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
                        : 'bg-[#0d0f14] border-[#1b202c] text-[#d8dae0]'
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
                        : 'bg-[#161922] border-[#1e222c] text-[#8a919e]'
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
