# WASM API

`modern_colorthief` ships a [WebAssembly](https://webassembly.org/) package
built from the same Rust core as the Python extension.  It exposes an
identical two-function surface so you can extract colour palettes entirely
in the browser or in Node.js — no server round-trip required.

## Building the WASM package

Install [wasm-pack](https://rustwasm.github.io/wasm-pack/) and then run:

```bash
# for use with a bundler (webpack, Vite, …)
wasm-pack build src/wasm --target bundler

# for use directly in a browser <script type="module">
wasm-pack build src/wasm --target web --out-dir pkg
```

The generated package is placed in `src/wasm/pkg/` by default.

## Usage

```js
import init, { get_palette, get_color } from './pkg/modern_colorthief_wasm.js';

await init();   // boot the WASM module once

const response = await fetch('/path/to/image.jpg');
const bytes    = new Uint8Array(await response.arrayBuffer());

console.log(get_palette(bytes, 5));   // [[r,g,b], …]
console.log(get_color(bytes));        // [r, g, b]
```

## Functions

```{eval-rst}
.. js:autofunction:: get_palette

.. js:autofunction:: get_color
```
