import React from 'react';
import { Sparkle, ArrowSquareOut, X, Info } from '@phosphor-icons/react';
import type { ReleaseInfo } from '../services/updater';
import { openExternalUrl } from '../services/updater';

interface UpdateBannerProps {
  release: ReleaseInfo;
  onDismiss: () => void;
  onOpenDetails: () => void;
}

export const UpdateBanner: React.FC<UpdateBannerProps> = ({
  release,
  onDismiss,
  onOpenDetails,
}) => {
  const handleOpenReleases = () => {
    openExternalUrl(release.htmlUrl);
  };

  return (
    <aside
      aria-label="New version notification"
      className="relative z-30 px-3.5 py-2 border-b border-good/30 bg-good/10 text-ink flex items-center justify-between gap-3 text-xs transition-colors shrink-0"
    >
      <div className="flex items-center gap-2.5 truncate">
        <div className="flex items-center justify-center w-6 h-6 rounded-full shrink-0 bg-good/20 text-good border border-good/30">
          <Sparkle size={13} weight="fill" />
        </div>

        <div className="flex items-center gap-2 truncate">
          <span className="font-semibold tracking-wide">
            New version available:
          </span>
          <span className="font-mono font-semibold px-1.5 py-0.5 rounded text-[11px] shrink-0 bg-good/15 text-good border border-good/30">
            {release.tagName || `v${release.version}`}
          </span>
          <span className="hidden sm:inline text-[11px] truncate text-muted">
            (Current: v{release.currentVersion})
          </span>
        </div>
      </div>

      <div className="flex items-center gap-2 shrink-0">
        <button
          type="button"
          onClick={onOpenDetails}
          className="gf-btn"
          title="View release notes and details" aria-label="View release notes and details"
        >
          <Info size={13} weight="bold" />
          <span>Release Notes</span>
        </button>

        <button
          type="button"
          onClick={handleOpenReleases}
          className="inline-flex items-center gap-1.5 px-3 py-1 rounded-md text-[11.5px] font-semibold transition-colors cursor-pointer shadow-xs bg-good text-canvas hover:opacity-90"
          title="Go to GitHub Releases page" aria-label="Go to GitHub Releases page"
        >
          <span>Go to Releases</span>
          <ArrowSquareOut size={13} weight="bold" />
        </button>

        <button
          type="button"
          onClick={onDismiss}
          className="p-1 rounded-md transition-colors cursor-pointer text-faint hover:text-ink hover:bg-line/50"
          title="Dismiss notification"
          aria-label="Dismiss banner"
        >
          <X size={14} weight="bold" />
        </button>
      </div>
    </aside>
  );
};
