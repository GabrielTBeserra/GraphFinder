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
}) => {
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
    const onKey = (e: KeyboardEvent) => {
      if (e.key === 'Escape') onClose();
    };
    window.addEventListener('keydown', onKey);
    return () => window.removeEventListener('keydown', onKey);
  }, [onClose]);

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
      } ${'bg-surface border-line shadow-sm'}`}
    >
      {/* Header */}
      <div className={`p-3.5 border-b flex items-start justify-between gap-3 transition-colors ${
        'bg-surface/70 border-line'
      }`}>
        <div className="flex items-start gap-2.5 min-w-0">
          <div className={`p-2 rounded-lg ${style.bg} border ${style.border} ${style.text} shrink-0 mt-0.5`}>
            <FileCode size={18} weight="duotone" />
          </div>
          <div className="min-w-0">
            <div className="flex items-center gap-2 flex-wrap">
              <h2 className={`text-sm font-semibold truncate ${'text-ink'}`} title={context?.name || fileId}>
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
            <p className="text-[11px] font-mono text-muted truncate mt-0.5" title={fileId}>
              {fileId}
            </p>
          </div>
        </div>

        <div className="flex items-center gap-1.5 shrink-0">
          <button
            type="button"
            onClick={copyDeepLink}
            className={`flex items-center gap-1 px-2.5 py-1 rounded-lg text-xs font-medium border transition-all cursor-pointer ${
              'bg-raise hover:bg-raise border-line text-ink'
            }`}
            title="Copy Deep Link wikilink for this file/symbol" aria-label="Copy Deep Link wikilink for this file/symbol"
          >
            {copiedLink ? (
              <>
                <Check size={12} weight="bold" className="text-good" />
                <span className="text-good text-[11px]">Copied!</span>
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
              'text-muted hover:text-ink hover:bg-raise'
            }`}
            title={isFullscreen ? 'Exit Fullscreen' : 'Focus Mode (Fullscreen)'} aria-label={isFullscreen ? 'Exit fullscreen' : 'Enter fullscreen'}
          >
            {isFullscreen ? <ArrowsIn size={14} /> : <ArrowsOut size={14} />}
          </button>
          <button
            type="button"
            onClick={onClose}
            className={`flex items-center justify-center w-7 h-7 rounded-lg transition-colors cursor-pointer ${
              'text-muted hover:text-ink hover:bg-raise'
            }`}
            title="Close" aria-label="Close"
          >
            <X size={15} />
          </button>
        </div>
      </div>

      {loading ? (
        <div
          className={`flex-1 flex flex-col items-center justify-center p-8 text-center ${
            'text-muted'
          }`}
        >
          <Cpu size={28} className="animate-spin text-accent mb-3" />
          <span
            className={`text-xs font-medium ${
              'text-ink'
            }`}
          >
            Analyzing AST & Dependencies...
          </span>
        </div>
      ) : !context ? (
        <div
          className={`flex-1 flex flex-col items-center justify-center p-8 text-center ${
            'text-muted'
          }`}
        >
          <span className="text-xs">File not found in index.</span>
        </div>
      ) : (
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Metadata telemetry pill bar */}
          <div
            className={`px-4 py-2 border-b flex items-center justify-between text-[11px] font-mono ${
              'border-line bg-surface text-muted'
            }`}
          >
            <div className="flex items-center gap-3">
              <span>{(context.size_bytes / 1024).toFixed(1)} KB</span>
              <span>•</span>
              <span>~{context.token_estimate} tokens</span>
            </div>
            <div className="flex items-center gap-2">
              <span className={'text-good font-medium'}>
                AST Indexed
              </span>
            </div>
          </div>

          {/* Navigation Tabs */}
          <div
            className={`flex items-center border-b px-3 ${
              'border-line bg-raise'
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
                  ? 'border-accent text-ink'
                  : 'border-transparent text-muted hover:text-ink'
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
                  ? 'border-accent text-ink'
                  : 'border-transparent text-muted hover:text-ink'
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
                  ? 'border-accent text-ink'
                  : 'border-transparent text-muted hover:text-ink'
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
                    'text-muted'
                  }`}
                >
                  Declared Symbols (Functions, Classes, Structs)
                </div>
                {context.symbols.length === 0 ? (
                  <p className={`text-xs italic ${'text-muted'}`}>
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
                              ? 'bg-accent/10 border-accent/50 text-ink'
                              : 'bg-raise border-line hover:bg-raise text-ink'
                          }`}
                        >
                          <div className="flex items-center justify-between">
                            <div className="flex items-center gap-2 min-w-0">
                              <span
                                className={`w-1.5 h-1.5 rounded-full shrink-0 ${
                                  'bg-accent'
                                }`}
                              />
                              <span className="text-xs font-mono font-medium truncate">
                                {sym.name}
                              </span>
                            </div>
                            <div
                              className={`flex items-center gap-2 shrink-0 text-[10px] font-mono ${
                                'text-muted'
                              }`}
                            >
                              <span>
                                L{sym.start_line}–{sym.end_line}
                              </span>
                              <span
                                className={`px-1.5 py-0.5 rounded border ${
                                  'bg-raise border-line-strong text-muted'
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
                                    'bg-sky-500/10 text-sky-400'
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
                                    'bg-accent-soft text-accent'
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
                      'text-sky-500'
                    }`}
                  >
                    <ArrowUpRight size={13} weight="bold" />
                    <span>Imported Files ({context.imported_files.length})</span>
                  </div>
                  {context.imported_files.length === 0 ? (
                    <p className={`text-xs italic ${'text-muted'}`}>
                      Does not import other local project files.
                    </p>
                  ) : (
                    <div className="space-y-1">
                      {context.imported_files.map((imp) => (
                        <div
                          key={imp}
                          onClick={() => onNavigateFile(imp)}
                          className={`flex items-center justify-between p-2 rounded-lg border text-xs font-mono transition-all cursor-pointer group ${
                            'bg-raise border-line hover:border-sky-500/40 hover:bg-raise text-ink'
                          }`}
                        >
                          <span className="truncate">{imp}</span>
                          <ArrowSquareOut
                            size={12}
                            className={`shrink-0 ${
                              'text-muted group-hover:text-sky-400'
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
                      'text-good'
                    }`}
                  >
                    <Package size={13} weight="bold" />
                    <span>External Packages ({context.external_packages.length})</span>
                  </div>
                  {context.external_packages.length === 0 ? (
                    <p className={`text-xs italic ${'text-muted'}`}>
                      No external dependencies declared.
                    </p>
                  ) : (
                    <div className="flex flex-wrap gap-1.5">
                      {context.external_packages.map((pkg) => (
                        <span
                          key={pkg}
                          className={`px-2 py-1 rounded-md border text-[11px] font-mono ${
                            'bg-raise border-line text-muted'
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
                      'text-purple-400'
                    }`}
                  >
                    <ArrowDownLeft size={13} weight="bold" />
                    <span>Imported By ({context.dependent_files.length})</span>
                  </div>
                  {context.dependent_files.length === 0 ? (
                    <p className={`text-xs italic ${'text-muted'}`}>
                      No local files import this file directly.
                    </p>
                  ) : (
                    <div className="space-y-1">
                      {context.dependent_files.map((dep) => (
                        <div
                          key={dep}
                          onClick={() => onNavigateFile(dep)}
                          className={`flex items-center justify-between p-2 rounded-lg border text-xs font-mono transition-all cursor-pointer group ${
                            'bg-raise border-line hover:border-accent-line hover:bg-raise text-ink'
                          }`}
                        >
                          <span className="truncate">{dep}</span>
                          <ArrowSquareOut
                            size={12}
                            className={`shrink-0 ${
                              'text-muted group-hover:text-accent'
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
                      'text-muted'
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
                        'text-muted hover:text-ink'
                      }`}
                    >
                      {copiedCode ? (
                        <>
                          <Check size={11} className="text-good" />
                          <span className="text-good font-medium">Copied</span>
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
                      'text-muted'
                    }`}
                  >
                    <Cpu size={20} className="animate-spin text-accent mx-auto mb-2" />
                    Loading surgical excerpt...
                  </div>
                ) : activeChunk ? (
                  <div
                    className={`relative rounded-xl border overflow-hidden ${
                      'bg-surface border-line text-ink'
                    }`}
                  >
                    <pre className="p-3 text-[11.5px] font-mono leading-relaxed overflow-x-auto whitespace-pre">
                      {activeChunk.content}
                    </pre>
                  </div>
                ) : (
                  <div
                    className={`p-4 rounded-lg border text-xs text-center ${
                      'bg-raise border-line text-muted'
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
