// Save eval before importing webgpu to preserve wasm-bindgen's globalThis binding
const _eval = globalThis.eval;

import { create, globals } from 'webgpu';

// Restore eval if webgpu import corrupted it
if (typeof globalThis.eval !== 'function') {
    Object.defineProperty(globalThis, 'eval', { value: _eval, writable: true, configurable: true, enumerable: false });
}

// Polyfill navigator.gpu for Node.js so the JS helper code works
if (typeof globalThis.navigator === 'undefined') {
    globalThis.navigator = {};
}
globalThis.navigator.gpu = create([]);
