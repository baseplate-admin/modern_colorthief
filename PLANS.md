# PLANS.md

## Current Implementation Status

### Completed
- [x] Manual median-cut algorithm in core with rayon parallelism
- [x] Removed `color-thief` and `image` crate dependencies
- [x] Python bindings use Pillow for image decoding (transparent to user)
- [x] Node bindings with napi-rs + Sharp for image decoding
- [x] WASM bindings with Canvas API for image decoding
- [x] JVM bindings with JNI
- [x] Ruby bindings with magnus
- [x] CI workflows split per binding
- [x] Node CI tests against Node, Deno, Bun
- [x] JVM CI tests Java 26 + Kotlin
- [x] npm provenance attestation (`--provenance` flag)
- [x] Test suite ported to node bindings
- [x] TypeScript for node bindings
- [x] esbuild config for bundling
- [x] uv monorepo setup
- [x] AGENTS.md created
- [x] bench.py uses tabulate

### Pending

#### GPU Bindings (Vulkan Compute)
- [ ] Create `python-bindings-gpu/` crate — Vulkan-accelerated median-cut
- [ ] Create `node-bindings-gpu/` crate — Vulkan-accelerated median-cut
- [ ] Create `jvm-bindings-gpu/` crate — Vulkan-accelerated median-cut
- [ ] Create `ruby-bindings-gpu/` crate — Vulkan-accelerated median-cut
- [ ] CI jobs to test GPU bindings (require GPU runner)
- [ ] Vulkan Compute shaders for pixel sampling and median cut
- [ ] Cross-vendor validation (Intel, AMD, NVIDIA)

#### WASM Vulkan Migration (Future)
- [ ] Research WebGPU support for median-cut in WASM
- [ ] Plan migration path from CPU to GPU for WASM
- [ ] WebGPU compute shader for color quantization
- [ ] Fallback to CPU for browsers without WebGPU

#### Documentation
- [ ] Create llms.txt index file
- [ ] Create llms-full.txt (complete API docs)
- [ ] Create per-binding llms.txt files
- [ ] Set up hooks to auto-regenerate llms.txt on API changes

#### Cleanup
- [ ] Remove `crates/python-bindings/examples/benchmark.py`
- [ ] Copy test images to all binding test directories
- [ ] Port remaining tests (concurrent, multithread, cli, path, deduplication) to all bindings
- [ ] Remove old `examples/` directory at root

#### Node Bindings
- [ ] Update package.json with esbuild build script
- [ ] Add vitest config
- [ ] Set up TSGO (@typescript/native-preview)
