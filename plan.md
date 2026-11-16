# plan.md

Single source of truth for this repository's roadmap. Replaces the former `PLANS.md`,
`improvement_plan.md`, and `async-correctness-webgpu-plan.md`.

Last verified against the tree at commit `18e0fa1` (docs fix landed, Rust work not started).

---

## Decisions

- **Async only where it matters.** Node, WASM, and Python gain real awaitable APIs. JVM, Ruby, and PHP
  keep synchronous signatures but stop blocking the host (thread offload, GVL release, JNI detach).
- **Python async uses `pyo3-async-runtimes`** (tokio + `spawn_blocking`), not an `asyncio.to_thread` shim.
- **Fix every correctness defect found in the audit**, not just the async obstacles.
- **The docs site gets a live WebGPU page and demo** for `crates/wasm-bindings-webgpu`.
- **100% coverage is the gate**, made reachable by moving logic out of FFI glue and deleting dead code —
  not by blanket-excluding files.

Reality check that shapes everything below: today's async surface is cosmetic. `crates/wasm-bindings`
hand-rolls promises that resolve synchronously, `crates/node-bindings-wasm` wraps work in
`future_to_promise` with zero `.await`, and the median cut is always caller-thread work. Only the host
wrappers (`sharp`, Pillow, AWT) are genuinely async. "Async" here means moving compute off the caller's
thread.

---

## Status

### Done

**Docs site (commit `18e0fa1`)**
- RTD sub-path handling: base derived from `READTHEDOCS_LANGUAGE`/`READTHEDOCS_VERSION` in
  `docs/astro.config.ts`; `site` points at the RTD domain. This is what fixed the blank site.
- Config converted to TypeScript plus `docs/tsconfig.json` and `docs/src/env.d.ts`; `@astrojs/react` added.
- `/demo/` page with a React island (`docs/src/components/PaletteDemo.tsx` + CSS module) replacing the
  static `docs/public/demo/index.html`.
