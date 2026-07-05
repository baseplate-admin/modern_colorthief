# Fuzz Testing

LibFuzzer targets for modern_colorthief. Modeled after [wasmtime's fuzz harness](https://github.com/bytecodealliance/wasmtime/tree/main/fuzz).

## Fuzz Targets

| Target | Description |
|--------|-------------|
| `color_extract` | Fuzz dominant color extraction (CPU + GPU) |
| `palette_extract` | Fuzz palette extraction (CPU + GPU) |
| `cpu_gpu_consistency` | Differential fuzzing: CPU vs GPU backend consistency |

## Usage

```bash
# Run a fuzz target
cargo +nightly fuzz run color_extract -- -max_len=4096

# Run with existing corpus
cargo +nightly fuzz run palette_extract -- -max_time=300

# Minimize corpus
cargo +nightly fuzz run cpu_gpu_consistency -- -minimize_crashes=0 -max_total_time=60
```

## Corpus

Corpus and artifacts are stored in `corpus/` and `artifacts/` (git-ignored).
To seed the fuzzer, place `.wasm` or raw image data files in `corpus/<target>/`.

## Safety

These fuzz targets exercise the core color extraction pipeline with arbitrary
image data. They are safe to run locally but may trigger panics on edge cases
(that's the point). Do not feed untrusted fuzz artifacts to production code
without reviewing the crash first.
