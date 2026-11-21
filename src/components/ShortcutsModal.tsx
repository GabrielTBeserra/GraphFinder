import React from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { X } from '@phosphor-icons/react';

interface ShortcutsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

const SHORTCUTS: { keys: string; action: string }[] = [
  { keys: 'Ctrl+K', action: 'Omnisearch — notes, code & actions' },
  { keys: 'Ctrl+B', action: 'Toggle sidebar' },
  { keys: 'Ctrl+S', action: 'Save note (in editor)' },
  { keys: 'Ctrl+/', action: 'Show this cheatsheet' },
  { keys: 'Esc', action: 'Close editor / inspector / modal' },
  { keys: '↑ ↓', action: 'Navigate results (Omnisearch)' },
  { keys: 'Enter', action: 'Open result (Omnisearch)' },
  { keys: 'Alt+Enter', action: 'Reveal node in graph (Omnisearch)' },
];

export const ShortcutsModal: React.FC<ShortcutsModalProps> = ({ isOpen, onClose }) => (
  <Dialog.Root open={isOpen} onOpenChange={(o) => !o && onClose()}>
    <Dialog.Portal>
      <Dialog.Overlay className="fixed inset-0 z-[60] gf-overlay backdrop-blur-sm" />
      <Dialog.Content
        aria-label="Keyboard shortcuts"
        className="fixed left-1/2 top-1/2 z-[60] w-full max-w-xs -translate-x-1/2 -translate-y-1/2 rounded-xl gf-dialog shadow-2xl p-4 focus:outline-none"
      >
        <div className="flex items-center justify-between mb-3">
          <Dialog.Title className="text-sm font-semibold text-ink">
            Keyboard shortcuts
          </Dialog.Title>
          <button
            type="button"
            onClick={onClose}
            aria-label="Close"
            className="p-1 rounded text-faint hover:text-ink hover:bg-raise transition-colors cursor-pointer"
          >
            <X size={13} />
          </button>
        </div>
        <div className="space-y-1">
          {SHORTCUTS.map((s) => (
            <div key={s.keys} className="flex items-center justify-between gap-4 py-1">
              <span className="text-[11px] text-muted">{s.action}</span>
              <kbd className="shrink-0 px-1.5 py-0.5 rounded border border-line bg-canvas text-[10px] font-mono text-ink">
                {s.keys}
              </kbd>
            </div>
          ))}
        </div>
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
);
