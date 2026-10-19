import React, { useState } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import { Folder, FolderPlus, X, Database, Check, CheckCircle } from '@phosphor-icons/react';

interface NewProjectModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreate: (name: string) => Promise<void>;
}

export const NewProjectModal: React.FC<NewProjectModalProps> = ({
  isOpen,
  onClose,
  onCreate,
}) => {
  const [name, setName] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const slug = name
    .trim()
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '');

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) {
      setError('Por favor, informe o nome do projeto.');
      return;
    }

    setIsSubmitting(true);
    setError(null);
    try {
      await onCreate(name.trim());
      setName('');
      onClose();
    } catch (err: unknown) {
      console.error('Erro ao criar projeto:', err);
      const errMsg = err instanceof Error ? err.message : 'Falha ao criar novo cofre de projeto.';
      setError(errMsg);
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm animate-in fade-in-0 duration-200" />
        <Dialog.Content className="fixed left-1/2 top-1/2 z-50 w-full max-w-md -translate-x-1/2 -translate-y-1/2 rounded-xl bg-zinc-900 border border-zinc-800 p-6 shadow-2xl animate-in fade-in-0 zoom-in-95 duration-200 focus:outline-none">
          {/* Header */}
          <div className="flex items-start justify-between gap-4 mb-5">
            <div className="flex items-center gap-3">
              <div className="flex items-center justify-center w-8 h-8 rounded-lg bg-zinc-800/80 border border-zinc-700/60 text-zinc-300">
                <FolderPlus size={16} weight="regular" />
              </div>
              <div>
                <Dialog.Title className="text-sm font-semibold text-zinc-100">
                  Criar Novo Projeto & Vault
                </Dialog.Title>
                <Dialog.Description className="text-xs text-zinc-400 mt-0.5">
                  Cofre isolado de notas Markdown com indice FastEmbed independente.
                </Dialog.Description>
              </div>
            </div>

            <Dialog.Close asChild>
              <button
                type="button"
                className="rounded-md p-1.5 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/60 transition-colors cursor-pointer"
                title="Fechar"
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
                className="block text-xs font-medium text-zinc-300 mb-1.5"
              >
                Nome do Projeto
              </label>
              <input
                id="project-name-input"
                type="text"
                className="w-full px-3 py-2 text-xs rounded-lg bg-zinc-950/80 border border-zinc-800 text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-indigo-500/80 focus:ring-1 focus:ring-indigo-500/50 transition-all"
                placeholder="Ex: HomeLab, Estudos IA, Financas, Startup..."
                value={name}
                onChange={(e) => {
                  setName(e.target.value);
                  if (error) setError(null);
                }}
                autoFocus
                disabled={isSubmitting}
              />
            </div>

            {/* Architecture Preview */}
            <div className="rounded-lg bg-zinc-950/50 border border-zinc-800/70 p-3 space-y-2">
              <div className="flex items-center gap-2 text-zinc-400 text-xs font-medium">
                <Database size={13} />
                <span>Estrutura do Cofre Isolado:</span>
              </div>

              <div className="font-mono text-[11px] space-y-1 text-zinc-400">
                <div className="flex items-center gap-1.5 truncate">
                  <Folder size={12} className="text-zinc-500 shrink-0" />
                  <span>Pasta Vault:</span>
                  <span className="text-zinc-200">./vaults/{slug || 'nome-do-projeto'}/</span>
                </div>
                <div className="flex items-center gap-1.5 truncate">
                  <Database size={12} className="text-zinc-500 shrink-0" />
                  <span>Indice SQLite:</span>
                  <span className="text-zinc-200">./vaults/{slug || 'nome-do-projeto'}/index.db</span>
                </div>
              </div>

              <div className="flex items-center gap-1.5 text-[11px] text-zinc-500 pt-1">
                <CheckCircle size={12} className="text-zinc-500 shrink-0" />
                <span>Compativel com Obsidian e isolado no protocolo MCP.</span>
              </div>
            </div>

            {error && (
              <div className="text-xs font-medium text-rose-400 bg-rose-950/20 border border-rose-900/30 px-3 py-1.5 rounded-md">
                {error}
              </div>
            )}

            {/* Actions */}
            <div className="flex items-center justify-end gap-2 pt-2">
              <button
                type="button"
                onClick={onClose}
                disabled={isSubmitting}
                className="px-3 py-1.5 text-xs font-medium text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800/60 rounded-lg transition-colors cursor-pointer"
              >
                Cancelar
              </button>
              <button
                type="submit"
                disabled={isSubmitting || !name.trim()}
                className="inline-flex items-center gap-1.5 px-3.5 py-1.5 text-xs font-semibold rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white transition-all disabled:opacity-50 disabled:cursor-not-allowed cursor-pointer shadow-sm shadow-indigo-950"
              >
                <Check size={13} weight="bold" />
                <span>{isSubmitting ? 'Criando Vault...' : 'Criar Vault'}</span>
              </button>
            </div>
          </form>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
