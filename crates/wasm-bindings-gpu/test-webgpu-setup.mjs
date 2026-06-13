import { create, globals } from 'webgpu';

// Inject WebGPU globals into globalThis for Node.js
Object.assign(globalThis, globals);

// Polyfill navigator.gpu for Node.js so the JS helper code works
if (typeof globalThis.navigator === 'undefined') {
    globalThis.navigator = {};
}
globalThis.navigator.gpu = create([]);
