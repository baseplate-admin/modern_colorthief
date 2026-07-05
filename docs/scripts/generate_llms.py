#!/usr/bin/env python3
"""Generate llms.txt and llms-full.txt dynamically from binding source files.

Mirrors the DaisyUI approach: read structured API specs from Rust source files,
concatenate them in a defined order, and emit plain-text documentation for LLMs.

Usage:
    python scripts/generate_llms.py          # writes to docs/llms/
    python scripts/generate_llms.py --check  # exits 1 if output would differ
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
DOCS = ROOT / "docs" / "llms"

# ---------------------------------------------------------------------------
# Binding definitions -- mirrors workspace.members for binding crates
# ---------------------------------------------------------------------------

BINDINGS = [
    {
        "name": "Python",
        "lang": "py",
        "crate": "python-bindings",
        "install": "pip install modern_colorthief",
        "gpu_crate": "python-bindings-gpu",
        "gpu_install": "pip install modern_colorthief_gpu",
    },
    {
        "name": "Node.js",
        "lang": "js",
        "crate": "node-bindings",
        "install": "npm install modern-colorthief",
        "gpu_crate": "node-bindings-gpu",
        "gpu_install": "npm install modern-colorthief-gpu",
    },
    {
        "name": "Ruby",
        "lang": "rb",
        "crate": "ruby-bindings/ext",
        "install": "gem install modern_colorthief",
        "gpu_crate": "ruby-bindings-gpu/ext",
        "gpu_install": "gem install modern_colorthief_gpu",
    },
    {
        "name": "Java (JVM)",
        "lang": "java",
        "crate": "jvm-bindings",
        "install": "Download JAR from GitHub Releases",
        "gpu_crate": "jvm-bindings-gpu",
        "gpu_install": "Download GPU JAR from GitHub Releases",
    },
    {
        "name": "PHP",
        "lang": "php",
        "crate": "php-bindings",
        "install": "Build and install as a PHP extension",
        "gpu_crate": "php-bindings-gpu",
        "gpu_install": "Build and install as a PHP extension",
    },
    {
        "name": "WebAssembly",
        "lang": "js",
        "crate": "wasm-bindings",
        "install": "npm install colorthief-wasm",
        "gpu_crate": "wasm-bindings-webgpu",
        "gpu_install": "npm install colorthief-wasm-webgpu",
    },
]

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------


def read_cargo_version(crate_path: str) -> str:
    """Extract version from a crate's Cargo.toml."""
    toml = ROOT / "crates" / crate_path / "Cargo.toml"
    if toml.is_file():
        for line in toml.read_text().splitlines():
            if line.startswith("version ="):
                m = re.search(r'"([^"]+)"', line)
                if m:
                    return m.group(1)
    return "0.3.0"


def extract_rust_doc_comments(lib_path: str) -> str | None:
    """Read lib.rs and extract doc comments for public functions."""
    librs = ROOT / "crates" / lib_path / "src" / "lib.rs"
    if not librs.is_file():
        return None
    return librs.read_text()


def parse_function_docs(source: str) -> list[dict]:
    """Parse Rust source to extract function signatures and doc comments."""
    functions = []
    lines = source.splitlines()
    i = 0
    while i < len(lines):
        line = lines[i]
        # Collect doc comments
        docs = []
        while i < len(lines) and (
            lines[i].strip().startswith("///") or lines[i].strip() == ""
        ):
            if lines[i].strip().startswith("///"):
                docs.append(lines[i].strip()[3:].strip())
            i += 1
        # Look for function definitions
        func_match = re.search(
            r"(?:pub\s+)?fn\s+(\w+)\s*\(([^)]*)\)", line, re.IGNORECASE
        )
        if not func_match and docs:
            # Try #[napi] or #[pyfunction] or #[php_function] decorated functions
            func_match = re.search(r"fn\s+(\w+)\s*\(([^)]*)\)", line, re.IGNORECASE)
        if func_match and docs:
            name = func_match.group(1)
            params_str = func_match.group(2)
            # Parse params
            params = []
            for p in params_str.split(","):
                p = p.strip()
                if p and not p.startswith("py:") and not p.startswith("_"):
                    param_name = p.split(":")[0].split("<")[0].split("[")[0].strip()
                    param_type = p.split(":")[1].strip() if ":" in p else "unknown"
                    if param_name and param_name not in (
                        "env",
                        "mut",
                        "py",
                    ):
                        params.append({"name": param_name, "type": param_type})
            functions.append(
                {
                    "name": name,
                    "params": params,
                    "docs": docs[:3],  # First 3 doc lines as summary
                }
            )
        i += 1
    return functions


# ---------------------------------------------------------------------------
# Content generators
# ---------------------------------------------------------------------------


