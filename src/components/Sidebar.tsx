import React, { useState, useMemo, useEffect } from 'react';
import {
  PushPin,
  Plus,
  FileText,
  Hash,
  Lightning,
  Trash,
  Code,
  FileCode,
  ArrowUpRight,
  ArrowDownLeft,
  ArrowClockwise,
  Crosshair,
  SortAscending,
  SortDescending,
  CaretDown,
} from '@phosphor-icons/react';
import type { ChunkHit, CodeChunkHit, CodeFileRecord, NoteSummary } from '../types';
import { getLanguageStyle } from './languageStyle';

interface SidebarProps {
  projectId?: string;
  notes: NoteSummary[];
  codeFiles?: CodeFileRecord[];
  selectedNoteId: string | null;
  selectedCodePath?: string | null;
  onSelectNote: (id: string) => void;
  onSelectCodeFile?: (path: string, symbol?: string) => void;
  onNewNote: () => void;
  onDeleteNote?: (id: string, title: string) => Promise<void>;
  onSearchSemantic: (query: string) => Promise<void>;
  onSearchCode?: (query: string) => Promise<void>;
  onRevealInGraph?: (id: string) => void;
  chunkResults: ChunkHit[];
  codeChunkResults?: CodeChunkHit[];
  isSearching: boolean;
  isCollapsed?: boolean;
  theme?: 'dark' | 'light';
  appVersion?: string;
  hasUpdate?: boolean;
  onOpenReleases?: () => void;
}

const PINNED_KEY = 'gf_pinned_notes';

function loadPinned(): Set<string> {
  try {
    return new Set(JSON.parse(localStorage.getItem(PINNED_KEY) || '[]'));
  } catch {
    return new Set();
  }
}

function formatRelativeTime(timestamp?: number): string {
  if (!timestamp) return '';
  const now = Math.floor(Date.now() / 1000);
  const diff = Math.max(0, now - timestamp);
  if (diff < 60) return 'just now';
  if (diff < 3600) return `${Math.floor(diff / 60)}m ago`;
  if (diff < 86400) return `${Math.floor(diff / 3600)}h ago`;
  if (diff < 604800) return `${Math.floor(diff / 86400)}d ago`;
  return new Date(timestamp * 1000).toLocaleDateString(undefined, {
    month: 'short',
    day: 'numeric',
  });
}

