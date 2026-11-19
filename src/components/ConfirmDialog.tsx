import React from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { WarningCircle } from '@phosphor-icons/react';

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  description: string;
  confirmLabel?: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export const ConfirmDialog: React.FC<ConfirmDialogProps> = ({
  open,
  title,
  description,
  confirmLabel = 'Delete',
  onConfirm,
  onCancel,
}) => (
  <Dialog.Root open={open} onOpenChange={(o) => !o && onCancel()}>
    <Dialog.Portal>
      <Dialog.Overlay className="fixed inset-0 z-[60] gf-overlay backdrop-blur-sm" />
      <Dialog.Content className="fixed left-1/2 top-1/2 z-[60] w-full max-w-sm -translate-x-1/2 -translate-y-1/2 rounded-xl gf-dialog shadow-2xl p-5 focus:outline-none text-ink">
        <div className="flex items-start gap-3">
          <div className="flex items-center justify-center w-8 h-8 rounded-full bg-bad/15 text-bad shrink-0">
            <WarningCircle size={18} weight="fill" />
          </div>
          <div className="min-w-0">
            <Dialog.Title className="text-sm font-semibold text-ink">{title}</Dialog.Title>
            <Dialog.Description className="mt-1 text-xs text-muted leading-relaxed">
              {description}
            </Dialog.Description>
          </div>
        </div>
        <div className="mt-5 flex justify-end gap-2">
          <button
            type="button"
            onClick={onCancel}
            className="gf-btn"
            autoFocus
          >
            Cancel
          </button>
          <button
            type="button"
            onClick={onConfirm}
            className="inline-flex items-center gap-1.5 px-3 py-1.5 rounded-md text-xs font-semibold transition-colors cursor-pointer bg-bad text-canvas hover:opacity-90"
          >
            {confirmLabel}
          </button>
        </div>
      </Dialog.Content>
    </Dialog.Portal>
  </Dialog.Root>
);
