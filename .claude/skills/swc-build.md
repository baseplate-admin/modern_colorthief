---
name: swc-build
description: Use SWC for all JS/TS transformation and minification in Rust build scripts; use naga for WGSL shader validation
metadata:
  type: skill
  model: haiku
---

# SWC Build-Step Skill

## Core Principle

All JavaScript/TypeScript transformation and minification in this project MUST use the **Rust SWC crates** exclusively, running as **build-time** operations in `build.rs`. No runtime JS minification, no non-SWC tools, no SWC at runtime.

## What is SWC

SWC (Speedy Web Compiler) is a Rust-based compiler suite for JavaScript/TypeScript. In this project, it's used in `build.rs` to:
- Parse TypeScript source into an AST
- Strip TypeScript type annotations
- Minify the output (compress + mangle)
- Embed the result as a compile-time string in the final WASM module

### Documentation
- SWC Rust crates: https://github.com/swc-project/swc/tree/main/crates
- SWC examples: https://github.com/swc-project/swc/blob/main/tests

## Why SWC in build.rs

This project embeds a JavaScript helper (containing a color-extraction algorithm with embedded WGSL shaders) inside a Rust WASM module. The JS is written as TypeScript for maintainability but must be transformed at build time:

1. **TypeScript → JavaScript**: Strip type annotations so the embedded string is valid JS
2. **Minification**: Reduce the embedded string size to shrink the final WASM binary
3. **Zero runtime cost**: All transformation happens at compile time; the final binary contains only the processed JS string

## Architecture

```
src/helper.ts (TypeScript source)
    │
    │  build.rs runs at compile time:
    │  1. Read helper.ts
    │  2. Parse with SWC (swc_ecma_parser)
    │  3. Strip TS types (swc_ecma_transforms_typescript)
    │  4. Minify (swc_ecma_minifier)
    │  5. Write to $OUT_DIR/helper.min.js
    │
    ▼
src/lib.rs: include_str!(concat!(env!("OUT_DIR"), "/helper.min.js"))
    │
    ▼
Final WASM binary with embedded, minified JS
```

## SWC Reference Implementation

The current `crates/core_wasm/build.rs` is the canonical reference. Key patterns:

### Required Crates (build-dependencies only)

| Crate                            | Purpose                              |
| -------------------------------- | ------------------------------------ |
| `swc_common`                     | Source maps, error handling, globals |
| `swc_ecma_parser`                | Parse JS/TS into AST                 |
| `swc_ecma_codegen`               | Emit AST back to JS source           |
| `swc_ecma_transforms_typescript` | Strip TS types                       |
| `swc_ecma_minifier`              | Minify (compress + mangle)           |
| `swc_ecma_utils`                 | AST manipulation utilities           |
| `swc_ecma_visit`                 | AST visitor traits                   |
| `naga`                           | Parse + validate WGSL shaders        |
| `serde_json`                     | Parse tsconfig.json                  |

### build.rs Structure

```rust
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=src/helper.ts");
    // Optionally: rerun-if-changed for any other inputs

    let input_path = Path::new("src/helper.ts");
    let source = std::fs::read_to_string(input_path).expect("read helper.ts");

    // Parse + transform + minify using SWC
    let minified = process_with_swc(&source, input_path);

    // Write to OUT_DIR
    let out_path = Path::new(&std::env::var("OUT_DIR").unwrap()).join("helper.min.js");
    std::fs::write(&out_path, &minified).expect("write minified helper");

    eprintln!(
        "cargo:warning=helper.ts: {} -> {} bytes ({:.1}% reduction)",
        source.len(),
        minified.len(),
        (1.0 - minified.len() as f64 / source.len() as f64) * 100.0,
    );
}
```

### SWC Processing Pipeline

The pipeline has 3 stages:

**Stage 1: Parse** — Read TS source into SWC AST
```rust
use swc_common::{sync::Lrc, source_map::SourceMap, errors::Handler, input::InputTrait, input::SourceFileInput};
use swc_ecma_parser::{Parser, StringInput, Syntax, TsConfig, CapturingSyntaxExt};

let cm = Lrc::new(SourceMap::default());
let handler = Handler::default(); // or custom for better error reporting
let gm = swc_common::globals::Globals::new();
swc_common::globals::SET_GLOBALS.with(|state| state.set(gm));

let parsed = parser_parse(
    &cm,
    &handler,
    SourceFileInput::from(cm.new_source_file(input_spanned.src, source.into())),
    Syntax::Typescript(TsConfig { ..Default::default() }),
    Default::default(),
    &[],
    &[],
).map_err(|e| format!("SWC parse error: {}", e))?;
```

