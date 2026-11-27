import { execFileSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync } from 'node:fs';
import { dirname, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const docsRoot = dirname(dirname(fileURLToPath(import.meta.url)));
const repoRoot = join(docsRoot, '..');
const outFile = join(docsRoot, 'src', 'generated', 'releases.json');
const maxReleases = 10;

function git(args) {
  return execFileSync('git', args, { cwd: repoRoot, encoding: 'utf8', stdio: ['ignore', 'pipe', 'ignore'] }).trim();
}

function escapeHtml(value) {
  return value
    .replaceAll('&', '&amp;')
    .replaceAll('<', '&lt;')
    .replaceAll('>', '&gt;')
    .replaceAll('"', '&quot;');
}

function inlineMarkdown(value) {
  return escapeHtml(value)
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\*\*([^*]+)\*\*/g, '<strong>$1</strong>')
    .replace(/\[([^\]]+)\]\((https?:[^)]+)\)/g, '<a href="$2" target="_blank" rel="noopener">$1</a>');
}

function parseChangelogSection(body) {
  const sections = [];
  let current = null;
  for (const line of body.split('\n')) {
    const heading = line.match(/^###\s+(.+)\s*$/);
    if (heading) {
      current = { heading: inlineMarkdown(heading[1].trim()), items: [] };
      sections.push(current);
      continue;
    }
    const bullet = line.match(/^\s*[*-]\s+(.+)\s*$/);
    if (bullet) {
      if (!current) {
        current = { heading: 'Changes', items: [] };
        sections.push(current);
      }
      current.items.push(inlineMarkdown(bullet[1].trim()));
    }
  }
  return sections.filter((section) => section.items.length > 0);
}

function changelogBody(changelog, version) {
  const marker = `## [${version}]`;
  const start = changelog.indexOf(marker);
  if (start < 0) return '';
  const bodyStart = changelog.indexOf('\n', start) + 1;
  const next = changelog.indexOf('\n## [', bodyStart);
  return changelog.slice(bodyStart, next < 0 ? changelog.length : next + 1);
}

function fallbackCommits(tag, previousTag) {
  try {
    const range = previousTag ? `${previousTag}..${tag}` : tag;
    return git(['log', '--no-merges', '--pretty=format:%s', range])
      .split('\n')
      .map((line) => line.trim())
      .filter(Boolean)
      .slice(0, 12)
      .map(inlineMarkdown);
  } catch {
    return [];
  }
}

const tags = git(['tag', '--list', 'v*', '--sort=-v:refname'])
  .split('\n')
  .map((tag) => tag.trim())
  .filter(Boolean)
  .slice(0, maxReleases);

const changelogPath = join(repoRoot, 'CHANGELOG.md');
const changelog = readFileSync(changelogPath, 'utf8');

const releases = tags.map((tag, index) => {
  const version = tag.replace(/^v/, '');
  const date = git(['log', '-1', '--format=%as', tag]);
  const body = changelogBody(changelog, version);
  let sections = parseChangelogSection(body);
  if (sections.length === 0) {
    const commits = fallbackCommits(tag, tags[index + 1]);
    sections = commits.length ? [{ heading: 'Changes', items: commits }] : [];
  }
  return { tag, version, date, sections };
});

const payload = {
  latestVersion: tags[0] ?? 'v0.0.0',
  generatedAt: new Date().toISOString(),
  source: 'git-tags+CHANGELOG.md',
  releases,
};

mkdirSync(dirname(outFile), { recursive: true });
writeFileSync(outFile, `${JSON.stringify(payload, null, 2)}\n`, 'utf8');
console.log(`generated ${outFile} with ${releases.length} releases; latest ${payload.latestVersion}`);
