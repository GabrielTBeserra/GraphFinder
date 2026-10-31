import React from 'react';
import { Sparkle, ArrowSquareOut, X, Info } from '@phosphor-icons/react';
import type { ReleaseInfo } from '../services/updater';
import { openExternalUrl } from '../services/updater';

interface UpdateBannerProps {
  release: ReleaseInfo;
  theme?: 'dark' | 'light';
  onDismiss: () => void;
  onOpenDetails: () => void;
}

export const UpdateBanner: React.FC<UpdateBannerProps> = ({
  release,
  theme = 'dark',
  onDismiss,
  onOpenDetails,
}) => {
  const isLight = theme === 'light';

  const handleOpenReleases = () => {
    openExternalUrl(release.htmlUrl);
  };

  return (
    <aside
      aria-label="New version notification"
      className={`relative z-30 px-3.5 py-2 border-b flex items-center justify-between gap-3 text-xs transition-colors shrink-0 ${
        isLight
          ? 'bg-gradient-to-r from-emerald-50 via-teal-50 to-indigo-50/50 border-emerald-200/80 text-emerald-950'
          : 'bg-gradient-to-r from-[#0d1f18] via-[#0f1d24] to-[#131728] border-emerald-500/30 text-emerald-200'
      }`}
    >
      <div className="flex items-center gap-2.5 truncate">
        <div
          className={`flex items-center justify-center w-6 h-6 rounded-full shrink-0 ${
            isLight
              ? 'bg-emerald-500 text-white shadow-xs'
              : 'bg-emerald-500/20 text-emerald-400 border border-emerald-500/30'
          }`}
        >
          <Sparkle size={13} weight="fill" />
        </div>

        <div className="flex items-center gap-2 truncate">
          <span className="font-semibold tracking-wide">
            New version available:
          </span>
          <span
            className={`font-mono font-bold px-1.5 py-0.5 rounded text-[11px] shrink-0 ${
              isLight
                ? 'bg-emerald-100 text-emerald-800 border border-emerald-200'
                : 'bg-emerald-500/15 text-emerald-300 border border-emerald-500/30'
            }`}
          >
            {release.tagName || `v${release.version}`}
          </span>
          <span
            className={`hidden sm:inline text-[11px] truncate ${
              isLight ? 'text-slate-600' : 'text-slate-400'
            }`}
          >
            (Current: v{release.currentVersion})
          </span>
        </div>
      </div>

      <div className="flex items-center gap-2 shrink-0">
        <button
          type="button"
          onClick={onOpenDetails}
          className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-md text-[11.5px] font-medium transition-colors cursor-pointer ${
            isLight
              ? 'bg-white/80 hover:bg-white text-slate-700 border border-slate-200 shadow-2xs'
              : 'bg-[#182030] hover:bg-[#202b40] text-slate-200 border border-[#2b3952]'
          }`}
          title="View release notes and details"
        >
          <Info size={13} weight="bold" />
          <span>Release Notes</span>
        </button>

        <button
          type="button"
          onClick={handleOpenReleases}
          className={`inline-flex items-center gap-1.5 px-3 py-1 rounded-md text-[11.5px] font-semibold transition-all cursor-pointer shadow-xs ${
            isLight
              ? 'bg-emerald-600 hover:bg-emerald-500 text-white'
              : 'bg-emerald-500 hover:bg-emerald-400 text-emerald-950 font-bold'
          }`}
          title="Go to GitHub Releases page"
        >
          <span>Go to Releases</span>
          <ArrowSquareOut size={13} weight="bold" />
        </button>

        <button
          type="button"
          onClick={onDismiss}
          className={`p-1 rounded-md transition-colors cursor-pointer ${
            isLight
              ? 'text-slate-400 hover:text-slate-700 hover:bg-slate-200/50'
              : 'text-slate-400 hover:text-slate-200 hover:bg-white/10'
          }`}
          title="Dismiss notification"
          aria-label="Dismiss banner"
        >
          <X size={14} weight="bold" />
        </button>
      </div>
    </aside>
  );
};
