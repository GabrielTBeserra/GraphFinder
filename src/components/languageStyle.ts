export const LANGUAGE_COLORS: Record<string, { bg: string; text: string; border: string; hex: string }> = {
  typescript: { bg: 'bg-sky-500/10', text: 'text-sky-400', border: 'border-sky-500/30', hex: '#5fa8d9' },
  javascript: { bg: 'bg-amber-500/10', text: 'text-amber-400', border: 'border-amber-500/30', hex: '#facc15' },
  rust: { bg: 'bg-orange-500/10', text: 'text-orange-400', border: 'border-orange-500/30', hex: '#e08b5f' },
  python: { bg: 'bg-emerald-500/10', text: 'text-emerald-400', border: 'border-emerald-500/30', hex: '#4fb08c' },
  go: { bg: 'bg-cyan-500/10', text: 'text-cyan-400', border: 'border-cyan-500/30', hex: '#4fb8c8' },
  c: { bg: 'bg-blue-500/10', text: 'text-blue-400', border: 'border-blue-500/30', hex: '#3b82f6' },
  cpp: { bg: 'bg-blue-600/10', text: 'text-blue-400', border: 'border-blue-600/30', hex: '#2563eb' },
  csharp: { bg: 'bg-purple-500/10', text: 'text-purple-400', border: 'border-purple-500/30', hex: '#b08be0' },
  java: { bg: 'bg-amber-600/10', text: 'text-amber-400', border: 'border-amber-600/30', hex: '#ea580c' },
  kotlin: { bg: 'bg-violet-500/10', text: 'text-violet-400', border: 'border-violet-500/30', hex: '#9b8ae0' },
  php: { bg: 'bg-indigo-500/10', text: 'text-indigo-400', border: 'border-indigo-500/30', hex: '#6d7ae0' },
  ruby: { bg: 'bg-rose-500/10', text: 'text-rose-400', border: 'border-rose-500/30', hex: '#d96f8a' },
  swift: { bg: 'bg-pink-500/10', text: 'text-pink-400', border: 'border-pink-500/30', hex: '#f43f5e' },
  css: { bg: 'bg-pink-600/10', text: 'text-pink-400', border: 'border-pink-600/30', hex: '#e07c9b' },
  shell: { bg: 'bg-teal-500/10', text: 'text-teal-400', border: 'border-teal-500/30', hex: '#14b8a6' },
  generic: { bg: 'bg-slate-500/10', text: 'text-slate-400', border: 'border-slate-500/30', hex: '#8a919e' },
};

export function getLanguageStyle(lang: string) {
  const key = lang.toLowerCase();
  return LANGUAGE_COLORS[key] || LANGUAGE_COLORS.generic;
}
