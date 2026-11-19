import React from 'react';
import * as Tabs from '@radix-ui/react-tabs';
import { FileText, FileCode, X } from '@phosphor-icons/react';
import type { ChunkHit, CodeChunkHit, ProjectInfo, SyncProgress } from '../types';

interface BottomPanelProps {
  open: boolean;
  chunkResults: ChunkHit[];
  codeChunkResults: CodeChunkHit[];
  isSearching: boolean;
  onSelectNote: (id: string) => void;
  onSelectCode: (path: string) => void;
  onClose: () => void;
}

export const BottomPanel: React.FC<BottomPanelProps> = ({
  open,
  chunkResults,
  codeChunkResults,
  isSearching,
  onSelectNote,
  onSelectCode,
  onClose,
}) => {
  if (!open) return null;
  return (
    <section
      aria-label="Panel"
      className="h-44 border-t border-line bg-surface shrink-0 flex flex-col"
    >
      <Tabs.Root defaultValue="results" className="flex flex-col h-full">
        <div className="flex items-center border-b border-line shrink-0">
          <Tabs.List className="flex items-center h-8">
            <Tabs.Trigger
              value="results"
              className="px-3 h-8 text-[10.5px] font-mono uppercase tracking-[0.12em] text-faint data-[state=active]:text-accent data-[state=active]:border-b-2 data-[state=active]:border-accent transition-colors cursor-pointer"
            >
              Results
            </Tabs.Trigger>
            <Tabs.Trigger
              value="log"
              className="px-3 h-8 text-[10.5px] font-mono uppercase tracking-[0.12em] text-faint data-[state=active]:text-accent data-[state=active]:border-b-2 data-[state=active]:border-accent transition-colors cursor-pointer"
            >
              Engine log
            </Tabs.Trigger>
          </Tabs.List>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close panel"
            className="ml-auto mr-2 p-1 rounded text-faint hover:text-ink hover:bg-raise transition-colors cursor-pointer"
          >
            <X size={12} />
          </button>
        </div>
        <Tabs.Content value="results" className="flex-1 overflow-y-auto p-2 space-y-1">
          {isSearching ? (
            <p className="text-[11px] text-faint py-4 text-center">Computing embeddings…</p>
          ) : chunkResults.length === 0 && codeChunkResults.length === 0 ? (
            <p className="text-[11px] text-faint py-4 text-center">
              No results — run a semantic search (Ctrl+K).
            </p>
          ) : (
            <>
              {chunkResults.map((c) => (
                <button
                  key={c.chunk_id}
                  type="button"
                  onClick={() => onSelectNote(c.note_id)}
                  className="w-full flex items-center gap-2 px-2 py-1.5 rounded text-left hover:bg-raise transition-colors cursor-pointer"
                >
                  <FileText size={12} className="text-faint shrink-0" />
                  <span className="text-[11px] text-ink truncate">{c.note_title}</span>
                  {c.heading && (
                    <span className="text-[10px] font-mono text-accent truncate">§ {c.heading}</span>
                  )}
                  <span className="ml-auto text-[10px] font-mono text-good shrink-0 tabular-nums">
                    {Math.round(c.score * 100)}%
                  </span>
                </button>
              ))}
              {codeChunkResults.map((c, i) => (
                <button
                  key={`${c.file_id}-${i}`}
                  type="button"
                  onClick={() => onSelectCode(c.file_id)}
                  className="w-full flex items-center gap-2 px-2 py-1.5 rounded text-left hover:bg-raise transition-colors cursor-pointer"
                >
                  <FileCode size={12} className="text-sky-400 shrink-0" />
                  <span className="text-[11px] font-mono text-ink truncate">
                    {c.symbol_name || c.file_name}
                  </span>
                  <span className="text-[10px] font-mono text-faint truncate">{c.file_id}</span>
                  <span className="ml-auto text-[10px] font-mono text-good shrink-0 tabular-nums">
                    {Math.round(c.score * 100)}%
                  </span>
                </button>
              ))}
            </>
          )}
        </Tabs.Content>
        <Tabs.Content value="log" className="flex-1 overflow-y-auto p-2">
          <EngineLog />
        </Tabs.Content>
      </Tabs.Root>
    </section>
  );
};

const EngineLog: React.FC = () => {
  const [entries, setEntries] = React.useState<string[]>([]);
  React.useEffect(() => {
    try {
      const raw = sessionStorage.getItem('gf_engine_log');
      if (raw) setEntries(JSON.parse(raw));
    } catch { /* noop */ }
    const handler = (e: Event) => {
      const msg = (e as CustomEvent<string>).detail;
      setEntries((prev) => {
        const next = [...prev, `${new Date().toLocaleTimeString()} ${msg}`].slice(-100);
        try { sessionStorage.setItem('gf_engine_log', JSON.stringify(next)); } catch { /* noop */ }
        return next;
      });
    };
    window.addEventListener('gf:log', handler);
    return () => window.removeEventListener('gf:log', handler);
  }, []);
  if (entries.length === 0) {
    return <p className="text-[11px] text-faint py-4 text-center">Engine is quiet. Sync events appear here.</p>;
  }
  return (
    <pre className="text-[10.5px] font-mono text-muted leading-relaxed whitespace-pre-wrap">
      {entries.join('\n')}
    </pre>
  );
};

interface StatusBarProps {
  project: ProjectInfo | null;
  noteCount: number;
  fileCount: number;
  linkCount: number;
  engineReady: boolean;
  engineError: boolean;
  engineStatusMessage: string;
  syncProgress: SyncProgress | null;
  onOpenMcp: () => void;
}

export const StatusBar: React.FC<StatusBarProps> = ({
  project,
  noteCount,
  fileCount,
  linkCount,
  engineReady,
  engineError,
  engineStatusMessage,
  syncProgress,
  onOpenMcp,
}) => (
  <footer
    aria-label="Status bar"
    className="h-6 px-3 flex items-center gap-3 border-t border-line bg-surface text-[10px] font-mono text-faint shrink-0 select-none"
  >
    <span className="flex items-center gap-1.5 truncate">
      <span
        className={`size-1.5 rounded-full ${
          engineError ? 'bg-bad' : engineReady ? 'bg-good' : 'bg-warn'
        }`}
      />
      <span className="truncate" title={engineStatusMessage}>
        {project?.name || 'default'}
      </span>
    </span>
    <span className="tabular-nums">{noteCount} notes</span>
    <span className="tabular-nums">{fileCount} files</span>
    <span className="tabular-nums">{linkCount} links</span>
    <span className="ml-auto flex items-center gap-3">
      {syncProgress && (
        <span className="tabular-nums text-accent">
          {syncProgress.percent !== null ? `${Math.round(syncProgress.percent)}%` : 'syncing…'}
        </span>
      )}
      <button
        type="button"
        onClick={onOpenMcp}
        className="flex items-center gap-1 hover:text-ink transition-colors cursor-pointer"
        aria-label="Open MCP settings"
      >
        <span className="size-1.5 rounded-full bg-good" />
        MCP
      </button>
    </span>
  </footer>
);
