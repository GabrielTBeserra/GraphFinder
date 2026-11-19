import React from 'react';
import {
  SidebarSimple,
  MagnifyingGlass,
  Plus,
  ArrowClockwise,
  Cpu,
  Gear,
} from '@phosphor-icons/react';

interface ActivityBarProps {
  sidebarOpen: boolean;
  isSyncing: boolean;
  hasUpdate?: boolean;
  onToggleSidebar: () => void;
  onSearch: () => void;
  onNewNote: () => void;
  onSync: () => void;
  onOpenMcp: () => void;
  onOpenReleases: () => void;
}

export const ActivityBar: React.FC<ActivityBarProps> = ({
  sidebarOpen,
  isSyncing,
  hasUpdate = false,
  onToggleSidebar,
  onSearch,
  onNewNote,
  onSync,
  onOpenMcp,
  onOpenReleases,
}) => {
  const item = (
    icon: React.ReactNode,
    label: string,
    onClick: () => void,
    active = false,
  ) => (
    <button
      type="button"
      onClick={onClick}
      aria-label={label}
      title={label}
      className={`relative w-full h-11 flex items-center justify-center transition-colors cursor-pointer ${
        active
          ? 'text-accent before:absolute before:left-0 before:top-2 before:bottom-2 before:w-[2px] before:bg-accent before:rounded-r'
          : 'text-faint hover:text-ink'
      }`}
    >
      {icon}
    </button>
  );

  return (
    <nav
      aria-label="Activity bar"
      className="w-12 h-full flex flex-col items-center border-r border-line bg-surface shrink-0 select-none z-10"
    >
      {item(<SidebarSimple size={18} weight={sidebarOpen ? 'fill' : 'regular'} />, 'Explorer (Ctrl+B)', onToggleSidebar, sidebarOpen)}
      {item(<MagnifyingGlass size={18} />, 'Search (Ctrl+K)', onSearch)}
      {item(<Plus size={18} weight="bold" />, 'New note', onNewNote)}
      {item(<ArrowClockwise size={18} className={isSyncing ? 'animate-spin' : undefined} />, 'Synchronize vault', onSync)}
      <div className="mt-auto" />
      {item(<Cpu size={18} />, 'MCP connection', onOpenMcp)}
      <button
        type="button"
        onClick={onOpenReleases}
        aria-label="Releases"
        title={hasUpdate ? 'Releases — update available' : 'Releases'}
        className="relative w-full h-11 flex items-center justify-center text-faint hover:text-ink transition-colors cursor-pointer"
      >
        <Gear size={18} />
        {hasUpdate && (
          <span className="absolute top-2.5 right-2.5 size-2 rounded-full bg-good border border-surface" />
        )}
      </button>
    </nav>
  );
};
