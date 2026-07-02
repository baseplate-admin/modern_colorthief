#!/usr/bin/env python3
"""Static-analyze Rust binding crates to generate llms-<language>.txt files.

Parses lib.rs source files to extract:
- Function signatures and doc comments
- Parameter names, types, and defaults
- Return types
- Module/class names
- Crate metadata (version, name)

Usage:
    python docs/scripts/generate_llms_per_lang.py        # generate all
    python docs/scripts/generate_llms_per_lang.py --check # verify up to date
"""

from __future__ import annotations

import argparse
import re
import sys
from dataclasses import dataclass, field
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent.parent
DOCS = ROOT / "docs"

# ---------------------------------------------------------------------------
# Data types
# ---------------------------------------------------------------------------


@dataclass
class Param:
    name: str
    type_str: str
    default: str | None = None


@dataclass
class Function:
    name: str
    params: list[Param] = field(default_factory=list)
    return_type: str = ""
    docs: list[str] = field(default_factory=list)
    decorator: str = ""


# ---------------------------------------------------------------------------
# Cargo.toml parser
# ---------------------------------------------------------------------------


def read_cargo_toml(crate_path: str) -> dict:
    """Parse name and version from a Cargo.toml."""
    toml = ROOT / "crates" / crate_path / "Cargo.toml"
    result = {}
    if not toml.is_file():
        return result
    for line in toml.read_text().splitlines():
        line = line.strip()
        if "=" in line and not line.startswith("["):
            key, _, value = line.partition("=")
            key = key.strip()
            value = value.strip().strip('"').strip("'")
            if key in ("name", "version"):
                result[key] = value
    return result


# ---------------------------------------------------------------------------
# Rust source parser
# ---------------------------------------------------------------------------


def _collect_docs(lines: list[str], start: int) -> tuple[list[str], int]:
    """Collect /// doc comments starting at *start*, return (docs, next_idx)."""
    docs = []
    i = start
    while i < len(lines):
        stripped = lines[i].strip()
        if stripped.startswith("///"):
            docs.append(stripped[3:].strip())
        elif stripped == "" and i + 1 < len(lines) and lines[i + 1].strip().startswith("///"):
            pass  # blank between doc lines
        else:
            break
        i += 1
    return docs, i


def _collect_decorators(lines: list[str], start: int) -> tuple[str, int]:
    """Collect #[attr] decorators, return (primary_decorator, next_idx)."""
    decorator = ""
    i = start
    while i < len(lines):
        stripped = lines[i].strip()
        if stripped.startswith("#[") and not stripped.startswith("#[cfg"):
            m = re.search(r"#\[(\w+)", stripped)
            if m:
                decorator = m.group(1)
        else:
            break
        i += 1
    return decorator, i


def _split_params(param_block: str) -> list[str]:
    """Split a parameter block on commas, respecting generics and references."""
    parts = []
    depth = 0
    current = []
    for ch in param_block:
        if ch in ("<", "[", "{"):
            depth += 1
        elif ch in (">", "]", "}"):
            depth -= 1
        elif ch == "," and depth <= 0:
            parts.append("".join(current).strip())
            current = []
            continue
        current.append(ch)
    remainder = "".join(current).strip()
    if remainder:
        parts.append(remainder)
    return parts


def _parse_param(text: str) -> Param | None:
    """Parse one Rust parameter like `width: u32`."""
    text = text.strip()
    if not text or text.startswith("mut "):
        return None
    text = re.sub(r"'\w+\s*", "", text)  # strip lifetimes
    if ":" not in text:
        return None
    name, _, type_str = text.partition(":")
    name = name.strip().rstrip("*")
    if not name:
        return None
    type_str = type_str.strip()
    # Clean up empty generics left after lifetime stripping
    while "<>" in type_str:
        type_str = type_str.replace("<>", "")
    default = None
    if "Option<" in type_str:
        inner = re.search(r"Option<(\w+)>", type_str)
        if inner:
            type_str = inner.group(1)
            default = "None"
    return Param(name=name, type_str=type_str, default=default)


