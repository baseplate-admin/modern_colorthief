use wasm_bindgen::prelude::*;

/// Extract a palette of dominant colors from raw pixel data using WebGPU.
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
    let buf = pixels.to_vec();
    wasm_bindgen_futures::future_to_promise(async move {
        let palette = modern_colorthief_core_wasm::extract_palette_from_buffer_webgpu(
            &buf, width, height, color_count, quality,
        ).await.map_err(|e| JsValue::from_str(&e))?;

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

/// Extract the dominant color from raw pixel data using WebGPU.
///
/// Accepts a `Uint8Array` of pixel data (RGBA format, row-major, top-to-bottom).
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
    let buf = pixels.to_vec();
    wasm_bindgen_futures::future_to_promise(async move {
        let (r, g, b) = modern_colorthief_core_wasm::extract_color_from_buffer_webgpu(
            &buf, width, height, quality,
        ).await.map_err(|e| JsValue::from_str(&e))?;

        let result = js_sys::Array::new();
        result.push(&JsValue::from(f64::from(r)));
        result.push(&JsValue::from(f64::from(g)));
        result.push(&JsValue::from(f64::from(b)));
        Ok::<JsValue, JsValue>(result.into())
    })
}
