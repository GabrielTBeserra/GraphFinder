import React, { useState, useEffect, useMemo, useRef } from 'react';
import {
  MagnifyingGlass,
  FileText,
  Code,
  X,
  ArrowRight,
  Hash,
} from '@phosphor-icons/react';
import type { NoteSummary, CodeFileRecord } from '../types';
import { getLanguageStyle } from './CodeInspector';

interface OmnisearchModalProps {
  isOpen: boolean;
  onClose: () => void;
  notes: NoteSummary[];
  codeFiles: CodeFileRecord[];
  onSelectNote: (id: string) => void;
  onSelectCodeFile: (path: string, symbol?: string) => void;
  theme?: 'dark' | 'light';
}

type SearchItem =
  | { type: 'note'; item: NoteSummary; key: string }
  | { type: 'code'; item: CodeFileRecord; key: string };

export const OmnisearchModal: React.FC<OmnisearchModalProps> = ({
  isOpen,
  onClose,
  notes,
  codeFiles,
  onSelectNote,
  onSelectCodeFile,
  theme = 'dark',
}) => {
  const isLight = theme === 'light';
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  const results = useMemo<SearchItem[]>(() => {
    const q = query.trim().toLowerCase();
    if (!q) {
      // Return recent 5 notes and 5 code files
      const recentNotes: SearchItem[] = notes.slice(0, 5).map((n) => ({
        type: 'note',
        item: n,
        key: `note-${n.id}`,
      }));
      const recentCode: SearchItem[] = codeFiles.slice(0, 5).map((c) => ({
        type: 'code',
        item: c,
        key: `code-${c.id}`,
      }));
      return [...recentNotes, ...recentCode];
    }

    const matchedNotes: SearchItem[] = notes
      .filter(
        (n) =>
          n.title.toLowerCase().includes(q) ||
          n.summary.toLowerCase().includes(q) ||
          n.tags.some((t) => t.toLowerCase().includes(q))
      )
      .slice(0, 6)
      .map((n) => ({ type: 'note', item: n, key: `note-${n.id}` }));

    const matchedCode: SearchItem[] = codeFiles
      .filter(
        (c) =>
          c.name.toLowerCase().includes(q) ||
          c.path.toLowerCase().includes(q) ||
          c.language.toLowerCase().includes(q)
      )
      .slice(0, 6)
      .map((c) => ({ type: 'code', item: c, key: `code-${c.id}` }));

    return [...matchedNotes, ...matchedCode];
  }, [query, notes, codeFiles]);

  const handleSelect = (result: SearchItem) => {
    if (result.type === 'note') {
      onSelectNote(result.item.id);
    } else {
      onSelectCodeFile(result.item.path);
    }
    onClose();
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev + 1) % Math.max(1, results.length));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((prev) => (prev - 1 + results.length) % Math.max(1, results.length));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      if (results[selectedIndex]) {
        handleSelect(results[selectedIndex]);
      }
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  };

  if (!isOpen) return null;

  return (
    <div
      className={`fixed inset-0 z-50 flex items-start justify-center pt-20 px-4 backdrop-blur-sm animate-in fade-in duration-150 ${
        isLight ? 'bg-slate-900/40' : 'bg-black/60'
      }`}
      onClick={onClose}
    >
      <div
        className={`w-full max-w-xl rounded-2xl border shadow-2xl overflow-hidden flex flex-col animate-in zoom-in-95 duration-150 ${
          isLight
            ? 'bg-white border-slate-200'
            : 'bg-[#0f121a] border-[#222736]'
        }`}
        onClick={(e) => e.stopPropagation()}
      >
        {/* Search Input Bar */}
        <div
          className={`flex items-center gap-3 px-4 py-3.5 border-b ${
            isLight
              ? 'border-slate-200 bg-slate-50/70'
              : 'border-[#1f2433] bg-[#131622]'
          }`}
        >
          <MagnifyingGlass
            size={18}
            className={`${isLight ? 'text-slate-400' : 'text-[#64748b]'} shrink-0`}
          />
          <input
            ref={inputRef}
            type="text"
            className={`flex-1 bg-transparent text-sm focus:outline-none ${
              isLight
                ? 'text-slate-900 placeholder-slate-400'
                : 'text-[#f8fafc] placeholder-[#64748b]'
            }`}
            placeholder="Search notes, tags, or code files..."
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
          />
          {query && (
            <button
              type="button"
              onClick={() => setQuery('')}
              className={`p-1 rounded cursor-pointer transition-colors ${
                isLight
                  ? 'text-slate-400 hover:text-slate-700'
                  : 'text-[#64748b] hover:text-[#f8fafc]'
              }`}
            >
              <X size={14} />
            </button>
          )}
          <kbd
            className={`hidden sm:inline-flex items-center px-1.5 py-0.5 text-[10px] font-mono rounded border ${
              isLight
                ? 'text-slate-500 bg-white border-slate-200 shadow-2xs'
                : 'text-[#64748b] bg-[#1a1e2b] border-[#272d3e]'
            }`}
          >
            ESC
          </kbd>
        </div>

        {/* Results List */}
        <div className="max-h-[380px] overflow-y-auto p-2 space-y-1">
          {results.length === 0 ? (
            <div
              className={`py-12 text-center text-xs ${
                isLight ? 'text-slate-400' : 'text-[#64748b]'
              }`}
            >
              No results found for &quot;{query}&quot;.
            </div>
          ) : (
            results.map((res, idx) => {
              const isSelected = idx === selectedIndex;
              if (res.type === 'note') {
                const note = res.item;
                return (
                  <div
                    key={res.key}
                    onMouseEnter={() => setSelectedIndex(idx)}
                    onClick={() => handleSelect(res)}
                    className={`flex items-center justify-between gap-3 px-3 py-2.5 rounded-xl cursor-pointer transition-all ${
                      isSelected
                        ? isLight
                          ? 'bg-indigo-50 border border-indigo-200 text-indigo-950 font-medium'
                          : 'bg-indigo-600/15 border border-indigo-500/30 text-[#f8fafc]'
                        : isLight
                        ? 'hover:bg-slate-50 border border-transparent text-slate-800'
                        : 'hover:bg-[#141824] border border-transparent text-[#cbd5e1]'
                    }`}
                  >
                    <div className="flex items-center gap-2.5 min-w-0">
                      <div
                        className={`flex items-center justify-center w-7 h-7 rounded-lg shrink-0 ${
                          isLight
                            ? 'bg-indigo-50 text-indigo-600 border border-indigo-100'
                            : 'bg-indigo-500/10 text-indigo-400'
                        }`}
                      >
                        <FileText size={15} />
                      </div>
                      <div className="min-w-0">
                        <div className="text-xs font-semibold truncate flex items-center gap-1.5">
                          <span>{note.title.replace(/\.md$/, '')}</span>
                          {note.tags.slice(0, 2).map((t) => (
                            <span
                              key={t}
                              className={`inline-flex items-center gap-0.5 text-[9.5px] font-mono px-1.5 py-0.2 rounded ${
                                isLight
                                  ? 'bg-slate-100 text-slate-600 border border-slate-200'
                                  : 'bg-[#1a1e2b] text-[#94a3b8]'
                              }`}
                            >
                              <Hash size={8} />
                              {t}
                            </span>
                          ))}
                        </div>
                        {note.summary && (
                          <p
                            className={`text-[11px] truncate mt-0.5 ${
                              isLight ? 'text-slate-500' : 'text-[#64748b]'
                            }`}
                          >
                            {note.summary}
                          </p>
                        )}
                      </div>
                    </div>
                    {isSelected && (
                      <ArrowRight
                        size={14}
                        className={`${isLight ? 'text-indigo-600' : 'text-indigo-400'} shrink-0`}
                      />
                    )}
                  </div>
                );
              }

              const code = res.item;
              const style = getLanguageStyle(code.language);
              return (
                <div
                  key={res.key}
                  onMouseEnter={() => setSelectedIndex(idx)}
                  onClick={() => handleSelect(res)}
                  className={`flex items-center justify-between gap-3 px-3 py-2.5 rounded-xl cursor-pointer transition-all ${
                    isSelected
                      ? isLight
                        ? 'bg-sky-50 border border-sky-200 text-sky-950 font-medium'
                        : 'bg-sky-500/15 border border-sky-500/30 text-[#f8fafc]'
                      : isLight
                      ? 'hover:bg-slate-50 border border-transparent text-slate-800'
                      : 'hover:bg-[#141824] border border-transparent text-[#cbd5e1]'
                  }`}
                >
                  <div className="flex items-center gap-2.5 min-w-0">
                    <div
                      className={`flex items-center justify-center w-7 h-7 rounded-lg ${style.bg} ${style.text} shrink-0`}
                    >
                      <Code size={15} />
                    </div>
                    <div className="min-w-0">
                      <div className="text-xs font-semibold truncate flex items-center gap-2">
                        <span>{code.name}</span>
                        <span
                          className={`text-[9px] font-mono px-1.5 py-0.2 rounded ${style.bg} ${style.text}`}
                        >
                          {code.language}
                        </span>
                      </div>
                      <p
                        className={`text-[10.5px] font-mono truncate mt-0.5 ${
                          isLight ? 'text-slate-500' : 'text-[#64748b]'
                        }`}
                      >
                        {code.path}
                      </p>
                    </div>
                  </div>
                  {isSelected && (
                    <ArrowRight
                      size={14}
                      className={`${isLight ? 'text-sky-600' : 'text-sky-400'} shrink-0`}
                    />
                  )}
                </div>
              );
            })
          )}
        </div>

        {/* Footer Navigation Hints */}
        <div
          className={`px-4 py-2 border-t flex items-center justify-between text-[10.5px] ${
            isLight
              ? 'border-slate-200 bg-slate-50 text-slate-500'
              : 'border-[#1a1f2c] bg-[#0c0e15] text-[#64748b]'
          }`}
        >
          <div className="flex items-center gap-3">
            <span>
              <kbd
                className={`font-mono text-[9px] px-1 py-0.5 rounded border ${
                  isLight
                    ? 'bg-white border-slate-200 text-slate-600'
                    : 'bg-[#171b26] border-[#222736]'
                }`}
              >
                ↑↓
              </kbd>{' '}
              navigate
            </span>
            <span>
              <kbd
                className={`font-mono text-[9px] px-1 py-0.5 rounded border ${
                  isLight
                    ? 'bg-white border-slate-200 text-slate-600'
                    : 'bg-[#171b26] border-[#222736]'
                }`}
              >
                ↵
              </kbd>{' '}
              open
            </span>
            <span>
              <kbd
                className={`font-mono text-[9px] px-1 py-0.5 rounded border ${
                  isLight
                    ? 'bg-white border-slate-200 text-slate-600'
                    : 'bg-[#171b26] border-[#222736]'
                }`}
              >
                esc
              </kbd>{' '}
              close
            </span>
          </div>
          <span className="font-mono text-[10px]">GraphFinder Omnisearch</span>
        </div>
      </div>
    </div>
  );
};
