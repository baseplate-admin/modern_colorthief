import { create, globals } from 'webgpu';

// Install WebGPU global types (GPUBufferUsage, GPUMapMode, etc.) on globalThis
Object.assign(globalThis, globals);

// Polyfill navigator.gpu for Node.js so the JS helper code works
if (typeof globalThis.navigator === 'undefined') {
    globalThis.navigator = {};
}
const gpu = create([]);
globalThis.navigator.gpu = gpu;

// Set a flag readable from the Function constructor's scope
// (navigator.gpu is invisible because Function scope sees Node's built-in navigator)
globalThis.__webgpu_available = true;
