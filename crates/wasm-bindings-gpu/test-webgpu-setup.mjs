import { create, globals } from 'webgpu';

// Install WebGPU global types (GPUBufferUsage, GPUMapMode, etc.) on globalThis
Object.assign(globalThis, globals);

// Polyfill navigator.gpu for Node.js so the JS helper code works
if (typeof globalThis.navigator === 'undefined') {
    globalThis.navigator = {};
}
const gpu = create(['backend=vulkan']);
globalThis.navigator.gpu = gpu;
