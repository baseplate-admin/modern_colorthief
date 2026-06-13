import { create } from 'webgpu';

console.log('[webgpu-setup] webgpu imported successfully');
console.log('[webgpu-setup] create type:', typeof create);

// Polyfill navigator.gpu for Node.js so the JS helper code works
if (typeof globalThis.navigator === 'undefined') {
    globalThis.navigator = {};
}
const gpu = create([]);
console.log('[webgpu-setup] create([]) returned:', gpu);
globalThis.navigator.gpu = gpu;
console.log('[webgpu-setup] navigator.gpu set:', !!globalThis.navigator.gpu);