def parse_lib_rs(crate_path: str) -> list[Function]:
    """Static-analyze a lib.rs and return extracted Functions."""
    librs = ROOT / "crates" / crate_path / "src" / "lib.rs"
    if not librs.is_file():
        return []
    source = librs.read_text()
    lines = source.splitlines()
    functions = []
    i = 0
    while i < len(lines):
        # 1. Collect doc comments (may be empty)
        docs, doc_end = _collect_docs(lines, i)

        # 2. Collect decorators (may be empty)
        decorator, dec_end = _collect_decorators(lines, doc_end)

        # 3. Check for function definition at current position
        fn_start = dec_end if dec_end < len(lines) else doc_end
        if fn_start >= len(lines):
            break
        line = lines[fn_start].strip()

        # Only process if this line looks like a function definition
        if "fn " not in line:
            i += 1
            continue

        # Build a joined signature by consuming lines until balanced parens
        sig_lines = [line]
        depth = line.count("(") - line.count(")")
        idx = fn_start
        while depth > 0 and idx + 1 < len(lines):
            idx += 1
            sig_lines.append(lines[idx].strip())
            depth = "".join(sig_lines).count("(") - "".join(sig_lines).count(")")
        full_sig = " ".join(sig_lines)

        # Match fn name (allow generics like <'a> between name and paren)
        func_m = re.search(r"fn\s+(\w+)\s*(?:<[^>]*>)?\s*\(", full_sig)
        if not func_m:
            i += 1
            continue

        name = func_m.group(1)

        # 4. Extract parameter block
        after_name = full_sig[func_m.end(1) :]
        paren_idx = after_name.find("(")
        if paren_idx < 0:
            i += 1
            continue
        after_paren = after_name[paren_idx + 1 :]

        # Find the closing ')' with balanced depth
        depth = 1
        close_idx = -1
        for ci, ch in enumerate(after_paren):
            if ch == "(":
                depth += 1
            elif ch == ")":
                depth -= 1
                if depth == 0:
                    close_idx = ci
                    break
        if close_idx < 0:
            i += 1
            continue

        param_inner = after_paren[:close_idx]
        remainder = after_paren[close_idx + 1 :]

        # 5. Extract return type
        return_type = ""
        ret_m = re.search(r"->\s*(\w+)", remainder)
        if ret_m:
            return_type = ret_m.group(1)
        if not return_type:
            for j in range(idx, min(idx + 3, len(lines))):
                ret_m = re.search(r"->\s*(\w+)", lines[j])
                if ret_m:
                    return_type = ret_m.group(1)
                    break

        # 6. Parse parameters
        arrow_idx = param_inner.find("->")
        if arrow_idx >= 0:
            param_inner = param_inner[:arrow_idx]
        raw_params = _split_params(param_inner)
        params = []
        skip = {"env", "py", "_class", "_py", "mut", "__"}
        for rp in raw_params:
            p = _parse_param(rp)
            if p and p.name not in skip and not p.name.startswith("_"):
                params.append(p)

        functions.append(
            Function(
                name=name,
                params=params,
                return_type=return_type,
                docs=docs[:5],
                decorator=decorator,
            )
        )
        i = idx + 1
    return functions


# ---------------------------------------------------------------------------
# Binding definitions
# ---------------------------------------------------------------------------


@dataclass
class Binding:
    lang: str
    crate_path: str
    install_cmd: str
    package_name: str
    module_name: str
    version: str = "0.3.0"
    functions: list[Function] = field(default_factory=list)


BINDINGS: list[Binding] = [
    Binding(
        lang="python",
        crate_path="python-bindings",
        install_cmd="pip install modern_colorthief",
        package_name="modern_colorthief",
        module_name="modern_colorthief",
    ),
    Binding(
        lang="nodejs",
        crate_path="node-bindings",
        install_cmd="npm install modern-colorthief",
        package_name="modern-colorthief",
        module_name="modern-colorthief",
    ),
    Binding(
        lang="ruby",
        crate_path="ruby-bindings/ext",
        install_cmd="gem install modern_colorthief",
        package_name="modern_colorthief",
        module_name="Colorthief",
    ),
    Binding(
        lang="java",
        crate_path="jvm-bindings",
        install_cmd="Download JAR from GitHub Releases",
        package_name="modern_colorthief",
        module_name="modern_colorthief.Colorthief",
    ),
    Binding(
        lang="php",
        crate_path="php-bindings",
        install_cmd="Build and install as a PHP extension",
        package_name="modern_colorthief",
        module_name="modern_colorthief",
    ),
    Binding(
        lang="wasm",
        crate_path="wasm-bindings",
        install_cmd="npm install colorthief-wasm",
        package_name="colorthief-wasm",
        module_name="colorthief-wasm",
    ),
]


# ---------------------------------------------------------------------------
# Type / name mapping
# ---------------------------------------------------------------------------

