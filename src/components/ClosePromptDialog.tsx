import React from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { Tray } from '@phosphor-icons/react';
import type { CloseBehavior } from '../services/api';

interface ClosePromptDialogProps {
  open: boolean;
  onChoose: (behavior: CloseBehavior) => void;
  onCancel: () => void;
}

export const ClosePromptDialog: React.FC<ClosePromptDialogProps> = ({
  open,
  onChoose,
  onCancel,
}) => (
  <Dialog.Root open={open} onOpenChange={(o) => !o && onCancel()}>
    <Dialog.Portal>
      <Dialog.Overlay className="fixed inset-0 z-[60] gf-overlay backdrop-blur-sm" />
      <Dialog.Content className="fixed left-1/2 top-1/2 z-[60] w-full max-w-sm -translate-x-1/2 -translate-y-1/2 rounded-xl gf-dialog shadow-2xl p-5 focus:outline-none text-ink">
        <div className="flex items-start gap-3">
          <div className="flex items-center justify-center w-8 h-8 rounded-full bg-accent/15 text-accent shrink-0">
            <Tray size={18} weight="fill" />
          </div>
          <div className="min-w-0">
            <Dialog.Title className="text-sm font-semibold text-ink">Minimize to tray?</Dialog.Title>
            <Dialog.Description className="mt-1 text-xs text-muted leading-relaxed">
              Keeps the MCP server running for your AI agents. You can change this in Settings.
            </Dialog.Description>
          </div>
        </div>
        <div className="mt-5 flex justify-end gap-2">
          <button type="button" onClick={() => onChoose('quit')} className="gf-btn">
            Quit
          </button>
          <button
            type="button"
            onClick={() => onChoose('tray')}
            className="gf-btn-primary"
            autoFocus
          >
            Minimize
          </button>
        </div>
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
);
