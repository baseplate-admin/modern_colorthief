use color_thief::ColorFormat;
use image::DynamicImage;
use itertools::Itertools;
use js_sys::Array;
use wasm_bindgen::prelude::*;

fn get_image_buffer(img: DynamicImage) -> (Vec<u8>, ColorFormat) {
    match img {
        DynamicImage::ImageRgb8(buffer) => (buffer.to_vec(), ColorFormat::Rgb),
        DynamicImage::ImageRgba8(buffer) => (buffer.to_vec(), ColorFormat::Rgba),
        other => {
            // Convert any other format to RGB8 for compatibility
            let rgb = other.to_rgb8();
            (rgb.to_vec(), ColorFormat::Rgb)
        }
    }
}

fn palette_inner(
    image: &[u8],
    color_count: Option<u8>,
    quality: Option<u8>,
) -> Result<Vec<(u8, u8, u8)>, String> {
    let img = image::load_from_memory(image)
        .map_err(|e| format!("Failed to load image from memory: {e}"))?;

    let (buffer, color_type) = get_image_buffer(img);

    let colors = color_thief::get_palette(
        &buffer,
        color_type,
        quality.unwrap_or(10),
        color_count.unwrap_or(10),
    )
    .map_err(|e| format!("Failed to extract palette: {e:?}"))?;

    Ok(colors.iter().map(|c| (c.r, c.g, c.b)).unique().collect())
}

/// Sets up a browser-friendly panic hook so that Rust panics appear as
/// readable error messages in the browser console instead of generic
/// `RuntimeError: unreachable` messages.
///
/// This is called automatically when the WASM module is first loaded.
#[wasm_bindgen(start)]
pub fn init_panic_hook() {
    console_error_panic_hook::set_once();
}

/// Returns the color palette extracted from the given image bytes.
///
/// Accepts raw image bytes in any format supported by the `image` crate
/// (PNG, JPEG, WebP, GIF, BMP, …).  The palette is deduplicated so that
/// each returned color is unique.
///
/// @param {Uint8Array} image - Raw image bytes.
/// @param {number} [color_count=10] - Maximum number of colors to return (1–255).
/// @param {number} [quality=10] - Sampling quality (1–10).  Lower values give
///   higher quality at the cost of more CPU time.
/// @returns {Array<[number, number, number]>} Array of `[r, g, b]` tuples,
///   each component in the range 0–255.
/// @throws {Error} If the image bytes cannot be decoded.
///
/// @example
/// ```js
/// import init, { get_palette } from 'modern_colorthief_wasm';
///
/// await init();
/// const resp = await fetch('/photo.jpg');
/// const bytes = new Uint8Array(await resp.arrayBuffer());
/// const palette = get_palette(bytes, 5);
/// // => [[255, 128, 0], [30, 140, 255], …]
/// ```
#[wasm_bindgen]
pub fn get_palette(
    image: &[u8],
    color_count: Option<u8>,
    quality: Option<u8>,
) -> Result<Array, JsError> {
    let palette = palette_inner(image, color_count, quality).map_err(|e| JsError::new(&e))?;

    let result = Array::new();
    for (r, g, b) in palette {
        let color = Array::new();
        color.push(&JsValue::from(r as u32));
        color.push(&JsValue::from(g as u32));
        color.push(&JsValue::from(b as u32));
        result.push(&color);
    }
    Ok(result)
}

/// Returns the single dominant color extracted from the given image bytes.
///
/// Internally extracts a small palette and returns its first (most dominant)
/// entry.  Accepts raw image bytes in any format supported by the `image`
/// crate (PNG, JPEG, WebP, GIF, BMP, …).
///
/// @param {Uint8Array} image - Raw image bytes.
/// @param {number} [quality=10] - Sampling quality (1–10).  Lower values give
///   higher quality at the cost of more CPU time.
/// @returns {[number, number, number]} A single `[r, g, b]` tuple representing
///   the dominant color, each component in the range 0–255.
/// @throws {Error} If the image bytes cannot be decoded or no colors are found.
///
/// @example
/// ```js
/// import init, { get_color } from 'modern_colorthief_wasm';
///
/// await init();
/// const resp = await fetch('/photo.jpg');
/// const bytes = new Uint8Array(await resp.arrayBuffer());
/// const [r, g, b] = get_color(bytes);
/// console.log(`Dominant color: rgb(${r}, ${g}, ${b})`);
/// ```
#[wasm_bindgen]
pub fn get_color(image: &[u8], quality: Option<u8>) -> Result<Array, JsError> {
    let palette = palette_inner(image, Some(5), quality).map_err(|e| JsError::new(&e))?;

    let (r, g, b) = palette
        .into_iter()
        .next()
        .ok_or_else(|| JsError::new("No colors found in image"))?;

    let color = Array::new();
    color.push(&JsValue::from(r as u32));
    color.push(&JsValue::from(g as u32));
    color.push(&JsValue::from(b as u32));
    Ok(color)
}
