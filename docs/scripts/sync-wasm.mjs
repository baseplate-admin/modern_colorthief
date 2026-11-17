#!/usr/bin/env node
/**
 * Build the WASM bundles for the docs site and copy them into the
 * `docs/public/` directories.
 *
 * Usage:
 *   node docs/scripts/sync-wasm.mjs            # build + copy
 *   node docs/scripts/sync-wasm.mjs --check    # verify bundles are in sync
 *
 * The script builds:
 *   crates/wasm-bindings         → docs/public/wasm/
 *   crates/wasm-bindings-webgpu  → docs/public/wasm-webgpu/
 */
import { execSync } from 'node:child_process';
import { existsSync, cpSync, readdirSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = join(__dirname, '..', '..');

const CRATES = [
  {
    dir: 'crates/wasm-bindings',
    out: 'docs/public/wasm',
    pkg: 'pkg',
  },
  {
    dir: 'crates/wasm-bindings-webgpu',
    out: 'docs/public/wasm-webgpu',
    pkg: 'pkg',
  },
];

function run(cmd, opts = {}) {
  console.log(`  $ ${cmd}`);
  execSync(cmd, { stdio: 'inherit', cwd: REPO_ROOT, ...opts });
}

function copyBundle(src, dest) {
  const files = readdirSync(src);
  console.log(`  copy ${src} → ${dest}`);
  cpSync(src, dest, { recursive: true, force: true });
  return files.length;
}

function checkSync() {
  let ok = true;
  for (const { dir, out, pkg } of CRATES) {
    const srcDir = join(REPO_ROOT, dir, pkg);
    const destDir = join(REPO_ROOT, out);
    if (!existsSync(srcDir)) {
      console.error(`✗ ${dir}/${pkg} does not exist — run wasm-pack build first`);
      ok = false;
      continue;
    }
    if (!existsSync(destDir)) {
      console.error(`✗ ${out} does not exist — run sync-wasm.mjs (without --check)`);
      ok = false;
      continue;
    }
    // Compare file lists
    const srcFiles = readdirSync(srcDir).sort();
    const destFiles = readdirSync(destDir).sort();
    if (JSON.stringify(srcFiles) !== JSON.stringify(destFiles)) {
      console.error(`✗ ${out} is out of sync with ${dir}/${pkg}`);
      console.error(`  source: ${srcFiles.join(', ')}`);
      console.error(`  dest:   ${destFiles.join(', ')}`);
      ok = false;
    } else {
      console.log(`✓ ${out} is in sync (${srcFiles.length} files)`);
    }
  }
  return ok;
}

const check = process.argv.includes('--check');

if (check) {
  const ok = checkSync();
  process.exit(ok ? 0 : 1);
}

console.log('Building WASM bundles for docs…\n');
for (const { dir, out, pkg } of CRATES) {
  console.log(`${dir}:`);
  run(`cd ${dir} && npx wasm-pack build --release --target web`);
  const src = join(REPO_ROOT, dir, pkg);
  const dest = join(REPO_ROOT, out);
  copyBundle(src, dest);
  console.log('');
}
console.log('Done. Docs bundles are up to date.');
