import React from 'react';
import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import { Folder, FolderPlus, CaretDown, Check, Database } from '@phosphor-icons/react';
import type { ProjectInfo } from '../types';

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
  theme = 'dark',
}) => {
  const isLight = theme === 'light';
  const displayName = activeProject ? activeProject.name : 'Default';

  return (
    <DropdownMenu.Root>
      <DropdownMenu.Trigger asChild>
        <button
          type="button"
          className={`flex items-center gap-2 h-7 px-3 rounded-md border transition-all cursor-pointer text-left focus:outline-none focus:ring-1 focus:ring-indigo-500/40 ${
            isLight
              ? 'bg-slate-100/90 hover:bg-slate-200/80 border-slate-300 text-slate-800'
              : 'bg-[#161922] hover:bg-[#1a1e28] border-[#262a36] hover:border-[#333947] text-[#cbd5e1]'
          }`}
          title="Switch active knowledge vault"
        >
          {/* Emerald Status Dot from banner.svg */}
          <span className="w-2 h-2 rounded-full bg-[#10b981] shrink-0" />

          {/* Path notation: vaults / name */}
          <div className="flex items-center gap-1 text-[11px] font-medium">
            <span className={isLight ? 'text-slate-400' : 'text-[#64748b]'}>vaults /</span>
            <span
              className={`font-semibold truncate max-w-[150px] ${
                isLight ? 'text-slate-900' : 'text-[#f1f5f9]'
              }`}
            >
              {displayName}
            </span>
          </div>

          <CaretDown
            size={11}
            weight="bold"
            className={`ml-1 ${isLight ? 'text-slate-400' : 'text-[#64748b]'}`}
          />
        </button>
      </DropdownMenu.Trigger>

      <DropdownMenu.Portal>
        <DropdownMenu.Content
          align="center"
          sideOffset={6}
          className={`z-50 w-80 rounded-lg p-1.5 shadow-2xl backdrop-blur-md animate-in fade-in-0 zoom-in-95 border ${
            isLight
              ? 'bg-white border-slate-200 shadow-xl'
              : 'bg-[#12151c] border-[#1f232d]'
          }`}
        >
          {/* Header */}
          <div
            className={`px-2.5 py-2 border-b mb-1 ${
              isLight ? 'border-slate-100' : 'border-[#1f232d]'
            }`}
          >
            <div
              className={`flex items-center gap-1.5 ${
                isLight ? 'text-slate-900' : 'text-[#f1f5f9]'
              }`}
            >
              <Database
                size={13}
                weight="bold"
                className={isLight ? 'text-slate-400' : 'text-[#64748b]'}
              />
              <span className="text-xs font-semibold">Knowledge Vaults</span>
            </div>
            <p className={`text-[11px] mt-0.5 ${isLight ? 'text-slate-500' : 'text-[#64748b]'}`}>
              Local Markdown vaults with independent FastEmbed vector indices
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
                    onSwitchProject(proj.id);
                  }}
                  onClick={() => {
                    onSwitchProject(proj.id);
                  }}
                  className={`flex items-center justify-between w-full px-2.5 py-2 rounded-md text-left transition-colors cursor-pointer select-none outline-none ${
                    isActive
                      ? isLight
                        ? 'bg-indigo-50 text-indigo-950 border border-indigo-200'
                        : 'bg-[#1a1e28] text-[#f8fafc] border border-[#2c3242]'
                      : isLight
                      ? 'hover:bg-slate-50 text-slate-700'
                      : 'hover:bg-[#161922] text-[#94a3b8]'
                  }`}
                >
                  <div className="flex items-center gap-2.5 min-w-0 pr-2">
                    <div
                      className={`flex items-center justify-center w-6 h-6 rounded shrink-0 ${
                        isActive
                          ? isLight
                            ? 'bg-indigo-100 text-indigo-600 border border-indigo-200'
                            : 'bg-indigo-600/20 text-indigo-400 border border-indigo-500/30'
                          : isLight
                          ? 'bg-slate-100 text-slate-500 border border-slate-200'
                          : 'bg-[#161922] text-[#64748b] border border-[#262a36]'
                      }`}
                    >
                      <Folder size={13} weight={isActive ? 'fill' : 'regular'} />
                    </div>

                    <div className="min-w-0">
                      <div className="flex items-center gap-1.5">
                        <span
                          className={`text-xs font-medium truncate ${
                            isLight
                              ? isActive
                                ? 'text-indigo-950 font-semibold'
                                : 'text-slate-800'
                              : 'text-[#f1f5f9]'
                          }`}
                        >
                          {proj.name}
                        </span>
                        {proj.id === 'default' && (
                          <span
                            className={`text-[9px] font-mono px-1 py-0.2 rounded border ${
                              isLight
                                ? 'bg-slate-100 text-slate-600 border-slate-200'
                                : 'bg-[#1e222d] text-[#94a3b8] border-[#2d3342]'
                            }`}
                          >
                            Default
                          </span>
                        )}
                      </div>
                      <span
                        className={`block text-[10px] font-mono truncate max-w-[180px] ${
                          isLight ? 'text-slate-400' : 'text-[#64748b]'
                        }`}
                      >
                        {proj.path}
                      </span>
                    </div>
                  </div>

                  <div className="flex items-center gap-1.5 shrink-0">
                    <span
                      className={`text-[10px] font-mono ${
                        isLight ? 'text-slate-400' : 'text-[#64748b]'
                      }`}
                    >
                      {proj.note_count}
                    </span>
                    {isActive && (
                      <Check
                        size={13}
                        weight="bold"
                        className={isLight ? 'text-indigo-600' : 'text-indigo-400'}
                      />
                    )}
                  </div>
                </DropdownMenu.Item>
              );
            })}
          </div>

          {/* Footer Create Action */}
          <div
            className={`border-t pt-1 mt-1 ${
              isLight ? 'border-slate-100' : 'border-[#1f232d]'
            }`}
          >
            <DropdownMenu.Item
              onSelect={onOpenNewProjectModal}
              className={`flex items-center gap-2 w-full px-2.5 py-2 rounded-md text-xs font-medium transition-colors cursor-pointer select-none outline-none ${
                isLight
                  ? 'text-emerald-700 hover:text-emerald-800 hover:bg-emerald-50'
                  : 'text-[#10b981] hover:text-emerald-300 hover:bg-emerald-950/20'
              }`}
            >
              <FolderPlus size={14} weight="bold" />
              <span>New Vault / Choose Folder</span>
            </DropdownMenu.Item>
          </div>
        </DropdownMenu.Content>
      </DropdownMenu.Portal>
    </DropdownMenu.Root>
  );
};
