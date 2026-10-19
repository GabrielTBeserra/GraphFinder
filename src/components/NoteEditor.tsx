import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { marked } from 'marked';
import {
  FloppyDisk,
  Eye,
  PencilSimple,
  Columns,
  X,
  LinkSimple,
  ArrowRight,
  ArrowLeft,
  Lightning,
  Plus,
  Check,
} from '@phosphor-icons/react';
import type { LinkSuggestion, NoteRecord } from '../types';
import { suggestLinks } from '../services/api';

interface NoteEditorProps {
  note: NoteRecord | null;
  onSave: (title: string, content: string, filename?: string) => Promise<void>;
  onClose: () => void;
  onNavigateNote: (titleOrId: string) => void;
}

export const NoteEditor: React.FC<NoteEditorProps> = ({
  note,
  onSave,
  onClose,
  onNavigateNote,
}) => {
  const [title, setTitle] = useState(note?.title ?? (note ? '' : 'Nova Nota'));
  const [content, setContent] = useState(
    note?.content ??
      '# Nova Nota\n\nComece a escrever suas ideias aqui.\nConecte a outras notas com [[Nome da Nota]] e adicione #tags.'
  );
  const [viewMode, setViewMode] = useState<'edit' | 'preview' | 'split'>('split');
  const [isSaving, setIsSaving] = useState(false);
  const [isDirty, setIsDirty] = useState(!note);
  const [suggestions, setSuggestions] = useState<LinkSuggestion[]>([]);
  const [isSuggesting, setIsSuggesting] = useState(false);

  const handleSave = useCallback(async () => {
    if (!title.trim() || isSaving) return;
    setIsSaving(true);
    try {
      await onSave(title, content, note?.path);
      setIsDirty(false);
    } finally {
      setIsSaving(false);
    }
  }, [title, content, note?.path, isSaving, onSave]);

  // Keyboard shortcut Ctrl+S / Cmd+S
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if ((e.ctrlKey || e.metaKey) && e.key === 's') {
        e.preventDefault();
        handleSave();
      }
    };
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleSave]);

  const handleContentChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    setContent(e.target.value);
    setIsDirty(true);
  };

  const handleTitleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setTitle(e.target.value);
    setIsDirty(true);
  };

  const insertWikilink = () => {
    const text = '[[Nova Conexao]]';
    setContent((prev) => prev + '\n' + text);
    setIsDirty(true);
  };

  const handleSuggestLinks = async () => {
    if (!content.trim()) return;
    setIsSuggesting(true);
    try {
      const results = await suggestLinks(content, note?.id, 0.55);
      setSuggestions(results);
    } catch (err) {
      console.error('Erro ao sugerir links:', err);
    } finally {
      setIsSuggesting(false);
    }
  };

  const handleApplySuggestion = (s: LinkSuggestion) => {
    const linkStr = s.deep_link;
    if (!content.includes(linkStr)) {
      setContent((prev) => prev + `\n- Conexao Semantica: ${linkStr}`);
      setIsDirty(true);
    }
    setSuggestions((prev) => prev.filter((item) => item.deep_link !== s.deep_link));
  };

  const estimatedTokens = Math.max(1, Math.round(content.length / 4));

  const renderedHtml = useMemo(() => {
    try {
      const rawHtml = marked.parse(content) as string;
      return rawHtml.replace(/\[\[(.*?)\]\]/g, (_match, p1) => {
        const parts = p1.split('|');
        const target = parts[0].trim();
        const label = parts[1] ? parts[1].trim() : target;
        return `<span class="wikilink-btn" data-target="${target}">[[${label}]]</span>`;
      });
    } catch {
      return content;
    }
  }, [content]);

  const handlePreviewClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const target = (e.target as HTMLElement).closest('.wikilink-btn');
    if (target) {
      const linkTarget = target.getAttribute('data-target');
      if (linkTarget) {
        onNavigateNote(linkTarget);
      }
    }
  };

  return (
    <div className="absolute right-0 top-0 bottom-0 z-30 flex flex-col bg-zinc-950/95 border-l border-zinc-800 shadow-2xl backdrop-blur-md w-full md:w-[620px] lg:w-[720px] transition-all">
      {/* Editor Header */}
      <div className="flex items-center justify-between gap-3 px-4 py-2.5 border-b border-zinc-800/80 bg-zinc-900/50">
        <input
          type="text"
          className="flex-1 bg-transparent text-sm font-semibold text-zinc-100 placeholder-zinc-500 focus:outline-none"
          value={title}
          onChange={handleTitleChange}
          placeholder="Titulo da Nota..."
          aria-label="Titulo da Nota"
        />

        <div className="flex items-center gap-2">
          {/* Token Estimate Badge */}
          <span
            className={`inline-flex items-center gap-1 px-2 py-0.5 rounded text-[11px] font-mono border ${
              estimatedTokens < 150
                ? 'bg-emerald-950/40 text-emerald-400 border-emerald-800/40'
                : 'bg-indigo-950/40 text-indigo-400 border-indigo-800/40'
            }`}
            title="Estimativa de tokens para contexto de IA"
          >
            <Lightning size={11} className={estimatedTokens < 150 ? 'text-emerald-400' : 'text-indigo-400'} />
            ~{estimatedTokens} tok
          </span>

          <button
            type="button"
            onClick={handleSave}
            disabled={isSaving}
            className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-semibold transition-all cursor-pointer ${
              isDirty
                ? 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-sm shadow-indigo-950'
                : 'bg-zinc-800/80 hover:bg-zinc-700/80 text-zinc-300 border border-zinc-700/50'
            }`}
            title="Salvar alteracoes (Ctrl+S / Cmd+S)"
          >
            {isDirty ? (
              <>
                <FloppyDisk size={13} weight="bold" />
                <span>{isSaving ? 'Salvando...' : 'Salvar'}</span>
              </>
            ) : (
              <>
                <Check size={13} weight="bold" className="text-emerald-400" />
                <span className="text-zinc-300">Salvo</span>
              </>
            )}
          </button>

          <button
            type="button"
            onClick={onClose}
            className="p-1.5 rounded-md text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors cursor-pointer"
            title="Fechar Painel (Esc)"
          >
            <X size={15} />
          </button>
        </div>
      </div>

      {/* Editor Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-zinc-800/70 bg-zinc-950/80 text-xs">
        <div className="flex items-center gap-1 bg-zinc-900/90 border border-zinc-800 p-0.5 rounded-lg">
          <button
            type="button"
            role="tab"
            aria-selected={viewMode === 'edit'}
            onClick={() => setViewMode('edit')}
            className={`flex items-center gap-1 px-2.5 py-1 rounded-md transition-all cursor-pointer ${
              viewMode === 'edit'
                ? 'bg-zinc-800 text-zinc-100 shadow-xs'
                : 'text-zinc-400 hover:text-zinc-200'
            }`}
          >
            <PencilSimple size={12} />
            <span>Editar</span>
          </button>

          <button
            type="button"
            role="tab"
            aria-selected={viewMode === 'preview'}
            onClick={() => setViewMode('preview')}
            className={`flex items-center gap-1 px-2.5 py-1 rounded-md transition-all cursor-pointer ${
              viewMode === 'preview'
                ? 'bg-zinc-800 text-zinc-100 shadow-xs'
                : 'text-zinc-400 hover:text-zinc-200'
            }`}
          >
            <Eye size={12} />
            <span>Previa</span>
          </button>

          <button
            type="button"
            role="tab"
            aria-selected={viewMode === 'split'}
            onClick={() => setViewMode('split')}
            className={`flex items-center gap-1 px-2.5 py-1 rounded-md transition-all cursor-pointer ${
              viewMode === 'split'
                ? 'bg-zinc-800 text-zinc-100 shadow-xs'
                : 'text-zinc-400 hover:text-zinc-200'
            }`}
          >
            <Columns size={12} />
            <span>Lado a Lado</span>
          </button>
        </div>

        <div className="flex items-center gap-2">
          <button
            type="button"
            onClick={handleSuggestLinks}
            disabled={isSuggesting}
            className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md bg-zinc-900 border border-zinc-800 hover:bg-zinc-800 text-zinc-300 text-xs transition-colors cursor-pointer"
            title="Auto-Linker Semantico: detecta automaticamente afinidades com outras notas"
          >
            <LinkSimple size={12} />
            <span>{isSuggesting ? 'Analisando...' : 'Auto-Linker'}</span>
          </button>

          <button
            type="button"
            onClick={insertWikilink}
            className="p-1 rounded-md text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 transition-colors cursor-pointer"
            title="Inserir Wikilink [[Link]]"
          >
            <LinkSimple size={14} />
          </button>
        </div>
      </div>

      {/* Auto-Linker Drawer */}
      {suggestions.length > 0 && (
        <div className="p-3 bg-zinc-900/90 border-b border-indigo-900/40 space-y-2">
          <div className="flex items-center justify-between text-xs text-indigo-300">
            <span className="flex items-center gap-1.5 font-medium">
              <LinkSimple size={13} className="text-indigo-400" />
              Sugestoes de Conexao no Grafo ({suggestions.length})
            </span>
            <button
              type="button"
              onClick={() => setSuggestions([])}
              className="text-zinc-400 hover:text-zinc-200 cursor-pointer"
            >
              <X size={13} />
            </button>
          </div>

          <div className="space-y-1.5 max-h-36 overflow-y-auto">
            {suggestions.map((s) => (
              <div
                key={s.deep_link}
                className="flex items-center justify-between gap-2 p-2 rounded-md bg-zinc-950/70 border border-zinc-800/80 text-xs"
              >
                <div className="min-w-0">
                  <strong className="text-zinc-200 truncate block">{s.deep_link}</strong>
                  <span className="text-[10px] text-zinc-400">
                    {Math.round(s.similarity_score * 100)}% afinidade semantica
                  </span>
                </div>
                <button
                  type="button"
                  onClick={() => handleApplySuggestion(s)}
                  className="inline-flex items-center gap-1 px-2 py-0.5 rounded bg-indigo-600 hover:bg-indigo-500 text-white text-[11px] font-medium transition-colors cursor-pointer shrink-0"
                >
                  <Plus size={11} weight="bold" />
                  <span>Conectar</span>
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Editor Body */}
      <div className="flex-1 flex overflow-hidden min-h-0">
        {(viewMode === 'edit' || viewMode === 'split') && (
          <textarea
            className={`h-full p-4 font-mono text-xs text-zinc-200 bg-transparent resize-none focus:outline-none border-r border-zinc-800/60 leading-relaxed ${
              viewMode === 'split' ? 'w-1/2' : 'w-full'
            }`}
            value={content}
            onChange={handleContentChange}
            placeholder="Escreva em Markdown puro... Conecte notas com [[Titulo]] e adicione #tags."
            spellCheck={false}
          />
        )}

        {(viewMode === 'preview' || viewMode === 'split') && (
          <div
            className={`h-full p-4 overflow-y-auto preview-content ${
              viewMode === 'split' ? 'w-1/2 bg-zinc-950/40' : 'w-full'
            }`}
            dangerouslySetInnerHTML={{ __html: renderedHtml }}
            onClick={handlePreviewClick}
          />
        )}
      </div>

      {/* Connections Explorer (Footer) */}
      {note && (
        <div className="p-3 border-t border-zinc-800/80 bg-zinc-900/40 space-y-2 text-xs">
          {note.backlinks && note.backlinks.length > 0 && (
            <div>
              <div className="flex items-center gap-1.5 text-[11px] font-medium text-zinc-400 mb-1">
                <ArrowLeft size={11} />
                <span>Notas que conectam aqui ({note.backlinks.length})</span>
              </div>
              <div className="flex items-center gap-1.5 flex-wrap">
                {note.backlinks.map((b) => (
                  <button
                    key={b}
                    type="button"
                    onClick={() => onNavigateNote(b)}
                    className="px-2 py-0.5 rounded bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-indigo-300 font-mono text-[11px] transition-colors cursor-pointer"
                  >
                    [[{b}]]
                  </button>
                ))}
              </div>
            </div>
          )}

          {note.outgoing_links && note.outgoing_links.length > 0 && (
            <div>
              <div className="flex items-center gap-1.5 text-[11px] font-medium text-zinc-400 mb-1">
                <ArrowRight size={11} />
                <span>Links presentes nesta nota ({note.outgoing_links.length})</span>
              </div>
              <div className="flex items-center gap-1.5 flex-wrap">
                {note.outgoing_links.map((l) => (
                  <button
                    key={l}
                    type="button"
                    onClick={() => onNavigateNote(l)}
                    className="px-2 py-0.5 rounded bg-zinc-900 hover:bg-zinc-800 border border-zinc-800 hover:border-zinc-700 text-indigo-300 font-mono text-[11px] transition-colors cursor-pointer"
                  >
                    [[{l}]]
                  </button>
                ))}
              </div>
            </div>
          )}
        </div>
      )}
    </div>
  );
};