export const Sidebar: React.FC<SidebarProps> = ({
  projectId,
  notes,
  codeFiles = [],
  selectedNoteId,
  selectedCodePath,
  onSelectNote,
  onSelectCodeFile,
  onNewNote,
  onDeleteNote,
  onRevealInGraph,
  chunkResults,
  codeChunkResults = [],
  isSearching,
  isCollapsed = false,
  appVersion,
  hasUpdate = false,
  onOpenReleases,
}) => {
  const [mode, setMode] = useState<'notes' | 'code'>('notes');
  const [selectedTag, setSelectedTag] = useState<string | null>(null);
  const [codeLanguageFilter, setCodeLanguageFilter] = useState<string | null>(null);
  const [dismissedResults, setDismissedResults] = useState(false);
  const [sortAz, setSortAz] = useState(false);
  const [tagsOpen, setTagsOpen] = useState(false);
  const [pinned, setPinned] = useState<Set<string>>(loadPinned);

  const hasResults = chunkResults.length > 0 || codeChunkResults.length > 0;
  const showResults = hasResults && !dismissedResults;

  useEffect(() => {
    setSelectedTag(null);
    setCodeLanguageFilter(null);
    setDismissedResults(false);
  }, [projectId]);

  useEffect(() => {
    if (hasResults) setDismissedResults(false);
  }, [chunkResults.length, codeChunkResults.length]);

  useEffect(() => {
    if (notes.length === 0 && codeFiles.length > 0) {
      setMode('code');
    } else if (codeFiles.length === 0 && notes.length > 0) {
      setMode('notes');
    }
  }, [projectId, notes.length, codeFiles.length]);

  const togglePin = (id: string) => {
    setPinned((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      localStorage.setItem(PINNED_KEY, JSON.stringify([...next]));
      return next;
    });
  };

  const tagCounts = useMemo(() => {
    const map = new Map<string, number>();
    for (const n of notes) {
      for (const t of n.tags) {
        map.set(t, (map.get(t) || 0) + 1);
      }
    }
    return Array.from(map.entries()).sort((a, b) => b[1] - a[1]);
  }, [notes]);

  const languageCounts = useMemo(() => {
    const map = new Map<string, number>();
    for (const cf of codeFiles) {
      map.set(cf.language, (map.get(cf.language) || 0) + 1);
    }
    return Array.from(map.entries()).sort((a, b) => b[1] - a[1]);
  }, [codeFiles]);

  const filteredNotes = useMemo(() => {
    const list = notes.filter((n) => !selectedTag || n.tags.includes(selectedTag));
    const sorted = sortAz
      ? [...list].sort((a, b) => a.title.localeCompare(b.title))
      : list;
    return sorted;
  }, [notes, selectedTag, sortAz]);

  const pinnedNotes = useMemo(() => filteredNotes.filter((n) => pinned.has(n.id)), [filteredNotes, pinned]);
  const unpinnedNotes = useMemo(() => filteredNotes.filter((n) => !pinned.has(n.id)), [filteredNotes, pinned]);

  const filteredCodeFiles = useMemo(() => {
    const list = codeFiles.filter(
      (cf) => !codeLanguageFilter || cf.language.toLowerCase() === codeLanguageFilter.toLowerCase(),
    );
    return sortAz ? [...list].sort((a, b) => a.name.localeCompare(b.name)) : list;
  }, [codeFiles, codeLanguageFilter, sortAz]);

  if (isCollapsed) return null;

  const chipCls = (active: boolean, accent?: { bg: string; text: string; border: string }) =>
    `inline-flex items-center gap-1 px-2 py-0.5 rounded-md text-[10px] font-medium transition-colors cursor-pointer shrink-0 border ${
      active
        ? accent
          ? `${accent.bg} ${accent.text} ${accent.border}`
          : 'bg-accent/15 text-accent border-accent/30'
        : 'bg-raise text-muted border-line hover:text-ink hover:border-line-strong'
    }`;

  const sectionLabel = 'px-1 pt-1 pb-0.5 text-[9.5px] font-mono uppercase tracking-[0.12em] text-faint';

  const renderNoteItem = (note: NoteSummary) => {
    const isSelected = selectedNoteId === note.id;
    const isPinned = pinned.has(note.id);
    return (
      <div
        key={note.id}
        onClick={() => onSelectNote(note.id)}
        className={`group relative p-2 rounded-lg border transition-colors cursor-pointer ${
          isSelected
            ? 'bg-accent/10 border-accent/40 border-l-[3px] border-l-accent'
            : 'bg-surface border-line hover:bg-raise hover:border-line-strong'
        }`}
      >
        <div className="flex items-start justify-between gap-1.5">
          <span
            className={`text-[11.5px] font-semibold leading-tight truncate ${
              isSelected ? 'text-ink' : 'text-ink'
            }`}
          >
            {note.title.replace(/\.md$/, '')}
          </span>
          <div className="flex items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
            {onRevealInGraph && (
              <button
                type="button"
                title="Reveal in graph"
                onClick={(e) => {
                  e.stopPropagation();
                  onRevealInGraph(note.id);
                }}
                className="p-1 rounded text-faint hover:text-accent hover:bg-accent/10 transition-colors cursor-pointer"
              >
                <Crosshair size={11} />
              </button>
            )}
            <button
              type="button"
              title={isPinned ? 'Unpin' : 'Pin'}
              onClick={(e) => {
                e.stopPropagation();
                togglePin(note.id);
              }}
              className={`p-1 rounded transition-colors cursor-pointer ${
                isPinned ? 'text-accent' : 'text-faint hover:text-accent hover:bg-accent/10'
              }`}
            >
              <PushPin size={11} weight={isPinned ? 'fill' : 'regular'} />
            </button>
            {onDeleteNote && (
              <button
                type="button"
                title="Delete note"
                onClick={(e) => {
                  e.stopPropagation();
                  onDeleteNote(note.id, note.title);
                }}
                className="p-1 rounded text-faint hover:text-bad hover:bg-bad/10 transition-colors cursor-pointer"
              >
                <Trash size={11} />
              </button>
            )}
          </div>
          {isPinned && (
            <PushPin
              size={10}
              weight="fill"
              className="text-accent shrink-0 group-hover:hidden mt-0.5"
            />
          )}
        </div>
        {note.summary && (
          <p className="text-[10px] text-muted leading-snug mt-1 line-clamp-2">{note.summary}</p>
        )}
        <div className="flex items-center justify-between gap-1.5 mt-1.5">
          <div className="flex items-center gap-1 min-w-0">
            {note.tags.slice(0, 3).map((t) => (
              <button
                key={t}
                type="button"
                onClick={(e) => {
                  e.stopPropagation();
                  setMode('notes');
                  setSelectedTag(t);
                }}
                className="text-[9px] px-1 py-0.5 rounded bg-raise border border-line text-muted hover:text-accent hover:border-accent/30 transition-colors cursor-pointer font-mono"
              >
                #{t}
              </button>
            ))}
          </div>
          {note.updated_at && (
            <span className="text-[9px] text-faint shrink-0 font-mono">
              {formatRelativeTime(note.updated_at)}
            </span>
          )}
        </div>
      </div>
    );
  };

  return (
    <aside
      className="w-72 h-full flex flex-col border-r shrink-0 select-none z-10 bg-surface border-line transition-colors duration-200"
      aria-label="Sidebar"
    >
      {/* Mode tabs */}
      <div className="p-2 pb-1.5 flex items-center gap-1.5">
        <div className="flex-1 flex p-0.5 rounded-lg border bg-canvas border-line">
          {(
            [
              { key: 'notes', label: 'Notes', icon: FileText, count: notes.length },
              { key: 'code', label: 'Code', icon: Code, count: codeFiles.length },
            ] as const
          ).map(({ key, label, icon: Icon, count }) => (
            <button
              key={key}
              type="button"
              onClick={() => setMode(key)}
              className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-md text-[11px] font-medium transition-colors cursor-pointer ${
                mode === key ? 'bg-accent/15 text-accent' : 'text-muted hover:text-ink'
              }`}
            >
              <Icon size={13} weight={mode === key ? 'bold' : 'regular'} />
              <span>{label}</span>
              <span
                className={`px-1 rounded text-[9.5px] font-mono ${
                  mode === key ? 'bg-accent/20 text-accent' : 'bg-raise text-faint'
                }`}
              >
                {count}
              </span>
            </button>
          ))}
        </div>
        <button
          type="button"
          title={sortAz ? 'Sort by recency' : 'Sort A–Z'}
          onClick={() => setSortAz(!sortAz)}
          className={`p-1.5 rounded-md border transition-colors cursor-pointer ${
            sortAz
              ? 'text-accent border-accent/30 bg-accent/10'
              : 'text-muted border-line bg-canvas hover:text-ink'
          }`}
        >
          {sortAz ? <SortAscending size={13} /> : <SortDescending size={13} />}
        </button>
        {mode === 'notes' && (
          <button
            type="button"
            title="New note"
            onClick={onNewNote}
            className="p-1.5 rounded-md text-muted hover:text-accent hover:bg-accent/10 transition-colors cursor-pointer"
          >
            <Plus size={13} weight="bold" />
          </button>
        )}
      </div>

      {/* Tag / language chips — one scrollable row, expandable */}
      {mode === 'notes' && tagCounts.length > 0 && (
        <div className="px-2 pb-1.5 border-b border-line">
          <button
            type="button"
            onClick={() => setTagsOpen(!tagsOpen)}
            className="w-full flex items-center justify-between px-1 py-1 text-[9.5px] font-mono uppercase tracking-[0.12em] text-faint hover:text-muted transition-colors cursor-pointer"
          >
            <span className="flex items-center gap-1">
              <Hash size={10} />
              Tags {selectedTag && <span className="text-accent">· #{selectedTag}</span>}
            </span>
            <CaretDown size={10} className={`transition-transform ${tagsOpen ? 'rotate-180' : ''}`} />
          </button>
          <div
            className={`flex gap-1 overflow-x-auto pb-1 ${
              tagsOpen ? 'flex-wrap max-h-24 overflow-y-auto' : 'flex-nowrap'
            }`}
          >
            <button
              type="button"
              onClick={() => setSelectedTag(null)}
              className={chipCls(!selectedTag)}
            >
              All
            </button>
            {tagCounts.map(([tag, count]) => (
              <button
                key={tag}
                type="button"
                onClick={() => setSelectedTag(selectedTag === tag ? null : tag)}
                className={chipCls(selectedTag === tag)}
              >
                #{tag}
                <span className="opacity-60">{count}</span>
              </button>
            ))}
          </div>
        </div>
      )}
      {mode === 'code' && languageCounts.length > 0 && (
        <div className="px-2 pb-1.5 border-b border-line">
          <div className="px-1 py-1 text-[9.5px] font-mono uppercase tracking-[0.12em] text-faint">
            Languages {codeLanguageFilter && <span className="text-accent">· {codeLanguageFilter}</span>}
          </div>
          <div className="flex gap-1 overflow-x-auto pb-1 flex-nowrap">
            <button
              type="button"
              onClick={() => setCodeLanguageFilter(null)}
              className={chipCls(!codeLanguageFilter)}
            >
              All
            </button>
            {languageCounts.map(([lang, count]) => {
              const style = getLanguageStyle(lang);
              return (
                <button
                  key={lang}
                  type="button"
                  onClick={() =>
                    setCodeLanguageFilter(codeLanguageFilter === lang ? null : lang)
                  }
                  className={chipCls(codeLanguageFilter === lang, style)}
                >
                  {lang}
                  <span className="opacity-60">{count}</span>
                </button>
              );
            })}
          </div>
        </div>
      )}

      {/* Semantic results (fed by engine search / omnisearch) */}
      {showResults && (
        <div className="px-2.5 py-1.5 border-b border-line bg-accent/5 flex items-center justify-between gap-2">
          <div className="flex items-center gap-1.5 text-[10.5px] text-muted truncate">
            <Lightning size={11} className="text-accent shrink-0" weight="fill" />
            <span>Neural results</span>
          </div>
          <button
            type="button"
            onClick={() => setDismissedResults(true)}
            className="text-[10px] text-accent hover:underline cursor-pointer shrink-0"
          >
            Hide
          </button>
        </div>
      )}

      {/* List */}
      <div className="flex-1 overflow-y-auto p-2 space-y-1.5">
        {showResults ? (
          isSearching ? (
            <div className="py-12 text-center text-faint text-[11px]">
              <ArrowClockwise size={18} className="mx-auto mb-2 text-accent animate-spin" />
              <span>Computing embeddings...</span>
            </div>
          ) : mode === 'notes' ? (
            <div className="space-y-1.5">
              {chunkResults.map((chunk) => (
                <div
                  key={chunk.chunk_id}
                  onClick={() => onSelectNote(chunk.note_id)}
                  className={`p-2 rounded-lg border transition-colors cursor-pointer ${
                    selectedNoteId === chunk.note_id
                      ? 'bg-accent/10 border-accent/40 border-l-[3px] border-l-accent'
                      : 'bg-surface border-line hover:bg-raise'
                  }`}
                >
                  <div className="flex items-center justify-between gap-1.5">
                    <span className="text-[11px] font-semibold text-ink truncate">
                      {chunk.note_title}
                    </span>
                    <span className="text-[9px] font-mono text-good shrink-0">
                      {Math.round(chunk.score * 100)}%
                    </span>
                  </div>
                  {chunk.heading && (
                    <div className="text-[9.5px] font-mono text-accent mt-0.5 truncate">
                      § {chunk.heading}
                    </div>
                  )}
                  <p className="text-[10px] text-muted leading-snug mt-1 line-clamp-3">
                    {chunk.content}
                  </p>
                </div>
              ))}
            </div>
          ) : (
            <div className="space-y-1.5">
              {codeChunkResults.map((chunk, i) => (
                <div
                  key={`${chunk.file_id}-${i}`}
                  onClick={() => onSelectCodeFile?.(chunk.file_id, chunk.symbol_name || undefined)}
                  className="p-2 rounded-lg border bg-surface border-line hover:bg-raise transition-colors cursor-pointer"
                >
                  <div className="flex items-center justify-between gap-1.5">
                    <span className="text-[11px] font-semibold text-ink truncate font-mono">
                      {chunk.symbol_name || chunk.file_name}
                    </span>
                    <span className="text-[9px] font-mono text-good shrink-0">
                      {Math.round(chunk.score * 100)}%
                    </span>
                  </div>
                  <div className="text-[9.5px] font-mono text-faint mt-0.5 truncate">
                    {chunk.file_id}
                  </div>
                  {chunk.content && (
                    <pre className="text-[9px] text-muted leading-snug mt-1 line-clamp-3 font-mono whitespace-pre-wrap">
                      {chunk.content}
                    </pre>
                  )}
                </div>
              ))}
            </div>
          )
        ) : mode === 'notes' ? (
          filteredNotes.length === 0 ? (
            <div className="py-12 text-center text-faint text-[11px]">
              <FileText size={20} className="mx-auto mb-2 opacity-40" />
              {selectedTag ? `No notes tagged #${selectedTag}` : 'No notes yet — Ctrl+K to search or create.'}
            </div>
          ) : (
            <>
              {pinnedNotes.length > 0 && (
                <>
                  <div className={sectionLabel}>Pinned · {pinnedNotes.length}</div>
                  {pinnedNotes.map(renderNoteItem)}
                </>
              )}
              <div className={sectionLabel}>
                {selectedTag ? `#${selectedTag}` : 'Notes'} · {unpinnedNotes.length}
              </div>
              {unpinnedNotes.map(renderNoteItem)}
            </>
          )
        ) : filteredCodeFiles.length === 0 ? (
          <div className="py-12 text-center text-faint text-[11px]">
            <FileCode size={20} className="mx-auto mb-2 opacity-40" />
            {codeLanguageFilter
              ? `No ${codeLanguageFilter} files`
              : 'No code files indexed.'}
          </div>
        ) : (
          <>
            <div className={sectionLabel}>
              {codeLanguageFilter || 'Files'} · {filteredCodeFiles.length}
            </div>
            {filteredCodeFiles.map((cf) => {
              const style = getLanguageStyle(cf.language);
              const isSelected = selectedCodePath === cf.path;
              return (
                <div
                  key={cf.path}
                  onClick={() => onSelectCodeFile?.(cf.path)}
                  className={`group p-2 rounded-lg border transition-colors cursor-pointer ${
                    isSelected
                      ? 'bg-accent/10 border-accent/40 border-l-[3px] border-l-accent'
                      : 'bg-surface border-line hover:bg-raise hover:border-line-strong'
                  }`}
                >
                  <div className="flex items-center gap-1.5">
                    <span
                      className="w-1.5 h-1.5 rounded-full shrink-0"
                      style={{ backgroundColor: style.hex }}
                    />
                    <span className="text-[11px] font-semibold text-ink truncate font-mono">
                      {cf.name}
                    </span>
                    {onRevealInGraph && (
                      <button
                        type="button"
                        title="Reveal in graph"
                        onClick={(e) => {
                          e.stopPropagation();
                          onRevealInGraph(cf.path);
                        }}
                        className="ml-auto p-1 rounded text-faint opacity-0 group-hover:opacity-100 hover:text-accent hover:bg-accent/10 transition-all cursor-pointer shrink-0"
                      >
                        <Crosshair size={11} />
                      </button>
                    )}
                  </div>
                  <div className="text-[9.5px] font-mono text-faint mt-0.5 truncate pl-3">
                    {cf.path}
                  </div>
                  <div className="flex items-center gap-1.5 mt-1 pl-3">
                    <span className={`text-[9px] px-1 py-0.5 rounded border font-mono ${style.bg} ${style.text} ${style.border}`}>
                      {cf.language}
                    </span>
                    {cf.imports_count > 0 && (
                      <span className="text-[8.5px] font-mono text-faint flex items-center gap-0.5">
                        <ArrowUpRight size={8} />
                        {cf.imports_count}
                      </span>
                    )}
                    {cf.dependents_count > 0 && (
                      <span className="text-[8.5px] font-mono text-faint flex items-center gap-0.5">
                        <ArrowDownLeft size={8} />
                        {cf.dependents_count}
                      </span>
                    )}
                  </div>
                </div>
              );
            })}
          </>
        )}
      </div>

      {/* Footer */}
      <div className="px-2.5 py-1.5 border-t border-line flex items-center justify-between text-[9.5px] font-mono text-faint">
        <span>
          {notes.length} notes · {codeFiles.length} files
        </span>
        <button
          type="button"
          onClick={onOpenReleases}
          className={`flex items-center gap-1 transition-colors cursor-pointer ${
            hasUpdate ? 'text-accent hover:text-accent' : 'hover:text-muted'
          }`}
        >
          {hasUpdate && <span className="w-1.5 h-1.5 rounded-full bg-accent" />}
          {appVersion && `v${appVersion}`}
        </button>
      </div>
    </aside>
  );
};
