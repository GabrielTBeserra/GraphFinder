import React, { useState, useMemo, useEffect } from 'react';
import {
  MagnifyingGlass,
  Plus,
  FileText,
  Hash,
  Lightning,
  X,
  Trash,
  Code,
  FileCode,
  ArrowUpRight,
  ArrowDownLeft,
  ArrowClockwise,
  Clock,
  Sparkle,
} from '@phosphor-icons/react';
import type { ChunkHit, CodeChunkHit, CodeFileRecord, NoteSummary } from '../types';
import { getLanguageStyle } from './CodeInspector';

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
  chunkResults: ChunkHit[];
  codeChunkResults?: CodeChunkHit[];
  isSearching: boolean;
  isCollapsed?: boolean;
  theme?: 'dark' | 'light';
  appVersion?: string;
  hasUpdate?: boolean;
  onOpenReleases?: () => void;
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
  onSearchSemantic,
  onSearchCode,
  chunkResults,
  codeChunkResults = [],
  isSearching,
  isCollapsed = false,
  theme = 'dark',
  appVersion,
  hasUpdate = false,
  onOpenReleases,
}) => {
  const isLight = theme === 'light';
  const [mode, setMode] = useState<'notes' | 'code'>('notes');
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedTag, setSelectedTag] = useState<string | null>(null);
  const [codeLanguageFilter, setCodeLanguageFilter] = useState<string | null>(null);
  const [isSemanticView, setIsSemanticView] = useState(false);

  useEffect(() => {
    setSearchTerm('');
    setSelectedTag(null);
    setCodeLanguageFilter(null);
    setIsSemanticView(false);
  }, [projectId]);

  useEffect(() => {
    if (notes.length === 0 && codeFiles.length > 0) {
      setMode('code');
    } else if (codeFiles.length === 0 && notes.length > 0) {
      setMode('notes');
    }
  }, [projectId, notes.length, codeFiles.length]);

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

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchTerm.trim()) {
      setIsSemanticView(false);
      return;
    }
    setIsSemanticView(true);
    if (mode === 'code' && onSearchCode) {
      onSearchCode(searchTerm);
    } else {
      onSearchSemantic(searchTerm);
    }
  };

  const handleClearSearch = () => {
    setSearchTerm('');
    setIsSemanticView(false);
  };

  const handleResetFilters = () => {
    setSearchTerm('');
    setSelectedTag(null);
    setCodeLanguageFilter(null);
    setIsSemanticView(false);
  };

  const filteredNotes = useMemo(() => {
    return notes.filter((n) => {
      const matchesSearch =
        !searchTerm ||
        n.title.toLowerCase().includes(searchTerm.toLowerCase()) ||
        n.summary.toLowerCase().includes(searchTerm.toLowerCase()) ||
        n.tags.some((t) => t.toLowerCase().includes(searchTerm.toLowerCase()));

      const matchesTag = !selectedTag || n.tags.includes(selectedTag);
      return matchesSearch && matchesTag;
    });
  }, [notes, searchTerm, selectedTag]);

  const filteredCodeFiles = useMemo(() => {
    return codeFiles.filter((cf) => {
      const matchesSearch =
        !searchTerm ||
        cf.path.toLowerCase().includes(searchTerm.toLowerCase()) ||
        cf.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
        cf.language.toLowerCase().includes(searchTerm.toLowerCase());

      const matchesLang =
        !codeLanguageFilter || cf.language.toLowerCase() === codeLanguageFilter.toLowerCase();
      return matchesSearch && matchesLang;
    });
  }, [codeFiles, searchTerm, codeLanguageFilter]);

  if (isCollapsed) return null;

  return (
    <aside
      className={`w-72 h-full flex flex-col border-r shrink-0 select-none z-10 transition-colors duration-200 ${
        isLight ? 'bg-slate-50/95 border-slate-200' : 'bg-[#0b0d13] border-[#181c26]'
      }`}
      aria-label="Sidebar"
    >
      {/* 2-Mode Segmented Selector: Notes vs Code */}
      <div className="p-2.5 pb-1 flex items-center gap-1.5">
        <div
          className={`flex-1 flex p-0.5 rounded-lg border shadow-2xs transition-colors ${
            isLight ? 'bg-slate-200/60 border-slate-300/60' : 'bg-[#12151f] border-[#1f2536]'
          }`}
        >
          <button
            type="button"
            onClick={() => {
              setMode('notes');
              setIsSemanticView(false);
            }}
            className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-md text-xs font-semibold transition-all cursor-pointer ${
              mode === 'notes'
                ? 'bg-indigo-600 text-white shadow-xs'
                : isLight
                ? 'text-slate-600 hover:text-slate-900'
                : 'text-[#64748b] hover:text-[#cbd5e1]'
            }`}
          >
            <FileText size={13} weight={mode === 'notes' ? 'bold' : 'regular'} />
            <span>Notes</span>
            <span
              className={`px-1.5 py-0.2 rounded-full text-[10px] font-mono ${
                mode === 'notes'
                  ? 'bg-white/20 text-white'
                  : isLight
                  ? 'bg-slate-300/60 text-slate-700'
                  : 'bg-[#1a2030] text-[#94a3b8]'
              }`}
            >
              {notes.length}
            </span>
          </button>

          <button
            type="button"
            onClick={() => {
              setMode('code');
              setIsSemanticView(false);
            }}
            className={`flex-1 flex items-center justify-center gap-1.5 py-1.5 rounded-md text-xs font-semibold transition-all cursor-pointer ${
              mode === 'code'
                ? 'bg-indigo-600 text-white shadow-xs'
                : isLight
                ? 'text-slate-600 hover:text-slate-900'
                : 'text-[#64748b] hover:text-[#cbd5e1]'
            }`}
          >
            <Code size={13} weight={mode === 'code' ? 'bold' : 'regular'} />
            <span>Code</span>
            <span
              className={`px-1.5 py-0.2 rounded-full text-[10px] font-mono ${
                mode === 'code'
                  ? 'bg-white/20 text-white'
                  : isLight
                  ? 'bg-slate-300/60 text-slate-700'
                  : 'bg-[#1a2030] text-[#94a3b8]'
              }`}
            >
              {codeFiles.length}
            </span>
          </button>
        </div>

        {onNewNote && mode === 'notes' && (
          <button
            type="button"
            onClick={onNewNote}
            className={`flex items-center justify-center w-7 h-7 rounded-lg border transition-colors cursor-pointer shrink-0 shadow-2xs ${
              isLight
                ? 'bg-white hover:bg-slate-100 border-slate-200 text-slate-700 hover:text-indigo-600'
                : 'bg-[#12151f] hover:bg-[#181d2a] border-[#1f2536] text-[#cbd5e1] hover:text-indigo-400'
            }`}
            title="Create new note"
          >
            <Plus size={13} weight="bold" />
          </button>
        )}
      </div>

      {/* Filter and Search Bar */}
      <div className="px-2.5 py-1.5">
        <form onSubmit={handleSearchSubmit}>
          <div
            className={`relative flex items-center w-full rounded-lg border transition-all ${
              isLight
                ? 'bg-white border-slate-200 focus-within:border-indigo-500/70 focus-within:ring-1 focus-within:ring-indigo-500/20'
                : 'bg-[#12151f] border-[#1f2536] focus-within:border-indigo-500/70 focus-within:ring-1 focus-within:ring-indigo-500/20'
            }`}
          >
            <MagnifyingGlass
              size={13}
              className={`absolute left-2.5 ${
                isLight ? 'text-slate-400' : 'text-[#64748b]'
              } pointer-events-none`}
            />
            <input
              type="text"
              className={`w-full pl-8 pr-7 py-1.5 text-xs bg-transparent focus:outline-none ${
                isLight
                  ? 'text-slate-800 placeholder-slate-400'
                  : 'text-[#f1f5f9] placeholder-[#64748b]'
              }`}
              placeholder={
                mode === 'notes'
                  ? 'Filter notes... (↵ search)'
                  : 'Filter code... (↵ search)'
              }
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
            />
            {searchTerm ? (
              <button
                type="button"
                onClick={handleClearSearch}
                className="absolute right-2 p-0.5 rounded text-[#64748b] hover:text-slate-900 transition-colors cursor-pointer"
                title="Clear search"
              >
                <X size={12} />
              </button>
            ) : null}
          </div>
        </form>
      </div>

      {/* Chips: Tags (Notes) or Languages (Code) */}
      {mode === 'notes' && tagCounts.length > 0 && !isSemanticView && (
        <div
          className={`flex items-center gap-1 px-2.5 py-1.5 overflow-x-auto border-b scrollbar-none ${
            isLight ? 'border-slate-200 bg-white/40' : 'border-[#181c26]'
          }`}
        >
          {selectedTag && (
            <button
              type="button"
              onClick={() => setSelectedTag(null)}
              className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-medium bg-rose-500/10 text-rose-600 border border-rose-500/30 hover:bg-rose-500/20 transition-colors cursor-pointer shrink-0"
            >
              <X size={10} />
              Clear
            </button>
          )}
          {tagCounts.slice(0, 10).map(([tag, count]) => {
            const isSelected = selectedTag === tag;
            return (
              <button
                key={tag}
                type="button"
                onClick={() => setSelectedTag(isSelected ? null : tag)}
                className={`inline-flex items-center gap-0.5 px-2 py-0.5 rounded text-[10px] font-medium transition-all cursor-pointer shrink-0 ${
                  isSelected
                    ? 'bg-indigo-600 text-white shadow-2xs'
                    : isLight
                    ? 'bg-slate-200/70 text-slate-700 border border-slate-300/60 hover:bg-slate-200'
                    : 'bg-[#12151f] text-[#8e9bb2] border border-[#1f2536] hover:bg-[#181d2a] hover:text-[#f1f5f9]'
                }`}
              >
                <Hash size={9} />
                <span>{tag}</span>
                <span
                  className={`ml-0.5 ${
                    isSelected
                      ? 'text-white/80'
                      : isLight
                      ? 'text-slate-500'
                      : 'text-[#64748b]'
                  }`}
                >
                  {count}
                </span>
              </button>
            );
          })}
        </div>
      )}

      {mode === 'code' && languageCounts.length > 0 && !isSemanticView && (
        <div
          className={`flex items-center gap-1 px-2.5 py-1.5 overflow-x-auto border-b scrollbar-none ${
            isLight ? 'border-slate-200 bg-white/40' : 'border-[#181c26]'
          }`}
        >
          <button
            type="button"
            onClick={() => setCodeLanguageFilter(null)}
            className={`inline-flex items-center px-2 py-0.5 rounded text-[10px] font-medium transition-all cursor-pointer shrink-0 ${
              codeLanguageFilter === null
                ? 'bg-indigo-600 text-white shadow-2xs'
                : isLight
                ? 'bg-slate-200/70 text-slate-700 border border-slate-300/60 hover:bg-slate-200'
                : 'bg-[#12151f] text-[#8e9bb2] border border-[#1f2536] hover:bg-[#181d2a]'
            }`}
          >
            All
          </button>
          {languageCounts.map(([lang, count]) => {
            const isSelected = codeLanguageFilter?.toLowerCase() === lang.toLowerCase();
            const style = getLanguageStyle(lang);
            return (
              <button
                key={lang}
                type="button"
                onClick={() => setCodeLanguageFilter(isSelected ? null : lang)}
                className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-medium transition-all cursor-pointer shrink-0 ${
                  isSelected
                    ? `${style.bg} ${style.text} border ${style.border} font-semibold`
                    : isLight
                    ? 'bg-slate-200/70 text-slate-700 border border-slate-300/60 hover:bg-slate-200'
                    : 'bg-[#12151f] text-[#8e9bb2] border border-[#1f2536] hover:bg-[#181d2a]'
                }`}
              >
                <span>{lang}</span>
                <span className={isLight ? 'text-slate-500' : 'text-[#64748b]'}>({count})</span>
              </button>
            );
          })}
        </div>
      )}

      {/* Semantic Results Banner */}
      {isSemanticView && (
        <div
          className={`px-3 py-1.5 border-b flex items-center justify-between ${
            isLight ? 'bg-amber-50/70 border-amber-200/60' : 'bg-[#12151f] border-[#181c26]'
          }`}
        >
          <div className="flex items-center gap-1.5 text-[11px] truncate">
            <Lightning size={12} className="text-amber-500 shrink-0" />
            <span
              className={`truncate ${
                isLight ? 'text-slate-700 font-medium' : 'text-[#94a3b8]'
              }`}
            >
              Neural Search: &quot;{searchTerm}&quot;
            </span>
          </div>
          <button
            type="button"
            onClick={() => setIsSemanticView(false)}
            className="text-[10px] text-indigo-500 hover:text-indigo-600 font-medium cursor-pointer shrink-0"
          >
            Back
          </button>
        </div>
      )}

      {/* Items List */}
      <div className="flex-1 overflow-y-auto p-2 space-y-1.5">
        {/* SEMANTIC RESULTS MODE */}
        {isSemanticView ? (
          isSearching ? (
            <div className="py-12 text-center text-[#64748b] text-xs">
              <ArrowClockwise size={18} className="mx-auto mb-2 text-indigo-500 animate-spin" />
              <span>Computing embeddings...</span>
            </div>
          ) : mode === 'notes' ? (
            chunkResults.length === 0 ? (
              <div className="py-12 text-center text-[#64748b] text-xs">
                No note chunks found for &quot;{searchTerm}&quot;.
              </div>
            ) : (
              <div className="space-y-1.5">
                {chunkResults.map((chunk) => (
                  <div
                    key={chunk.chunk_id}
                    onClick={() => onSelectNote(chunk.note_id)}
                    className={`p-2.5 rounded-lg border transition-all cursor-pointer ${
                      selectedNoteId === chunk.note_id
                        ? isLight
                          ? 'bg-indigo-50/90 border-indigo-300 text-slate-900 shadow-xs border-l-[3px] border-l-indigo-600'
                          : 'bg-[#171b28] border-indigo-500/40 text-white border-l-[3px] border-l-indigo-500'
                        : isLight
                        ? 'bg-white border-slate-200 hover:bg-slate-50'
                        : 'bg-[#10131c] border-[#1c2130] hover:bg-[#141824]'
                    }`}
                  >
                    <div className="flex items-center justify-between gap-1.5">
                      <span
                        className={`text-xs font-semibold truncate ${
                          isLight ? 'text-slate-800' : 'text-[#e2e8f0]'
                        }`}
                      >
                        {chunk.deep_link}
                      </span>
                      <span className="text-[10px] font-mono font-semibold text-emerald-500 shrink-0">
                        {Math.round(chunk.score * 100)}% match
                      </span>
                    </div>
                    <p
                      className={`text-[10.5px] line-clamp-2 mt-1 leading-relaxed ${
                        isLight ? 'text-slate-600' : 'text-[#94a3b8]'
                      }`}
                    >
                      &quot;{chunk.content}&quot;
                    </p>
                  </div>
                ))}
              </div>
            )
          ) : codeChunkResults.length === 0 ? (
            <div className="py-12 text-center text-[#64748b] text-xs">
              No code symbols found for &quot;{searchTerm}&quot;.
            </div>
          ) : (
            <div className="space-y-1.5">
              {codeChunkResults.map((chunk) => (
                <div
                  key={chunk.chunk_id}
                  onClick={() =>
                    onSelectCodeFile?.(chunk.file_name, chunk.symbol_name || undefined)
                  }
                  className={`p-2.5 rounded-lg border transition-all cursor-pointer ${
                    isLight
                      ? 'bg-white border-slate-200 hover:border-sky-400 hover:bg-sky-50/30'
                      : 'bg-[#10131c] border-[#1c2130] hover:bg-[#141824] hover:border-sky-500/40'
                  }`}
                >
                  <div className="flex items-center justify-between gap-1.5">
                    <span
                      className={`text-xs font-mono font-semibold truncate ${
                        isLight ? 'text-sky-700' : 'text-sky-300'
                      }`}
                    >
                      {chunk.deep_link}
                    </span>
                    <span className="text-[10px] font-mono font-semibold text-emerald-500 shrink-0">
                      {Math.round(chunk.score * 100)}% match
                    </span>
                  </div>
                  <p
                    className={`text-[10px] font-mono line-clamp-2 mt-1 p-1.5 rounded border ${
                      isLight
                        ? 'bg-slate-50 text-slate-700 border-slate-200'
                        : 'bg-[#090b10] text-[#94a3b8] border-[#161a26]'
                    }`}
                  >
                    {chunk.content}
                  </p>
                  <div className="flex items-center justify-between text-[9px] font-mono text-[#64748b] mt-1.5">
                    <span>
                      L{chunk.start_line}–{chunk.end_line}
                    </span>
                    <span>~{chunk.token_estimate} tokens</span>
                  </div>
                </div>
              ))}
            </div>
          )
        ) : mode === 'notes' ? (
          /* REGULAR NOTES LIST */
          filteredNotes.length === 0 ? (
            <div className="py-12 px-4 text-center text-xs">
              <FileText
                size={26}
                className={`mx-auto mb-2 opacity-35 ${
                  isLight ? 'text-slate-400' : 'text-slate-500'
                }`}
              />
              <p className={isLight ? 'text-slate-600' : 'text-slate-400'}>
                {searchTerm || selectedTag
                  ? 'No notes match your filters.'
                  : 'No notes in this vault yet.'}
              </p>
              {searchTerm || selectedTag ? (
                <button
                  type="button"
                  onClick={handleResetFilters}
                  className="mt-2 text-xs text-indigo-500 hover:text-indigo-600 font-medium cursor-pointer"
                >
                  Reset filters
                </button>
              ) : onNewNote ? (
                <button
                  type="button"
                  onClick={onNewNote}
                  className="mt-2.5 inline-flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-semibold bg-indigo-600 hover:bg-indigo-500 text-white cursor-pointer shadow-xs"
                >
                  <Plus size={11} weight="bold" />
                  <span>Create First Note</span>
                </button>
              ) : null}
            </div>
          ) : (
            filteredNotes.map((note) => {
              const isSelected =
                selectedNoteId !== null &&
                (selectedNoteId === note.id ||
                  selectedNoteId.toLowerCase() === note.id.toLowerCase() ||
                  selectedNoteId.replace(/\.md$/i, '').toLowerCase() ===
                    note.id.replace(/\.md$/i, '').toLowerCase() ||
                  (note.title && selectedNoteId.toLowerCase() === note.title.toLowerCase()) ||
                  (note.title &&
                    selectedNoteId.replace(/\.md$/i, '').toLowerCase() ===
                      note.title.replace(/\.md$/i, '').toLowerCase()));

              const displayTitle = note.title.replace(/\.md$/, '');
              const relativeTime = formatRelativeTime(note.updated_at);

              return (
                <div
                  key={note.id}
                  onClick={() => onSelectNote(note.id)}
                  className={`group px-3 py-2.5 rounded-lg border transition-all cursor-pointer relative ${
                    isSelected
                      ? isLight
                        ? 'bg-indigo-50/90 border-indigo-200 text-slate-900 shadow-xs border-l-[3px] border-l-indigo-600'
                        : 'bg-[#181d2a] border-indigo-500/40 text-white shadow-xs border-l-[3px] border-l-indigo-500'
                      : isLight
                      ? 'bg-white/90 border-slate-200/80 hover:bg-white hover:border-slate-300 hover:shadow-xs'
                      : 'bg-transparent border-transparent hover:bg-[#12151f] hover:border-[#1d2230]'
                  }`}
                >
                  <div className="flex items-center justify-between gap-1.5">
                    <span
                      className={`text-xs font-semibold truncate flex-1 ${
                        isSelected
                          ? isLight
                            ? 'text-indigo-950 font-bold'
                            : 'text-[#f8fafc]'
                          : isLight
                          ? 'text-slate-800 group-hover:text-slate-950'
                          : 'text-[#cbd5e1] group-hover:text-[#f8fafc]'
                      }`}
                    >
                      {displayTitle}
                    </span>
                    {onDeleteNote && (
                      <button
                        type="button"
                        onClick={(e) => {
                          e.stopPropagation();
                          if (
                            window.confirm(
                              `Are you sure you want to delete note "${displayTitle}"?`
                            )
                          ) {
                            onDeleteNote(note.id, note.title);
                          }
                        }}
                        className={`opacity-0 group-hover:opacity-100 p-1 rounded transition-all shrink-0 cursor-pointer ${
                          isLight
                            ? 'text-slate-400 hover:text-rose-600 hover:bg-rose-50'
                            : 'text-[#64748b] hover:text-rose-400 hover:bg-rose-950/40'
                        }`}
                        title="Delete note"
                      >
                        <Trash size={12} />
                      </button>
                    )}
                  </div>

                  {note.summary && (
                    <p
                      className={`text-[11px] line-clamp-2 mt-0.5 leading-relaxed ${
                        isLight ? 'text-slate-500' : 'text-[#828f9f]'
                      }`}
                    >
                      {note.summary}
                    </p>
                  )}

                  <div className="flex items-center justify-between gap-1 mt-1.5 pt-0.5">
                    <div className="flex items-center gap-1 flex-wrap min-w-0">
                      {note.tags.slice(0, 3).map((t) => (
                        <span
                          key={t}
                          className={`text-[9px] font-mono px-1.5 py-0.2 rounded border truncate ${
                            isLight
                              ? 'bg-slate-100 text-slate-600 border-slate-200'
                              : 'bg-[#171a25] text-[#8e9bb2] border-[#202534]'
                          }`}
                        >
                          #{t}
                        </span>
                      ))}
                    </div>

                    {relativeTime && (
                      <span
                        className={`text-[9.5px] flex items-center gap-1 shrink-0 ${
                          isLight ? 'text-slate-400' : 'text-[#556276]'
                        }`}
                        title={`Updated ${relativeTime}`}
                      >
                        <Clock size={10} />
                        {relativeTime}
                      </span>
                    )}
                  </div>
                </div>
              );
            })
          )
        ) : (
          /* REGULAR CODE FILES LIST */
          filteredCodeFiles.length === 0 ? (
            <div className="py-12 px-4 text-center text-xs">
              <FileCode
                size={26}
                className={`mx-auto mb-2 opacity-35 ${
                  isLight ? 'text-slate-400' : 'text-slate-500'
                }`}
              />
              <p className={isLight ? 'text-slate-600' : 'text-slate-400'}>
                {searchTerm || codeLanguageFilter
                  ? 'No code files match your filters.'
                  : 'No code files found in this vault.'}
              </p>
              {searchTerm || codeLanguageFilter ? (
                <button
                  type="button"
                  onClick={handleResetFilters}
                  className="mt-2 text-xs text-indigo-500 hover:text-indigo-600 font-medium cursor-pointer"
                >
                  Reset filters
                </button>
              ) : null}
            </div>
          ) : (
            filteredCodeFiles.map((cf) => {
              const isSelected =
                selectedCodePath === cf.path ||
                selectedCodePath === cf.id ||
                (selectedNoteId !== null &&
                  (selectedNoteId === cf.path ||
                    selectedNoteId === cf.id ||
                    selectedNoteId.toLowerCase() === cf.path.toLowerCase()));

              const style = getLanguageStyle(cf.language);
              return (
                <div
                  key={cf.id}
                  onClick={() => onSelectCodeFile?.(cf.path)}
                  className={`group px-2.5 py-2.5 rounded-lg border transition-all cursor-pointer relative ${
                    isSelected
                      ? isLight
                        ? 'bg-sky-50 border-sky-300 shadow-xs border-l-[3px] border-l-sky-500'
                        : 'bg-[#181d2a] border-sky-500/40 shadow-xs border-l-[3px] border-l-sky-400'
                      : isLight
                      ? 'bg-white/90 border-slate-200/80 hover:bg-white hover:border-slate-300'
                      : 'bg-[#10131d]/60 border-transparent hover:bg-[#131722] hover:border-[#1d2232]'
                  }`}
                >
                  <div className="flex items-center justify-between gap-1.5">
                    <div className="flex items-center gap-1.5 min-w-0">
                      <span
                        className={`px-1.5 py-0.2 rounded text-[9px] font-mono font-semibold shrink-0 ${style.bg} ${style.text}`}
                      >
                        {cf.extension.toUpperCase().replace('.', '') ||
                          cf.language.slice(0, 3).toUpperCase()}
                      </span>
                      <span
                        className={`text-xs font-semibold truncate ${
                          isSelected
                            ? isLight
                              ? 'text-sky-950 font-bold'
                              : 'text-[#f8fafc]'
                            : isLight
                            ? 'text-slate-800 group-hover:text-slate-950'
                            : 'text-[#cbd5e1] group-hover:text-[#f8fafc]'
                        }`}
                      >
                        {cf.name}
                      </span>
                    </div>

                    <div className="flex items-center gap-1 shrink-0 text-[10px] font-mono text-[#64748b]">
                      {cf.imports_count > 0 && (
                        <span
                          className="flex items-center text-sky-500"
                          title={`${cf.imports_count} imports`}
                        >
                          <ArrowUpRight size={10} />
                          {cf.imports_count}
                        </span>
                      )}
                      {cf.dependents_count > 0 && (
                        <span
                          className="flex items-center text-violet-500"
                          title={`Imported by ${cf.dependents_count} files`}
                        >
                          <ArrowDownLeft size={10} />
                          {cf.dependents_count}
                        </span>
                      )}
                    </div>
                  </div>

                  <p
                    className={`text-[10px] font-mono truncate mt-1 ${
                      isLight ? 'text-slate-400' : 'text-[#64748b]'
                    }`}
                  >
                    {cf.path}
                  </p>
                </div>
              );
            })
          )
        )}
      </div>

      {/* Footer Info Pill */}
      <div
        className={`px-3 py-2 border-t flex items-center justify-between text-[10.5px] font-mono transition-colors ${
          isLight
            ? 'bg-slate-100/70 border-slate-200 text-slate-500'
            : 'bg-[#090b10] border-[#181c26] text-[#64748b]'
        }`}
      >
        <span className="truncate">
          {notes.length} notes • {codeFiles.length} files
        </span>

        <div className="flex items-center gap-2.5 shrink-0">
          {onOpenReleases && (
            <button
              type="button"
              onClick={onOpenReleases}
              className={`hover:underline cursor-pointer flex items-center gap-1 transition-colors ${
                hasUpdate
                  ? 'text-emerald-500 font-semibold'
                  : isLight
                  ? 'text-slate-500 hover:text-slate-800'
                  : 'text-[#64748b] hover:text-[#94a3b8]'
              }`}
              title={
                hasUpdate
                  ? 'New version available! Click to view'
                  : 'View installed version & releases'
              }
            >
              {hasUpdate ? (
                <>
                  <Sparkle size={11} weight="fill" className="text-emerald-400 animate-pulse" />
                  <span>Update</span>
                </>
              ) : (
                <span>v{appVersion || '0.14.0'}</span>
              )}
            </button>
          )}

          <span className="flex items-center gap-1.5 text-emerald-500">
            <span className="w-1.5 h-1.5 rounded-full bg-emerald-500" />
            Ready
          </span>
        </div>
      </div>
    </aside>
  );
};
