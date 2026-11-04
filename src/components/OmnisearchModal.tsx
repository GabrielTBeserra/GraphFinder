import React, { useState, useEffect, useMemo, useRef, useCallback } from 'react';
import {
  MagnifyingGlass,
  FileText,
  Code,
  X,
  ArrowRight,
  Hash,
  Lightning,
  Plus,
  ArrowClockwise,
  Sun,
  Moon,
  Crosshair,
} from '@phosphor-icons/react';
import type { NoteSummary, CodeFileRecord } from '../types';
import { searchChunks, searchCode } from '../services/api';

interface OmnisearchModalProps {
  isOpen: boolean;
  onClose: () => void;
  notes: NoteSummary[];
  codeFiles: CodeFileRecord[];
  onSelectNote: (id: string) => void;
  onSelectCodeFile: (path: string, symbol?: string) => void;
  onRevealNode?: (id: string) => void;
  onNewNote?: () => void;
  onSync?: () => void;
  onToggleTheme?: () => void;
  theme?: 'dark' | 'light';
}

type OmniItem =
  | { kind: 'note'; key: string; id: string; title: string; sub: string; tags: string[]; score: number }
  | { kind: 'code'; key: string; path: string; name: string; lang: string; score: number }
  | { kind: 'semantic-note'; key: string; noteId: string; deepLink: string; snippet: string; score: number }
  | { kind: 'semantic-code'; key: string; fileName: string; symbol: string | null; deepLink: string; snippet: string; score: number }
  | { kind: 'action'; key: string; actionId: string; label: string; hint?: string; score: number };

const RECENT_KEY = 'gf_omni_recent';
const MAX_RECENT = 8;

interface RecentEntry {
  kind: 'note' | 'code';
  id: string;
  label: string;
}

function loadRecents(): RecentEntry[] {
  try {
    return JSON.parse(localStorage.getItem(RECENT_KEY) || '[]');
  } catch {
    return [];
  }
}

function pushRecent(entry: RecentEntry) {
  try {
    const list = loadRecents().filter((r) => !(r.kind === entry.kind && r.id === entry.id));
    list.unshift(entry);
    localStorage.setItem(RECENT_KEY, JSON.stringify(list.slice(0, MAX_RECENT)));
  } catch {
    // ignore
  }
}

