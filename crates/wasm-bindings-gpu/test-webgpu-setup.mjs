import { create } from 'webgpu';

// Polyfill navigator.gpu for Node.js so the JS helper code works
if (typeof globalThis.navigator === 'undefined') {
    globalThis.navigator = {};
}
globalThis.navigator.gpu = create([]);
