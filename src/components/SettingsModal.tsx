import React, { useEffect, useState } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { X, Gear, WindowsLogo, Tray, RocketLaunch } from '@phosphor-icons/react';
import type { AppSettings } from '../types';
import { getAppSettings, updateAppSettings } from '../services/api';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
  onToast?: (message: string, kind: 'info' | 'error') => void;
}

interface ToggleRowProps {
  icon: React.ReactNode;
  title: string;
  description: string;
  checked: boolean;
  onChange: (value: boolean) => void;
}

const ToggleRow: React.FC<ToggleRowProps> = ({ icon, title, description, checked, onChange }) => (
  <button
    type="button"
    role="switch"
    aria-checked={checked}
    onClick={() => onChange(!checked)}
    className="w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-left transition-colors hover:bg-raise cursor-pointer group"
  >
    <span className="shrink-0 text-muted group-hover:text-accent transition-colors">{icon}</span>
    <span className="flex-1 min-w-0">
      <span className="block text-xs font-medium text-ink">{title}</span>
      <span className="block text-[10px] text-muted mt-0.5 leading-snug">{description}</span>
    </span>
    <span
      className={`relative shrink-0 w-8 h-4.5 rounded-full transition-colors ${
        checked ? 'bg-accent' : 'bg-line'
      }`}
    >
      <span
        className={`absolute top-0.5 size-3.5 rounded-full bg-white shadow transition-transform ${
          checked ? 'translate-x-4' : 'translate-x-0.5'
        }`}
      />
    </span>
  </button>
);

export const SettingsModal: React.FC<SettingsModalProps> = ({ isOpen, onClose, onToast }) => {
  const [settings, setSettings] = useState<AppSettings>({
    autostart: false,
    closeToTray: true,
    startMinimized: false,
  });
  const [loaded, setLoaded] = useState(false);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!isOpen) return;
    setLoaded(false);
    getAppSettings()
      .then((s) => {
        setSettings(s);
        setLoaded(true);
      })
      .catch(() => setLoaded(true));
  }, [isOpen]);

  const update = async (patch: Partial<AppSettings>) => {
    const next = { ...settings, ...patch };
    setSettings(next);
    setSaving(true);
    try {
      const saved = await updateAppSettings(next);
      setSettings(saved);
      if (patch.autostart !== undefined) {
        onToast?.(
          saved.autostart
            ? 'GraphFinder will start with Windows'
            : 'Windows autostart disabled',
          'info'
        );
      }
    } catch (e) {
      onToast?.(`Failed to save settings: ${e}`, 'error');
    } finally {
      setSaving(false);
    }
  };

  return (
    <Dialog.Root open={isOpen} onOpenChange={(o) => !o && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-[60] gf-overlay backdrop-blur-sm" />
        <Dialog.Content
          aria-label="Preferences"
          className="fixed left-1/2 top-1/2 z-[60] w-full max-w-sm -translate-x-1/2 -translate-y-1/2 rounded-xl gf-dialog shadow-2xl p-4 focus:outline-none"
        >
          <div className="flex items-center justify-between mb-3">
            <Dialog.Title className="flex items-center gap-2 text-sm font-semibold text-ink">
              <Gear size={15} className="text-accent" />
              Preferences
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

          <div className="space-y-1" aria-busy={!loaded}>
            <ToggleRow
              icon={<WindowsLogo size={16} />}
              title="Start with Windows"
              description="Launch GraphFinder automatically when you sign in."
              checked={settings.autostart}
              onChange={(v) => update({ autostart: v })}
            />
            <ToggleRow
              icon={<Tray size={16} />}
              title="Close to system tray"
              description="Closing the window keeps GraphFinder running in the tray."
              checked={settings.closeToTray}
              onChange={(v) => update({ closeToTray: v })}
            />
            <ToggleRow
              icon={<RocketLaunch size={16} />}
              title="Start minimized to tray"
              description="When launched at sign-in, stay hidden in the tray."
              checked={settings.startMinimized}
              onChange={(v) => update({ startMinimized: v })}
            />
          </div>

          <p className="mt-3 text-[10px] text-faint leading-relaxed">
            {saving
              ? 'Saving…'
              : 'Only one GraphFinder instance runs at a time — CLI commands are forwarded to the running app.'}
          </p>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
