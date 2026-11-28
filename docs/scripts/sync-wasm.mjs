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
import { execFileSync } from 'node:child_process';
import { existsSync, copyFileSync, mkdirSync, readFileSync } from 'node:fs';
import { join, dirname } from 'node:path';
import { fileURLToPath } from 'node:url';

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = join(__dirname, '..', '..');

const CRATES = [
  {
    dir: 'crates/wasm-bindings',
    out: 'docs/public/wasm',
    name: 'modern_colorthief_wasm',
    pkg: 'pkg',
  },
  {
    dir: 'crates/wasm-bindings-webgpu',
    out: 'docs/public/wasm-webgpu',
    name: 'modern_colorthief_webgpu',
    pkg: 'pkg',
  },
];

function bundleFiles(name) {
  return [`${name}.js`, `${name}_bg.wasm`];
}

function checkSync() {
  let ok = true;
  for (const { dir, out, name, pkg } of CRATES) {
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
    for (const file of bundleFiles(name)) {
      const source = join(srcDir, file);
      const destination = join(destDir, file);
      if (
        !existsSync(source) ||
        !existsSync(destination) ||
        !readFileSync(source).equals(readFileSync(destination))
      ) {
        console.error(`${out}/${file} is missing or out of sync with ${dir}/${pkg}`);
        ok = false;
      }
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
for (const { dir, out, name, pkg } of CRATES) {
  console.log(`${dir}:`);
  execFileSync('wasm-pack', ['build', '--release', '--target', 'web', '--out-name', name], {
    cwd: join(REPO_ROOT, dir),
    stdio: 'inherit',
  });
  const src = join(REPO_ROOT, dir, pkg);
  const dest = join(REPO_ROOT, out);
  mkdirSync(dest, { recursive: true });
  for (const file of bundleFiles(name)) {
    copyFileSync(join(src, file), join(dest, file));
  }
  console.log('');
}
console.log('Done. Docs bundles are up to date.');
