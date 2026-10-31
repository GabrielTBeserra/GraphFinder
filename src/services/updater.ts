import { invoke } from '@tauri-apps/api/core';

export const REPO_OWNER = 'GabrielTBeserra';
export const REPO_NAME = 'GraphFinder';
export const RELEASES_PAGE_URL = `https://github.com/${REPO_OWNER}/${REPO_NAME}/releases`;
export const LATEST_RELEASE_API_URL = `https://api.github.com/repos/${REPO_OWNER}/${REPO_NAME}/releases/latest`;

const isTauri = typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;

export interface ReleaseAsset {
  name: string;
  browser_download_url: string;
  size: number;
}

export interface ReleaseInfo {
  version: string;
  tagName: string;
  name: string;
  publishedAt: string;
  htmlUrl: string;
  body: string;
  hasUpdate: boolean;
  currentVersion: string;
  assets?: ReleaseAsset[];
}

export function parseSemver(versionStr: string): [number, number, number] {
  const cleaned = versionStr.trim().replace(/^v/, '').split('-')[0];
  const parts = cleaned.split('.').map((p) => parseInt(p, 10));
  return [parts[0] || 0, parts[1] || 0, parts[2] || 0];
}

export function compareSemver(v1: string, v2: string): number {
  const [maj1, min1, pat1] = parseSemver(v1);
  const [maj2, min2, pat2] = parseSemver(v2);

  if (maj1 !== maj2) return maj1 > maj2 ? 1 : -1;
  if (min1 !== min2) return min1 > min2 ? 1 : -1;
  if (pat1 !== pat2) return pat1 > pat2 ? 1 : -1;
  return 0;
}

const CACHE_KEY = 'gf_release_check_cache';
const CACHE_TTL_MS = 30 * 60 * 1000; // 30 minutes

interface CacheData {
  timestamp: number;
  data: ReleaseInfo;
}

export async function checkForUpdates(force = false): Promise<ReleaseInfo> {
  const currentVersion = typeof __APP_VERSION__ !== 'undefined' ? __APP_VERSION__ : '0.14.0';

  if (!force) {
    try {
      const cachedStr = localStorage.getItem(CACHE_KEY);
      if (cachedStr) {
        const cached: CacheData = JSON.parse(cachedStr);
        if (Date.now() - cached.timestamp < CACHE_TTL_MS) {
          // Re-evaluate hasUpdate in case currentVersion changed
          const hasUpdate = compareSemver(cached.data.version, currentVersion) > 0;
          return {
            ...cached.data,
            currentVersion,
            hasUpdate,
          };
        }
      }
    } catch {
      // Ignore cache reading errors
    }
  }

  try {
    const res = await fetch(LATEST_RELEASE_API_URL, {
      headers: {
        Accept: 'application/vnd.github.v3+json',
      },
    });

    if (!res.ok) {
      throw new Error(`GitHub API returned status ${res.status}`);
    }

    const data = await res.json();
    const tagName = data.tag_name || '';
    const latestVersion = tagName.replace(/^v/, '');
    const hasUpdate = compareSemver(latestVersion, currentVersion) > 0;

    const releaseInfo: ReleaseInfo = {
      version: latestVersion,
      tagName,
      name: data.name || tagName,
      publishedAt: data.published_at || '',
      htmlUrl: data.html_url || RELEASES_PAGE_URL,
      body: data.body || '',
      hasUpdate,
      currentVersion,
      assets: data.assets?.map((a: { name: string; browser_download_url: string; size: number }) => ({
        name: a.name,
        browser_download_url: a.browser_download_url,
        size: a.size,
      })),
    };

    try {
      localStorage.setItem(
        CACHE_KEY,
        JSON.stringify({
          timestamp: Date.now(),
          data: releaseInfo,
        })
      );
    } catch {
      // Ignore storage errors
    }

    return releaseInfo;
  } catch (err) {
    console.warn('[Updater] Failed to check for releases:', err);
    return {
      version: currentVersion,
      tagName: `v${currentVersion}`,
      name: `GraphFinder v${currentVersion}`,
      publishedAt: '',
      htmlUrl: RELEASES_PAGE_URL,
      body: '',
      hasUpdate: false,
      currentVersion,
    };
  }
}

export async function openExternalUrl(url: string): Promise<void> {
  const targetUrl = url || RELEASES_PAGE_URL;
  if (isTauri) {
    try {
      await invoke('open_url', { url: targetUrl });
      return;
    } catch (err) {
      console.warn('[Updater] Tauri open_url command failed, falling back to window.open:', err);
    }
  }
  window.open(targetUrl, '_blank', 'noopener,noreferrer');
}
