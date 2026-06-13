---
name: wasm-pack
description: Authoritative wasm-pack skill — all WebAssembly build, test, target, and publishing decisions must follow the official wasm-pack book documentation
metadata:
    type: project-skill
---

# wasm-pack Authoritative Skill

**Source of truth:** https://wasm-bindgen.github.io/wasm-pack/book/
**Book source:** https://github.com/wasm-bindgen/wasm-pack/tree/gh-pages/src

Every decision regarding WebAssembly builds, targets, testing, packaging, publishing, or Cargo.toml configuration in this project MUST be grounded in the wasm-pack book. Never invent approaches, skip required steps, or substitute alternatives without checking the book first.

## When This Skill Applies

This skill activates on ANY task involving:

- Building WebAssembly from Rust (`wasm-pack build`, `cargo build --target wasm32-unknown-unknown`)
- Configuring `Cargo.toml` for wasm targets (`crate-type`, `[package.metadata.wasm-pack]`)
- Running wasm tests (`wasm-pack test`)
- Publishing wasm packages (`wasm-pack pack`, `wasm-pack publish`)
- Choosing wasm build targets (`--target web|nodejs|bundler|deno|no-modules`)
- Setting up a new wasm crate (`wasm-pack new`)
- Configuring `wasm-opt`, panic strategy, profiles, or 64-bit wasm
- Any `wasm-bindgen` attribute usage or JS interop patterns

## Core Principles (from the book)

### 1. wasm-pack is a One-Stop Shop

The book states: _"This tool seeks to be a one-stop shop for building and working with rust-generated WebAssembly that you would like to interop with JavaScript, in the browser or with Node.js."_

Do not reach for manual `cargo build --target wasm32` + separate `wasm-bindgen-cli` steps. Use `wasm-pack build` as the primary build command. It handles compilation, binding generation, and npm package construction in a single step.

### 2. Quickstart Flow (book chapter: Quickstart)

The canonical wasm workflow per the book:

```
1. Install rust via rustup
2. Install wasm-pack
3. wasm-pack new hello-wasm
4. cd hello-wasm
5. wasm-pack build --target web
6. Import: import init, { greet } from "./pkg/hello_wasm.js"
7. Initialize: await init()
8. Use: greet()
9. Publish: wasm-pack publish
```

Any wasm crate in this project must follow this flow. Deviations require explicit book justification.

### 3. Prerequisites (book chapter: Prerequisites)

- **Rustup required:** `wasm-pack` relies on `wasm32-unknown-unknown` target. Rustup auto-installs it. Non-rustup setups must manually install the target from `static.rust-lang.org/dist/`.
- **npm required:** `pack`, `publish`, and `login` commands wrap the npm CLI and require npm installed. Alternative package managers are fine for consuming, but publishing requires npm CLI.
- **Node.js fetch polyfill:** Node.js target modules require a `fetch` polyfill. Import `node-fetch` and assign `Headers`, `Request`, `Response`, `fetch` to `global`.

## Build Configuration Rules

### `wasm-pack build` (book chapter: Commands/build)

#### Profile Selection

| Flag                  | Debug Assertions | Debug Info | Optimizations | Use When                  |
| --------------------- | ---------------- | ---------- | ------------- | ------------------------- |
| `--dev`               | Yes              | Yes        | No            | Development, debugging    |
| `--profiling`         | No               | Yes        | Yes           | Performance investigation |
| `--release` (default) | No               | No         | Yes           | Production shipping       |

**Rule:** Always use `--release` for production builds. If no profile is specified, `--release` is the default.

#### Target Selection

| `--target`          | Output                           | `package.json` key | Use When               |
| ------------------- | -------------------------------- | ------------------ | ---------------------- |
| `bundler` (default) | ES modules for webpack/rollup    | `module`           | Bundler pipelines      |
| `nodejs`            | CommonJS                         | `main`             | Node.js `require()`    |
| `web`               | ES modules, manual instantiation | —                  | Native browser imports |
| `no-modules`        | Global script, no ES modules     | —                  | Script tag inclusion   |
| `deno`              | ES modules for Deno              | —                  | Deno runtime           |

**Rule:** Match the target to the consumption environment. For this project's wasm bindings consumed by bundlers, use `--target bundler`. For Node.js native consumption, use `--target nodejs`.

#### Output Configuration

```bash
# Default output directory
wasm-pack build                     # → pkg/

# Custom output directory
wasm-pack build --out-dir out       # → out/

# Custom file name prefix
wasm-pack build --out-name index    # → index.js, index_bg.wasm, etc.
```

