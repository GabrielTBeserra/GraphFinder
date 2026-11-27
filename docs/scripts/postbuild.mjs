import { copyFileSync, existsSync } from 'node:fs';
import { join } from 'node:path';

const root = process.cwd();
const dist = join(root, 'dist');

copyFileSync(join(dist, 'index.html'), join(dist, '404.html'));
for (const file of ['.nojekyll', 'CNAME']) {
  const source = join(root, file);
  if (existsSync(source)) copyFileSync(source, join(dist, file));
}
