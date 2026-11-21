import React from 'react';
import type { ProjectInfo, SyncProgress } from '../types';

interface StatusBarProps {
  project: ProjectInfo | null;
  noteCount: number;
  fileCount: number;
  linkCount: number;
  engineReady: boolean;
  engineError: boolean;
  engineStatusMessage: string;
  syncProgress: SyncProgress | null;
  mcpUp: boolean;
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
  mcpUp,
  onOpenMcp,
}) => (
  <footer
    aria-label="Status bar"
    className="h-6 px-3 flex items-center gap-3 border-t border-line bg-surface text-[10px] font-mono text-faint shrink-0 select-none"
  >
    <span className="flex items-center gap-1.5 truncate" title={engineStatusMessage}>
      <span
        className={`size-1.5 rounded-full ${
          engineError ? 'bg-bad' : engineReady ? 'bg-good' : 'bg-warn'
        }`}
      />
      <span className="truncate">
        {project?.name || 'default'}
        {!engineReady && !engineError && ' · engine loading'}
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
        title={mcpUp ? 'MCP server: online' : 'MCP server: offline'}
      >
        <span className={`size-1.5 rounded-full ${mcpUp ? 'bg-good' : 'bg-warn'}`} />
        MCP
      </button>
    </span>
  </footer>
);