The `pkg` directory is auto-`.gitignore`d. Use `--no-gitignore` only when committing build artifacts is intentional (GitHub Pages, Deno packages, monorepo setups).

#### Scope

```bash
wasm-pack build --scope my-org      # → package.json name: "@my-org/crate-name"
```

#### Extra Cargo Options

Pass extra options to `cargo build` by appending after `--`:

```bash
wasm-pack build -- --offline
wasm-pack build -- --target wasm32-unknown-unknown
```

#### Mode

| `--mode`           | Description                                  |
| ------------------ | -------------------------------------------- |
| `normal` (default) | Install `wasm-bindgen` tools if missing      |
| `no-install`       | Build without auto-installing `wasm-bindgen` |

#### Panic Strategy

```bash
# Default: panic=abort (abort wasm instance on panic)
wasm-pack build

# Recoverable: panic=unwind (catch at FFI boundaries → JS exceptions)
wasm-pack build --panic-unwind
```

`--panic-unwind` requires nightly toolchain, `rust-src` component, and `-Z build-std=std,panic_unwind`.

#### 64-bit WebAssembly

For `wasm64-unknown-unknown` (memory64, tier-3 target):

```toml
# .cargo/config.toml
[build]
target = "wasm64-unknown-unknown"

[unstable]
build-std = ["std", "panic_abort"]
```

```toml
# rust-toolchain.toml
[toolchain]
channel = "nightly"
components = ["rust-src"]
```

### Cargo.toml Configuration (book chapter: Cargo.toml Configuration)

#### `[package.metadata.wasm-pack.profile.*]`

```toml
[package.metadata.wasm-pack.profile.dev]
wasm-opt = false

[package.metadata.wasm-pack.profile.dev.wasm-bindgen]
debug-js-glue = true
demangle-name-section = true
dwarf-debug-info = false

[package.metadata.wasm-pack.profile.release]
wasm-opt = false

[package.metadata.wasm-pack.profile.release.wasm-bindgen]
debug-js-glue = false
demangle-name-section = true
dwarf-debug-info = false
omit-default-module-path = false
split-linked-modules = false
```

**Rule:** Production wasm crates in this project MUST include `[package.metadata.wasm-pack.profile.release]` with `wasm-opt = false` (or `['-O']` if optimization is desired).

#### `crate-type` (book chapter: npm browser packages/template-deep-dive/Cargo.toml)

```toml
[lib]
crate-type = ["cdylib", "rlib"]
```

- `"cdylib"` — produces the `.wasm` file (C-compatible dynamic library)
- `"rlib"` — required for `wasm-pack test` to work (unit tests need rlib format)

**Rule:** All wasm crates MUST include both `"cdylib"` and `"rlib"` in `crate-type`.

#### `wasm-bindgen` Dependency

```toml
[dependencies]
wasm-bindgen = "0.2"
```

**Rule:** Every wasm crate MUST depend on `wasm-bindgen = "0.2"`. In Rust, `^` is implied — do not add `~` or pin to patch versions.

#### Optional Dependencies Pattern

```toml
[features]
default = ["console_error_panic_hook"]

[dependencies]
wasm-bindgen = "0.2"
console_error_panic_hook = { version = "0.1", optional = true }
wee_alloc = { version = "0.4", optional = true }
```

**Rule:** Use optional dependencies gated by features for dev-only or size-optimized crates. `console_error_panic_hook` should be the default feature for better debugging.

## Testing Rules

### `wasm-pack test` (book chapter: Commands/test)

**MANDATORY: Always test on BOTH Firefox and Chrome — never run tests on a single browser.**

```bash
# Run tests on both Firefox and Chrome in headless mode
wasm-pack test --headless --firefox --chrome

# Include Node.js tests
wasm-pack test --headless --firefox --chrome --node

# Run specific test file
wasm-pack test --headless --firefox --chrome --test web

# Run specific test by name filter
wasm-pack test --headless --firefox --chrome --test web palette_test
```

**Rule:** Every CI job and local test run MUST include at least `--firefox --chrome`. Running tests on only one browser is insufficient — wasm behavior differs across JS engines.

### Test File Structure (book chapter: template-deep-dive/tests-web-rs)

```rust
#![cfg(target_arch = "wasm32")]

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn pass() {
    assert_eq!(1 + 1, 2);
}
```

**Rules:**

