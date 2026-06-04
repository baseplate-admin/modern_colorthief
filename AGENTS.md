# AGENTS.md

This file provides guidance for AI agents working on the modern_colorthief project.

## Project Overview

Rust-based Python extension for extracting dominant colors and color palettes from images.
~100x faster than pure Python colorthief. Built with PyO3 + maturin. MIT licensed.

## Tech Stack

- **Rust** (edition 2024): Core image processing via `color_thief` + `image` crates
- **Python** (>3.10): Wrapper layer, public API (`get_palette`, `get_color`), CLI
- **PyO3** 0.28: FFI bindings, `gil_used = false` (ABI3-free mode)
- **maturin**: Build system (Rust cdylib → Python wheel)
- **uv**: Package manager, dependency resolution
- **Sphinx + Shibuya**: Documentation (RST only, no Markdown)

## Directory Structure

```
src/
  rust/
    lib.rs              # Rust core: 4 internal functions, 1 module init
  python/
    modern_colorthief/
      __init__.py       # Public API: get_palette(), get_color()
      _modern_colorthief.pyi  # Type stubs for Rust functions
      cli.py            # CLI entry point
      py.typed          # PEP 561 marker
docs/                   # Sphinx docs, RST format, shibuya theme
tests/                  # pytest tests with sample images
bench/                  # Benchmark scripts vs colorthief, fast-colorthief
examples/               # Usage examples
```

## Rust Conventions

- Functions are `#[pyfunction]` with `#[pyo3(signature = (...))]` for defaults
- Path functions take `&str`, bytes functions take `&[u8]` (zero-copy at FFI boundary)
- Errors use `.map_err(PyValueError::new_err(...))` pattern
- `extract_palette()` is the shared core: loads image → quantizes → deduplicates via `itertools::unique()`
- `get_color` functions extract a small palette (5) and return the first entry
- Module uses `gil_used = false` -- no GIL pointers stored in module state
- Release profile: `opt-level = 3`, fat LTO, strip, single codegen unit, panic=abort

## Python Conventions

- Public API in `__init__.py`: `get_palette()`, `get_color()` with full docstrings
- Input dispatch via `match` on type: `str` → location, `bytes`/`BytesIO` → bytes
- Type hints on all functions, `py.typed` marker present
- CLI uses `argparse`, entry point: `modern-colorthief`

## Development Commands

```bash
# Build
uv run maturin develop        # Install in dev mode
uv run maturin build          # Build wheel

# Test
uv run pytest tests/

# Docs
cd docs && make html

# Lint / format (Rust)
cargo clippy -- -D warnings
cargo fmt
```

## Key Rules

1. **No new runtime dependencies** -- the library has zero Python runtime deps. Keep it that way.
2. **Rust functions are internal** (prefixed `_`). Public API is the Python wrapper.
3. **Type stubs** (`_modern_colorthief.pyi`) must stay in sync with Rust signatures.
4. **Documentation is RST only** -- no Markdown in docs/, no myst-parser.
5. **Performance-first** -- this is a compute-bound library. Profile before adding abstractions.
6. **Cross-platform** -- wheels built for manylinux, musllinux, Windows, macOS, Android.

## CI/CD

GitHub Actions (maturin-generated): builds wheels for all target platforms on push,
publishes to PyPI + GitHub Releases on tag.
