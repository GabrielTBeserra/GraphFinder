import fs from 'node:fs';
import path from 'node:path';

const rawTag = process.argv[2] || process.env.GITHUB_REF_NAME || '';
const version = rawTag.trim().replace(/^v/, '');

if (!version || version === 'main' || version === 'master') {
  console.log(`[sync-version] Nenhuma versão válida detectada no argumento/tag ('${rawTag}'). Ignorando sincronização.`);
  process.exit(0);
}

console.log(`[sync-version] Sincronizando versão para: ${version}`);

// 1. Atualizar package.json
const pkgPath = path.resolve('package.json');
if (fs.existsSync(pkgPath)) {
  const pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8'));
  pkg.version = version;
  fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n', 'utf8');
  console.log(`  ✓ package.json atualizado para ${version}`);
}

// 2. Atualizar src-tauri/tauri.conf.json
const tauriConfPath = path.resolve('src-tauri/tauri.conf.json');
if (fs.existsSync(tauriConfPath)) {
  const tauriConf = JSON.parse(fs.readFileSync(tauriConfPath, 'utf8'));
  tauriConf.version = version;
  fs.writeFileSync(tauriConfPath, JSON.stringify(tauriConf, null, 2) + '\n', 'utf8');
  console.log(`  ✓ src-tauri/tauri.conf.json atualizado para ${version}`);
}

// 3. Atualizar src-tauri/Cargo.toml
const cargoPath = path.resolve('src-tauri/Cargo.toml');
if (fs.existsSync(cargoPath)) {
  let cargo = fs.readFileSync(cargoPath, 'utf8');
  cargo = cargo.replace(/^version\s*=\s*"[^"]*"/m, `version = "${version}"`);
  fs.writeFileSync(cargoPath, cargo, 'utf8');
  console.log(`  ✓ src-tauri/Cargo.toml atualizado para ${version}`);
}

console.log(`[sync-version] Sincronização concluída com sucesso!`);
