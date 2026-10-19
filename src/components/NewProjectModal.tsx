import React, { useState } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import {
  Folder,
  FolderPlus,
  FolderOpen,
  X,
  Database,
  Check,
  CheckCircle,
  HardDrive,
} from '@phosphor-icons/react';
import { pickDirectory } from '../services/api';

interface NewProjectModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (name: string, path?: string) => Promise<void>;
}

export const NewProjectModal: React.FC<NewProjectModalProps> = ({
  isOpen,
  onClose,
  onCreate,
}) => {
  const [name, setName] = useState('');
  const [customPath, setCustomPath] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const slug = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');

  const handleBrowseFolder = async () => {
    try {
      const selected = await pickDirectory();
      if (selected) {
        setCustomPath(selected);
        if (!name.trim()) {
          const parts = selected.replace(/\\/g, '/').split('/').filter(Boolean);
          if (parts.length > 0) {
            setName(parts[parts.length - 1]);
          }
        }
      }
    } catch (err) {
      console.error('Error opening folder picker:', err);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) {
      setError('Please provide a project name.');
      return;
    }

    setIsSubmitting(true);
    setError(null);
    try {
      await onCreate(name.trim(), customPath.trim() || undefined);
      setName('');
      setCustomPath('');
      onClose();
    } catch (err: unknown) {
      console.error('Error creating project:', err);
      const errMsg = err instanceof Error ? err.message : 'Failed to create new project vault.';
      setError(errMsg);
    } finally {
      setIsSubmitting(false);
    }
  };

  const effectiveVaultPath = customPath.trim() || `./vaults/${slug || 'project-name'}/`;

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/75 backdrop-blur-sm" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-lg -translate-x-1/2 -translate-y-1/2 rounded-xl bg-[#111318] border border-[#262933] p-6 shadow-2xl focus:outline-none text-[#f1f5f9]">
          {/* Header */}
          <div className="flex items-start justify-between gap-4 mb-5 pb-3 border-b border-[#1f232d]">
            <div className="flex items-center gap-3">
              <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-[#161922] border border-[#262a36] text-[#cbd5e1]">
                <FolderPlus size={16} weight="regular" />
              </div>
              <div>
                <Dialog.Title className="text-sm font-semibold text-[#f8fafc]">
                  Create or Open Knowledge Vault
                </Dialog.Title>
                <Dialog.Description className="text-xs text-[#64748b] mt-0.5">
                  Local-first storage in any directory on your computer.
                </Dialog.Description>
              </div>
            </div>

            <Dialog.Close asChild>
              <button
                type="button"
                className="rounded-md p-1.5 text-[#64748b] hover:text-[#f8fafc] hover:bg-[#1a1e28] transition-colors cursor-pointer"
                title="Close"
              >
                <X size={15} />
              </button>
            </Dialog.Close>
          </div>

          {/* Form */}
          <form onSubmit={handleSubmit} className="space-y-4">
            <div>
              <label
                htmlFor="project-name-input"
                className="block text-xs font-medium text-[#cbd5e1] mb-1.5"
              >
                Project / Vault Name
              </label>
              <input
                id="project-name-input"
                type="text"
                className="w-full px-3 py-2 text-xs rounded-md bg-[#0d0f14] border border-[#1f232d] text-[#f8fafc] placeholder-[#64748b] focus:outline-none focus:border-indigo-500/80 focus:ring-1 focus:ring-indigo-500/40 transition-all"
                placeholder="e.g., Research Notes, AI Engineering, Second Brain..."
                value={name}
                onChange={(e) => {
                  setName(e.target.value);
                  if (error) setError(null);
                }}
                autoFocus
                disabled={isSubmitting}
              />
            </div>

            {/* Folder Location Selector */}
            <div>
              <div className="flex items-center justify-between mb-1.5">
                <label
                  htmlFor="project-path-input"
                  className="text-xs font-medium text-[#cbd5e1]"
                >
                  Storage Location on Disk
                </label>
                {customPath && (
                  <button
                    type="button"
                    onClick={() => setCustomPath('')}
                    className="text-[10px] text-[#64748b] hover:text-[#cbd5e1] transition-colors cursor-pointer"
                  >
                    Restore default location
                  </button>
                )}
              </div>

              <div className="flex items-center gap-2">
                <div className="relative flex-1">
                  <input
                    id="project-path-input"
                    type="text"
                    className="w-full px-3 py-2 text-xs rounded-md bg-[#0d0f14] border border-[#1f232d] text-[#f8fafc] placeholder-[#64748b] focus:outline-none focus:border-indigo-500/80 focus:ring-1 focus:ring-indigo-500/40 transition-all font-mono"
                    placeholder="Select a folder or leave blank for default..."
                    value={customPath}
                    onChange={(e) => setCustomPath(e.target.value)}
                    disabled={isSubmitting}
                  />
                </div>

                <button
                  type="button"
                  onClick={handleBrowseFolder}
                  className="inline-flex items-center gap-1.5 px-3 py-2 text-xs font-medium rounded-md bg-[#13161f] hover:bg-[#1a1e28] text-[#cbd5e1] border border-[#1e222d] hover:border-[#2d3342] transition-colors cursor-pointer shrink-0"
                  title="Open folder picker dialog"
                >
                  <FolderOpen size={14} className="text-[#38bdf8]" />
                  <span>Choose Folder</span>
                </button>
              </div>
            </div>

            {/* Architecture Preview matching banner.svg lines 53-56 */}
            <div className="rounded-lg bg-[#12151c] border border-[#1e222d] p-3 space-y-2">
              <div className="flex items-center gap-2 text-[#cbd5e1] text-xs font-medium">
                <Database size={13} className="text-[#64748b]" />
                <span>Selected Directory:</span>
              </div>

              <div className="font-mono text-[11px] space-y-1 text-[#94a3b8]">
                <div className="flex items-center gap-1.5 truncate">
                  {customPath ? (
                    <HardDrive size={12} className="text-[#38bdf8] shrink-0" />
                  ) : (
                    <Folder size={12} className="text-[#64748b] shrink-0" />
                  )}
                  <span>Vault Folder:</span>
                  <span className="text-[#f8fafc] truncate">{effectiveVaultPath}</span>
                </div>
                <div className="flex items-center gap-1.5 truncate">
                  <Database size={12} className="text-[#64748b] shrink-0" />
                  <span>SQLite Index:</span>
                  <span className="text-[#f8fafc] truncate">{effectiveVaultPath.replace(/[\\/]$/, '')}/index.db</span>
                </div>
              </div>

              <div className="flex items-center gap-1.5 text-[11px] text-[#64748b] pt-1">
                <CheckCircle size={12} className="text-[#10b981] shrink-0" />
                <span>Existing Markdown notes in the folder will be indexed automatically.</span>
              </div>
            </div>

            {error && (
              <div className="text-xs font-medium text-rose-400 bg-rose-950/20 border border-rose-900/30 px-3 py-1.5 rounded-md">
                {error}
              </div>
            )}

            {/* Actions */}
            <div className="flex items-center justify-end gap-2 pt-2 border-t border-[#1f232d]">
              <button
                type="button"
                onClick={onClose}
                disabled={isSubmitting}
                className="px-3 py-1.5 text-xs font-medium text-[#64748b] hover:text-[#f8fafc] hover:bg-[#1a1e28] rounded-md transition-colors cursor-pointer"
              >
                Cancel
              </button>
              <button
                type="submit"
                disabled={isSubmitting || !name.trim()}
                className="inline-flex items-center gap-1.5 px-3.5 py-1.5 text-xs font-semibold rounded-md bg-indigo-600 hover:bg-indigo-500 text-white transition-all disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer shadow-sm"
              >
                <Check size={13} weight="bold" />
                <span>{isSubmitting ? 'Saving...' : 'Save Vault'}</span>
              </button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