TYPE_MAP: dict[str, dict[str, str]] = {
    "python": {
        "u8": "int", "u16": "int", "u32": "int", "u64": "int",
        "&[u8]": "bytes", "Vec<u8>": "list[int]",
        "Vec<Vec<u8>>": "list[list[int]]",
        "Vec<(u8, u8, u8)>": "list[tuple[int, int, int]]",
        "PyResult": "list[tuple[int, int, int]]",
        "String": "str",
    },
    "nodejs": {
        "u8": "number", "u16": "number", "u32": "number", "u64": "number",
        "&[u8]": "Uint8Array", "Vec<u8>": "number[]",
        "Vec<Vec<u8>>": "number[][]",
        "napi": "number[]",
        "String": "string",
    },
    "ruby": {
        "u8": "Integer", "u16": "Integer", "u32": "Integer", "u64": "Integer",
        "&[u8]": "String", "Vec<u8>": "Array<Integer>",
        "Vec<Vec<u8>>": "Array<Array<Integer>>",
        "RString": "String",
        "Result": "Array<Integer>",
        "usize": "Integer",
    },
    "java": {
        "u8": "byte", "u16": "short", "u32": "int", "u64": "long",
        "jint": "int", "jlong": "long", "jsize": "int", "jbyte": "byte",
        "&[u8]": "byte[]", "Vec<u8>": "byte[]",
        "Vec<Vec<u8>>": "byte[][]",
        "JObject": "Object", "JByteArray": "byte[]",
        "&JByteArray": "byte[]",
        "Result": "byte[][]",
        "String": "String",
    },
    "php": {
        "u8": "int", "u16": "int", "u32": "int", "u64": "int", "i64": "int",
        "&[u8]": "array<int>", "Vec<u8>": "array<int>",
        "Vec<Vec<u8>>": "array[array<int>]",
        "Vec<i64>": "array<int>",
        "Vec": "array<int>",
        "PhpResult": "array<int>",
        "String": "string",
    },
    "wasm": {
        "u8": "number", "u16": "number", "u32": "number", "u64": "number",
        "&[u8]": "Uint8Array", "Vec<u8>": "number[]",
        "Vec<Vec<u8>>": "number[][]",
        "JsValue": "any",
        "js_sys::Uint8Array": "Uint8Array",
        "js_sys": "Promise",
    },
}


def type_to_lang(rust_type: str, lang: str) -> str:
    """Map Rust type to target language, stripping generics/lifetimes."""
    direct = TYPE_MAP.get(lang, {}).get(rust_type, None)
    if direct:
        return direct
    # Strip module prefix like js_sys::
    base = re.sub(r"^\w+::", "", rust_type).strip()
    # Strip generics like <'a> or <T>
    base = re.sub(r"<.+?>", "", base).strip()
    # Strip leading &
    base = base.lstrip("& ").strip()
    mapped = TYPE_MAP.get(lang, {}).get(base, None)
    if mapped:
        return mapped
    # Return cleaned base if no mapping found
    return base if base else rust_type


def func_name_for_lang(name: str, lang: str) -> str:
    """Convert function name to the target language convention."""
    if lang == "java":
        # JNI: Java_pkg_Class_methodName -> methodName
        parts = name.split("_")
        # Skip "Java" prefix, then find the class part (capitalized)
        method_start = 0
        for idx in range(1, len(parts)):
            if parts[idx] and parts[idx][0].isupper():
                method_start = idx + 1
                break
        method_parts = parts[method_start:]
        return method_parts[0] + "".join(p.capitalize() for p in method_parts[1:])
    if lang in ("nodejs", "wasm"):
        parts = name.split("_")
        return parts[0] + "".join(p.capitalize() for p in parts[1:])
    if lang == "python":
        # Map Rust names to Python-facing names
        name_map = {
            "_get_palette_given_pixels": "get_palette",
            "_get_color_given_pixels": "get_color",
        }
        if name in name_map:
            return name_map[name]
        return name.lstrip("_")
    return name  # Ruby, PHP keep snake_case


def is_public_api(func: Function, lang: str) -> bool:
    """Filter to only public-facing functions."""
    name = func.name
    # Skip test functions
    if name.startswith("test_") or name.startswith("test"):
        return False
    # Skip internal helpers
    if name.startswith("resolve") or name in ("reject", "reject_err", "jni_err"):
        return False
    # Skip module init functions
    if "init_" in name or name.startswith("_modern_"):
        return False
    # Skip type conversion helpers
    if name in ("pixels_to_bytes", "rstring", "solid_pixels", "two_color_pixels", "build_image"):
        return False
    # Skip internal JVM helpers
    if name.startswith("extract_") and lang == "java":
        return False
    # Skip PHP module init
    if name.startswith("php_module") and lang == "php":
        return False
    # Skip WASM info/sync functions
    if name in ("version", "decode_image_sync") and lang == "wasm":
        return False
    # Skip non-function items
    if not func.params and not func.docs and func.decorator not in (
        "pyfunction", "napi_derive", "php_function", "wasm_bindgen",
    ):
        return False
    return True