def generate_header() -> str:
    """Generate the llms.txt header."""
    version = read_cargo_version("core")
    return f"""# modern_colorthief — API Index

Modern ColorThief: High-performance color palette extraction from images.

Rust core with bindings for multiple languages. Manual median-cut algorithm with rayon parallelism.
Version: {version}

## Bindings

- Python — `pip install modern_colorthief`
- Node.js — `npm install modern-colorthief`
- WASM — `npm install colorthief-wasm`
- JVM — Java/Kotlin JAR from GitHub Releases
- Ruby — Ruby gem
- PHP — PHP extension

## Common API

All bindings expose:

- `getPalette(pixels, width, height, colorCount?, quality?)` → Array of [R, G, B] colors
- `getColor(pixels, width, height, quality?)` → [R, G, B] dominant color

Parameters:
- `pixels` — Raw RGBA pixel data (4 bytes per pixel, row-major order)
- `width` — Image width in pixels
- `height` — Image height in pixels
- `colorCount` — Number of colors to extract (default: 10, range: 1..255)
- `quality` — Sampling quality 1-10, higher is faster but less accurate (default: 10)

## GPU Acceleration

GPU-accelerated variants available for each binding using Vulkan Compute (native)
or WebGPU (WASM). Package names append `-gpu` or `-webgpu`.

See llms-full.txt for complete per-binding documentation."""


def generate_binding_section(binding: dict) -> str:
    """Generate API documentation for a single binding."""
    name = binding["name"]
    crate = binding["crate"]
    install = binding["install"]
    version = read_cargo_version(crate.replace("/ext", ""))

    section = f"\n## {name} Binding\n\n"
    section += f"Install: `{install}`\n"
    section += f"Version: {version}\n\n"

    # Try to read the actual source
    source = extract_rust_doc_comments(crate)
    if source:
        funcs = parse_function_docs(source)
        if funcs:
            section += "Functions:\n"
            for func in funcs:
                if func["name"] in ("get_palette", "get_color", "getPalette", "getColor"):
                    section += f"  - `{func['name']}`: {'; '.join(func['docs'][:2])}\n"

    return section


