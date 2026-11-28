import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';

delete globalThis.global;
Object.defineProperty(navigator, 'gpu', {
  value: { requestAdapter: async () => null },
});

const { default: init, getPaletteGpu } = await import('./pkg/modern_colorthief_webgpu.js');
const wasm = await readFile(new URL('./pkg/modern_colorthief_webgpu_bg.wasm', import.meta.url));
await init({ module_or_path: wasm });

await assert.rejects(getPaletteGpu(new Uint8Array([255, 0, 0, 255]), 1, 1, 5, 1), (error) =>
  String(error).includes('No GPU adapter is available')
);