def _clean_type(rust_type: str) -> str:
    """Strip generics, lifetimes, and references from a type for display."""
    cleaned = re.sub(r"<.+?>", "", rust_type).strip()
    cleaned = cleaned.lstrip("& ").strip()
    return cleaned


# ---------------------------------------------------------------------------
# Content generation
# ---------------------------------------------------------------------------

USAGE_EXAMPLES: dict[str, str] = {
    "python": """```python
from __MODULE__ import get_palette, get_color

palette = get_palette(pixels, width, height, color_count=5, quality=1)
color = get_color(pixels, width, height, quality=1)
```""",
    "nodejs": """```javascript
import { getPalette, getColor } from '__MODULE__';

const palette = await getPalette(pixels, width, height, 5, 1);
const color = await getColor(pixels, width, height, 1);
```""",
    "ruby": """```ruby
require "__MODULE__"

palette = __MODULE__.get_palette(pixels, width, height, 5, 1)
color   = __MODULE__.get_color(pixels, width, height, 1)
```""",
    "java": """```java
import __MODULE__;

byte[][] palette = Colorthief.getPalette(pixels, width, height, 5, 1);
byte[]   color  = Colorthief.getColor(pixels, width, height, 1);
```""",
    "php": """```php
<?php
$palette = get_palette($pixels, $width, $height, 5, 1);
$color   = get_color($pixels, $width, $height, 1);
```""",
    "wasm": """```javascript
import { getPalette, getColor } from '__MODULE__';

const palette = await getPalette(pixels, width, height, 5, 1);
const color   = await getColor(pixels, width, height, 1);
```""",
}


def generate_llms_for_binding(binding: Binding) -> str:
    """Generate llms-<lang>.txt content for one binding."""
    lang = binding.lang
    out = []
    out.append(f"# modern_colorthief — {lang.upper()} Binding API")
    out.append("")
    out.append(f"Package: `{binding.package_name}`")
    out.append(f"Module: `{binding.module_name}`")
    out.append(f"Install: `{binding.install_cmd}`")
    out.append(f"Version: {binding.version}")
    out.append("")

    functions = parse_lib_rs(binding.crate_path)
    if not functions:
        out.append("No functions found in source.")
        return "\n".join(out)

    out.append("## Functions")
    out.append("")

    for func in functions:
        if not is_public_api(func, lang):
            continue
        display = func_name_for_lang(func.name, lang)
        param_parts = []
        for p in func.params:
            lt = type_to_lang(p.type_str, lang)
            optional = "?: " if p.default else ": "
            param_parts.append(f"{p.name}{optional}{lt}")
        params_str = ", ".join(param_parts)
        ret = type_to_lang(func.return_type, lang) if func.return_type else "void"
        out.append(f"### `{display}({params_str})` → `{ret}`")
        out.append("")
        if func.docs:
            for doc in func.docs:
                out.append(f"  {doc}")
            out.append("")
        if func.params:
            out.append("  Parameters:")
            for p in func.params:
                lt = type_to_lang(p.type_str, lang)
                opt = " (optional)" if p.default else ""
                out.append(f"  - `{p.name}` ({lt}){opt}")
            out.append("")

    out.append("## Usage Example")
    out.append("")
    tpl = USAGE_EXAMPLES.get(lang, "")
    out.append(tpl.replace("__MODULE__", binding.module_name))

    return "\n".join(out)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------


def main() -> None:
    parser = argparse.ArgumentParser(description="Generate per-language llms files")
    parser.add_argument(
        "--check",
        action="store_true",
        help="Compare generated output with existing files, exit 1 if different",
    )
    args = parser.parse_args()

    for binding in BINDINGS:
        toml = read_cargo_toml(binding.crate_path)
        if "version" in toml:
            binding.version = toml["version"]

    changed = []
    for binding in BINDINGS:
        filename = f"llms-{binding.lang}.txt"
        path = DOCS / filename
        content = generate_llms_for_binding(binding)
        existing = path.read_text(encoding="utf-8") if path.is_file() else None
        if existing is not None and existing == content:
            continue
        changed.append(filename)
        if not args.check:
            path.write_text(content, encoding="utf-8")
            print(f"  Generated {filename}")

    if args.check:
        if changed:
            print(f"Would update: {', '.join(changed)}")
            sys.exit(1)
        print("All per-language llms files are up to date")
    else:
        print(f"Generated {len(changed)} per-language llms files")


if __name__ == "__main__":
    main()