def generate_full_llms() -> str:
    """Generate the full llms-full.txt with examples for each binding."""
    version = read_cargo_version("core")
    sections = [
        f"""# modern_colorthief — Full API Documentation

## Overview

Modern ColorThief extracts dominant colors and color palettes from images.
Rust core with manual median-cut algorithm and rayon parallelism.
Bindings for Python, Node.js, WASM, JVM (Java/Kotlin), Ruby, and PHP.
Version: {version}

## Architecture

- Core algorithm: Manual median-cut with rayon parallel processing
- Image decoding: Language-native (Pillow, Sharp, Canvas, AWT, image_processing)
- GPU acceleration: Vulkan Compute (native), WebGPU (WASM)
- Multi-GPU: Vulkan device groups for scaling across GPUs

## Common API (all bindings)

### getPalette(pixels, width, height, colorCount?, quality?) → Array<[R, G, B]>

Extract a palette of dominant colors from raw RGBA pixel data.

Parameters:
- `pixels` — Raw RGBA pixel buffer (4 bytes per pixel, row-major order)
- `width` — Image width in pixels
- `height` — Image height in pixels
- `colorCount` — Number of colors to extract (default: 10, range: 1..255)
- `quality` — Sampling quality 1-10 (default: 10). Higher is faster but less accurate.

Returns: Deduplicated array of [R, G, B] color tuples (0-255).

### getColor(pixels, width, height, quality?) → [R, G, B]

Extract the single dominant color from raw RGBA pixel data.

Parameters:
- `pixels` — Raw RGBA pixel buffer (4 bytes per pixel, row-major order)
- `width` — Image width in pixels
- `height` — Image height in pixels
- `quality` — Sampling quality 1-10 (default: 10)

Returns: [R, G, B] color tuple (0-255)."""
    ]

    # Python
    sections.append("""
## Python Binding

Install: `pip install modern_colorthief`

```python
from modern_colorthief import get_palette, get_color

# File path (auto-decodes via Pillow)
palette = get_palette("photo.jpg", color_count=10, quality=10)
color = get_color("photo.jpg", quality=10)

# Bytes
with open("photo.jpg", "rb") as f:
    palette = get_palette(f.read())

# BytesIO
import io
palette = get_palette(io.BytesIO(image_data))

# PIL Image
from PIL import Image
img = Image.open("photo.jpg")
palette = get_palette(img)
```

Image decoding uses Pillow (transparent, auto-imported).

### Python GPU

Install: `pip install modern_colorthief_gpu`

```python
from modern_colorthief_gpu import get_palette_gpu, get_color_gpu

palette = get_palette_gpu(pixels, width, height, color_count=5)
color   = get_color_gpu(pixels, width, height)
```

## Node.js Binding

Install: `npm install modern-colorthief`

```typescript
import { getPalette, getColor } from 'modern-colorthief';

// File path (auto-decodes via Sharp)
const palette = await getPalette('photo.jpg', 10, 10);
const color = await getColor('photo.jpg', 10);

// Buffer
import { readFileSync } from 'fs';
const buffer = readFileSync('photo.jpg');
const palette = await getPalette(buffer);

// Raw pixels
const pixels = new Uint8Array([255, 0, 0, 255]); // 1 red pixel
const palette = await getPalette(pixels, 1, 1, 5, 1);
```

Image decoding uses Sharp (peer dependency, auto-used).

### Node.js GPU

Install: `npm install modern-colorthief-gpu`

```typescript
import { getPaletteGpu, getColorGpu } from 'modern-colorthief-gpu';

const palette = await getPaletteGpu(pixels, width, height, 5, 1);
const color   = await getColorGpu(pixels, width, height, 1);
```

## WASM Binding

Install: `npm install colorthief-wasm`

```javascript
import { getPalette, getColor, decodeImage } from 'colorthief-wasm';

// URL (auto-decodes via Canvas)
const palette = await getPalette('photo.jpg', 10, 10);
const color = await getColor('photo.jpg', 10);

// ArrayBuffer
const response = await fetch('photo.jpg');
const buffer = await response.arrayBuffer();
const palette = await getPalette(buffer, 10, 10);

// Raw pixels
const pixels = new Uint8Array([255, 0, 0, 255]);
const palette = await getPaletteFromPixels(pixels, 1, 1, 5, 10);

// Decode to raw pixels
const { pixels, width, height } = await decodeImage('photo.jpg');
```

Image decoding uses browser Canvas API.

### WASM WebGPU

Install: `npm install colorthief-wasm-webgpu`

Uses WebGPU for GPU acceleration in the browser.

## JVM Binding (Java/Kotlin)

Package: `modern_colorthief.Colorthief`
Install: Download JAR from GitHub Releases

```java
import modern_colorthief.Colorthief;

byte[] pixels = new byte[]{(byte)255, 0, 0, (byte)255}; // 1 red pixel
byte[][] palette = Colorthief.getPalette(pixels, 1, 1, 5, 1);
byte[] color = Colorthief.getColor(pixels, 1, 1, 1);
```

```kotlin
import modern_colorthief.Colorthief

val palette = Colorthief.getPalette(pixels, 1, 1, 5, 1)
val color = Colorthief.getColor(pixels, 1, 1, 1)
```

Tested against Java 26. Android compatible.

## Ruby Binding

Install: `gem install modern_colorthief`

```ruby
require "colorthief_ruby"

# Raw RGBA pixel data
pixels = ("\xFF\x00\x00\xFF" * 100).b  # 10x10 solid red
palette = Colorthief.get_palette(pixels, 10, 10, 5, 1)
color   = Colorthief.get_color(pixels, 10, 10, 1)
```

### Ruby GPU

Install: `gem install modern_colorthief_gpu`

```ruby
require "colorthief_gpu"

palette = ColorthiefGpu.get_palette(pixels, 10, 10, 5, 1)
color   = ColorthiefGpu.get_color(pixels, 10, 10, 1)
```

## PHP Binding

Install: Build and install as a PHP extension

```php
<?php
// Raw RGBA pixel data as integer array
$pixels = [255, 0, 0, 255]; // 1 red pixel
$palette = get_palette($pixels, 1, 1, 5, 1);
$color   = get_color($pixels, 1, 1, 1);
```

## GPU Bindings

GPU-accelerated versions available using Vulkan Compute (cross-vendor: Intel, AMD, NVIDIA).
Package names: `<language>-bindings-gpu`.

WASM uses WebGPU for GPU acceleration (no Vulkan in browsers).

## Performance

~100x faster than pure Python colorthief. Uses all available CPU cores via rayon.
Release profile: opt-level=3, fat LTO, strip, panic=abort.""")

    return "\n".join(sections)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate llms.txt files")
    parser.add_argument(
        "--check",
        action="store_true",
        help="Compare generated output with existing files, exit 1 if different",
    )
    args = parser.parse_args()

    llms_content = generate_header()
    llms_full_content = generate_full_llms()

    llms_path = DOCS / "llms.txt"
    llms_full_path = DOCS / "llms-full.txt"

    if args.check:
        changed = []
        if llms_path.is_file() and llms_path.read_text(encoding="utf-8") != llms_content:
            changed.append("llms.txt")
        if llms_full_path.is_file() and llms_full_path.read_text(encoding="utf-8") != llms_full_content:
            changed.append("llms-full.txt")
        if changed:
            print(f"Would update: {', '.join(changed)}")
            sys.exit(1)
        print("llms.txt files are up to date")
        return

    llms_path.write_text(llms_content, encoding="utf-8")
    llms_full_path.write_text(llms_full_content, encoding="utf-8")
    print(f"Generated {llms_path} and {llms_full_path}")


if __name__ == "__main__":
    main()
