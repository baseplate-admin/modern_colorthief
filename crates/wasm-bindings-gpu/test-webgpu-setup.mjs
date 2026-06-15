import { create, globals } from 'webgpu';

// Install WebGPU global types (GPUBufferUsage, GPUMapMode, etc.) on globalThis
Object.assign(globalThis, globals);

// Polyfill navigator.gpu for Node.js so the JS helper code works
if (typeof globalThis.navigator === 'undefined') {
    globalThis.navigator = {};
}
const gpu = create(['backend=vulkan']);
console.error(`[webgpu-setup] create() returned: ${gpu ? 'GPU object' : 'null/undefined'}`);
console.error(`[webgpu-setup] typeof gpu: ${typeof gpu}`);
globalThis.navigator.gpu = gpu;
console.error(`[webgpu-setup] globalThis.navigator.gpu: ${globalThis.navigator.gpu ? 'set' : 'null/undefined'}`);

// Set a flag readable from the Function constructor's scope
// (navigator.gpu is invisible because Function scope sees Node's built-in navigator)
globalThis.__webgpu_available = true;