**Stage 2: Transform** — Strip TS types
```rust
use swc_ecma_transforms_typescript::{typescript, inline_global_types};
use swc_ecma_utils::ExprFactory;

let pass = typescript::ts({})
    .and(inline_global_types::inline(global_types, false));

let module = parsed.apply(pass).compile();
```

**Stage 3: Codegen + Minify** — Emit minified JS
```rust
use swc_ecma_minifier::{optimize, pass::OptimizerPassPhase, OptimizeOptions, compress, mangle};

// Codegen
let passed = cm.new_source_map_handler().with_emitter(js_stderr_emitter());
let mut emitter = swc_ecma_codegen::TextWriter::new(&passed);
let cfg = swc_ecma_codegen::Config::default();
let expr = swc_ecma_codegen::Node::Module(module);
expr.emit_with(&mut emitter, &cfg).unwrap();
let transformed = passed.into_bundle().into_code().unwrap();

// Minify
let minified_cm = Lrc::new(SourceMap::default());
let minified_handler = Handler::default();
let minified_result = optimize(
    &minified_cm,
    &minified_handler,
    &minified_cm.new_source_file(input_spanned.src, transformed.into()),
    None,
    &OptimizeOptions {
        mangle: false,
        ..OptimizeOptions::default()
    },
    OptimizeOptions::none().as_f64(),
    false,
    Some(OptimizerPassPhase::All),
    Default::default(),
).map_err(|e| format!("SWC minify error: {}", e))?;
let minified = minified_result.code;
```

### Consuming Build Artifacts

```rust
// In lib.rs:
const JS_HELPER: &str = include_str!(concat!(env!("OUT_DIR"), "/helper.min.js"));
```

**Rule:** Artifacts produced by `build.rs` are consumed via `include_str!`/`include_bytes!` with `concat!(env!("OUT_DIR"), "/filename")`. Never hardcode paths to `src/` for build artifacts.

## Cargo.toml Configuration

```toml
[package]
name = "my-wasm-crate"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib", "rlib"]

[dependencies]
# Runtime deps only — NO SWC crates here
wasm-bindgen = "0.2"

[build-dependencies]
# SWC crates + naga — build-time only
swc_common = "23.0"
swc_ecma_parser = "41.0"
swc_ecma_codegen = "28.0"
swc_ecma_transforms_typescript = "49.0"
swc_ecma_utils = "31.0"
swc_ecma_visit = "25.0"
swc_ecma_minifier = "55.0"
naga = { version = "29", default-features = false, features = ["wgsl-in"] }
serde_json = "1"

[dev-dependencies]
# Test deps — NO SWC crates here
wasm-bindgen-test = "0.3"
```

**Rule:** SWC crates, naga, and serde_json appear EXCLUSIVELY in `[build-dependencies]`. If you find them in `[dependencies]` or `[dev-dependencies]`, it's a bug.

## tsconfig.json Integration

SWC does NOT have a built-in tsconfig.json parser. The `swc_ecma_transforms_typescript::Config` struct maps only a small subset of tsconfig `compilerOptions`:

| tsconfig field | SWC Config field | Notes |
|---|---|---|
| `verbatimModuleSyntax` | `verbatim_module_syntax` | bool |
| `importNotUsedAsValues` | `import_not_used_as_values` | `"preserve"` → `Preserve`, else `Remove` |
| `nativeClassProperties` | `native_class_properties` | bool |
| `useDefineForClassFields` | `native_class_properties` | `false` → disable |
| `noEmptyExport` | `no_empty_export` | bool |
| `tsEnumIsMutable` | `ts_enum_is_mutable` | bool |
| `flow` | `flow_syntax` | bool |

Most tsconfig fields (`target`, `module`, `lib`, `strict`, `types`, `noEmit`) are TypeScript-compiler concepts that don't apply to SWC's type-stripping pipeline. SWC strips types purely syntactically — no type resolution needed.

**Pattern:** Read tsconfig.json in `build.rs`, deserialize with `serde_json::Value`, extract supported fields, construct `Config`, pass to `swc_ecma_transforms_typescript::typescript(config, ...)`. Always add `println!("cargo:rerun-if-changed=tsconfig.json")`.

**Anti-pattern:** Using `strip(unresolved_mark, top_level_mark)` instead of `typescript(config, unresolved_mark, top_level_mark)` — the `strip()` function is a convenience wrapper that always uses `Config::default()` and ignores tsconfig.

## Decision Checklist

Before implementing any JS/TS transformation or minification:

