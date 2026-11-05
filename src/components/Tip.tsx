import React from 'react';

interface TipProps {
  label: React.ReactNode;
  children: React.ReactNode;
}

/** Lightweight CSS-only tooltip — appears below the trigger (toolbar is at top). */
export const Tip: React.FC<TipProps> = ({ label, children }) => (
  <span className="relative inline-flex group/tip">
    {children}
    <span className="pointer-events-none absolute left-1/2 top-full z-50 mt-1.5 w-max max-w-[240px] -translate-x-1/2 translate-y-[-3px] rounded-md border border-line bg-surface px-2.5 py-1.5 text-[10.5px] font-normal leading-snug text-muted shadow-lg opacity-0 transition-all duration-150 group-hover/tip:translate-y-0 group-hover/tip:opacity-100">
      {label}
    </span>
  </span>
);
