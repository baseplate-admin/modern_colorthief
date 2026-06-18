use modern_colorthief_core_cpu::extract_palette_from_buffer;
use wasm_bindgen::prelude::*;

/// Extract a palette of dominant colors from raw RGBA pixel data.
///
/// Accepts a `Uint8Array` of pixel data (RGBA format, row-major, top-to-bottom).
///
/// # Returns
/// Promise resolving to `number[][]` — each inner array is `[R, G, B]`.
///
/// # Example
/// ```js
/// const palette = await getPalette(pixels, width, height, 10, 10);
/// console.log(palette); // [[139, 69, 19], [220, 20, 60], ...]
/// ```
#[wasm_bindgen]
pub fn get_palette(
    pixels: js_sys::Uint8Array,
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> js_sys::Promise {
    wasm_bindgen_futures::future_to_promise(async move {
        let palette = extract_palette_from_buffer(
            &pixels.to_vec(), width, height, color_count, quality,
        ).map_err(|e| JsValue::from_str(&e))?;

        let result = js_sys::Array::new();
        for (r, g, b) in palette {
            let tuple = js_sys::Array::new();
            tuple.push(&JsValue::from(f64::from(r)));
            tuple.push(&JsValue::from(f64::from(g)));
            tuple.push(&JsValue::from(f64::from(b)));
            result.push(&tuple);
        }
        Ok::<JsValue, JsValue>(result.into())
    })
}

/// Extract the dominant color from raw RGBA pixel data.
///
/// Accepts a `Uint8Array` of pixel data (RGBA format, row-major, top-to-bottom).
///
/// # Returns
/// Promise resolving to `number[]` — `[R, G, B]`.
///
/// # Example
/// ```js
/// const color = await getColor(pixels, width, height, 10);
/// console.log(color); // [139, 69, 19]
/// ```
#[wasm_bindgen]
pub fn get_color(
    pixels: js_sys::Uint8Array,
    width: u32,
    height: u32,
    quality: u8,
) -> js_sys::Promise {
    wasm_bindgen_futures::future_to_promise(async move {
        let mut palette = extract_palette_from_buffer(
            &pixels.to_vec(), width, height, 5, quality,
        ).map_err(|e| JsValue::from_str(&e))?;

        if let Some((r, g, b)) = palette.pop() {
            let result = js_sys::Array::new();
            result.push(&JsValue::from(f64::from(r)));
            result.push(&JsValue::from(f64::from(g)));
            result.push(&JsValue::from(f64::from(b)));
            Ok::<JsValue, JsValue>(result.into())
        } else {
            Err(JsValue::from_str("Image contains no extractable colors"))
        }
    })
}
