// Muted palette aligned with the design tokens (desaturated accents)
export const COMMUNITY_DARK = [
  '#6d7ae0', '#4fb08c', '#5fa8d9', '#d9a05f', '#e07c9b',
  '#9b8ae0', '#4fb8c8', '#e08b5f', '#66b8a0', '#d96f8a',
  '#a3b86a', '#b08be0', '#6a9fd8', '#d8bd6a',
];

export const COMMUNITY_LIGHT = [
  '#4f5bc0', '#346538', '#1f6c9f', '#956400', '#b03a5f',
  '#6d54b8', '#0f7f86', '#b25a2e', '#2e7f6b', '#b03a5f',
  '#6f8a2e', '#7d54c0', '#2e6db0', '#9a7d1e',
];

export function communityColor(id: number, isLight: boolean): string {
  const p = isLight ? COMMUNITY_LIGHT : COMMUNITY_DARK;
  return p[Math.abs(id) % p.length];
}

export const LANG_COLORS: Record<string, string> = {
  typescript: '#5fa8d9',
  javascript: '#d8bd6a',
  rust: '#e08b5f',
  python: '#4fb08c',
  go: '#4fb8c8',
  c: '#6a9fd8',
  cpp: '#4f5bc0',
  csharp: '#9b8ae0',
  java: '#d96f5f',
  kotlin: '#8a7ae0',
  php: '#6d7ae0',
  ruby: '#d96f8a',
  swift: '#e07c7c',
  css: '#e07c9b',
  shell: '#66b8a0',
};

const ACCENTS_DARK = COMMUNITY_DARK.slice(0, 10);
const ACCENTS_LIGHT = COMMUNITY_LIGHT.slice(0, 10);

export function hashStringToColor(str: string, isLight: boolean): string {
  let hash = 0;
  for (let i = 0; i < str.length; i++) {
    hash = (hash << 5) - hash + str.charCodeAt(i);
    hash |= 0;
  }
  const index = Math.abs(hash) % ACCENTS_DARK.length;
  return isLight ? ACCENTS_LIGHT[index] : ACCENTS_DARK[index];
}

const TAG_PALETTES: Record<string, { dark: string; light: string }> = {
  architecture: { dark: '#6d7ae0', light: '#4f5bc0' },
  system: { dark: '#6d7ae0', light: '#4f5bc0' },
  design: { dark: '#6d7ae0', light: '#4f5bc0' },
  rust: { dark: '#e08b5f', light: '#b25a2e' },
  embeddings: { dark: '#4fb08c', light: '#346538' },
  rag: { dark: '#4fb8c8', light: '#0f7f86' },
  ai: { dark: '#4fb08c', light: '#346538' },
  mcp: { dark: '#9b8ae0', light: '#6d54b8' },
  neural: { dark: '#4fb8c8', light: '#0f7f86' },
  guide: { dark: '#d9a05f', light: '#956400' },
  tutorial: { dark: '#d9a05f', light: '#956400' },
  welcome: { dark: '#e07c9b', light: '#b03a5f' },
  onboarding: { dark: '#e07c9b', light: '#b03a5f' },
  sqlite: { dark: '#9b8ae0', light: '#6d54b8' },
  database: { dark: '#9b8ae0', light: '#6d54b8' },
  storage: { dark: '#b08be0', light: '#7d54c0' },
  security: { dark: '#e07c7c', light: '#9f2f2d' },
  auth: { dark: '#e07c9b', light: '#b03a5f' },
  docs: { dark: '#5fa8d9', light: '#1f6c9f' },
};

export function getNodeColor(
  node: any,
  isLight: boolean,
  colorMode: 'communities' | 'default',
): string {
  if (colorMode === 'communities' && typeof node.community === 'number') {
    return communityColor(node.community, isLight);
  }
  if (node.node_type === 'code') {
    const lang = (node.tags && node.tags[0]) || '';
    return LANG_COLORS[lang.toLowerCase()] || (isLight ? '#1f6c9f' : '#5fa8d9');
  }
  if (node.node_type === 'unresolved') {
    return isLight ? '#a8a49c' : '#565e6b';
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
