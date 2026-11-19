import React from 'react';
import { FileText, FileCode, X } from '@phosphor-icons/react';

export interface DocTab {
  key: string;
  type: 'note' | 'code';
  label: string;
}

interface DocTabsProps {
  tabs: DocTab[];
  activeKey: string | null;
  onSelect: (tab: DocTab) => void;
  onClose: (tab: DocTab) => void;
  onShowGraph: () => void;
}

export const DocTabs: React.FC<DocTabsProps> = ({
  tabs,
  activeKey,
  onSelect,
  onClose,
  onShowGraph,
}) => {
  const base =
    'group flex items-center gap-1.5 px-3 h-8 text-[11px] border-r border-line cursor-pointer transition-colors whitespace-nowrap shrink-0';
  return (
    <div
      role="tablist"
      aria-label="Open documents"
      className="flex items-stretch h-8 border-b border-line bg-surface overflow-x-auto shrink-0"
    >
      <button
        type="button"
        role="tab"
        aria-selected={activeKey === null}
        onClick={onShowGraph}
        className={`${base} ${activeKey === null ? 'bg-canvas text-ink' : 'text-muted hover:text-ink'}`}
      >
        <span className="font-mono text-[10px] text-faint">◈</span>
        <span>Grafo</span>
      </button>
      {tabs.map((tab) => {
        const isActive = activeKey === tab.key;
        return (
          <div
            key={tab.key}
            role="tab"
            aria-selected={isActive}
            onClick={() => onSelect(tab)}
            className={`${base} ${isActive ? 'bg-canvas text-ink' : 'text-muted hover:text-ink hover:bg-raise'}`}
          >
            {tab.type === 'note' ? (
              <FileText size={12} className="shrink-0 text-faint" />
            ) : (
              <FileCode size={12} className="shrink-0 text-sky-400" />
            )}
            <span className="max-w-36 truncate">{tab.label}</span>
            <button
              type="button"
              aria-label={`Close ${tab.label}`}
              onClick={(e) => {
                e.stopPropagation();
                onClose(tab);
              }}
              className="p-0.5 rounded opacity-0 group-hover:opacity-100 hover:bg-line text-faint hover:text-ink transition-opacity cursor-pointer"
            >
              <X size={10} />
            </button>
          </div>
        );
      })}
    </div>
  );
};
