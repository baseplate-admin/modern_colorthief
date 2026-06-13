// Save a reference to eval BEFORE importing webgpu, because the webgpu native
// module corrupts wasm-bindgen's `globalThis` binding, making js_eval fail.
globalThis.jsEval = eval;

import { create } from 'webgpu';

// Polyfill navigator.gpu for Node.js so the JS helper code works
if (typeof globalThis.navigator === 'undefined') {
    globalThis.navigator = {};
}
globalThis.navigator.gpu = create([]);
