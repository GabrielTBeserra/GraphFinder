import fs from 'node:fs';
import path from 'node:path';

const rawTag = process.argv[2] || process.env.GITHUB_REF_NAME || '';
const version = rawTag.trim().replace(/^v/, '');

if (!version || version === 'main' || version === 'master') {
  console.log(`[sync-version] No valid version detected in argument/tag ('${rawTag}'). Skipping synchronization.`);
  process.exit(0);
}

console.log(`[sync-version] Synchronizing version to: ${version}`);

// 1. Update package.json
const pkgPath = path.resolve('package.json');
if (fs.existsSync(pkgPath)) {
  const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
  pkg.version = version;
  fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n', 'utf8');
  console.log(`  [OK] package.json updated to ${version}`);
}

// 2. Update src-tauri/tauri.conf.json
const tauriConfPath = path.resolve('src-tauri/tauri.conf.json');
if (fs.existsSync(tauriConfPath)) {
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
  tauriConf.version = version;
  fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n', 'utf8');
  console.log(`  [OK] src-tauri/tauri.conf.json updated to ${version}`);
}

// 3. Update src-tauri/Cargo.toml
const cargoPath = path.resolve('src-tauri/Cargo.toml');
if (fs.existsSync(cargoPath)) {
  let cargo = fs.readFileSync(cargoPath, 'utf8');
  cargo = cargo.replace(/^version\s*=\s*"[^"]*"/m, `version = "${version}"`);
  fs.writeFileSync(cargoPath, cargo, 'utf8');
  console.log(`  [OK] src-tauri/Cargo.toml updated to ${version}`);
}

console.log(`[sync-version] Version synchronization completed successfully.`);
