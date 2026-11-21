import React, { useState } from 'react';
import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import {
  Folder,
  FolderPlus,
  List,
  Check,
  Database,
  DownloadSimple,
  CircleNotch,
  WarningCircle,
} from '@phosphor-icons/react';
import type { ProjectInfo } from '../types';
import { installAgentKit } from '../services/api';

interface ProjectSwitcherProps {
  projects: ProjectInfo[];
  activeProject: ProjectInfo | null;
  onSwitchProject: (id: string) => Promise<void>;
  onOpenNewProjectModal: () => void;
  theme?: 'dark' | 'light';
}

export const ProjectSwitcher: React.FC<ProjectSwitcherProps> = ({
  projects,
  activeProject,
  onSwitchProject,
  onOpenNewProjectModal,
}) => {
  const displayName = activeProject ? activeProject.name : 'Default';
  const [installingId, setInstallingId] = useState<string | null>(null);
  const [installStatus, setInstallStatus] = useState<{
    id: string;
    kind: 'ok' | 'err';
    message: string;
  } | null>(null);

  const handleInstallKit = async (e: React.MouseEvent, proj: ProjectInfo) => {
    e.preventDefault();
    e.stopPropagation();
    if (installingId) return;
    setInstallingId(proj.id);
    setInstallStatus(null);
    try {
      const res = await installAgentKit(proj.id);
      setInstallStatus({ id: proj.id, kind: 'ok', message: res.target });
    } catch (err) {
      console.error('Agent kit install failed:', err);
      setInstallStatus({
        id: proj.id,
        kind: 'err',
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setInstallingId(null);
      setTimeout(() => {
        setInstallStatus((s) => (s && s.id === proj.id ? null : s));
      }, 5000);
    }
  };

  return (
    <div className="flex items-center gap-1.5 min-w-0">
      {/* Static vault label */}
      <span
        className="flex items-center gap-1.5 text-[11px] font-medium min-w-0"
        title={activeProject?.path || displayName}
      >
        <span className="w-2 h-2 rounded-full bg-good shrink-0" />
        <span className="text-muted shrink-0">vaults /</span>
        <span className="font-semibold truncate max-w-[110px] text-ink">
          {displayName}
        </span>
      </span>

      {/* Hamburger actions menu (VS Code-style) */}
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild>
          <button
            type="button"
            className={`flex items-center justify-center w-6 h-6 rounded-md transition-all cursor-pointer shrink-0 focus:outline-none focus:ring-1 focus:ring-accent/40 ${
              'text-muted hover:bg-raise hover:text-ink'
            }`}
            title="Vault actions — switch vault, install agent kit, new vault"
            aria-label="Vault actions menu"
          >
            <List size={14} weight="bold" />
          </button>
        </DropdownMenu.Trigger>

      <DropdownMenu.Portal>
        <DropdownMenu.Content
          align="end"
          sideOffset={6}
          className={`z-50 w-80 rounded-lg p-1.5 shadow-2xl backdrop-blur-md animate-in fade-in-0 zoom-in-95 border ${
            'bg-raise border-line'
          }`}
        >
          {/* Header */}
          <div
            className={`px-2.5 py-2 border-b mb-1 ${
              'border-line'
            }`}
          >
            <div
              className={`flex items-center gap-1.5 ${
                'text-ink'
              }`}
            >
              <Database
                size={13}
                weight="bold"
                className={'text-muted'}
              />
              <span className="text-xs font-semibold">Knowledge Vaults</span>
            </div>
            <p className={`text-[11px] mt-0.5 ${'text-muted'}`}>
              Local Markdown vaults with independent FastEmbed vector indices.
              Use <DownloadSimple size={10} className="inline -mt-0.5" /> to
              install agent skills + rules into a folder.
            </p>
          </div>

          {/* Project List */}
          <div className="max-h-60 overflow-y-auto space-y-0.5 py-0.5">
            {projects.map((proj) => {
              const isActive = (activeProject?.id || '').toLowerCase() === proj.id.toLowerCase();
              return (
                <DropdownMenu.Item
                  key={proj.id}
                  onSelect={() => {
                    if (!isActive) {
                      onSwitchProject(proj.id);
                    }
                  }}
                  className={`flex items-center justify-between w-full px-2.5 py-2 rounded-md text-left transition-colors cursor-pointer select-none outline-none ${
                    isActive
                      ? 'bg-raise text-ink border border-line-strong'
                      : 'hover:bg-raise text-muted'
                  }`}
                >
                  <div className="flex items-center gap-2.5 min-w-0 pr-2">
                    <div
                      className={`flex items-center justify-center w-6 h-6 rounded shrink-0 ${
                        isActive
                          ? 'bg-accent/20 text-accent border border-accent-line'
                          : 'bg-raise text-muted border border-line'
                      }`}
                    >
                      <Folder size={13} weight={isActive ? 'fill' : 'regular'} />
                    </div>

                    <div className="min-w-0">
                      <div className="flex items-center gap-1.5">
                        <span
                          className={`text-xs font-medium truncate ${
                            isActive ? 'text-accent-strong font-semibold' : 'text-ink'
                          }`}
                        >
                          {proj.name}
                        </span>
                        {proj.id === 'default' && (
                          <span
                            className={`text-[10px] font-mono px-1 py-0.2 rounded border ${
                              'bg-line text-muted border-line-strong'
                            }`}
                          >
                            Default
                          </span>
                        )}
                      </div>
                      <span
                        className={`block text-[10px] font-mono truncate max-w-[180px] ${
                          'text-muted'
                        }`}
                      >
                        {proj.path}
                      </span>
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5 shrink-0">
                    <button
                      type="button"
                      onClick={(e) => handleInstallKit(e, proj)}
                      disabled={installingId === proj.id}
                      className={`flex items-center justify-center w-6 h-6 rounded border transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-wait ${
                        'border-line text-muted hover:text-accent hover:border-accent-line hover:bg-accent/10'
                      }`}
                      title={`Install agent skills + AGENTS.md rules into ${proj.path}`}
                      aria-label={`Install agent kit into ${proj.name}`}
                    >
                      {installingId === proj.id ? (
                        <CircleNotch size={12} className="animate-spin" />
                      ) : installStatus?.id === proj.id ? (
                        installStatus.kind === 'ok' ? (
                          <Check size={12} weight="bold" className="text-good" />
                        ) : (
                          <WarningCircle size={12} weight="bold" className="text-bad" />
                        )
                      ) : (
                        <DownloadSimple size={12} />
                      )}
                    </button>
                    <span
                      className={`text-[10px] font-mono ${
                        'text-muted'
                      }`}
                    >
                      {proj.note_count}
                      {(proj.code_count ?? 0) > 0 ? ` +${proj.code_count}` : ''}
                    </span>
                    {isActive && (
                      <Check
                        size={13}
                        weight="bold"
                        className={'text-accent-strong'}
                      />
                    )}
                  </div>
                </DropdownMenu.Item>
              );
            })}
          </div>

          {/* Install feedback */}
          {installStatus && (
            <div
              className={`mx-1 mt-1 px-2 py-1.5 rounded-md text-[10px] font-mono truncate border ${
                installStatus.kind === 'ok'
                  ? 'text-good bg-good/10 border-good/30'
                  : 'text-bad bg-bad/10 border-bad/30'
              }`}
            >
              {installStatus.kind === 'ok'
                ? `Agent kit installed → ${installStatus.message}`
                : `Install failed: ${installStatus.message}`}
            </div>
          )}

          {/* Footer Create Action */}
          <div
            className={`border-t pt-1 mt-1 ${
              'border-line'
            }`}
          >
            <DropdownMenu.Item
              onSelect={() => setTimeout(onOpenNewProjectModal, 0)}
              className={`flex items-center gap-2 w-full px-2.5 py-2 rounded-md text-xs font-medium transition-colors cursor-pointer select-none outline-none ${
                'text-good hover:text-good hover:bg-good/15'
              }`}
            >
              <FolderPlus size={14} weight="bold" />
              <span>New Vault / Choose Folder</span>
            </DropdownMenu.Item>
          </div>
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
      </DropdownMenu.Root>
    </div>
  );
};
