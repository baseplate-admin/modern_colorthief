use modern_colorthief_core_gpu::extract_palette_from_buffer;
use wasm_bindgen::prelude::*;

fn resolve(p: &js_sys::Function, val: &JsValue) {
    let _ = p.call1(&JsValue::UNDEFINED, val);
}

fn reject(p: &js_sys::Function, msg: &str) {
    let _ = p.call1(&JsValue::UNDEFINED, &JsValue::from_str(msg));
}

fn reject_err(p: &js_sys::Function, msg: String) {
    let _ = p.call1(&JsValue::UNDEFINED, &JsValue::from_str(&msg));
}

/// Extract a palette of dominant colors from raw pixel data using the GPU backend.
///
/// Accepts a `Uint8Array` of pixel data (RGBA format, row-major, top-to-bottom).
///
/// # Returns
/// Promise resolving to `number[][]` — each inner array is `[R, G, B]`.
///
/// # Example
/// ```js
/// const palette = await getPaletteGpu(pixels, width, height, 10, 10);
/// console.log(palette); // [[139, 69, 19], [220, 20, 60], ...]
/// ```
#[wasm_bindgen(js_name = "getPaletteGpu")]
pub fn get_palette_gpu_promise(
    pixels: js_sys::Uint8Array,
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> js_sys::Promise {
    js_sys::Promise::new(&mut |res_fn, rej_fn| {
        let buf = pixels.to_vec();
        match extract_palette_from_buffer(&buf, width, height, color_count, quality) {
            Ok(colors) => {
                let result = js_sys::Array::new();
                for (r, g, b) in colors {
                    let tuple = js_sys::Array::new();
                    tuple.push(&JsValue::from(f64::from(r)));
                    tuple.push(&JsValue::from(f64::from(g)));
                    tuple.push(&JsValue::from(f64::from(b)));
                    result.push(&tuple);
                }
                resolve(&res_fn, &result);
            }
            Err(e) => reject_err(&rej_fn, e),
        }
    })
}

/// Extract the dominant color from raw pixel data using the GPU backend.
///
/// Accepts a `Uint8Array` of pixel data (RGBA format, row-major, top-to-bottom).
/// Internally extracts a small palette and returns the top color.
///
/// # Returns
/// Promise resolving to `number[]` — `[R, G, B]`.
///
/// # Example
/// ```js
/// const color = await getColorGpu(pixels, width, height, 10);
/// console.log(color); // [139, 69, 19]
/// ```
#[wasm_bindgen(js_name = "getColorGpu")]
pub fn get_color_gpu_promise(
    pixels: js_sys::Uint8Array,
    width: u32,
    height: u32,
    quality: u8,
) -> js_sys::Promise {
    js_sys::Promise::new(&mut |res_fn, rej_fn| {
        let buf = pixels.to_vec();
        match extract_palette_from_buffer(&buf, width, height, 5, quality) {
            Ok(mut colors) => {
                if let Some((r, g, b)) = colors.pop() {
                    let result = js_sys::Array::new();
                    result.push(&JsValue::from(f64::from(r)));
                    result.push(&JsValue::from(f64::from(g)));
                    result.push(&JsValue::from(f64::from(b)));
                    resolve(&res_fn, &result);
                } else {
                    reject(&rej_fn, "Image contains no extractable colors");
                }
            }
            Err(e) => reject_err(&rej_fn, e),
        }
    })
}
