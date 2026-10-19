import React, { useState } from 'react';
import {
  MagnifyingGlass,
  Plus,
  Stack,
  FileText,
  Tag,
  Hash,
  Lightning,
  X,
  ArrowElbowDownLeft,
} from '@phosphor-icons/react';
import type { ChunkHit, NoteSummary } from '../types';

interface SidebarProps {
  notes: NoteSummary[];
  selectedNoteId: string | null;
  onSelectNote: (id: string) => void;
  onNewNote: () => void;
  onSearchSemantic: (query: string) => Promise<void>;
  chunkResults: ChunkHit[];
  isSearching: boolean;
  isCollapsed?: boolean;
}

export const Sidebar: React.FC<SidebarProps> = ({
  notes,
  selectedNoteId,
  onSelectNote,
  onNewNote,
  onSearchSemantic,
  chunkResults,
  isSearching,
  isCollapsed = false,
}) => {
  const [activeTab, setActiveTab] = useState<'notes' | 'semantic' | 'tags'>('notes');
  const [searchTerm, setSearchTerm] = useState('');
  const [selectedTag, setSelectedTag] = useState<string | null>(null);

  const tagCounts = React.useMemo(() => {
    const map = new Map<string, number>();
    for (const n of notes) {
      for (const t of n.tags) {
        map.set(t, (map.get(t) || 0) + 1);
      }
    }
    return Array.from(map.entries()).sort((a, b) => b[1] - a[1]);
  }, [notes]);

  const handleSearchSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!searchTerm.trim()) return;
    setActiveTab('semantic');
    onSearchSemantic(searchTerm);
  };

  const filteredNotes = React.useMemo(() => {
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

  if (isCollapsed) return null;

  return (
    <aside
      className="w-72 h-full flex flex-col bg-[#0d0f14] border-r border-[#1f232d] shrink-0 select-none z-10"
      aria-label="Painel de Navegacao e Busca"
    >
      {/* Section Label matching banner.svg line 41 */}
      <div className="px-3.5 pt-3 pb-1 flex items-center justify-between">
        <span className="text-[11px] font-bold text-[#64748b] tracking-[0.8px] uppercase">
          Knowledge Vault
        </span>
        <button
          type="button"
          onClick={onNewNote}
          className="flex items-center gap-1 px-2 py-0.5 rounded bg-[#161922] hover:bg-[#1a1e28] border border-[#262a36] text-[11px] font-medium text-[#cbd5e1] transition-all cursor-pointer"
          title="Create new Markdown note"
        >
          <Plus size={11} weight="bold" />
          <span>Note</span>
        </button>
      </div>

      {/* Search Bar matching banner.svg line 129 */}
      <div className="p-3 pt-1.5 border-b border-[#1f232d]">
        <form onSubmit={handleSearchSubmit}>
          <div className="relative flex items-center w-full rounded-md bg-[#171b26] border border-[#2d3446] focus-within:border-indigo-500/80 focus-within:ring-1 focus-within:ring-indigo-500/30 transition-all">
            <MagnifyingGlass size={13} className="absolute left-2.5 text-[#64748b] pointer-events-none" />
            <input
              type="text"
              className="w-full pl-8 pr-7 py-1.5 text-xs text-[#f1f5f9] placeholder-[#64748b] bg-transparent focus:outline-none"
              placeholder="search_semantic()..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
            />
            {searchTerm ? (
              <button
                type="button"
                onClick={() => setSearchTerm('')}
                className="absolute right-2 p-0.5 rounded text-[#64748b] hover:text-[#f1f5f9] transition-colors cursor-pointer"
                title="Clear search"
              >
                <X size={12} />
              </button>
            ) : (
              <span
                className="absolute right-2 flex items-center justify-center w-3.5 h-3.5 rounded bg-[#1e222d] text-[#64748b] text-[9px]"
                title="Press Enter for semantic search"
              >
                <ArrowElbowDownLeft size={9} />
              </span>
            )}
          </div>
        </form>
      </div>

      {/* Segmented Navigation Tabs */}
      <div className="flex items-center gap-1 px-3 py-1.5 border-b border-[#1f232d]" role="tablist">
        <button
          type="button"
          role="tab"
          aria-selected={activeTab === 'notes'}
          onClick={() => setActiveTab('notes')}
          className={`flex items-center gap-1.5 px-2 py-1 rounded text-xs font-medium transition-all cursor-pointer ${
            activeTab === 'notes'
              ? 'bg-[#1a1e28] text-[#f8fafc] border border-[#2c3242]'
              : 'text-[#64748b] hover:text-[#cbd5e1] hover:bg-[#13161f]'
          }`}
        >
          <FileText size={12} />
          <span>Notes ({notes.length})</span>
        </button>

        <button
          type="button"
          role="tab"
          aria-selected={activeTab === 'semantic'}
          onClick={() => {
            setActiveTab('semantic');
            if (searchTerm.trim() && chunkResults.length === 0) {
              onSearchSemantic(searchTerm);
            }
          }}
          className={`flex items-center gap-1.5 px-2 py-1 rounded text-xs font-medium transition-all cursor-pointer ${
            activeTab === 'semantic'
              ? 'bg-[#1a1e28] text-[#f8fafc] border border-[#2c3242]'
              : 'text-[#64748b] hover:text-[#cbd5e1] hover:bg-[#13161f]'
          }`}
        >
          <Stack size={12} />
          <span>Chunks ({chunkResults.length})</span>
        </button>

        <button
          type="button"
          role="tab"
          aria-selected={activeTab === 'tags'}
          onClick={() => setActiveTab('tags')}
          className={`flex items-center gap-1.5 px-2 py-1 rounded text-xs font-medium transition-all cursor-pointer ${
            activeTab === 'tags'
              ? 'bg-[#1a1e28] text-[#f8fafc] border border-[#2c3242]'
              : 'text-[#64748b] hover:text-[#cbd5e1] hover:bg-[#13161f]'
          }`}
        >
          <Tag size={12} />
          <span>Tags ({tagCounts.length})</span>
        </button>
      </div>

      {/* Tag Filters Bar */}
      {activeTab === 'tags' && (
        <div className="flex items-center gap-1.5 p-2 overflow-x-auto border-b border-[#1f232d] bg-[#12151c]">
          {selectedTag && (
            <button
              type="button"
              onClick={() => setSelectedTag(null)}
              className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-medium bg-rose-950/60 text-rose-300 border border-rose-800/40 hover:bg-rose-900/50 transition-colors cursor-pointer shrink-0"
            >
              <X size={10} />
              Clear Filter
            </button>
          )}
          {tagCounts.map(([tag, count]) => {
            const isSelected = selectedTag === tag;
            return (
              <button
                key={tag}
                type="button"
                onClick={() => setSelectedTag(isSelected ? null : tag)}
                className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-[10px] font-medium transition-all cursor-pointer shrink-0 ${
                  isSelected
                    ? 'bg-indigo-600 text-white'
                    : 'bg-[#171b26] text-[#94a3b8] border border-[#262a36] hover:bg-[#1e222d] hover:text-[#f1f5f9]'
                }`}
              >
                <Hash size={9} />
                {tag} <span className="text-[#64748b]">({count})</span>
              </button>
            );
          })}
        </div>
      )}

      {/* List Container */}
      <div className="flex-1 overflow-y-auto p-2 space-y-1">
        {activeTab === 'semantic' ? (
          <div>
            {isSearching ? (
              <div className="py-8 text-center text-[#64748b] text-xs">
                <Stack size={18} className="mx-auto mb-2 text-indigo-400 animate-spin" />
                <span className="font-medium text-[#cbd5e1] block">Calculating Embeddings...</span>
                <span className="text-[11px] text-[#64748b] block mt-1">
                  Local FastEmbed isolating chunks
                </span>
              </div>
            ) : chunkResults.length === 0 ? (
              <div className="py-8 px-3 text-center text-[#64748b] text-xs">
                <div className="w-8 h-8 rounded-lg bg-[#12151c] border border-[#1e222d] flex items-center justify-center mx-auto mb-2 text-[#64748b]">
                  <Stack size={14} />
                </div>
                <strong className="block text-[#cbd5e1] mb-1">Semantic Search</strong>
                Enter a query and press Enter to retrieve atomic sections.
              </div>
            ) : (
              <div className="space-y-1.5">
                {chunkResults.map((chunk) => (
                  <div
                    key={chunk.chunk_id}
                    onClick={() => onSelectNote(chunk.note_id)}
                    className={`p-2.5 rounded-md border transition-all cursor-pointer ${
                      selectedNoteId === chunk.note_id
                        ? 'bg-[#1a1e28] border-[#2c3242]'
                        : 'bg-[#13161f] border-[#222736] hover:bg-[#171b26]'
                    }`}
                  >
                    <div className="flex items-center justify-between gap-1.5">
                      <span className="text-[11px] font-semibold text-[#e2e8f0] truncate">
                        {chunk.deep_link}
                      </span>
                      <span className="text-[10px] font-mono font-semibold text-[#10b981] shrink-0">
                        {Math.round(chunk.score * 100)}% match
                      </span>
                    </div>
                    <p className="text-[10.5px] text-[#94a3b8] line-clamp-2 mt-1 leading-relaxed">
                      &quot;{chunk.content}&quot;
                    </p>
                    <div className="flex items-center gap-1 text-[9.5px] font-mono text-[#64748b] mt-1.5">
                      <Lightning size={9} className="text-amber-400" />
                      <span>~{chunk.token_estimate} tokens</span>
                    </div>
                  </div>
                ))}
              </div>
            )}
          </div>
        ) : filteredNotes.length === 0 ? (
          <div className="py-8 text-center text-[#64748b] text-xs">
            <FileText size={20} className="mx-auto mb-2 opacity-40" />
            No notes found {searchTerm ? `for "${searchTerm}"` : ''}.
          </div>
        ) : (
          filteredNotes.map((note) => {
            const isSelected = selectedNoteId === note.id;
            return (
              <div
                key={note.id}
                onClick={() => onSelectNote(note.id)}
                className={`px-3 py-2 rounded-md border transition-all cursor-pointer ${
                  isSelected
                    ? 'bg-[#1a1e28] border-[#2c3242] shadow-xs'
                    : 'bg-transparent border-transparent hover:bg-[#13161f] hover:border-[#1e222d]'
                }`}
              >
                <div
                  className={`text-xs font-semibold truncate ${
                    isSelected ? 'text-[#f8fafc]' : 'text-[#94a3b8] hover:text-[#f1f5f9]'
                  }`}
                >
                  {note.title.endsWith('.md') ? note.title : `${note.title}.md`}
                </div>
                {note.summary && (
                  <p className="text-[11px] text-[#64748b] line-clamp-1 mt-0.5 leading-tight">
                    {note.summary}
                  </p>
                )}
              </div>
            );
          })
        )}
      </div>

      {/* Vault Stats Box matching banner.svg lines 53-56 */}
      <div className="p-3 border-t border-[#1f232d] bg-[#0d0f14]">
        <div className="p-2.5 rounded-lg bg-[#12151c] border border-[#1e222d] space-y-1">
          <span className="text-[11px] font-semibold text-[#cbd5e1] block">
            Vault Overview
          </span>
          <div className="font-mono text-[11px] text-[#64748b] flex items-center justify-between">
            <span>Nodes: <strong className="text-[#f8fafc] font-semibold">{notes.length}</strong></span>
            <span>Edges: <strong className="text-[#f8fafc] font-semibold">{notes.reduce((acc, n) => acc + (n.tags.length || 1), 0)}</strong></span>
          </div>
          <div className="font-mono text-[10px] text-[#64748b]">
            Local-First &amp; SQLite WAL
          </div>
        </div>
      </div>
    </aside>
  );
};
