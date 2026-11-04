// Vibrant palette restored from the pre-sigma renderer (same values).
export const COMMUNITY_PALETTE = [
  '#6366f1', '#10b981', '#38bdf8', '#f59e0b', '#ec4899',
  '#8b5cf6', '#06b6d4', '#f97316', '#14b8a6', '#e11d48',
  '#84cc16', '#a855f7', '#0ea5e9', '#eab308',
];

export function communityColor(id: number, _isLight: boolean): string {
  return COMMUNITY_PALETTE[Math.abs(id) % COMMUNITY_PALETTE.length];
}

export const LANG_COLORS: Record<string, string> = {
  typescript: '#38bdf8',
  javascript: '#facc15',
  rust: '#f97316',
  python: '#10b981',
  go: '#06b6d4',
  c: '#3b82f6',
  cpp: '#2563eb',
  csharp: '#a855f7',
  java: '#ea580c',
  kotlin: '#8b5cf6',
  php: '#6366f1',
  ruby: '#e11d48',
  swift: '#f43f5e',
  css: '#ec4899',
  shell: '#14b8a6',
};

const VIBRANT_ACCENTS: { dark: string; light: string }[] = [
  { dark: '#6366f1', light: '#4f46e5' }, // Indigo
  { dark: '#10b981', light: '#059669' }, // Emerald
  { dark: '#38bdf8', light: '#0284c7' }, // Sky
  { dark: '#f59e0b', light: '#d97706' }, // Amber
  { dark: '#8b5cf6', light: '#7c3aed' }, // Violet
  { dark: '#ec4899', light: '#db2777' }, // Pink
  { dark: '#06b6d4', light: '#0891b2' }, // Cyan
  { dark: '#f97316', light: '#ea580c' }, // Orange
  { dark: '#14b8a6', light: '#0d9488' }, // Teal
  { dark: '#e11d48', light: '#be123c' }, // Rose
];

export function hashStringToColor(str: string, isLight: boolean): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = (hash << 5) - hash + str.charCodeAt(i);
    hash |= 0;
  }
  const index = Math.abs(hash) % VIBRANT_ACCENTS.length;
  return isLight ? VIBRANT_ACCENTS[index].light : VIBRANT_ACCENTS[index].dark;
}

const TAG_PALETTES: Record<string, { dark: string; light: string }> = {
  architecture: { dark: '#6366f1', light: '#4f46e5' },
  system: { dark: '#6366f1', light: '#4f46e5' },
  design: { dark: '#6366f1', light: '#4f46e5' },
  rust: { dark: '#f97316', light: '#ea580c' },
  embeddings: { dark: '#10b981', light: '#059669' },
  rag: { dark: '#06b6d4', light: '#0891b2' },
  ai: { dark: '#10b981', light: '#059669' },
  mcp: { dark: '#8b5cf6', light: '#7c3aed' },
  neural: { dark: '#06b8d4', light: '#0891b2' },
  guide: { dark: '#f59e0b', light: '#d97706' },
  tutorial: { dark: '#f59e0b', light: '#d97706' },
  welcome: { dark: '#e11d48', light: '#be123c' },
  onboarding: { dark: '#ec4899', light: '#db2777' },
  sqlite: { dark: '#8b5cf6', light: '#7c3aed' },
  database: { dark: '#8b5cf6', light: '#7c3aed' },
  storage: { dark: '#a855f7', light: '#9333ea' },
  security: { dark: '#ef4444', light: '#dc2626' },
  auth: { dark: '#f43f5e', light: '#e11d48' },
  docs: { dark: '#38bdf8', light: '#0284c7' },
};

export function getNodeColor(
  node: any,
  isLight: boolean,
  colorMode: 'communities' | 'default',
): string {
  if (colorMode === 'communities' && typeof node.community === 'number') {
    return COMMUNITY_PALETTE[Math.abs(node.community) % COMMUNITY_PALETTE.length];
  }
  if (node.node_type === 'code') {
    const lang = (node.tags && node.tags[0]) || '';
    return LANG_COLORS[lang.toLowerCase()] || (isLight ? '#0284c7' : '#38bdf8');
  }
  if (node.node_type === 'unresolved') {
    return isLight ? '#94a3b8' : '#64748b';
  }
  if (node.tags && Array.isArray(node.tags)) {
    for (const t of node.tags) {
      const clean = t.toLowerCase().replace(/^#/, '');
      if (TAG_PALETTES[clean]) {
        return isLight ? TAG_PALETTES[clean].light : TAG_PALETTES[clean].dark;
      }
    }
  }
  const title = (node.title || node.id || '').replace(/\.md$/, '').toLowerCase();
  for (const [key, palette] of Object.entries(TAG_PALETTES)) {
    if (title.includes(key)) {
      return isLight ? palette.light : palette.dark;
    }
  }
  return hashStringToColor(node.id || 'note', isLight);
}