- Demo decodes via `createImageBitmap` + `getPaletteFromPixels`, sidestepping the main-thread deadlock.
- Internal links made base-relative (they 404'd under the RTD prefix).
- Leptonica MMCQ paper restored to `docs/public/pdfs/mediancut.pdf` and embedded on the median-cut page
  through a React `PdfEmbed` (server-rendered, no hydration).
- `favicon.svg` added; stale `docs/test_*.png` removed.

**Elsewhere**
- llms documents exist: `docs/llms/llms.txt`, `llms-full.txt`, six per-binding files (content stale).
- Node: `crates/node-bindings/vitest.config.ts`; TSGO wired via `rolldown-plugin-dts` +
  `@typescript/native-preview`.
- WebGPU: `crates/core_webgpu/src/shader.wgsl` + `helper.ts` implement real compute for chunk means.
- GPU CI jobs exist with SwiftShader/lavapipe/Dawn across the binding workflows.
- Test images present in 7 crate test dirs (py, py-gpu, node, php, php-gpu, wasm, wasm-webgpu).

### Partial
- WebGPU compute: shader does per-chunk means; median cut and dedup still run on the CPU.
- GPU CI: emulated adapters only, no real Intel/AMD/NVIDIA validation.
- Test images: JVM uses `tests/src/test/resources/`; Ruby has none.
- Test porting: only `node-bindings` has `test_concurrent.ts` / `test_deduplication.ts`; no
  multithread/path/cli equivalents outside Python.
- llms files: generated but stale (old package name, no `/demo/`, no `--check` in CI).

### Not started
- Vulkan Compute: no `create_compute_pipeline` / `create_shader_module` / `cmd_dispatch` anywhere in
  `crates/core_gpu` — `gpu_extract` averages chunks on the CPU (`vulkan.rs:312`).
- `docs/package.json:16` still pins `modern-colorthief-wasm: file:../crates/wasm-bindings/pkg`, so
  `.readthedocs.yml` still installs rustup + wasm-pack and compiles wasm on every docs build.
- `docs/public/wasm/` is still a hand-copied bundle with no regeneration script.
- No docs CI (`astro check`, link check, llms `--check`).
- All Rust workstreams A–F and the coverage gate (I).

---

## 0. Environment prerequisites (blocking)

This machine has **no Rust toolchain**: `~/.cargo/bin` is on `PATH` but empty, so `cargo`, `rustup`, and
`wasm-pack` resolve to nothing. Nothing in Workstreams A–F or the WebGPU demo can be built or verified
until this exists.

```powershell
winget install --id Rustlang.Rustup -e --accept-source-agreements --accept-package-agreements
# new shell, then:
rustup default stable
rustup target add wasm32-unknown-unknown
rustup component add llvm-tools-preview
cargo install wasm-pack cargo-llvm-cov cargo-nextest
```

Also required: Node 26 (present), `pytest`, `pytest-asyncio`, `pytest-cov`. Add `scripts/doctor.ps1` and
`scripts/doctor.sh` asserting every tool/version so CI and contributors fail fast.

---

## Workstream A — WASM (reference implementation)

Files: `crates/wasm-bindings/src/lib.rs`, `crates/wasm-bindings/Cargo.toml`, root `web-sys` features.

1. Delete `decode_image_sync` (`lib.rs:193-266`) including the busy-wait at `lib.rs:229`; replace with
   `load_image(image) -> impl Future<Output = Result<HtmlImageElement, String>>` built on `onload`/
   `onerror` + `JsFuture`, handling the already-complete race synchronously, revoking object URLs in all
   paths (currently leaked, `lib.rs:220`), with optional `AbortSignal`/timeout.
2. Make `getPalette`, `getColor`, `decodeImage` genuine `pub async fn` (wasm-bindgen emits real Promises).
3. Promote `wasm-bindgen-futures` from `[dev-dependencies]` to `[dependencies]`
   (`crates/wasm-bindings/Cargo.toml:24-26`).
4. Yield between `median_cut` chunks (`JsFuture::from(Promise::resolve(...))`) with the chunk size as a
   named constant. Worker + `OffscreenCanvas` is the follow-on (needs `Worker`,
   `DedicatedWorkerGlobalScope`, `OffscreenCanvas`, `MessageEvent`, `ErrorEvent` in the root manifest).
5. Reject with real `Error` objects, not `JsValue::from_str` (`lib.rs:15-21`).
6. One `colors_to_js(&[(u8,u8,u8)]) -> JsValue` replaces four `f64` push loops.
7. Fix `getColor`: `.pop()` (`lib.rs:80`) returns the worst color; core's trait default is `.first()`.

Verify: `wasm-pack test --node`, `--headless --chrome`, `--headless --firefox`; a browser test that
`await getPalette(url)` resolves and a bad URL rejects (today it spins forever).

---

## Workstream B — Node.js

1. Move compute to the libuv pool with `AsyncTask`/`impl Task` for `getPalette`/`getColor` (CPU + GPU).
   `napi` already compiles with `features = ["full"]` (`Cargo.toml:41`) and is unused; the public TS
   surface already returns `Promise`, so `index.ts` does not change.
2. Update `native.ts:43-56` and committed `index.d.ts` to async signatures; add `getPaletteSync`/
   `getColorSync` if needed.
3. Fix `native.ts` arch resolution: `getSuffix()` has no arm64 mapping, so published
   `darwin-arm64.node` / `linux-arm64-gnu.node` can never be found.
4. Delete `crates/node-bindings-wasm`: never published, never tested (only compile-checked at
   `wasm-ci.yaml:37`), `vite.config.ts` points at a non-existent `tests/node.test.js`, and it has the
   `.pop()` bug. Deleting it removes an untestable crate from the coverage gate.
5. Give `node-bindings-gpu` a real wrapper: `package.json` points at `index.js`/`index.d.ts` that do not
   exist; no scripts, no workflow, its tests skip at collection time.
6. Error fidelity: `Status::InvalidArg` for bad input, `GenericFailure` for core failures.

---

## Workstream C — Python

1. Awaitable path: `pyo3-async-runtimes` (`tokio`) + `tokio` (`rt-multi-thread`); export
   `_get_palette_async`/`_get_color_async` via `future_into_py` wrapping
   `tokio::task::spawn_blocking(|| extract_palette_from_buffer(...))`. The `py.detach` closure at
   `src/lib.rs:29` is the unit to move.
2. Move the real cost: Pillow decode + `tobytes()` (`python/modern_colorthief/__init__.py:11-30`) runs
   with the GIL. Expose `async def get_palette_from_pixels(...)` as the true awaitable; keep decode in the
   thread hop.
3. Keep sync: `get_palette`/`get_color` untouched; add `get_palette_async`/`get_color_async` on both
   packages.
4. GPU cleanup: `gil_used = false` on `#[pymodule]` (`python-bindings-gpu/src/lib.rs:100`), detach around
   `list_gpus_py` (`:83-97`), reconcile `quality` defaults (Python 10 vs Rust 1 at `:12`/`:27`) and the
   `bytes` → `ValueError` asymmetry.
5. Extend `_modern_colorthief.pyi`, add `pytest-asyncio` cases, validate tokio on musllinux and android
   before locking the dependency in.

---

## Workstream D — JVM (sync API, non-blocking host)

1. Fix the JNI symbol mismatch — shipped bug. Rust exports `Java_modern_colorthief_Colorthief_*`
   (`jvm-bindings/src/lib.rs:8`) but the class is `io.baseplate_admin.modern_colorthief.Colorthief`
   (`tests/src/io/.../Colorthief.java:3`) → `UnsatisfiedLinkError`. Rename exports (match the GPU pair) or
   move the class.
2. Stop discarding error text: `Error::JavaException` + `ThrowRuntimeExAndDefault` (`lib.rs:48`,
   `-gpu:48`) throws a message-less `RuntimeException`. Use `env.throw_new(...)` with the core string.
3. Java-side async: `ColorthiefAsync.getPaletteAsync(...) -> CompletableFuture<byte[][]>` over
   `Executors.newVirtualThreadPerTaskExecutor()` on Java 21+, cached pool otherwise. Native signature
   unchanged; CI covers 17/21/25/26 so both branches are exercised.
4. Ship a loader: the JAR embeds `META-INF/natives` (`tests/build.gradle.kts:84-89`) but nothing extracts
   or `System.loadLibrary`s it.
5. Validation parity: the GPU crate skips the `len >= w*h*4` check the CPU crate does (`:22-29`).
6. Kotlin: `AGENTS.md:15` claims Kotlin bindings; only test sources exist. Publish the coroutine wrapper
   or correct the docs.

---

## Workstream E — Ruby and PHP (sync API, no host stall)

**Ruby** (`crates/ruby-bindings/ext/src/lib.rs`)
1. Release the GVL around compute (`rb_thread_call_without_gvl` / `Ruby::thread_call_without_gvl`).
   Copy the input first — `unsafe { pixels.as_slice() }` (`:11`, `:28`) borrows `RString` storage and is
   illegal across a GVL release.
2. Build the Ruby array after re-acquiring the GVL.
3. Replace `Ruby::get().unwrap()` in error paths (`:18-24`, `:36-41`) — a panic there aborts the
   interpreter.

**PHP** (`crates/php-bindings/src/lib.rs`)
1. Change the pixel input type: `Vec<i64>` (`:21`, `:44`) costs 8 bytes per pixel byte; switch to a
   binary string (`Binary<u8>`/`&[u8]`) with `pixels_to_bytes` kept as a compat shim.
2. Validation parity; range-check in one place.
3. Document reality: ext-php-rs 0.16 has no future surface and fibers only switch at explicit suspension
   points. Keep the API sync; document "run in a worker process/FPM pool".

---

## Workstream F — Core: correctness and shared semantics

1. `core_gpu` does not use the GPU. `gpu_extract` (`core_gpu/src/vulkan.rs:312`) creates no pipeline,
   shader module, or dispatch; it enumerates adapters then averages chunks on the CPU (`:355-384`).
   Recommendation: implement it — port the WGSL kernel from `crates/core_webgpu/src/shader.wgsl` to
   SPIR-V (naga) and dispatch, mirroring `core_webgpu/src/helper.ts`. Until then every `-gpu` package is
   documented as CPU fallback.
2. One `extract_color` rule: two WASM surfaces use `.pop()`. Centralise in
   `core::PaletteExtractor::extract_color` and delete per-binding copies.
3. Real error type (`thiserror`): `ColorthiefError { EmptyBuffer, NoPixels, InvalidDimensions, Backend(String) }`
   with `Display` preserving today's strings. Lets JVM keep messages and Node pick `Status::InvalidArg`.
4. Validate once, in core: `extract_palette_from_buffer` ignores `width`/`height`
   (`core_cpu/src/lib.rs:144-146`) while only JVM checks length. Enforce `buffer.len() >= w*h*4` and update
   the permissive tests (`wasm-bindings/tests/web.rs` `accepts_partial_pixel_data`).
5. De-duplicate `GpuInfo`/`GpuDevice` (`core/src/lib.rs:4-19` vs `core_gpu/src/lib.rs:4-20`; the `core`
   copy is dead).
6. `panic = "abort"` + FFI: root `Cargo.toml:87` aborts the host on any panic and no FFI crate uses
   `catch_unwind`. Switch cdylib profiles to `unwind` and wrap each exported boundary.
7. Dedup vendor crates: PHP CPU/GPU are byte-identical apart from two strings; node/ruby/jvm pairs share
   bodies. Extract `bindings-common` for `colors_to_*` marshalling and the `get_color`-from-5-colors
   logic. This is also what makes 100% coverage reachable.

---

## Workstream G — Docs site: WebGPU, async docs, honest content

1. New `/webgpu/` page + live demo for `crates/wasm-bindings-webgpu`:
   `docs/src/content/docs/webgpu.mdx` + `WebGpuDemo.tsx` (`client:load`), reading
   `import.meta.env.BASE_URL + 'wasm-webgpu/…'`. Feature-detect `navigator.gpu`, show the adapter name
   from `requestAdapter()`, fall back to "WebGPU unavailable — see the CPU demo" linking `/demo/`.
   Document honestly: `getPaletteGpu`/`getColorGpu` are async-only, the shader does chunk means with
   CPU-side dedup, and `getVersion` vs `__version__` differ between the two wasm crates.
2. Ship the right bundle: `crates/wasm-bindings-webgpu/package.json:7-9` points at
   `pkg/modern_colorthief_wasm_gpu.js`, which wasm-pack does not emit for crate
   `modern-colorthief-webgpu`. Fix paths, build `--target web`, add `docs/scripts/sync-wasm.mjs`
   (`npm run wasm:sync`) that rebuilds and copies **both** bundles into `docs/public/wasm/` and
   `docs/public/wasm-webgpu/`.
3. Drop `modern-colorthief-wasm: file:../crates/wasm-bindings/pkg` (`docs/package.json:16`) and the
   rustup/wasm-pack steps in `.readthedocs.yml`; let CI run the sync script and fail on drift.
4. Document async per language, honestly: WASM/Python/Node awaitable examples + cancellation; JVM
   `CompletableFuture`; Ruby "does not hold the GVL"; PHP "blocking by design".
5. Content accuracy: package-name drift (`colorthief-wasm` vs `modern-colorthief-wasm` vs
   `@modern_colorthief/wasm`; `docs/scripts/generate_llms*.py:71,316`), `getPalette(file, …)` examples
   (`languages/wasm.mdx:32` — a `File` is rejected), object-vs-positional option drift
   (`quickstart.mdx:28` vs `:42`), missing `init()` in WASM examples, and regenerate all eight
   `docs/llms/*.txt` (zero mentions of `/demo/`).
6. Warnings: the `i18n` collection warning and the missing 404 entry (add `404.mdx`).

---

## Workstream H — CI and release plumbing

1. Docs workflow (does not exist): `astro check`, `npm run build`, link check,
   `python docs/scripts/generate_llms.py --check`, `npm run wasm:sync -- --check`.
2. `core_webgpu` is in no workflow `paths:` filter — shader/helper changes trigger nothing.
3. Actually run fuzzing: `fuzz.yml` only does `cargo check --bins`; install `cargo-fuzz` and run a
   bounded session per PR.
4. Release consistency: Ruby publishes only on `refs/tags/v*` while everything else uses
   `refs/tags/*`; unify, and pick one version source (`0.3.0` hardcoded in 12+ places).
5. Metadata: add `license`, `repository`, `description`, `rust-version` to the workspace; the two
   published crates currently have none.
6. Lint gates: prettier/eslint for `docs/` and node packages, `ruff`, `ktlint`, `rubocop`, PHP CS — at
   minimum the JS/TS one, since `docs/` is now a TypeScript project.

---

## Workstream I — Coverage to 100%

100% across 17 crates including JNI/PHP/magnus glue and hardware-gated Vulkan is not reachable. The
plan is: move logic into testable crates, delete dead code, then enforce.

| Tier | Scope | Gate |
| --- | --- | --- |
| 1 | `core`, `core_cpu`, `bindings-common` (new) | 100% lines, functions, regions |
| 2 | Pure logic in bindings and CLI | 100% |
| 3 | FFI glue (JNI, `#[pymodule]`, `#[napi]`, `#[wasm_bindgen]` shims, `build.rs`) | `#[coverage(off)]` (stable since Rust 1.86), measured not gated |
| 4 | Hardware/browser paths (Vulkan, WebGPU, DOM decode) | measured; mocks where feasible, otherwise documented |

Tools: `cargo-llvm-cov` + `cargo-nextest` (`--fail-under-lines 100 --fail-under-functions 100`,
`--doctests` for `core`); `pytest-cov --cov-fail-under=100 --cov-branch`; `vitest --coverage` with v8
thresholds; JaCoCo/Kover; SimpleCov `minimum_coverage 100`; PCOV via Pest for PHP; WASM measured through
the host target.

Before 100% is reachable: delete dead code (`core`'s duplicate `GpuInfo`, `core_gpu`'s unused `image`,
unused `rayon` in `jvm-bindings`/`node-bindings`, `core`'s placeholder test); convert the `Err(_) => {}`
tests in `core_gpu`, the JVM `assertTrue(true)` stubs, and the collection-time-skipped node-gpu tests into
real assertions or explicit ignores; add the missing error-path and boundary tests; give `core_webgpu`
tests (it has none, and its `build.rs` naga/swc pipeline is uncovered).

Thresholds live in the repo (`codecov.yml`, `vitest.config.ts`, `build.gradle.kts`, `.simplecov`,
`phpunit.xml`), not in workflow YAML. Nightly runs the Tier 4 hardware suites.

---

## Open checklist (carried over from `PLANS.md`)

### GPU acceleration
- [ ] Vulkan compute for sampling and median cut — **not implemented**; `core_gpu` averages on the CPU
- [ ] WebGPU compute for WASM — **partial**: chunk means on GPU, median cut + dedup on CPU
- [ ] Cross-vendor validation (Intel, AMD, NVIDIA) — **partial**: CI uses SwiftShader/lavapipe only
- [x] CI jobs exercising GPU bindings (CPU-fallback with a warning)

### Documentation
- [x] `llms.txt` index — `docs/llms/llms.txt`
- [x] `llms-full.txt` — present
- [x] Per-binding llms files — six present
- [ ] Auto-regenerate llms on API change — no hook; scripts are manual (and output is stale)

### Cleanup
- [ ] Remove `crates/python-bindings/examples/benchmark.py` — still present (with `test.py`)
- [ ] Copy test images to all binding test dirs — 7 crates done; Ruby missing
- [ ] Port remaining tests (concurrent, multithread, cli, path, deduplication) to all bindings —
  only `node-bindings` has concurrent + dedup
- [ ] Remove the root `examples/` directory — still present (`test.jpg`, `test.py`, `test_time.py`)

### Node bindings
- [x] Add vitest config — `crates/node-bindings/vitest.config.ts`
- [x] Set up TSGO — `rolldown-plugin-dts` + `@typescript/native-preview`

---

## Sequencing

1. Section 0 (toolchain) → F6 (`catch_unwind`, panic profile) → F3 (error enum) → F4 (validation).
2. F7 (`bindings-common`) → I (tiering, exclusions, tests).
3. A (WASM) + G2/G3 (bundle sync) — fixes the tab-hang bug and unblocks the demo pages.
4. B (Node) + C (Python) — where async has real consumers.
5. D (JVM symbols/errors/async wrapper), E (Ruby GVL, PHP input type).
6. F1 (`core_gpu` Vulkan dispatch) — largest item; `-gpu` packages stay labelled CPU fallback.
7. G1/G4/G5 (WebGPU page, async docs, content) → H (CI) → final 100% enforcement.

## Risks

- **No local Rust toolchain** — nothing in A–F can be verified until section 0 is done.
- **tokio in the Python wheel matrix** (musl, android, abi3-py39) — validate the two hardest targets first.
- **`panic = "unwind"` changes release codegen** (LTO, size, perf) — measure wasm/node binaries before and
  after.
- **Coverage gates can incentivise bad tests** — enforce on Tier 1/2 only; a coverage-driven test must
  assert behaviour, not implementation.
- **WASM yielding** trades throughput for responsiveness — chunk size as a constant, benchmark before and
  after.
- **Vulkan compute** may not port cleanly across vendors — own milestone, WebGPU path as reference.
- **API drift** across three async ecosystems makes `docs/` and `llms/` the surface most at risk — hence
  the `--check` gates in H1.