1. **SWC only:** Am I using SWC (not Terser, UglifyJS, esbuild, or custom solutions)?
2. **Build-step:** Is the SWC code running in `build.rs` (not at runtime)?
3. **Build-dependencies:** Are SWC crates in `[build-dependencies]` (not `[dependencies]`)?
4. **No SWC at runtime:** Does the final binary/wasm contain zero SWC code?
5. **OUT_DIR:** Is the artifact written to `$OUT_DIR` and consumed via `include_str!`?
6. **rerun-if-changed:** Did I add `println!("cargo:rerun-if-changed=...")` for each input file?
7. **Error handling:** Does the build script fail fast on parse/transform errors?
8. **WGSL validation:** If embedding WGSL shaders, are they validated with naga before AND after minification?
9. **naga parse-only:** Am I using `naga::front::wgsl::parse_str()` (not `Validator::validate()`) to avoid naga rejecting valid WebGPU shaders?
10. **Strip @entry_point:** Am I stripping `@entry_point("...")` attributes before naga parsing since naga doesn't support them yet?

## Anti-Patterns (What NOT to Do)

- **DO NOT** add SWC crates or naga to `[dependencies]` — they are build-time only
- **DO NOT** add SWC crates or naga to `[dev-dependencies]` — they are build-time only
- **DO NOT** use Terser, UglifyJS, esbuild minify, or any non-SWC minifier
- **DO NOT** run SWC transformations at runtime — they belong in `build.rs`
- **DO NOT** write build artifacts to `src/` — use `$OUT_DIR`
- **DO NOT** skip `cargo:rerun-if-changed` — incremental builds will break
- **DO NOT** bundle SWC output as a npm dependency — embed it in the wasm module
- **DO NOT** use JavaScript-based SWC (`@swc/core`) for Rust build steps — use the Rust crates
- **DO NOT** use `naga::valid::Validator::validate()` for WGSL validation — it's stricter than WebGPU and will reject valid shaders; use parse-only (`naga::front::wgsl::parse_str()`) instead
- **DO NOT** pass `@entry_point("...")` attributes to naga — strip them first or naga will fail to parse
- **DO NOT** enable naga default features — use `default-features = false, features = ["wgsl-in"]` to minimize build time

## Testing

When modifying the SWC build pipeline:
1. Run `cargo clean -p <crate>` to force a full rebuild (build.rs only runs when inputs change)
2. Verify the build compiles: `cargo check --target wasm32-unknown-unknown -p <crate>`
3. Check the build output for size reduction messages
4. Run the wasm-bindgen tests: `wasm-pack test --node --release` (from the bindings crate directory)

## Post-Minification Shader Pass

After SWC processes the TypeScript, a secondary Rust pass minifies the embedded WGSL shader string within the JS output. This pass:
- Extracts the WGSL string from JS string literals
- Removes comments, extra whitespace, and normalizes formatting
- Re-embeds the minified shader

**Rule:** This post-minification pass is complementary to SWC — it handles edge cases specific to embedded shader strings. Do not remove it unless SWC's native minifier fully covers these cases.

### WGSL Shader Validation (naga)

When a build script embeds WGSL shaders, they MUST be validated with `naga` at build time to catch syntax and type errors before shipping.

```toml
# Cargo.toml
[build-dependencies]
naga = { version = "29", default-features = false, features = ["wgsl-in"] }
```

```rust
// build.rs — validate shader before embedding
fn main() {
    println!("cargo:rerun-if-changed=src/shader.wgsl");

    let shader_source = std::fs::read_to_string("src/shader.wgsl").unwrap();

    // Validate original shader
    validate_wgsl(&shader_source, "original shader");

    // Minify WGSL
    let minified = minify_wgsl(&shader_source);

    // Validate minified output (confirms minifier didn't break the shader)
    validate_wgsl(&minified, "minified shader");

    // Embed in helper.ts, run through SWC pipeline...
}

fn validate_wgsl(source: &str, label: &str) {
    let sanitized = strip_entry_point_attrs(source);
    let module = naga::front::wgsl::parse_str(&sanitized).unwrap_or_else(|e| {
        eprintln!("error: failed to parse {label}:\n{e}");
        std::process::exit(1);
    });
    println!(
        "cargo:warning={label}: {} types, {} functions, {} global vars, {} entry points",
        module.types.len(),
        module.functions.len(),
        module.global_variables.len(),
        module.entry_points.len(),
    );
}
```

**Key rules:**

1. **Validate before AND after minification** — original shader to catch author errors, minified to confirm the minifier didn't break the shader
2. **Parse-only validation** — use `naga::front::wgsl::parse_str()` for syntax/type checking. Do NOT use `naga::valid::Validator` for full validation — naga is stricter than the WebGPU runtime and will reject valid WebGPU shaders (e.g. early returns in compute shaders, non-uniform control flow)
3. **Strip `@entry_point("...")`** — naga doesn't support the `@entry_point` attribute yet; strip it before parsing (it's redundant when the function name already matches the entry point name)
4. **`default-features = false`** — only enable `wgsl-in` feature; no need for GLSL, HLSL, SPIR-V, or output features
5. **Fail the build** — if the shader fails to parse, `process::exit(1)` with a clear error message