/** Subsequence fuzzy match — returns a score (higher = better) or null. */
function fuzzyScore(query: string, target: string): number | null {
  if (!query) return 0;
  const q = query.toLowerCase();
  const t = target.toLowerCase();
  let score = 0;
  let ti = 0;
  let streak = 0;
  let lastMatch = -2;
  for (let qi = 0; qi < q.length; qi++) {
    const ch = q[qi];
    const found = t.indexOf(ch, ti);
    if (found === -1) return null;
    streak = found === lastMatch + 1 ? streak + 1 : 0;
    score += 1 + streak * 4;
    if (found === 0 || /[\s\-_/.#]/.test(t[found - 1])) score += 6; // word boundary bonus
    lastMatch = found;
    ti = found + 1;
  }
  score -= Math.max(0, t.length - q.length) * 0.05;
  return score;
}

export const OmnisearchModal: React.FC<OmnisearchModalProps> = ({
  isOpen,
  onClose,
  notes,
  codeFiles,
  onSelectNote,
  onSelectCodeFile,
  onRevealNode,
  onNewNote,
  onSync,
  onToggleTheme,
  theme = 'dark',
}) => {
  const isLight = theme === 'light';
  const [query, setQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [semanticNotes, setSemanticNotes] = useState<OmniItem[]>([]);
  const [semanticCode, setSemanticCode] = useState<OmniItem[]>([]);
  const [semanticLoading, setSemanticLoading] = useState(false);
  const inputRef = useRef<HTMLInputElement>(null);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (isOpen) {
      setQuery('');
      setSelectedIndex(0);
      setSemanticNotes([]);
      setSemanticCode([]);
      setTimeout(() => inputRef.current?.focus(), 50);
    }
  }, [isOpen]);

  // Debounced semantic search via engine
  useEffect(() => {
    if (debounceRef.current) clearTimeout(debounceRef.current);
    const q = query.trim();
    if (!isOpen || q.length < 3) {
      setSemanticNotes([]);
      setSemanticCode([]);
      setSemanticLoading(false);
      return;
    }
    setSemanticLoading(true);
    debounceRef.current = setTimeout(async () => {
      try {
        const [chunks, codeChunks] = await Promise.allSettled([
          searchChunks(q, 5),
          searchCode(q, undefined, 4),
        ]);
        if (chunks.status === 'fulfilled') {
          setSemanticNotes(
            chunks.value.map((c) => ({
              kind: 'semantic-note' as const,
              key: `sem-${c.chunk_id}`,
              noteId: c.note_id,
              deepLink: c.deep_link,
              snippet: c.content,
              score: c.score,
            }))
          );
        }
        if (codeChunks.status === 'fulfilled') {
          setSemanticCode(
            codeChunks.value.map((c) => ({
              kind: 'semantic-code' as const,
              key: `semcode-${c.chunk_id}`,
              fileName: c.file_name,
              symbol: c.symbol_name,
              deepLink: c.deep_link,
              snippet: c.content,
              score: c.score,
            }))
          );
        }
      } catch {
        // engine not ready — ignore
      } finally {
        setSemanticLoading(false);
      }
    }, 250);
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, [query, isOpen]);

  const actions = useMemo<OmniItem[]>(() => {
    const q = query.trim().toLowerCase();
    const defs: { id: string; label: string; keywords: string; hint?: string }[] = [
      { id: 'new-note', label: 'New Note', keywords: 'new note create blank' },
      { id: 'sync', label: 'Sync Vault', keywords: 'sync vault reindex index refresh' },
      { id: 'theme', label: isLight ? 'Switch to Dark Theme' : 'Switch to Light Theme', keywords: 'theme dark light toggle' },
    ];
    return defs
      .filter((a) => !q || fuzzyScore(q, `${a.label} ${a.keywords}`) !== null)
      .map((a) => ({ kind: 'action' as const, key: `action-${a.id}`, actionId: a.id, label: a.label, hint: a.hint, score: 0 }));
  }, [query, isLight]);

  const groups = useMemo(() => {
    const q = query.trim();

    if (!q) {
      const recents = loadRecents()
        .map((r): OmniItem | null => {
          if (r.kind === 'note') {
            const n = notes.find((x) => x.id === r.id);
            if (!n) return null;
            return { kind: 'note', key: `note-${n.id}`, id: n.id, title: n.title, sub: n.summary, tags: n.tags, score: 0 };
          }
          const c = codeFiles.find((x) => x.path === r.id || x.id === r.id);
          if (!c) return null;
          return { kind: 'code', key: `code-${c.id}`, path: c.path, name: c.name, lang: c.language, score: 0 };
        })
        .filter((x): x is OmniItem => x !== null);
      const fallback: OmniItem[] = [
        ...notes.slice(0, 5 - Math.min(5, recents.length)).map((n) => ({
          kind: 'note' as const, key: `note-${n.id}`, id: n.id, title: n.title, sub: n.summary, tags: n.tags, score: 0,
        })),
        ...codeFiles.slice(0, 4).map((c) => ({
          kind: 'code' as const, key: `code-${c.id}`, path: c.path, name: c.name, lang: c.language, score: 0,
        })),
      ];
      return [
        { label: recents.length ? 'Recent' : 'Quick Access', items: [...recents, ...fallback].slice(0, 9) },
        { label: 'Actions', items: actions },
      ];
    }

    const scoredNotes: OmniItem[] = notes
      .map((n): OmniItem | null => {
        const s = Math.max(
          fuzzyScore(q, n.title) ?? -Infinity,
          n.tags.length ? Math.max(...n.tags.map((t) => fuzzyScore(q, t) ?? -Infinity)) : -Infinity,
          (fuzzyScore(q, n.summary) ?? -Infinity) * 0.5
        );
        return s === -Infinity ? null : { kind: 'note' as const, key: `note-${n.id}`, id: n.id, title: n.title, sub: n.summary, tags: n.tags, score: s };
      })
      .filter((x): x is OmniItem => x !== null)
      .sort((a, b) => b.score - a.score)
      .slice(0, 6);

    const scoredCode: OmniItem[] = codeFiles
      .map((c): OmniItem | null => {
        const s = Math.max(
          fuzzyScore(q, c.name) ?? -Infinity,
          fuzzyScore(q, c.path) ?? -Infinity,
          (fuzzyScore(q, c.language) ?? -Infinity) * 0.6
        );
        return s === -Infinity ? null : { kind: 'code' as const, key: `code-${c.id}`, path: c.path, name: c.name, lang: c.language, score: s };
      })
      .filter((x): x is OmniItem => x !== null)
      .sort((a, b) => b.score - a.score)
      .slice(0, 6);

    const out: { label: string; items: OmniItem[] }[] = [];
    if (scoredNotes.length) out.push({ label: 'Notes', items: scoredNotes });
    if (scoredCode.length) out.push({ label: 'Code', items: scoredCode });
    if (semanticNotes.length || semanticCode.length) {
      out.push({ label: 'Semantic', items: [...semanticNotes, ...semanticCode] });
    }
    if (actions.length) out.push({ label: 'Actions', items: actions });
    return out;
  }, [query, notes, codeFiles, semanticNotes, semanticCode, actions]);

  const flatItems = useMemo(() => groups.flatMap((g) => g.items), [groups]);

  useEffect(() => {
    setSelectedIndex(0);
  }, [flatItems.length]);

  const commitSelect = useCallback(
    (item: OmniItem, reveal = false) => {
      if (item.kind === 'note' || item.kind === 'semantic-note') {
        const id = item.kind === 'note' ? item.id : item.noteId;
        pushRecent({ kind: 'note', id, label: 'title' in item ? item.title : item.deepLink });
        if (reveal && onRevealNode) {
          onRevealNode(id);
        } else {
          onSelectNote(id);
        }
      } else if (item.kind === 'code' || item.kind === 'semantic-code') {
        const path = item.kind === 'code' ? item.path : item.fileName;
        pushRecent({ kind: 'code', id: path, label: 'name' in item ? item.name : item.deepLink });
        if (reveal && onRevealNode) {
          onRevealNode(path);
        } else {
          onSelectCodeFile(path, item.kind === 'semantic-code' ? item.symbol || undefined : undefined);
        }
      } else if (item.kind === 'action') {
        if (item.actionId === 'new-note') onNewNote?.();
        if (item.actionId === 'sync') onSync?.();
        if (item.actionId === 'theme') onToggleTheme?.();
      }
      onClose();
    },
    [onSelectNote, onSelectCodeFile, onRevealNode, onNewNote, onSync, onToggleTheme, onClose]
  );

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'ArrowDown') {
      e.preventDefault();
      setSelectedIndex((p) => (p + 1) % Math.max(1, flatItems.length));
    } else if (e.key === 'ArrowUp') {
      e.preventDefault();
      setSelectedIndex((p) => (p - 1 + flatItems.length) % Math.max(1, flatItems.length));
    } else if (e.key === 'Enter') {
      e.preventDefault();
      const item = flatItems[selectedIndex];
      if (item) commitSelect(item, e.altKey);
    } else if (e.key === 'Escape') {
      e.preventDefault();
      onClose();
    }
  };

  if (!isOpen) return null;

  const itemIndex = (item: OmniItem) => {
    const i = flatItems.indexOf(item);
    return i;
  };

  const iconFor = (item: OmniItem) => {
    switch (item.kind) {
      case 'note':
        return <FileText size={14} />;
      case 'code':
        return <Code size={14} />;
      case 'semantic-note':
      case 'semantic-code':
        return <Lightning size={14} />;
      case 'action':
        if (item.actionId === 'new-note') return <Plus size={14} />;
        if (item.actionId === 'sync') return <ArrowClockwise size={14} />;
        if (item.actionId === 'theme') return isLight ? <Moon size={14} /> : <Sun size={14} />;
        return <ArrowRight size={14} />;
    }
  };

  const titleOf = (item: OmniItem): string => {
    if (item.kind === 'note') return item.title.replace(/\.md$/, '');
    if (item.kind === 'code') return item.name;
    if (item.kind === 'semantic-note' || item.kind === 'semantic-code') return item.deepLink;
    return item.label;
  };

  const subOf = (item: OmniItem): string => {
    if (item.kind === 'note') return item.sub;
    if (item.kind === 'code') return item.path;
    if (item.kind === 'semantic-note' || item.kind === 'semantic-code') return item.snippet;
    return '';
  };

  return (
    <div
      className="fixed inset-0 z-50 flex items-start justify-center pt-20 px-4 backdrop-blur-sm bg-black/50"
      onClick={onClose}
    >
      <div
        className="w-full max-w-xl rounded-xl border border-line bg-surface overflow-hidden flex flex-col"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Input */}
        <div className="flex items-center gap-3 px-4 py-3 border-b border-line bg-raise">
          <MagnifyingGlass size={16} className="text-faint shrink-0" />
          <input
            ref={inputRef}
            type="text"
            className="flex-1 bg-transparent text-sm text-ink placeholder-faint focus:outline-none"
            placeholder="Search notes, code, actions..."
            value={query}
            onChange={(e) => {
              setQuery(e.target.value);
              setSelectedIndex(0);
            }}
            onKeyDown={handleKeyDown}
          />
          {semanticLoading && (
            <ArrowClockwise size={13} className="text-accent animate-spin shrink-0" />
          )}
          {query && (
            <button
              type="button"
              onClick={() => setQuery('')}
              className="p-1 rounded text-faint hover:text-ink transition-colors cursor-pointer"
            >
              <X size={13} />
            </button>
          )}
          <kbd>ESC</kbd>
        </div>

        {/* Results */}
        <div className="max-h-[400px] overflow-y-auto p-1.5">
          {flatItems.length === 0 ? (
            <div className="py-12 text-center text-xs text-faint">
              No results for &quot;{query}&quot;.
            </div>
          ) : (
            groups.map((group) => (
              <div key={group.label}>
                <div className="px-3 pt-2.5 pb-1 text-[9.5px] font-mono uppercase tracking-[0.12em] text-faint">
                  {group.label}
                </div>
                {group.items.map((item) => {
                  const idx = itemIndex(item);
                  const isSel = idx === selectedIndex;
                  return (
                    <div
                      key={item.key}
                      onMouseEnter={() => setSelectedIndex(idx)}
                      onClick={() => commitSelect(item)}
                      className={`flex items-center justify-between gap-3 px-3 py-2 rounded-lg cursor-pointer transition-colors border ${
                        isSel
                          ? 'bg-accent/10 border-accent/30 text-ink'
                          : 'border-transparent text-muted hover:bg-raise'
                      }`}
                    >
                      <div className="flex items-center gap-2.5 min-w-0">
                        <div
                          className={`flex items-center justify-center w-6 h-6 rounded-md shrink-0 ${
                            isSel ? 'text-accent' : 'text-faint'
                          }`}
                        >
                          {iconFor(item)}
                        </div>
                        <div className="min-w-0">
                          <div className="text-xs font-medium truncate flex items-center gap-1.5">
                            <span className={isSel ? 'text-ink' : 'text-ink/90'}>{titleOf(item)}</span>
                            {item.kind === 'note' &&
                              item.tags.slice(0, 2).map((t) => (
                                <span
                                  key={t}
                                  className="inline-flex items-center gap-0.5 text-[9px] font-mono px-1 py-0.5 rounded bg-raise border border-line text-faint"
                                >
                                  <Hash size={7} />
                                  {t}
                                </span>
                              ))}
                            {item.kind === 'code' && (
                              <span className="text-[9px] font-mono px-1 py-0.5 rounded bg-raise border border-line text-faint">
                                {item.lang}
                              </span>
                            )}
                            {(item.kind === 'semantic-note' || item.kind === 'semantic-code') && (
                              <span className="text-[9px] font-mono text-good">
                                {Math.round(item.score * 100)}%
                              </span>
                            )}
                          </div>
                          {subOf(item) && (
                            <p className="text-[10.5px] truncate mt-0.5 text-faint font-mono">
                              {subOf(item)}
                            </p>
                          )}
                        </div>
                      </div>
                      {isSel && (
                        <div className="flex items-center gap-1.5 shrink-0">
                          {(item.kind === 'note' || item.kind === 'semantic-note' ||
                            item.kind === 'code' || item.kind === 'semantic-code') &&
                            onRevealNode && (
                              <button
                                type="button"
                                onClick={(e) => {
                                  e.stopPropagation();
                                  commitSelect(item, true);
                                }}
                                className="flex items-center gap-1 px-1.5 py-0.5 rounded text-[9.5px] font-mono text-accent hover:bg-accent/15 cursor-pointer"
                                title="Reveal in graph (Alt+Enter)"
                              >
                                <Crosshair size={10} />
                                reveal
                              </button>
                            )}
                          <ArrowRight size={12} className="text-accent" />
                        </div>
                      )}
                    </div>
                  );
                })}
              </div>
            ))
          )}
        </div>

        {/* Footer */}
        <div className="px-4 py-2 border-t border-line bg-canvas flex items-center justify-between text-[10px] text-faint">
          <div className="flex items-center gap-3">
            <span><kbd>↑↓</kbd> navigate</span>
            <span><kbd>↵</kbd> open</span>
            <span><kbd>alt+↵</kbd> reveal</span>
            <span><kbd>esc</kbd> close</span>
          </div>
          <span className="font-mono">Omnisearch</span>
        </div>
      </div>
    </div>
  );
};
