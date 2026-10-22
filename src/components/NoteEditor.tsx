import React, { useState, useEffect, useCallback, useMemo, useRef } from 'react';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
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
  Hash,
  FileText,
  Trash,
  ArrowsOut,
  ArrowsIn,
} from '@phosphor-icons/react';
import type { LinkSuggestion, NoteRecord, NoteSummary } from '../types';
import { suggestLinks } from '../services/api';

interface NoteEditorProps {
  note: NoteRecord | null;
  onSave: (title: string, content: string, filename?: string) => Promise<void>;
  onDelete?: (id: string, title: string) => Promise<void>;
  onClose: () => void;
  onNavigateNote: (titleOrId: string) => void;
  availableNotes?: NoteSummary[];
  theme?: 'dark' | 'light';
}

interface AutocompleteState {
  isOpen: boolean;
  query: string;
  isSectionQuery: boolean;
  baseNoteTitle: string;
  startIndex: number;
  cursorIndex: number;
  selectedIndex: number;
}

interface AutocompleteItem {
  id: string;
  title: string;
  heading?: string;
  tag?: string;
  type: 'note' | 'section';
  preview?: string;
}

export const NoteEditor: React.FC<NoteEditorProps> = ({
  note,
  onSave,
  onDelete,
  onClose,
  onNavigateNote,
  availableNotes = [],
  theme = 'dark',
}) => {
  const isLight = theme === 'light';
  const [title, setTitle] = useState(note?.title ?? (note ? '' : 'New Note'));
  const [content, setContent] = useState(
    note?.content ??
      '# New Note\n\nStart writing your thoughts here.\nConnect to other notes with [[Note Title]] and add #tags.'
  );
  const [viewMode, setViewMode] = useState<'edit' | 'preview' | 'split'>('split');
  const [isFullscreen, setIsFullscreen] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [isDirty, setIsDirty] = useState(!note);
  const [suggestions, setSuggestions] = useState<LinkSuggestion[]>([]);
  const [isSuggesting, setIsSuggesting] = useState(false);

  const textareaRef = useRef<HTMLTextAreaElement>(null);

  // Autocomplete state
  const [autocomplete, setAutocomplete] = useState<AutocompleteState>({
    isOpen: false,
    query: '',
    isSectionQuery: false,
    baseNoteTitle: '',
    startIndex: -1,
    cursorIndex: -1,
    selectedIndex: 0,
  });

  const lastLoadedIdRef = useRef<string | undefined>(undefined);

  useEffect(() => {
    const targetKey = note?.id || note?.title || 'new';
    const isNewTarget = lastLoadedIdRef.current !== targetKey;
    if (isNewTarget) {
      lastLoadedIdRef.current = targetKey;
      setTitle(note?.title ?? (note ? '' : 'New Note'));
      setContent(
        note?.content ??
          '# New Note\n\nStart writing your thoughts here.\nConnect to other notes with [[Note Title]] and add #tags.'
      );
      setIsDirty(!note);
      setSuggestions([]);
      setAutocomplete((prev) => ({ ...prev, isOpen: false }));
    }
  }, [note?.id, note?.title, note?.path, note?.content]);

  const handleSave = useCallback(async () => {
    if (!title.trim() || isSaving) return;
    setIsSaving(true);
    try {
      await onSave(title, content, note?.path);
      setIsDirty(false);
      lastLoadedIdRef.current = note?.id || title;
    } finally {
      setIsSaving(false);
    }
  }, [title, content, note?.path, note?.id, isSaving, onSave]);

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

  // Extract headings from the current document
  const currentHeadings = useMemo(() => {
    const lines = content.split('\n');
    const headings: string[] = [];
    for (const line of lines) {
      const match = line.match(/^#{1,3}\s+(.+)$/);
      if (match) {
        headings.push(match[1].trim());
      }
    }
    return headings;
  }, [content]);

  // Check cursor for open [[
  const checkAutocomplete = useCallback(
    (text: string, cursorPos: number) => {
      const textBefore = text.slice(0, cursorPos);
      const lastOpen = textBefore.lastIndexOf('[[');

      if (lastOpen !== -1) {
        const textAfterOpen = textBefore.slice(lastOpen + 2);
        // Only trigger if no newline and no closing ]] between [[ and cursor
        if (!textAfterOpen.includes(']]') && !textAfterOpen.includes('\n')) {
          const isSection = textAfterOpen.includes('#');
          let baseNote = '';
          let q = textAfterOpen;

          if (isSection) {
            const parts = textAfterOpen.split('#');
            baseNote = parts[0].trim();
            q = parts[1] || '';
          }

          setAutocomplete({
            isOpen: true,
            query: q,
            isSectionQuery: isSection,
            baseNoteTitle: baseNote,
            startIndex: lastOpen,
            cursorIndex: cursorPos,
            selectedIndex: 0,
          });
          return;
        }
      }

      setAutocomplete((prev) => (prev.isOpen ? { ...prev, isOpen: false } : prev));
    },
    []
  );

  const handleContentChange = (e: React.ChangeEvent<HTMLTextAreaElement>) => {
    const val = e.target.value;
    const cursorPos = e.target.selectionStart;
    setContent(val);
    setIsDirty(true);
    checkAutocomplete(val, cursorPos);
  };

  const handleEditorSelectionChange = () => {
    if (textareaRef.current) {
      checkAutocomplete(content, textareaRef.current.selectionStart);
    }
  };

  const handleTitleChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setTitle(e.target.value);
    setIsDirty(true);
  };

  // Generate autocomplete candidate list
  const autocompleteCandidates = useMemo<AutocompleteItem[]>(() => {
    if (!autocomplete.isOpen) return [];

    const q = autocomplete.query.toLowerCase().trim();

    // Case 1: Searching for section [[#Section]] or [[Note#Section]]
    if (autocomplete.isSectionQuery) {
      // If baseNoteTitle is empty, search current note's headings
      if (!autocomplete.baseNoteTitle || autocomplete.baseNoteTitle.toLowerCase() === title.toLowerCase()) {
        return currentHeadings
          .filter((h) => !q || h.toLowerCase().includes(q))
          .map((h) => ({
            id: `sec:${h}`,
            title: title || 'Current Note',
            heading: h,
            type: 'section',
            preview: `Current section: #${h}`,
          }));
      }

      // If user specified an external note, suggest sections or match note title
      return [
        {
          id: `sec:${autocomplete.baseNoteTitle}#${autocomplete.query}`,
          title: autocomplete.baseNoteTitle,
          heading: autocomplete.query || 'Section',
          type: 'section',
          preview: `Deep link to [[${autocomplete.baseNoteTitle}#${autocomplete.query}]]`,
        },
      ];
    }

    // Case 2: Searching for notes [[Query]]
    const matchedNotes = availableNotes.filter((n) => {
      if (!q) return true;
      const titleMatch = n.title.toLowerCase().includes(q);
      const tagMatch = n.tags.some((t) => t.toLowerCase().includes(q));
      return titleMatch || tagMatch;
    });

    const items: AutocompleteItem[] = matchedNotes.slice(0, 8).map((n) => ({
      id: n.id,
      title: n.title,
      tag: n.tags[0],
      type: 'note',
      preview: n.summary || (n.tags.length > 0 ? `#${n.tags.join(' #')}` : undefined),
    }));

    // If query is not empty and doesn't exactly match an existing note, offer creating a new note link
    const exactExists = availableNotes.some((n) => n.title.toLowerCase() === q);
    if (q && !exactExists) {
      items.push({
        id: `new:${autocomplete.query}`,
        title: autocomplete.query,
        type: 'note',
        preview: 'Create new note upon clicking graph link',
      });
    }

    return items;
  }, [
    autocomplete.isOpen,
    autocomplete.isSectionQuery,
    autocomplete.baseNoteTitle,
    autocomplete.query,
    title,
    currentHeadings,
    availableNotes,
  ]);

  // Insert selected autocomplete item
  const applyAutocomplete = useCallback(
    (item: AutocompleteItem) => {
      let insertText: string;
      if (item.type === 'section') {
        if (!autocomplete.baseNoteTitle || autocomplete.baseNoteTitle.toLowerCase() === title.toLowerCase()) {
          insertText = `[[#${item.heading}]]`;
        } else {
          insertText = `[[${item.title}#${item.heading}]]`;
        }
      } else {
        insertText = `[[${item.title}]]`;
      }

      const before = content.slice(0, autocomplete.startIndex);
      const after = content.slice(autocomplete.cursorIndex);
      const newContent = before + insertText + after;

      setContent(newContent);
      setIsDirty(true);
      setAutocomplete((prev) => ({ ...prev, isOpen: false }));

      // Move cursor right after the closing brackets
      setTimeout(() => {
        if (textareaRef.current) {
          const nextPos = before.length + insertText.length;
          textareaRef.current.focus();
          textareaRef.current.setSelectionRange(nextPos, nextPos);
        }
      }, 15);
    },
    [autocomplete, content, title]
  );

  // Keyboard navigation for autocomplete inside textarea
  const handleEditorKeyDown = (e: React.KeyboardEvent<HTMLTextAreaElement>) => {
    if (autocomplete.isOpen && autocompleteCandidates.length > 0) {
      if (e.key === 'ArrowDown') {
        e.preventDefault();
        setAutocomplete((prev) => ({
          ...prev,
          selectedIndex: (prev.selectedIndex + 1) % autocompleteCandidates.length,
        }));
        return;
      }
      if (e.key === 'ArrowUp') {
        e.preventDefault();
        setAutocomplete((prev) => ({
          ...prev,
          selectedIndex: (prev.selectedIndex - 1 + autocompleteCandidates.length) % autocompleteCandidates.length,
        }));
        return;
      }
      if (e.key === 'Enter' || e.key === 'Tab') {
        e.preventDefault();
        const selected = autocompleteCandidates[autocomplete.selectedIndex];
        if (selected) {
          applyAutocomplete(selected);
        }
        return;
      }
      if (e.key === 'Escape') {
        e.preventDefault();
        setAutocomplete((prev) => ({ ...prev, isOpen: false }));
        return;
      }
    }
  };

  const insertWikilink = () => {
    const text = '[[New Connection]]';
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
      console.error('Error suggesting links:', err);
    } finally {
      setIsSuggesting(false);
    }
  };

  const handleApplySuggestion = (s: LinkSuggestion) => {
    const linkStr = s.deep_link;
    if (!content.includes(linkStr)) {
      setContent((prev) => prev + `\n- Semantic Connection: ${linkStr}`);
      setIsDirty(true);
    }
    setSuggestions((prev) => prev.filter((item) => item.deep_link !== s.deep_link));
  };

  const estimatedTokens = Math.max(1, Math.round(content.length / 4));

  const renderedHtml = useMemo(() => {
    try {
      const rawHtml = marked.parse(content) as string;
      const withWikilinks = rawHtml.replace(/\[\[(.*?)\]\]/g, (_match, p1) => {
        const parts = p1.split('|');
        const target = parts[0].trim();
        const label = parts[1] ? parts[1].trim() : target;
        return `<span class="wikilink-btn" data-target="${target}">[[${label}]]</span>`;
      });
      return DOMPurify.sanitize(withWikilinks, {
        ADD_TAGS: ['span'],
        ADD_ATTR: ['data-target', 'class'],
      });
    } catch {
      return DOMPurify.sanitize(content);
    }
  }, [content]);

  const handlePreviewClick = (e: React.MouseEvent<HTMLDivElement>) => {
    const anchor = (e.target as HTMLElement).closest('a');
    if (anchor) {
      e.preventDefault();
      const href = anchor.getAttribute('href');
      if (href) {
        if (!href.startsWith('http://') && !href.startsWith('https://')) {
          onNavigateNote(href.replace(/\.md$/, ''));
        }
      }
      return;
    }

    const target = (e.target as HTMLElement).closest('.wikilink-btn');
    if (target) {
      const linkTarget = target.getAttribute('data-target');
      if (linkTarget) {
        onNavigateNote(linkTarget);
      }
    }
  };

  const displayFilename = title.endsWith('.md') ? title : `${title}.md`;
  const wordCount = useMemo(() => content.trim().split(/\s+/).filter(Boolean).length, [content]);

  return (
    <div
      className={`absolute top-0 bottom-0 z-30 flex flex-col border-l shadow-2xl backdrop-blur-md transition-all duration-200 ${
        isFullscreen
          ? 'left-0 right-0 w-full'
          : 'right-0 w-full md:w-[480px] lg:w-[540px]'
      } ${isLight ? 'bg-white border-slate-200 shadow-slate-200/50' : 'bg-[#0b0d13] border-[#181c26]'}`}
      aria-label="Note Editor"
    >
      {/* Editor Header */}
      <div className={`flex items-center justify-between gap-3 px-4 py-3 border-b transition-colors ${
        isLight ? 'bg-slate-50/70 border-slate-200' : 'bg-[#0e1118] border-[#181c26]'
      }`}>
        <div className="flex-1 min-w-0">
          <input
            type="text"
            className={`w-full bg-transparent text-sm font-bold focus:outline-none ${
              isLight ? 'text-slate-900 placeholder:text-slate-400' : 'text-[#f8fafc] placeholder-[#64748b]'
            }`}
            value={title}
            onChange={handleTitleChange}
            placeholder="Note Title..."
            aria-label="Note Title"
          />
          <div className={`flex items-center gap-2 mt-0.5 text-[11px] ${isLight ? 'text-slate-500' : 'text-[#64748b]'}`}>
            <span className="font-mono">{displayFilename}</span>
            <span>•</span>
            <span className="font-mono text-emerald-500">~{estimatedTokens} tokens</span>
            <span>•</span>
            <span>{wordCount} words</span>
          </div>
        </div>

        <div className="flex items-center gap-1.5 shrink-0">
          <button
            type="button"
            onClick={handleSave}
            disabled={isSaving}
            className={`inline-flex items-center gap-1.5 h-7 px-2.5 rounded-lg text-xs font-semibold transition-all cursor-pointer ${
              isDirty
                ? 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-xs'
                : isLight
                ? 'bg-slate-100 hover:bg-slate-200 text-slate-700 border border-slate-200'
                : 'bg-[#131620] hover:bg-[#181c28] text-[#cbd5e1] border border-[#202534]'
            }`}
            title="Save changes (Ctrl+S / Cmd+S)"
          >
            {isDirty ? (
              <>
                <FloppyDisk size={12} weight="bold" />
                <span>{isSaving ? 'Saving...' : 'Save'}</span>
              </>
            ) : (
              <>
                <Check size={12} weight="bold" className="text-emerald-500" />
                <span className={isLight ? 'text-slate-700' : 'text-[#cbd5e1]'}>Saved</span>
              </>
            )}
          </button>

          {note?.id && onDelete && (
            <button
              type="button"
              onClick={async () => {
                if (window.confirm(`Are you sure you want to delete note "${note.title}"?`)) {
                  await onDelete(note.id, note.title);
                  onClose();
                }
              }}
              className={`flex items-center justify-center w-7 h-7 rounded-lg transition-colors cursor-pointer ${
                isLight
                  ? 'text-slate-400 hover:text-rose-600 hover:bg-rose-50 border border-transparent'
                  : 'text-[#64748b] hover:text-rose-400 hover:bg-rose-950/40 border border-transparent hover:border-rose-900/40'
              }`}
              title="Delete note permanently"
            >
              <Trash size={13} />
            </button>
          )}

          <button
            type="button"
            onClick={() => setIsFullscreen(!isFullscreen)}
            className={`flex items-center justify-center w-7 h-7 rounded-lg transition-colors cursor-pointer ${
              isLight
                ? 'text-slate-500 hover:text-slate-900 hover:bg-slate-100'
                : 'text-[#64748b] hover:text-[#f8fafc] hover:bg-[#161a24]'
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
                : 'text-[#64748b] hover:text-[#f8fafc] hover:bg-[#161a24]'
            }`}
            title="Close Panel (Esc)"
          >
            <X size={14} />
          </button>
        </div>
      </div>

      {/* Editor Toolbar */}
      <div className={`flex items-center justify-between px-4 py-1.5 border-b text-xs transition-colors ${
        isLight ? 'bg-white border-slate-200' : 'bg-[#0c0e15] border-[#181c26]'
      }`}>
        <div className={`flex items-center gap-0.5 border p-0.5 rounded-lg ${
          isLight ? 'bg-slate-100 border-slate-200' : 'bg-[#12151f] border-[#1f2536]'
        }`}>
          <button
            type="button"
            role="tab"
            aria-selected={viewMode === 'edit'}
            onClick={() => setViewMode('edit')}
            className={`flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium transition-all cursor-pointer ${
              viewMode === 'edit'
                ? 'bg-indigo-600 text-white shadow-xs'
                : isLight
                ? 'text-slate-600 hover:text-slate-900'
                : 'text-[#64748b] hover:text-[#cbd5e1]'
            }`}
          >
            <PencilSimple size={12} />
            <span>Edit</span>
          </button>

          <button
            type="button"
            role="tab"
            aria-selected={viewMode === 'split'}
            onClick={() => setViewMode('split')}
            className={`flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium transition-all cursor-pointer ${
              viewMode === 'split'
                ? 'bg-indigo-600 text-white shadow-xs'
                : isLight
                ? 'text-slate-600 hover:text-slate-900'
                : 'text-[#64748b] hover:text-[#cbd5e1]'
            }`}
          >
            <Columns size={12} />
            <span>Split</span>
          </button>

          <button
            type="button"
            role="tab"
            aria-selected={viewMode === 'preview'}
            onClick={() => setViewMode('preview')}
            className={`flex items-center gap-1 px-2.5 py-1 rounded-md text-xs font-medium transition-all cursor-pointer ${
              viewMode === 'preview'
                ? 'bg-indigo-600 text-white shadow-xs'
                : isLight
                ? 'text-slate-600 hover:text-slate-900'
                : 'text-[#64748b] hover:text-[#cbd5e1]'
            }`}
          >
            <Eye size={12} />
            <span>Preview</span>
          </button>
        </div>

        <div className="flex items-center gap-1.5">
          <button
            type="button"
            onClick={handleSuggestLinks}
            disabled={isSuggesting}
            className={`inline-flex items-center gap-1.5 h-6 px-2 rounded-md text-[11px] transition-colors cursor-pointer ${
              isLight
                ? 'bg-slate-100 border border-slate-200 hover:bg-slate-200 text-slate-700'
                : 'bg-[#131722] border border-[#202636] hover:bg-[#181d2a] text-[#cbd5e1]'
            }`}
            title="Semantic Auto-Linker: discover conceptual connections"
          >
            <LinkSimple size={11} className={isLight ? 'text-sky-600' : 'text-[#38bdf8]'} />
            <span>{isSuggesting ? 'Analyzing...' : 'Semantic Links'}</span>
          </button>

          <button
            type="button"
            onClick={insertWikilink}
            className={`flex items-center justify-center w-6 h-6 rounded-md border transition-colors cursor-pointer ${
              isLight
                ? 'bg-slate-100 border-slate-200 text-slate-600 hover:text-slate-900'
                : 'bg-[#131722] border border-[#202636] text-[#64748b] hover:text-[#f8fafc]'
            }`}
            title="Insert Wikilink [[Link]]"
          >
            <LinkSimple size={12} />
          </button>
        </div>
      </div>

      {/* Auto-Linker Drawer */}
      {suggestions.length > 0 && (
        <div className={`p-3 border-b space-y-2 ${
          isLight ? 'bg-indigo-50/50 border-indigo-100' : 'bg-[#111318] border-[#1f232d]'
        }`}>
          <div className="flex items-center justify-between text-xs">
            <span className={`flex items-center gap-1.5 font-medium ${isLight ? 'text-slate-800' : 'text-[#cbd5e1]'}`}>
              <LinkSimple size={12} className={isLight ? 'text-sky-600' : 'text-[#38bdf8]'} />
              Semantic Suggestions ({suggestions.length})
            </span>
            <button
              type="button"
              onClick={() => setSuggestions([])}
              className={`cursor-pointer ${isLight ? 'text-slate-400 hover:text-slate-800' : 'text-[#64748b] hover:text-[#f8fafc]'}`}
            >
              <X size={12} />
            </button>
          </div>

          <div className="space-y-1.5 max-h-36 overflow-y-auto">
            {suggestions.map((s) => (
              <div
                key={s.deep_link}
                className={`flex items-center justify-between gap-2 p-2 rounded-lg border text-xs ${
                  isLight ? 'bg-white border-slate-200 shadow-xs' : 'bg-[#13161f] border-[#222736]'
                }`}
              >
                <div className="min-w-0">
                  <div className="font-mono text-[10px] font-semibold text-emerald-500">
                    Match {Math.round(s.similarity_score * 100)}% affinity
                  </div>
                  <strong className={`font-sans text-xs truncate block ${isLight ? 'text-slate-900' : 'text-[#e2e8f0]'}`}>
                    {s.deep_link}
                  </strong>
                </div>
                <button
                  type="button"
                  onClick={() => handleApplySuggestion(s)}
                  className="inline-flex items-center gap-1 h-6 px-2 rounded-md bg-indigo-600 hover:bg-indigo-500 text-white text-[10.5px] font-medium transition-colors cursor-pointer shrink-0"
                >
                  <Plus size={10} weight="bold" />
                  <span>Connect</span>
                </button>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Editor Body with Wikilink Autocomplete */}
      <div className={`flex-1 flex overflow-hidden min-h-0 relative ${isLight ? 'bg-white' : 'bg-[#0d0f14]'}`}>
        {(viewMode === 'edit' || viewMode === 'split') && (
          <div className={`relative h-full flex flex-col ${viewMode === 'split' ? 'w-1/2' : 'w-full'}`}>
            <textarea
              ref={textareaRef}
              className={`w-full h-full p-4 font-mono text-xs bg-transparent resize-none focus:outline-none border-r leading-relaxed ${
                isLight
                  ? 'text-slate-800 border-slate-200 placeholder-slate-400 selection:bg-indigo-100'
                  : 'text-[#e2e8f0] border-[#1f232d] placeholder-[#64748b] selection:bg-indigo-950'
              }`}
              value={content}
              onChange={handleContentChange}
              onKeyDown={handleEditorKeyDown}
              onClick={handleEditorSelectionChange}
              onKeyUp={handleEditorSelectionChange}
              placeholder="Write in Markdown... Type [[ to trigger autocomplete for notes or sections and add #tags."
              spellCheck={false}
            />

            {/* Floating Autocomplete Dropdown */}
            {autocomplete.isOpen && autocompleteCandidates.length > 0 && (
              <div className={`absolute left-4 bottom-4 z-40 w-80 max-h-60 rounded-xl border shadow-2xl backdrop-blur-xl overflow-hidden flex flex-col transition-all ${
                isLight ? 'bg-white/95 border-slate-300' : 'bg-[#10131d]/95 border-[#2b334a]'
              }`}>
                <div className={`flex items-center justify-between px-3 py-1.5 border-b text-[10.5px] font-semibold ${
                  isLight ? 'bg-slate-100 border-slate-200 text-slate-700' : 'bg-[#141824] border-[#1e2436] text-[#94a3b8]'
                }`}>
                  <span className="flex items-center gap-1.5 text-indigo-500">
                    <LinkSimple size={12} weight="bold" />
                    <span>Autocomplete [[Wikilink]]</span>
                  </span>
                  <span className="font-mono text-[9px] text-[#64748b]">↑↓ navigate • ↵ Tab insert</span>
                </div>

                <div className={`overflow-y-auto max-h-48 divide-y ${isLight ? 'divide-slate-100' : 'divide-[#181d2a]'}`}>
                  {autocompleteCandidates.map((item, idx) => {
                    const isSelected = idx === autocomplete.selectedIndex;
                    return (
                      <div
                        key={item.id}
                        onMouseDown={(e) => {
                          e.preventDefault(); // Prevent blur on textarea
                          applyAutocomplete(item);
                        }}
                        onMouseEnter={() =>
                          setAutocomplete((prev) => ({ ...prev, selectedIndex: idx }))
                        }
                        className={`px-3 py-2 text-xs flex items-center justify-between gap-2 cursor-pointer transition-colors ${
                          isSelected
                            ? isLight
                              ? 'bg-indigo-50 text-indigo-950 font-semibold'
                              : 'bg-indigo-600/20 text-[#f8fafc]'
                            : isLight
                            ? 'text-slate-700 hover:bg-slate-50'
                            : 'text-[#cbd5e1] hover:bg-[#151926]'
                        }`}
                      >
                        <div className="flex items-center gap-2 min-w-0">
                          {item.type === 'section' ? (
                            <Hash size={13} className="text-amber-500 shrink-0" />
                          ) : (
                            <FileText size={13} className="text-indigo-500 shrink-0" />
                          )}
                          <div className="min-w-0">
                            <div className="font-medium truncate flex items-center gap-1.5">
                              <span>{item.type === 'section' ? item.heading : item.title}</span>
                              {item.tag && (
                                <span className={`font-mono text-[9.5px] px-1.5 py-0.2 rounded border ${
                                  isLight ? 'bg-slate-100 text-slate-600 border-slate-200' : 'bg-[#1c2233] text-[#94a3b8] border-[#2b334a]'
                                }`}>
                                  #{item.tag}
                                </span>
                              )}
                            </div>
                            {item.preview && (
                              <div className="text-[10px] text-[#64748b] truncate mt-0.5 font-sans">
                                {item.preview}
                              </div>
                            )}
                          </div>
                        </div>

                        {isSelected && (
                          <span className={`text-[10px] font-mono px-1.5 py-0.5 rounded border shrink-0 ${
                            isLight ? 'bg-indigo-100 border-indigo-200 text-indigo-700' : 'bg-indigo-500/20 border-indigo-500/30 text-indigo-300'
                          }`}>
                            ↵ Enter
                          </span>
                        )}
                      </div>
                    );
                  })}
                </div>
              </div>
            )}
          </div>
        )}

        {(viewMode === 'preview' || viewMode === 'split') && (
          <div
            className={`h-full p-4 overflow-y-auto preview-content ${
              viewMode === 'split' ? (isLight ? 'w-1/2 bg-slate-50/40' : 'w-1/2 bg-[#090a0f]') : 'w-full'
            }`}
            dangerouslySetInnerHTML={{ __html: renderedHtml }}
            onClick={handlePreviewClick}
          />
        )}
      </div>

      {/* Connections Explorer (Footer) */}
      {note && (
        <div className={`p-3 border-t space-y-2 text-xs transition-colors ${
          isLight ? 'bg-slate-50/80 border-slate-200' : 'bg-[#111318] border-[#1f232d]'
        }`}>
          {note.backlinks && note.backlinks.length > 0 && (
            <div>
              <div className="flex items-center gap-1.5 text-[10.5px] font-bold text-[#64748b] uppercase tracking-[0.5px] mb-1">
                <ArrowLeft size={10} />
                <span>Backlinks to this note ({note.backlinks.length})</span>
              </div>
              <div className="flex items-center gap-1.5 flex-wrap">
                {note.backlinks.map((b) => (
                  <button
                    key={b}
                    type="button"
                    onClick={() => onNavigateNote(b)}
                    className={`px-2 py-0.5 rounded font-mono text-[10.5px] transition-colors cursor-pointer border ${
                      isLight
                        ? 'bg-white hover:bg-slate-100 border-slate-200 text-sky-700'
                        : 'bg-[#13161f] hover:bg-[#1a1e28] border-[#1e222d] hover:border-[#2c3242] text-[#38bdf8]'
                    }`}
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
                <span>Outgoing links in this note ({note.outgoing_links.length})</span>
              </div>
              <div className="flex items-center gap-1.5 flex-wrap">
                {note.outgoing_links.map((l) => (
                  <button
                    key={l}
                    type="button"
                    onClick={() => onNavigateNote(l)}
                    className={`px-2 py-0.5 rounded font-mono text-[10.5px] transition-colors cursor-pointer border ${
                      isLight
                        ? 'bg-white hover:bg-slate-100 border-slate-200 text-sky-700'
                        : 'bg-[#13161f] hover:bg-[#1a1e28] border-[#1e222d] hover:border-[#2c3242] text-[#38bdf8]'
                    }`}
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
