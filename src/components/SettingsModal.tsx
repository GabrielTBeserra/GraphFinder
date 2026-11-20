import React, { useEffect, useState } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { X } from '@phosphor-icons/react';
import { getCloseBehavior, setCloseBehavior } from '../services/api';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const SettingsModal: React.FC<SettingsModalProps> = ({ isOpen, onClose }) => {
  const [minimizeToTray, setMinimizeToTray] = useState(false);
  const [isLoaded, setIsLoaded] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    let cancelled = false;
    getCloseBehavior()
      .then((behavior) => {
        if (!cancelled) setMinimizeToTray(behavior === 'tray');
      })
      .catch((err) => console.error('Failed to read close behavior:', err))
      .finally(() => {
        if (!cancelled) setIsLoaded(true);
      });
    return () => {
      cancelled = true;
    };
  }, [isOpen]);

  const toggleMinimizeToTray = async () => {
    const next = !minimizeToTray;
    setMinimizeToTray(next);
    try {
      await setCloseBehavior(next ? 'tray' : 'quit');
    } catch (err) {
      console.error('Failed to save close behavior:', err);
      setMinimizeToTray(!next);
    }
  };

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 gf-overlay backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2 rounded-xl gf-dialog shadow-2xl p-5 focus:outline-none text-ink">
          <div className="flex items-center justify-between gap-4 pb-3 border-b border-line">
            <div>
              <Dialog.Title className="text-sm font-semibold text-ink">Settings</Dialog.Title>
              <Dialog.Description className="mt-0.5 text-xs text-muted">
                Application preferences.
              </Dialog.Description>
            </div>
            <Dialog.Close asChild>
              <button
                type="button"
                className="p-1.5 rounded-md text-muted hover:text-ink hover:bg-line transition-colors cursor-pointer"
                title="Close"
                aria-label="Close"
              >
                <X size={16} />
              </button>
            </Dialog.Close>
          </div>

          <section className="pt-4">
            <h3 className="text-[11px] font-semibold uppercase tracking-wider text-faint">General</h3>
            <div className="mt-3 flex items-start justify-between gap-4">
              <div className="min-w-0">
                <label htmlFor="setting-minimize-to-tray" className="text-xs font-medium text-ink">
                  Minimize to tray on close
                </label>
                <p className="mt-0.5 text-xs text-muted leading-relaxed">
                  Closing the window keeps GraphFinder running in the system tray, so the MCP
                  server stays available to your AI agents.
                </p>
              </div>
              <button
                id="setting-minimize-to-tray"
                type="button"
                role="switch"
                aria-checked={minimizeToTray}
                disabled={!isLoaded}
                onClick={toggleMinimizeToTray}
                className={`relative mt-0.5 inline-flex h-5 w-9 shrink-0 items-center rounded-full border border-line transition-colors cursor-pointer disabled:opacity-50 disabled:cursor-default ${
                  minimizeToTray ? 'bg-accent' : 'bg-raise'
                }`}
              >
                <span
                  className={`inline-block size-3.5 rounded-full shadow-sm transition-transform ${
                    minimizeToTray ? 'translate-x-4 bg-on-accent' : 'translate-x-0.5 bg-muted'
                  }`}
                />
              </button>
            </div>
          </section>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
