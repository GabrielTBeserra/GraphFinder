import React, { useState } from 'react';
import * as Dialog from '@radix-ui/react-dialog';
import {
  Sparkle,
  ArrowSquareOut,
  X,
  ArrowClockwise,
  CheckCircle,
  DownloadSimple,
  GitBranch,
  CalendarBlank,
  Tag,
} from '@phosphor-icons/react';
import { marked } from 'marked';
import DOMPurify from 'dompurify';
import type { ReleaseInfo } from '../services/updater';
import { checkForUpdates, openExternalUrl, RELEASES_PAGE_URL } from '../services/updater';

interface ReleaseModalProps {
  isOpen: boolean;
  onClose: () => void;
  releaseInfo: ReleaseInfo | null;
  onUpdateReleaseInfo: (info: ReleaseInfo) => void;
  theme?: 'dark' | 'light';
}

export const ReleaseModal: React.FC<ReleaseModalProps> = ({
  isOpen,
  onClose,
  releaseInfo,
  onUpdateReleaseInfo,
  theme = 'dark',
}) => {
  const isLight = theme === 'light';
  const [isChecking, setIsChecking] = useState(false);
  const [statusMessage, setStatusMessage] = useState<string | null>(null);

  const handleCheckNow = async () => {
    setIsChecking(true);
    setStatusMessage(null);
    try {
      const freshInfo = await checkForUpdates(true);
      onUpdateReleaseInfo(freshInfo);
      if (freshInfo.hasUpdate) {
        setStatusMessage(`New version ${freshInfo.tagName} is available!`);
      } else {
        setStatusMessage('You are already using the latest version.');
      }
    } catch {
      setStatusMessage('Unable to check for updates. Please try again later.');
    } finally {
      setIsChecking(false);
    }
  };

  const handleOpenReleases = () => {
    const url = releaseInfo?.htmlUrl || RELEASES_PAGE_URL;
    openExternalUrl(url);
  };

  const renderedNotes = React.useMemo(() => {
    if (!releaseInfo?.body) return '';
    try {
      const raw = marked.parse(releaseInfo.body) as string;
      return DOMPurify.sanitize(raw);
    } catch {
      return releaseInfo.body;
    }
  }, [releaseInfo?.body]);

  const currentVer = releaseInfo?.currentVersion || '0.14.0';
  const latestVer = releaseInfo?.version || currentVer;
  const hasUpdate = releaseInfo?.hasUpdate ?? false;

  return (
    <Dialog.Root open={isOpen} onOpenChange={(open) => !open && onClose()}>
      <Dialog.Portal>
        <Dialog.Overlay className="fixed inset-0 z-50 bg-black/80 backdrop-blur-md" />
        <Dialog.Content
          className={`fixed left-1/2 top-1/2 z-50 w-full max-w-xl max-h-[88vh] -translate-x-1/2 -translate-y-1/2 rounded-2xl border p-6 shadow-2xl flex flex-col focus:outline-none overflow-hidden transition-colors ${
            isLight
              ? 'bg-white border-slate-200 text-slate-800'
              : 'bg-[#0f1117] border-[#232838] text-[#f1f5f9]'
          }`}
        >
          {/* Header */}
          <div
            className={`flex items-start justify-between gap-4 pb-4 border-b ${
              isLight ? 'border-slate-200' : 'border-[#1f2433]'
            }`}
          >
            <div className="flex items-center gap-3">
              <div
                className={`flex items-center justify-center w-10 h-10 rounded-xl border ${
                  hasUpdate
                    ? isLight
                      ? 'bg-emerald-50 border-emerald-200 text-emerald-600'
                      : 'bg-emerald-500/10 border-emerald-500/25 text-emerald-400'
                    : isLight
                    ? 'bg-indigo-50 border-indigo-200 text-indigo-600'
                    : 'bg-indigo-500/10 border-indigo-500/20 text-indigo-400'
                }`}
              >
                {hasUpdate ? (
                  <Sparkle size={22} weight="duotone" />
                ) : (
                  <CheckCircle size={22} weight="duotone" />
                )}
              </div>
              <div>
                <Dialog.Title className="text-base font-semibold flex items-center gap-2">
                  <span>Version &amp; Releases</span>
                  <span
                    className={`inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10.5px] font-mono font-medium ${
                      hasUpdate
                        ? isLight
                          ? 'bg-emerald-100 text-emerald-800 border border-emerald-200'
                          : 'bg-emerald-500/15 text-emerald-300 border border-emerald-500/30'
                        : isLight
                        ? 'bg-slate-100 text-slate-700 border border-slate-200'
                        : 'bg-[#181d2a] text-slate-300 border border-[#2b3347]'
                    }`}
                  >
                    {hasUpdate ? 'Update Available' : 'Up to Date'}
                  </span>
                </Dialog.Title>
                <Dialog.Description
                  className={`text-xs mt-0.5 ${
                    isLight ? 'text-slate-500' : 'text-[#64748b]'
                  }`}
                >
                  Check installed version against official GitHub releases.
                </Dialog.Description>
              </div>
            </div>

            <Dialog.Close asChild>
              <button
                type="button"
                className={`p-1.5 rounded-lg transition-colors cursor-pointer ${
                  isLight
                    ? 'text-slate-400 hover:text-slate-700 hover:bg-slate-100'
                    : 'text-[#64748b] hover:text-[#cbd5e1] hover:bg-[#181c26]'
                }`}
                title="Close"
              >
                <X size={16} />
              </button>
            </Dialog.Close>
          </div>

          {/* Versions Summary Bar */}
          <div
            className={`my-4 p-3.5 rounded-xl border flex flex-col sm:flex-row sm:items-center justify-between gap-3 text-xs ${
              isLight
                ? 'bg-slate-50 border-slate-200'
                : 'bg-[#121520] border-[#1f2536]'
            }`}
          >
            <div className="flex items-center gap-4">
              <div>
                <span
                  className={`block text-[10px] font-semibold uppercase tracking-wider ${
                    isLight ? 'text-slate-400' : 'text-[#64748b]'
                  }`}
                >
                  Installed Version
                </span>
                <span className="font-mono font-bold text-sm">
                  v{currentVer}
                </span>
              </div>

              <div
                className={`w-[1px] h-7 ${
                  isLight ? 'bg-slate-200' : 'bg-[#1f2536]'
                }`}
              />

              <div>
                <span
                  className={`block text-[10px] font-semibold uppercase tracking-wider ${
                    isLight ? 'text-slate-400' : 'text-[#64748b]'
                  }`}
                >
                  Latest Available
                </span>
                <span
                  className={`font-mono font-bold text-sm ${
                    hasUpdate
                      ? 'text-emerald-500'
                      : isLight
                      ? 'text-slate-700'
                      : 'text-slate-300'
                  }`}
                >
                  v{latestVer}
                </span>
              </div>
            </div>

            <div className="flex items-center gap-2">
              <button
                type="button"
                onClick={handleCheckNow}
                disabled={isChecking}
                className={`inline-flex items-center gap-1.5 px-3 py-1.5 rounded-lg font-medium transition-all cursor-pointer disabled:opacity-50 ${
                  isLight
                    ? 'bg-white hover:bg-slate-100 text-slate-700 border border-slate-200 shadow-2xs'
                    : 'bg-[#181e2e] hover:bg-[#20293d] text-slate-200 border border-[#2b354e]'
                }`}
              >
                <ArrowClockwise
                  size={13}
                  className={isChecking ? 'animate-spin text-indigo-500' : ''}
                />
                <span>{isChecking ? 'Checking...' : 'Check for Updates'}</span>
              </button>
            </div>
          </div>

          {/* Feedback status message */}
          {statusMessage && (
            <div
              className={`mb-3 px-3 py-1.5 rounded-lg text-xs font-medium border ${
                hasUpdate
                  ? isLight
                    ? 'bg-emerald-50 border-emerald-200 text-emerald-700'
                    : 'bg-emerald-500/10 border-emerald-500/25 text-emerald-300'
                  : isLight
                  ? 'bg-slate-100 border-slate-200 text-slate-700'
                  : 'bg-[#151926] border-[#22293d] text-slate-300'
              }`}
            >
              {statusMessage}
            </div>
          )}

          {/* Release Notes Body */}
          <div className="flex-1 overflow-y-auto min-h-[160px] max-h-[340px] pr-1 space-y-3">
            {releaseInfo?.name && (
              <div className="flex items-center gap-3 text-xs">
                <span className="flex items-center gap-1 font-semibold">
                  <Tag size={13} className="text-indigo-400" />
                  {releaseInfo.name}
                </span>
                {releaseInfo.publishedAt && (
                  <span
                    className={`flex items-center gap-1 ${
                      isLight ? 'text-slate-400' : 'text-[#64748b]'
                    }`}
                  >
                    <CalendarBlank size={13} />
                    {new Date(releaseInfo.publishedAt).toLocaleDateString()}
                  </span>
                )}
              </div>
            )}

            {renderedNotes ? (
              <div
                className={`text-xs leading-relaxed p-3.5 rounded-xl border prose max-w-none ${
                  isLight
                    ? 'bg-slate-50/60 border-slate-200 prose-slate'
                    : 'bg-[#0b0d13] border-[#181d2a] prose-invert text-slate-300'
                }`}
                dangerouslySetInnerHTML={{ __html: renderedNotes }}
              />
            ) : (
              <div
                className={`py-8 text-center text-xs ${
                  isLight ? 'text-slate-400' : 'text-[#64748b]'
                }`}
              >
                No release notes found or you are offline.
              </div>
            )}

            {/* Assets List (if any) */}
            {releaseInfo?.assets && releaseInfo.assets.length > 0 && (
              <div className="mt-3">
                <span
                  className={`block text-[11px] font-semibold uppercase tracking-wider mb-1.5 ${
                    isLight ? 'text-slate-400' : 'text-[#64748b]'
                  }`}
                >
                  Binary Downloads ({releaseInfo.assets.length})
                </span>
                <div className="grid grid-cols-1 sm:grid-cols-2 gap-1.5">
                  {releaseInfo.assets.map((asset) => (
                    <button
                      key={asset.name}
                      type="button"
                      onClick={() => openExternalUrl(asset.browser_download_url)}
                      className={`flex items-center justify-between p-2 rounded-lg border text-[11px] font-mono transition-colors text-left truncate cursor-pointer ${
                        isLight
                          ? 'bg-slate-50 hover:bg-indigo-50/60 border-slate-200 text-slate-700'
                          : 'bg-[#121520] hover:bg-[#181d2d] border-[#1f2536] text-slate-300 hover:text-white'
                      }`}
                      title={asset.name}
                    >
                      <span className="truncate mr-2">{asset.name}</span>
                      <DownloadSimple size={13} className="shrink-0 text-indigo-400" />
                    </button>
                  ))}
                </div>
              </div>
            )}
          </div>

          {/* Footer Actions */}
          <div
            className={`mt-4 pt-3 border-t flex items-center justify-between gap-3 text-xs ${
              isLight ? 'border-slate-200' : 'border-[#1f2433]'
            }`}
          >
            <div
              className={`flex items-center gap-1.5 ${
                isLight ? 'text-slate-400' : 'text-[#64748b]'
              }`}
            >
              <GitBranch size={13} />
              <span className="text-[11px]">github.com/GabrielTBeserra/GraphFinder</span>
            </div>

            <div className="flex items-center gap-2">
              <Dialog.Close asChild>
                <button
                  type="button"
                  className={`px-3 py-1.5 rounded-lg font-medium transition-colors cursor-pointer ${
                    isLight
                      ? 'bg-slate-100 hover:bg-slate-200 text-slate-700'
                      : 'bg-[#181c26] hover:bg-[#202533] text-slate-300'
                  }`}
                >
                  Close
                </button>
              </Dialog.Close>

              <button
                type="button"
                onClick={handleOpenReleases}
                className="inline-flex items-center gap-1.5 px-3.5 py-1.5 rounded-lg font-semibold bg-indigo-600 hover:bg-indigo-500 text-white shadow-xs transition-all cursor-pointer"
              >
                <span>Go to Releases Page</span>
                <ArrowSquareOut size={13} weight="bold" />
              </button>
            </div>
          </div>
        </Dialog.Content>
      </Dialog.Portal>
    </Dialog.Root>
  );
};