1. Web tests in `tests/web.rs` MUST use `#[wasm_bindgen_test]` attribute
2. Configure with `wasm_bindgen_test_configure!(run_in_browser)` for browser tests
3. Gate with `#![cfg(target_arch = "wasm32")]` for cross-platform crates
4. Use `--node` flag for Node.js-specific tests, `--firefox`/`--chrome` for browser tests

### Test Filtering

```bash
# Run all tests in a specific file
wasm-pack test --headless --firefox --chrome --test diff_patch

# Run tests matching a name pattern
wasm-pack test --headless --firefox --chrome --test diff_patch replace

# Run lib module tests
wasm-pack test --headless --firefox --chrome --lib diff::tests
```

## Publishing (book chapter: Commands/pack-and-publish)

```bash
# Create tarball only
wasm-pack pack

# Publish to npm registry
wasm-pack publish

# Publish with tag
wasm-pack publish --tag next
```

**Rule:** `pack` and `publish` operate on the `pkg` directory produced by `wasm-pack build`. Build first, then pack/publish.

## `wasm-bindgen` Usage Patterns (book chapter: template-deep-dive/src-lib-rs)

### Import JS Functions into Rust

```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
```

### Export Rust Functions to JS

```rust
#[wasm_bindgen]
pub fn greet() -> String {
    "Hello, World!".to_string()
}
```

### Return Promises from Async Rust

```rust
#[wasm_bindgen]
pub async fn fetch_data() -> Result<String, JsValue> {
    // ...
}
```

Or manually with `wasm_bindgen_futures::future_to_promise`:

```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::future_to_promise;

#[wasm_bindgen]
pub fn get_palette() -> js_sys::Promise {
    future_to_promise(async {
        // async work here
        Ok::<JsValue, JsValue>(result.into())
    })
}
```

### Convert Rust Types to JS

```rust
// Vec<(u8, u8, u8)> → number[][]
let result = js_sys::Array::new();
for (r, g, b) in colors {
    let tuple = js_sys::Array::new();
    tuple.push(&JsValue::from(f64::from(r)));
    tuple.push(&JsValue::from(f64::from(g)));
    tuple.push(&JsValue::from(f64::from(b)));
    result.push(&tuple);
}
Ok::<JsValue, JsValue>(result.into())
```

### JS Interop with `web_sys`

```rust
let window = web_sys::window().ok_or("No window")?;
let document = window.document().ok_or("No document")?;
let canvas = document.create_element("canvas")
    .unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
```

## Log Levels (book chapter: Commands)

```bash
wasm-pack --log-level info build    # default, all messages
wasm-pack --log-level warn build    # warnings + errors only
wasm-pack --log-level error build   # errors only
wasm-pack --quiet build             # silence all stdout
wasm-pack --verbose build           # extra detail
```

## Decision Checklist

Before making any wasm-related change, verify against this checklist:

1. **Build command:** Am I using `wasm-pack build` (not manual cargo + wasm-bindgen-cli)?
2. **Target:** Did I select the correct `--target` for the consumption environment?
3. **Profile:** Am I using `--release` for production, `--dev` for debugging?
4. **Cargo.toml:** Does the crate have `crate-type = ["cdylib", "rlib"]`?
5. **wasm-bindgen:** Is `wasm-bindgen = "0.2"` a dependency?
6. **Metadata:** Is `[package.metadata.wasm-pack.profile.release]` configured?
7. **Testing:** Am I testing on **both Firefox AND Chrome** (`--firefox --chrome`)?
8. **Test attributes:** Am I using `#[wasm_bindgen_test]` for wasm tests?
9. **Browser test config:** Did I add `wasm_bindgen_test_configure!(run_in_browser)`?
10. **Cross-platform gate:** Is the test gated with `#![cfg(target_arch = "wasm32")]`?

## Anti-Patterns (What NOT to Do)

- **DO NOT** use `cargo build --target wasm32-unknown-unknown` directly — use `wasm-pack build`
- **DO NOT** run tests on only one browser — always `--firefox --chrome`
- **DO NOT** omit `"rlib"` from `crate-type` — tests will fail
- **DO NOT** use `--target no-modules` unless building a global script tag bundle
- **DO NOT** commit `pkg/` to git unless explicitly needed (Deno, GitHub Pages)
- **DO NOT** pin `wasm-bindgen` to exact patch versions — use `"0.2"` (caret implied)
- **DO NOT** skip `wasm-bindgen` dependency — it's required for all JS interop
- **DO NOT** use `wasm-pack init` — it's deprecated, use `wasm-pack build`
