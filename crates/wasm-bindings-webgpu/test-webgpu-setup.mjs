import { create, globals } from 'webgpu';

// Install WebGPU global types (GPUBufferUsage, GPUMapMode, etc.) on globalThis
Object.assign(globalThis, globals);

// Polyfill navigator.gpu for Node.js so the JS helper code works
if (typeof globalThis.navigator === 'undefined') {
    globalThis.navigator = {};
}
const gpu = create(['backend=vulkan']);
globalThis.navigator.gpu = gpu;

// Define getter on globalThis so Rust code can read navigator.gpu at call time.
// js_sys::global() caches the global object at WASM load time (before this polyfill runs),
// so Reflect::get on the cached object can't see navigator.gpu. A getter defined via
// Object.defineProperty is on the real globalThis and survives the cache.
Object.defineProperty(globalThis, '__wt_get_gpu', {
    value: () => globalThis.navigator && globalThis.navigator.gpu,
    configurable: true,
    writable: true,
});

console.log('[test-setup] WebGPU polyfill applied, navigator.gpu =', !!globalThis.navigator.gpu);
