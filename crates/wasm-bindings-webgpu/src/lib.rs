use wasm_bindgen::prelude::*;

fn err_to_js(err: modern_colorthief_core::ColorthiefError) -> JsValue {
    JsValue::from_str(&err.to_string())
}

/// Return the package version string (e.g. "0.3.0").
#[wasm_bindgen(js_name = "getVersion")]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Extract a palette of dominant colors from raw pixel data using WebGPU.
///
/// # Returns
/// Promise resolving to `number[][]` — each inner array is `[R, G, B]`.
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
        let palette = modern_colorthief_core_webgpu::extract_palette_from_buffer_webgpu(
            &buf,
            width,
            height,
            color_count,
            quality,
        )
        .await
        .map_err(err_to_js)?;

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
/// # Returns
/// Promise resolving to `number[]` — `[R, G, B]`.
#[wasm_bindgen(js_name = "getColorGpu")]
pub fn get_color_gpu_promise(
    pixels: js_sys::Uint8Array,
    width: u32,
    height: u32,
    quality: u8,
) -> js_sys::Promise {
    let buf = pixels.to_vec();
    wasm_bindgen_futures::future_to_promise(async move {
        let (r, g, b) = modern_colorthief_core_webgpu::extract_color_from_buffer_webgpu(
            &buf, width, height, quality,
        )
        .await
        .map_err(err_to_js)?;

        let result = js_sys::Array::new();
        result.push(&JsValue::from(f64::from(r)));
        result.push(&JsValue::from(f64::from(g)));
        result.push(&JsValue::from(f64::from(b)));
        Ok::<JsValue, JsValue>(result.into())
    })
}
