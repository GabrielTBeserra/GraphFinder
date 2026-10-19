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

  const displayFilename = title.endsWith('.md') ? title : `${title}.md`;

  return (
    <div className="absolute right-0 top-0 bottom-0 z-30 flex flex-col bg-[#0d0f14] border-l border-[#1f232d] shadow-2xl backdrop-blur-md w-full md:w-[620px] lg:w-[720px] transition-all">
      {/* Editor Header matching banner.svg lines 113-116 */}
      <div className="flex items-center justify-between gap-3 px-4 py-3 border-b border-[#1f232d] bg-[#111318]">
        <div className="flex-1 min-w-0">
          <input
            type="text"
            className="w-full bg-transparent text-sm font-bold text-[#f8fafc] placeholder-[#64748b] focus:outline-none"
            value={title}
            onChange={handleTitleChange}
            placeholder="Titulo da Nota..."
            aria-label="Titulo da Nota"
          />
          <div className="flex items-center gap-2 mt-0.5 text-[11px] text-[#64748b]">
            <span className="font-mono">{displayFilename}</span>
            <span>•</span>
            <span className="font-mono text-[#10b981]">~{estimatedTokens} tokens</span>
            <span>•</span>
            <span>Local FastEmbed &amp; SQLite WAL</span>
          </div>
        </div>

        <div className="flex items-center gap-2 shrink-0">
          <button
            type="button"
            onClick={handleSave}
            disabled={isSaving}
            className={`inline-flex items-center gap-1.5 h-7 px-2.5 rounded-md text-xs font-semibold transition-all cursor-pointer ${
              isDirty
                ? 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-sm'
                : 'bg-[#13161f] hover:bg-[#1a1e28] text-[#cbd5e1] border border-[#1e222d]'
            }`}
            title="Salvar alteracoes (Ctrl+S / Cmd+S)"
          >
            {isDirty ? (
              <>
                <FloppyDisk size={12} weight="bold" />
                <span>{isSaving ? 'Salvando...' : 'Salvar'}</span>
              </>
            ) : (
              <>
                <Check size={12} weight="bold" className="text-[#10b981]" />
                <span className="text-[#cbd5e1]">Salvo</span>
              </>
            )}
          </button>

          <button
            type="button"
            onClick={onClose}
            className="flex items-center justify-center w-7 h-7 rounded-md text-[#64748b] hover:text-[#f8fafc] hover:bg-[#1a1e28] transition-colors cursor-pointer"
            title="Fechar Painel (Esc)"
          >
            <X size={14} />
          </button>
        </div>
      </div>

      {/* Editor Toolbar */}
      <div className="flex items-center justify-between px-4 py-2 border-b border-[#1f232d] bg-[#0d0f14] text-xs">
        <div className="flex items-center gap-1 bg-[#13161f] border border-[#1e222d] p-0.5 rounded-md">
          <button
            type="button"
            role="tab"
            aria-selected={viewMode === 'edit'}
            onClick={() => setViewMode('edit')}
            className={`flex items-center gap-1 px-2.5 py-1 rounded text-xs transition-all cursor-pointer ${
              viewMode === 'edit'
                ? 'bg-[#1a1e28] text-[#f8fafc] border border-[#2c3242]'
                : 'text-[#64748b] hover:text-[#cbd5e1]'
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
            className={`flex items-center gap-1 px-2.5 py-1 rounded text-xs transition-all cursor-pointer ${
              viewMode === 'preview'
                ? 'bg-[#1a1e28] text-[#f8fafc] border border-[#2c3242]'
                : 'text-[#64748b] hover:text-[#cbd5e1]'
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
            className={`flex items-center gap-1 px-2.5 py-1 rounded text-xs transition-all cursor-pointer ${
              viewMode === 'split'
                ? 'bg-[#1a1e28] text-[#f8fafc] border border-[#2c3242]'
                : 'text-[#64748b] hover:text-[#cbd5e1]'
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
            className="inline-flex items-center gap-1.5 h-6 px-2 rounded bg-[#171b26] border border-[#2d3446] hover:bg-[#1e222d] text-[#cbd5e1] text-[11px] transition-colors cursor-pointer"
            title="Auto-Linker Semantico: detecta automaticamente afinidades conceituais"
          >
            <LinkSimple size={11} className="text-[#38bdf8]" />
            <span>{isSuggesting ? 'Analisando...' : 'Auto-Linker'}</span>
          </button>

          <button
            type="button"
            onClick={insertWikilink}
            className="flex items-center justify-center w-6 h-6 rounded bg-[#13161f] border border-[#1e222d] text-[#64748b] hover:text-[#f8fafc] transition-colors cursor-pointer"
            title="Inserir Wikilink [[Link]]"
          >
            <LinkSimple size={12} />
          </button>
        </div>
      </div>

      {/* Auto-Linker Drawer matching banner.svg lines 135-139 */}
      {suggestions.length > 0 && (
        <div className="p-3 bg-[#111318] border-b border-[#1f232d] space-y-2">
          <div className="flex items-center justify-between text-xs text-[#cbd5e1]">
            <span className="flex items-center gap-1.5 font-medium">
              <LinkSimple size={12} className="text-[#38bdf8]" />
              Sugestoes de Conexao Semantica ({suggestions.length})
            </span>
            <button
              type="button"
              onClick={() => setSuggestions([])}
              className="text-[#64748b] hover:text-[#f8fafc] cursor-pointer"
            >
              <X size={12} />
            </button>
          </div>

          <div className="space-y-1.5 max-h-36 overflow-y-auto">
            {suggestions.map((s) => (
              <div
                key={s.deep_link}
                className="flex items-center justify-between gap-2 p-2 rounded bg-[#13161f] border border-[#222736] text-xs"
              >
                <div className="min-w-0">
                  <div className="font-mono text-[10px] font-semibold text-[#10b981]">
                    Match {Math.round(s.similarity_score * 100)}% affinity
                  </div>
                  <strong className="text-[#e2e8f0] font-sans text-xs truncate block">
                    {s.deep_link}
                  </strong>
                </div>
                <button
                  type="button"
                  onClick={() => handleApplySuggestion(s)}
                  className="inline-flex items-center gap-1 h-6 px-2 rounded bg-indigo-600 hover:bg-indigo-500 text-white text-[10.5px] font-medium transition-colors cursor-pointer shrink-0"
                >
                  <Plus size={10} weight="bold" />
                  <span>Conectar</span>
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Editor Body */}
      <div className="flex-1 flex overflow-hidden min-h-0 bg-[#0d0f14]">
        {(viewMode === 'edit' || viewMode === 'split') && (
          <textarea
            className={`h-full p-4 font-mono text-xs text-[#e2e8f0] bg-transparent resize-none focus:outline-none border-r border-[#1f232d] leading-relaxed placeholder-[#64748b] ${
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
              viewMode === 'split' ? 'w-1/2 bg-[#090a0f]' : 'w-full'
            }`}
            dangerouslySetInnerHTML={{ __html: renderedHtml }}
            onClick={handlePreviewClick}
          />
        )}
      </div>

      {/* Connections Explorer (Footer) */}
      {note && (
        <div className="p-3 border-t border-[#1f232d] bg-[#111318] space-y-2 text-xs">
          {note.backlinks && note.backlinks.length > 0 && (
            <div>
              <div className="flex items-center gap-1.5 text-[10.5px] font-bold text-[#64748b] uppercase tracking-[0.5px] mb-1">
                <ArrowLeft size={10} />
                <span>Notas que conectam aqui ({note.backlinks.length})</span>
              </div>
              <div className="flex items-center gap-1.5 flex-wrap">
                {note.backlinks.map((b) => (
                  <button
                    key={b}
                    type="button"
                    onClick={() => onNavigateNote(b)}
                    className="px-2 py-0.5 rounded bg-[#13161f] hover:bg-[#1a1e28] border border-[#1e222d] hover:border-[#2c3242] text-[#38bdf8] font-mono text-[10.5px] transition-colors cursor-pointer"
                  >
                    [[{b}]]
                  </button>
                ))}
              </div>
            </div>
          )}

          {note.outgoing_links && note.outgoing_links.length > 0 && (
            <div>
              <div className="flex items-center gap-1.5 text-[10.5px] font-bold text-[#64748b] uppercase tracking-[0.5px] mb-1">
                <ArrowRight size={10} />
                <span>Links presentes nesta nota ({note.outgoing_links.length})</span>
              </div>
              <div className="flex items-center gap-1.5 flex-wrap">
                {note.outgoing_links.map((l) => (
                  <button
                    key={l}
                    type="button"
                    onClick={() => onNavigateNote(l)}
                    className="px-2 py-0.5 rounded bg-[#13161f] hover:bg-[#1a1e28] border border-[#1e222d] hover:border-[#2c3242] text-[#38bdf8] font-mono text-[10.5px] transition-colors cursor-pointer"
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
