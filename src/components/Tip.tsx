import React from 'react';

interface TipProps {
  label: React.ReactNode;
  children: React.ReactNode;
}

/** Lightweight CSS-only tooltip — appears below the trigger (toolbar is at top).
 *  Delayed on hover so it doesn't fire on every cursor sweep; also shows on
 *  keyboard focus so icon-only controls stay discoverable. */
export const Tip: React.FC<TipProps> = ({ label, children }) => (
  <span className="relative inline-flex group/tip">
    {children}
    <span className="pointer-events-none absolute left-1/2 top-full z-50 mt-1.5 w-max max-w-[240px] -translate-x-1/2 translate-y-[-3px] rounded-md border border-line bg-surface px-2.5 py-1.5 text-[11px] font-normal leading-snug text-muted shadow-lg opacity-0 transition-all duration-150 delay-0 group-hover/tip:translate-y-0 group-hover/tip:opacity-100 group-hover/tip:delay-300 group-focus-within/tip:translate-y-0 group-focus-within/tip:opacity-100 group-focus-within/tip:delay-0">
      {label}
    </span>
  </span>
);
