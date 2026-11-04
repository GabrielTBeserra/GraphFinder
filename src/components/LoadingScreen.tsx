import React from 'react';
import { Cpu, ShieldCheck, HardDrives } from '@phosphor-icons/react';

interface LoadingScreenProps {
  statusMessage?: string;
  isError?: boolean;
}

/**
 * Minimal loading screen — pure CSS, transform/opacity-only animation
 * so everything stays on the GPU compositor.
 */
export const LoadingScreen: React.FC<LoadingScreenProps> = ({
  statusMessage = 'Initializing neural embedding engine...',
  isError = false,
}) => {
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-canvas overflow-hidden select-none">
      {/* Slow ambient drift — GPU-composited transform only */}
      <div
        className="absolute inset-0 pointer-events-none opacity-[0.05] will-change-transform"
        style={{
          background:
            'radial-gradient(circle at 30% 35%, #6d7ae0 0%, transparent 45%), radial-gradient(circle at 70% 65%, #4fb8c8 0%, transparent 40%)',
          animation: 'gfDrift 24s cubic-bezier(0.16,1,0.3,1) infinite alternate',
        }}
      />
      <style>{`
        @keyframes gfDrift {
          from { transform: translate3d(-3%, -2%, 0) scale(1); }
          to { transform: translate3d(3%, 2%, 0) scale(1.06); }
        }
      `}</style>

      <div className="relative z-10 w-full max-w-sm mx-4 p-8 rounded-xl bg-surface border border-line flex flex-col items-center text-center">
        <div className="w-12 h-12 rounded-lg bg-raise border border-line flex items-center justify-center mb-5">
          <Cpu size={26} weight="duotone" className="text-accent" />
        </div>

        <h1 className="text-lg font-semibold tracking-tight text-ink mb-0.5 flex items-center gap-2">
          GraphFinder
          <span className="text-[10px] font-mono px-1.5 py-0.5 rounded-full bg-raise border border-line text-muted">
            v0.14
          </span>
        </h1>
        <p className="text-xs text-muted mb-6">Neural Knowledge Graph &amp; Semantic Vault</p>

        <div className="w-full p-4 rounded-lg bg-canvas border border-line mb-5 text-left">
          <div className="flex items-center justify-between text-xs mb-2.5">
            <span className="flex items-center gap-2 text-ink font-medium">
              <span
                className={`w-1.5 h-1.5 rounded-full ${
                  isError ? 'bg-bad' : 'bg-accent'
                }`}
                style={isError ? undefined : { animation: 'gfPulse 1.6s ease-in-out infinite' }}
              />
              {isError ? 'Initialization Error' : 'Initializing'}
            </span>
            <span className="font-mono text-[10px] text-good">LOCAL</span>
          </div>

          {/* Indeterminate bar — transform-only */}
          <div className="w-full h-1 rounded-full bg-raise overflow-hidden relative">
            <div
              className="absolute inset-y-0 w-1/3 rounded-full bg-accent will-change-transform"
              style={{ animation: 'gfSlide 1.4s cubic-bezier(0.45,0,0.55,1) infinite' }}
            />
          </div>
          <style>{`
            @keyframes gfSlide {
              from { transform: translate3d(-110%, 0, 0); }
              to { transform: translate3d(320%, 0, 0); }
            }
            @keyframes gfPulse {
              0%, 100% { opacity: 1; }
              50% { opacity: 0.35; }
            }
          `}</style>

          <p className="text-[11.5px] text-muted mt-2.5 leading-relaxed">{statusMessage}</p>
        </div>

        <div className="grid grid-cols-2 gap-2 w-full text-left text-[11px] text-muted">
          <div className="flex items-center gap-1.5 p-2 rounded-lg bg-raise border border-line">
            <ShieldCheck size={13} className="text-good shrink-0" />
            <span className="truncate">Local-first</span>
          </div>
          <div className="flex items-center gap-1.5 p-2 rounded-lg bg-raise border border-line">
            <HardDrives size={13} className="text-accent shrink-0" />
            <span className="truncate">FastEmbed ONNX</span>
          </div>
        </div>

        <div className="mt-5 text-[10.5px] text-faint">
          Model download only on first launch.
        </div>
      </div>
    </div>
  );
};
