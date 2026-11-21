import React from 'react';
import * as DropdownMenu from '@radix-ui/react-dropdown-menu';
import {
  SidebarSimple,
  MagnifyingGlass,
  Plus,
  ArrowClockwise,
  Cpu,
  Gear,
  Sun,
  Moon,
  SlidersHorizontal,
} from '@phosphor-icons/react';

interface ActivityBarProps {
  sidebarOpen: boolean;
  isSyncing: boolean;
  hasUpdate?: boolean;
  theme?: 'dark' | 'light';
  onToggleTheme?: () => void;
  onToggleSidebar: () => void;
  onSearch: () => void;
  onNewNote: () => void;
  onSync: () => void;
  onOpenMcp: () => void;
  onOpenReleases: () => void;
  onOpenSettings: () => void;
}

export const ActivityBar: React.FC<ActivityBarProps> = ({
  sidebarOpen,
  isSyncing,
  hasUpdate = false,
  theme = 'dark',
  onToggleTheme,
  onToggleSidebar,
  onSearch,
  onNewNote,
  onSync,
  onOpenMcp,
  onOpenReleases,
  onOpenSettings,
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
      className={`relative w-full h-11 flex items-center justify-center transition-colors cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-accent/60 ${
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
      <DropdownMenu.Root>
        <DropdownMenu.Trigger asChild>
          <button
            type="button"
            aria-label="Manage"
            title={hasUpdate ? 'Manage — update available' : 'Manage'}
            className="relative w-full h-11 flex items-center justify-center text-faint hover:text-ink transition-colors cursor-pointer focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-inset focus-visible:ring-accent/60"
          >
            <Gear size={18} />
            {hasUpdate && (
              <span className="absolute top-2.5 right-2.5 size-2 rounded-full bg-good border border-surface" />
            )}
          </button>
        </DropdownMenu.Trigger>
        <DropdownMenu.Portal>
          <DropdownMenu.Content
            side="right"
            align="end"
            sideOffset={8}
            className="z-[70] min-w-44 rounded-lg gf-dialog shadow-2xl p-1 animate-in fade-in-0 zoom-in-95"
          >
            <DropdownMenu.Item
              onSelect={onOpenReleases}
              className="flex items-center gap-2 px-2.5 py-1.5 rounded-md text-xs text-ink cursor-pointer outline-none data-[highlighted]:bg-raise"
            >
              <span className="relative">
                Releases
                {hasUpdate && <span className="absolute -right-3 top-1 size-1.5 rounded-full bg-good" />}
              </span>
            </DropdownMenu.Item>
            <DropdownMenu.Item
              onSelect={onOpenSettings}
              className="flex items-center gap-2 px-2.5 py-1.5 rounded-md text-xs text-ink cursor-pointer outline-none data-[highlighted]:bg-raise"
            >
              <SlidersHorizontal size={13} />
              Preferences
            </DropdownMenu.Item>
            <DropdownMenu.Item
              onSelect={onToggleTheme}
              className="flex items-center gap-2 px-2.5 py-1.5 rounded-md text-xs text-ink cursor-pointer outline-none data-[highlighted]:bg-raise"
            >
              {theme === 'light' ? <Moon size={13} /> : <Sun size={13} />}
              {theme === 'light' ? 'Dark theme' : 'Light theme'}
            </DropdownMenu.Item>
          </DropdownMenu.Content>
        </DropdownMenu.Portal>
      </DropdownMenu.Root>
    </nav>
  );
};
