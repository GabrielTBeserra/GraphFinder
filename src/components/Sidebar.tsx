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
      className="w-80 h-full flex flex-col bg-zinc-950/95 border-r border-zinc-800/80 shrink-0 select-none z-10"
      aria-label="Painel de Navegacao e Busca"
    >
      {/* Search Bar */}
      <div className="p-3 border-b border-zinc-800/70">
        <form onSubmit={handleSearchSubmit}>
          <div className="relative flex items-center w-full rounded-lg bg-zinc-900/80 border border-zinc-800 focus-within:border-indigo-500/80 focus-within:ring-1 focus-within:ring-indigo-500/40 transition-all">
            <MagnifyingGlass size={14} className="absolute left-3 text-zinc-400 pointer-events-none" />
            <input
              type="text"
              className="w-full pl-9 pr-8 py-2 text-xs text-zinc-200 placeholder-zinc-400 bg-transparent focus:outline-none"
              placeholder="Buscar ou Enter para IA..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
            />
            {searchTerm ? (
              <button
                type="button"
                onClick={() => setSearchTerm('')}
                className="absolute right-2.5 p-0.5 rounded text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors cursor-pointer"
                title="Limpar busca"
              >
                <X size={13} />
              </button>
            ) : (
              <span
                className="absolute right-2.5 flex items-center justify-center w-4 h-4 rounded bg-zinc-800 text-zinc-400 text-[10px]"
                title="Pressione Enter para busca semantica"
              >
                <ArrowElbowDownLeft size={10} />
              </span>
            )}
          </div>
        </form>
      </div>

      {/* Action Button */}
      <div className="px-3 pt-2 pb-1">
        <button
          type="button"
          onClick={onNewNote}
          className="flex items-center justify-center gap-2 w-full py-2 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs font-semibold transition-all cursor-pointer shadow-sm shadow-indigo-950"
          title="Criar uma nova nota Markdown"
        >
          <Plus size={14} weight="bold" />
          <span>Nova Nota</span>
        </button>
      </div>

      {/* Tabs */}
      <div className="flex items-center gap-1 px-3 py-2 border-b border-zinc-800/70" role="tablist">
        <button
          type="button"
          role="tab"
          aria-selected={activeTab === 'notes'}
          onClick={() => setActiveTab('notes')}
          className={`flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs font-medium transition-all cursor-pointer ${
            activeTab === 'notes'
              ? 'bg-zinc-800/90 text-zinc-100 border border-zinc-700/60 shadow-xs'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900'
          }`}
        >
          <FileText size={13} />
          <span>Notas ({notes.length})</span>
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
          className={`flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs font-medium transition-all cursor-pointer ${
            activeTab === 'semantic'
              ? 'bg-zinc-800/90 text-zinc-100 border border-zinc-700/60 shadow-xs'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900'
          }`}
        >
          <Stack size={13} />
          <span>Secoes ({chunkResults.length})</span>
        </button>

        <button
          type="button"
          role="tab"
          aria-selected={activeTab === 'tags'}
          onClick={() => setActiveTab('tags')}
          className={`flex items-center gap-1.5 px-2.5 py-1.5 rounded-md text-xs font-medium transition-all cursor-pointer ${
            activeTab === 'tags'
              ? 'bg-zinc-800/90 text-zinc-100 border border-zinc-700/60 shadow-xs'
              : 'text-zinc-400 hover:text-zinc-200 hover:bg-zinc-900'
          }`}
        >
          <Tag size={13} />
          <span>Tags ({tagCounts.length})</span>
        </button>
      </div>

      {/* Tag Filters Bar */}
      {activeTab === 'tags' && (
        <div className="flex items-center gap-1.5 p-2 overflow-x-auto border-b border-zinc-800/70 bg-zinc-900/30">
          {selectedTag && (
            <button
              type="button"
              onClick={() => setSelectedTag(null)}
              className="inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-medium bg-rose-950/60 text-rose-300 border border-rose-800/40 hover:bg-rose-900/50 transition-colors cursor-pointer shrink-0"
            >
              <X size={10} />
              Limpar Filtro
            </button>
          )}
          {tagCounts.map(([tag, count]) => {
            const isSelected = selectedTag === tag;
            return (
              <button
                key={tag}
                type="button"
                onClick={() => setSelectedTag(isSelected ? null : tag)}
                className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-medium transition-all cursor-pointer shrink-0 ${
                  isSelected
                    ? 'bg-indigo-600 text-white shadow-xs'
                    : 'bg-zinc-900/80 text-zinc-400 border border-zinc-800 hover:bg-zinc-800 hover:text-zinc-200'
                }`}
              >
                <Hash size={10} />
                {tag} <span className="opacity-70">({count})</span>
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
              <div className="py-8 text-center text-zinc-400 text-xs">
                <Stack size={20} className="mx-auto mb-2 text-indigo-400 animate-spin" />
                <span className="font-medium text-zinc-200 block">Calculando Embeddings...</span>
                <span className="text-[11px] text-zinc-400 block mt-1">
                  FastEmbed local isolando chunks semanticos
                </span>
              </div>
            ) : chunkResults.length === 0 ? (
              <div className="py-8 px-4 text-center text-zinc-400 text-xs">
                <div className="w-8 h-8 rounded-lg bg-zinc-900 border border-zinc-800 flex items-center justify-center mx-auto mb-2.5 text-zinc-400">
                  <Stack size={16} />
                </div>
                <strong className="block text-zinc-200 mb-1">Busca Semantica RAG</strong>
                Digite um conceito na barra e pressione Enter para recuperar secoes atomicas com economia de tokens.
              </div>
            ) : (
              <div className="space-y-1.5">
                {chunkResults.map((chunk) => (
                  <div
                    key={chunk.chunk_id}
                    onClick={() => onSelectNote(chunk.note_id)}
                    className={`p-2.5 rounded-lg border transition-all cursor-pointer ${
                      selectedNoteId === chunk.note_id
                        ? 'bg-indigo-950/40 border-indigo-500/40 shadow-xs'
                        : 'bg-zinc-900/40 border-zinc-800/60 hover:bg-zinc-900/80 hover:border-zinc-700/60'
                    }`}
                  >
                    <div className="flex items-center justify-between gap-2">
                      <span className="text-xs font-semibold text-indigo-300 truncate">
                        {chunk.deep_link}
                      </span>
                      <div className="flex items-center gap-1.5 shrink-0">
                        <span className="inline-flex items-center gap-0.5 px-1.5 py-0.2 rounded bg-zinc-800/80 border border-zinc-700/50 text-[10px] text-zinc-400">
                          <Lightning size={10} className="text-amber-400" />
                          ~{chunk.token_estimate} tok
                        </span>
                        <span className="text-[10px] font-mono px-1.5 py-0.2 rounded bg-indigo-950/60 text-indigo-300 border border-indigo-800/40">
                          {Math.round(chunk.score * 100)}%
                        </span>
                      </div>
                    </div>
                    <p className="text-[11px] text-zinc-400 line-clamp-3 mt-1.5 leading-relaxed font-sans">
                      &quot;{chunk.content}&quot;
                    </p>
                  </div>
                ))}
              </div>
            )}
          </div>
        ) : filteredNotes.length === 0 ? (
          <div className="py-8 text-center text-zinc-400 text-xs">
            <FileText size={22} className="mx-auto mb-2 opacity-40" />
            Nenhuma nota encontrada {searchTerm ? `para "${searchTerm}"` : ''}.
          </div>
        ) : (
          filteredNotes.map((note) => {
            const isSelected = selectedNoteId === note.id;
            return (
              <div
                key={note.id}
                onClick={() => onSelectNote(note.id)}
                className={`p-2.5 rounded-lg border transition-all cursor-pointer ${
                  isSelected
                    ? 'bg-zinc-900 border-indigo-500/50 shadow-sm'
                    : 'bg-zinc-950/60 border-zinc-900 hover:bg-zinc-900/60 hover:border-zinc-800'
                }`}
              >
                <div className="text-xs font-semibold text-zinc-200 truncate">
                  {note.title}
                </div>
                {note.summary && (
                  <p className="text-[11px] text-zinc-400 line-clamp-2 mt-1 leading-snug">
                    {note.summary}
                  </p>
                )}
                <div className="flex items-center justify-between gap-2 mt-2 pt-1 border-t border-zinc-800/40 text-[10px] text-zinc-400">
                  <span>{new Date(note.updated_at * 1000).toLocaleDateString()}</span>
                  <div className="flex items-center gap-1 flex-wrap">
                    {note.tags.slice(0, 2).map((t) => (
                      <span
                        key={t}
                        className="px-1.5 py-0.2 rounded bg-zinc-900 border border-zinc-800 text-zinc-400"
                      >
                        #{t}
                      </span>
                    ))}
                    {note.tags.length > 2 && (
                      <span className="px-1 py-0.2 rounded bg-zinc-900 border border-zinc-800 text-zinc-400">
                        +{note.tags.length - 2}
                      </span>
                    )}
                  </div>
                </div>
              </div>
            );
          })
        )}
      </div>
    </aside>
  );
};
