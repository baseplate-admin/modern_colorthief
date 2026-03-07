/**
 * @module modern_colorthief_wasm
 *
 * WebAssembly bindings for `modern_colorthief`, compiled from Rust with
 * `wasm-bindgen`.  The API mirrors the Python package: the same two
 * functions are exposed, accepting raw image bytes so they work in any
 * browser or Node.js environment without relying on the filesystem.
 *
 * ### Quick-start
 *
 * ```bash
 * # build the WASM package
 * wasm-pack build src/wasm --target web --out-dir pkg
 * ```
 *
 * ```js
 * import init, { get_palette, get_color } from './pkg/modern_colorthief_wasm.js';
 *
 * await init();                                     // boot the WASM module
 * const bytes = new Uint8Array(await (await fetch('/photo.jpg')).arrayBuffer());
 *
 * console.log(get_palette(bytes, 5));              // [[r,g,b], …]
 * console.log(get_color(bytes));                   // [r, g, b]
 * ```
 */

/**
 * Returns the color palette extracted from the given image bytes.
 *
 * Accepts raw image bytes in any format supported by the `image` crate
 * (PNG, JPEG, WebP, GIF, BMP, …).  Duplicate colors are removed so
 * every entry in the returned array is unique.
 *
 * @param image - Raw bytes of the image file.
 * @param colorCount - Maximum number of colors to return.  Must be in the
 *   range 1–255.  Defaults to `10`.
 * @param quality - Sampling quality in the range 1–10.  Lower values
 *   produce higher-quality results at the cost of more CPU time.
 *   Defaults to `10`.
 * @returns An array of `[r, g, b]` tuples where each component is an
 *   integer in the range 0–255.
 * @throws `Error` if the image bytes cannot be decoded.
 *
 * @example
 * ```ts
 * import init, { get_palette } from 'modern_colorthief_wasm';
 *
 * await init();
 * const resp  = await fetch('/photo.jpg');
 * const bytes = new Uint8Array(await resp.arrayBuffer());
 *
 * const palette = get_palette(bytes, 5, 10);
 * // => [[255, 128, 0], [30, 140, 255], [10, 200, 90]]
 * ```
 */
export declare function get_palette(
    image: Uint8Array,
    colorCount?: number,
    quality?: number,
): Array<[number, number, number]>;

/**
 * Returns the single dominant color extracted from the given image bytes.
 *
 * Internally extracts a small palette and returns its most dominant entry.
 * Accepts raw image bytes in any format supported by the `image` crate
 * (PNG, JPEG, WebP, GIF, BMP, …).
 *
 * @param image - Raw bytes of the image file.
 * @param quality - Sampling quality in the range 1–10.  Lower values
 *   produce higher-quality results at the cost of more CPU time.
 *   Defaults to `10`.
 * @returns A single `[r, g, b]` tuple where each component is an integer
 *   in the range 0–255.
 * @throws `Error` if the image bytes cannot be decoded or no colors are
 *   found.
 *
 * @example
 * ```ts
 * import init, { get_color } from 'modern_colorthief_wasm';
 *
 * await init();
 * const resp  = await fetch('/photo.jpg');
 * const bytes = new Uint8Array(await resp.arrayBuffer());
 *
 * const [r, g, b] = get_color(bytes);
 * console.log(`Dominant color: rgb(${r}, ${g}, ${b})`);
 * ```
 */
export declare function get_color(
    image: Uint8Array,
    quality?: number,
): [number, number, number];
