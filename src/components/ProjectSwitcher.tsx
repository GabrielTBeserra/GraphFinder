import React from 'react';
import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import { Folder, FolderPlus, CaretDown, Check, Database } from '@phosphor-icons/react';
import type { ProjectInfo } from '../types';

interface ProjectSwitcherProps {
  projects: ProjectInfo[];
  activeProject: ProjectInfo | null;
  onSwitchProject: (id: string) => Promise<void>;
  onOpenNewProjectModal: () => void;
}

export const ProjectSwitcher: React.FC<ProjectSwitcherProps> = ({
  projects,
  activeProject,
  onSwitchProject,
  onOpenNewProjectModal,
}) => {
  const displayName = activeProject ? activeProject.name : 'Selecionar Projeto';
  const noteCount = activeProject ? activeProject.note_count : 0;

  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        <button
          type="button"
          className="flex items-center gap-2.5 px-3 py-1.5 rounded-md bg-zinc-900/80 hover:bg-zinc-800/90 border border-zinc-800/90 hover:border-zinc-700/80 transition-all cursor-pointer text-left focus:outline-none focus:ring-1 focus:ring-indigo-500/50"
          title="Alternar cofre de notas / projeto ativo"
        >
          <div className="flex items-center justify-center w-5 h-5 rounded bg-zinc-800/80 text-zinc-400">
            <Folder size={13} weight="regular" />
          </div>

          <div className="flex flex-col">
            <span className="text-[10px] font-medium uppercase tracking-wider text-zinc-500 leading-none">
              Cofre Ativo
            </span>
            <span className="text-xs font-semibold text-zinc-200 leading-tight truncate max-w-[140px]">
              {displayName}
            </span>
          </div>

          <span className="text-[10px] font-mono px-1.5 py-0.5 rounded bg-zinc-800/60 text-zinc-400 border border-zinc-700/40 ml-1">
            {noteCount} {noteCount === 1 ? 'nota' : 'notas'}
          </span>

          <CaretDown size={12} weight="bold" className="text-zinc-500 ml-0.5" />
        </button>
      </DropdownMenu.Trigger>

      <DropdownMenu.Portal>
        <DropdownMenu.Content
          align="center"
          sideOffset={6}
          className="z-50 w-80 rounded-lg bg-zinc-900/95 border border-zinc-800 p-1.5 shadow-2xl backdrop-blur-md animate-in fade-in-0 zoom-in-95"
        >
          {/* Header */}
          <div className="px-2.5 py-2 border-b border-zinc-800/70 mb-1">
            <div className="flex items-center gap-1.5 text-zinc-200">
              <Database size={13} weight="bold" className="text-zinc-400" />
              <span className="text-xs font-semibold">Cofres & Projetos</span>
            </div>
            <p className="text-[11px] text-zinc-400 mt-0.5">
              Bases Markdown e vetores FastEmbed isolados
            </p>
          </div>

          {/* Project List */}
          <div className="max-h-60 overflow-y-auto space-y-0.5 py-0.5">
            {projects.map((proj) => {
              const isActive = activeProject?.id === proj.id;
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
                      ? 'bg-indigo-950/40 text-zinc-100 border border-indigo-800/40'
                      : 'hover:bg-zinc-800/60 text-zinc-300'
                  }`}
                >
                  <div className="flex items-center gap-2.5 min-w-0 pr-2">
                    <div
                      className={`flex items-center justify-center w-6 h-6 rounded shrink-0 ${
                        isActive
                          ? 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/30'
                          : 'bg-zinc-800/70 text-zinc-400 border border-zinc-700/40'
                      }`}
                    >
                      <Folder size={13} weight={isActive ? 'fill' : 'regular'} />
                    </div>

                    <div className="min-w-0">
                      <div className="flex items-center gap-1.5">
                        <span className="text-xs font-medium text-zinc-200 truncate">
                          {proj.name}
                        </span>
                        {proj.id === 'default' && (
                          <span className="text-[9px] font-mono px-1 py-0.2 rounded bg-zinc-800 text-zinc-400 border border-zinc-700/50">
                            Padrao
                          </span>
                        )}
                      </div>
                      <span className="block text-[10px] font-mono text-zinc-400 truncate max-w-[180px]">
                        {proj.path}
                      </span>
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5 shrink-0">
                    <span className="text-[10px] font-mono text-zinc-400">
                      {proj.note_count}
                    </span>
                    {isActive && (
                      <Check size={13} weight="bold" className="text-indigo-400" />
                    )}
                  </div>
                </DropdownMenu.Item>
              );
            })}
          </div>

          {/* Footer Create Action */}
          <div className="border-t border-zinc-800/70 pt-1 mt-1">
            <DropdownMenu.Item
              onSelect={onOpenNewProjectModal}
              className="flex items-center gap-2 w-full px-2.5 py-2 rounded-md text-xs font-medium text-emerald-400 hover:text-emerald-300 hover:bg-emerald-950/30 transition-colors cursor-pointer select-none outline-none"
            >
              <FolderPlus size={14} weight="bold" />
              <span>Criar Novo Projeto / Vault</span>
            </DropdownMenu.Item>
          </div>
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
};
