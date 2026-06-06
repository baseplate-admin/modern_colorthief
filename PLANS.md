# PLANS.md

## Current Implementation Status

### Pending

#### GPU Acceleration
- [ ] Vulkan Compute shaders for pixel sampling and median cut (current impl is CPU fallback)
- [ ] WebGPU compute shader implementation for WASM
- [ ] Cross-vendor validation (Intel, AMD, NVIDIA)
- [ ] CI jobs to test GPU bindings (fallback to CPU with with vulkan compute cpu with a warning)

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
- [ ] Add vitest config
- [ ] Set up TSGO (@typescript/native-preview)
