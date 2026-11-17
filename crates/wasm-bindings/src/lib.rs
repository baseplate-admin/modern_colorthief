use modern_colorthief_core::bindings::dominant_color;
use modern_colorthief_core::ColorthiefError;
use modern_colorthief_core_cpu::extract_palette_from_buffer;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Return the package version string (e.g. "0.3.0").
#[wasm_bindgen(js_name = "__version__")]
pub fn version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn resolve(p: &js_sys::Function, val: &JsValue) {
    let _ = p.call1(&JsValue::UNDEFINED, val);
}

fn reject_err(p: &js_sys::Function, err: ColorthiefError) {
    let _ = p.call1(&JsValue::UNDEFINED, &JsValue::from_str(&err.to_string()));
}

fn reject_str(p: &js_sys::Function, msg: &str) {
    let _ = p.call1(&JsValue::UNDEFINED, &JsValue::from_str(msg));
}

/// Build a JS array of `number[][]` from an `Rgb` palette.
fn palette_to_js_array(palette: &[(u8, u8, u8)]) -> js_sys::Array {
    let result = js_sys::Array::new();
    for (r, g, b) in palette {
        let tuple = js_sys::Array::new();
        tuple.push(&JsValue::from(f64::from(*r)));
        tuple.push(&JsValue::from(f64::from(*g)));
        tuple.push(&JsValue::from(f64::from(*b)));
        result.push(&tuple);
    }
    result
}

/// Build a JS array of `number[]` — `[R, G, B]` — from a single color.
fn color_to_js_array(r: u8, g: u8, b: u8) -> js_sys::Array {
    let result = js_sys::Array::new();
    result.push(&JsValue::from(f64::from(r)));
    result.push(&JsValue::from(f64::from(g)));
    result.push(&JsValue::from(f64::from(b)));
    result
}

/// Extract a palette of dominant colors from an image.
///
/// Accepts a string URL, `Uint8Array`, or `ArrayBuffer` of raw image bytes.
/// Uses the browser's Canvas API to decode, then runs Median Cut in Rust.
///
/// # Returns
/// Promise resolving to `number[][]` — each inner array is `[R, G, B]`.
#[wasm_bindgen(js_name = "getPalette")]
pub fn get_palette_promise(image: &JsValue, color_count: u8, quality: u8) -> js_sys::Promise {
    js_sys::Promise::new(&mut |res_fn, rej_fn| {
        let image = image.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match decode_image_async(&image).await {
                Ok((pixels, width, height)) => {
                    let buf = pixels.to_vec();
                    match extract_palette_from_buffer(&buf, width, height, color_count, quality) {
                        Ok(colors) => resolve(&res_fn, &palette_to_js_array(&colors).into()),
                        Err(e) => reject_err(&rej_fn, e),
                    }
                }
                Err(msg) => reject_str(&rej_fn, &msg),
            }
        });
    })
}

/// Extract the dominant color from an image.
///
/// The dominant color is the **first** entry of a 5-color palette.
///
/// # Returns
/// Promise resolving to `number[]` — `[R, G, B]`.
#[wasm_bindgen(js_name = "getColor")]
pub fn get_color_promise(image: &JsValue, quality: u8) -> js_sys::Promise {
    js_sys::Promise::new(&mut |res_fn, rej_fn| {
        let image = image.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match decode_image_async(&image).await {
                Ok((pixels, width, height)) => {
                    let buf = pixels.to_vec();
                    match extract_palette_from_buffer(&buf, width, height, 5, quality) {
                        Ok(colors) => {
                            let (r, g, b) = match dominant_color(&colors) {
                                Ok(c) => c,
                                Err(e) => {
                                    reject_err(&rej_fn, e);
                                    return;
                                }
                            };
                            resolve(&res_fn, &color_to_js_array(r, g, b).into());
                        }
                        Err(e) => reject_err(&rej_fn, e),
                    }
                }
                Err(msg) => reject_str(&rej_fn, &msg),
            }
        });
    })
}

/// Decode an image via the browser Canvas API.
///
/// # Returns
/// Promise resolving to `{ pixels: Uint8Array, width: number, height: number }`.
#[wasm_bindgen(js_name = "decodeImage")]
pub fn decode_image_promise(image: &JsValue) -> js_sys::Promise {
    js_sys::Promise::new(&mut |res_fn, rej_fn| {
        let image = image.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match decode_image_async(&image).await {
                Ok((pixels, width, height)) => {
                    let obj = js_sys::Object::new();
                    let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("pixels"), &pixels.into());
                    let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("width"), &JsValue::from(width));
                    let _ =
                        js_sys::Reflect::set(&obj, &JsValue::from_str("height"), &JsValue::from(height));
                    resolve(&res_fn, &obj);
                }
                Err(msg) => reject_str(&rej_fn, &msg),
            }
        });
    })
}

/// Extract a palette from raw RGBA pixel data (no image decoding).
#[wasm_bindgen(js_name = "getPaletteFromPixels")]
pub fn get_palette_from_pixels(
    pixels: js_sys::Uint8Array,
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> js_sys::Promise {
    js_sys::Promise::new(&mut |res_fn, rej_fn| {
        let buf = pixels.to_vec();
        wasm_bindgen_futures::spawn_local(async move {
            match extract_palette_from_buffer(&buf, width, height, color_count, quality) {
                Ok(colors) => resolve(&res_fn, &palette_to_js_array(&colors).into()),
                Err(e) => reject_err(&rej_fn, e),
            }
        });
    })
}

/// Extract the dominant color from raw RGBA pixel data (no image decoding).
#[wasm_bindgen(js_name = "getColorFromPixels")]
pub fn get_color_from_pixels(
    pixels: js_sys::Uint8Array,
    width: u32,
    height: u32,
    quality: u8,
) -> js_sys::Promise {
    js_sys::Promise::new(&mut |res_fn, rej_fn| {
        let buf = pixels.to_vec();
        wasm_bindgen_futures::spawn_local(async move {
            match extract_palette_from_buffer(&buf, width, height, 5, quality) {
                Ok(colors) => {
                    let (r, g, b) = match dominant_color(&colors) {
                        Ok(c) => c,
                        Err(e) => {
                            reject_err(&rej_fn, e);
                            return;
                        }
                    };
                    resolve(&res_fn, &color_to_js_array(r, g, b).into());
                }
                Err(e) => reject_err(&rej_fn, e),
            }
        });
    })
}

/// Decode an image via the browser Canvas API (async).
///
/// Returns a future resolving to `(Uint8Array, width, height)`.
async fn decode_image_async(image: &JsValue) -> Result<(js_sys::Uint8Array, u32, u32), String> {
    let window = web_sys::window().ok_or("No window available".to_string())?;
    let document = window.document().ok_or("No document available".to_string())?;
    let body = document.body().ok_or("No document body".to_string())?;

    let img = document
        .create_element("img")
        .map_err(|_| "Failed to create img element".to_string())?
        .dyn_into::<web_sys::HtmlImageElement>()
        .map_err(|_| "Failed to cast to img element".to_string())?;

    if image.is_string() {
        let url = image.as_string().unwrap_or_default();
        img.set_src(&url);
    } else {
        let bytes: Vec<u8> = if image.is_instance_of::<js_sys::Uint8Array>() {
            image.dyn_ref::<js_sys::Uint8Array>().unwrap().to_vec()
        } else if image.is_instance_of::<js_sys::ArrayBuffer>() {
            let buf = image.dyn_ref::<js_sys::ArrayBuffer>().unwrap();
            js_sys::Uint8Array::new(buf).to_vec()
        } else {
            return Err("Expected string URL, Uint8Array, or ArrayBuffer".to_string());
        };

        let blob =
            web_sys::Blob::new_with_u8_array_sequence(&js_sys::Uint8Array::from(bytes.as_slice()))
                .map_err(|_| "Failed to create blob".to_string())?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|_| "Failed to create blob URL".to_string())?;
        img.set_src(&url);
    }

    body.append_child(&img)
        .map_err(|_| "Failed to append image".to_string())?;

    // Wait for image load via onload promise (non-blocking).
    // Closures are leaked to stay alive for the JS event loop.
    let load_promise = js_sys::Promise::new(&mut |res_fn: js_sys::Function, rej_fn: js_sys::Function| {
        let onload: Closure<dyn Fn()> = Closure::new(move || {
            let _ = res_fn.call0(&JsValue::UNDEFINED);
        });
        let onerror: Closure<dyn Fn()> = Closure::new(move || {
            let _ = rej_fn.call1(&JsValue::UNDEFINED, &JsValue::from_str("Image load failed"));
        });
        // set_onload borrows the Function — pass a reference into the leaked closure
        img.set_onload(Some(onload.as_ref().unchecked_ref::<js_sys::Function>()));
        img.set_onerror(Some(onerror.as_ref().unchecked_ref::<js_sys::Function>()));
        // Leak so the JS event loop can call them later
        let _ = unsafe { std::ptr::read(&onload) };
        let _ = unsafe { std::ptr::read(&onerror) };
    });
    wasm_bindgen_futures::JsFuture::from(load_promise)
        .await
        .map_err(|_| "Image failed to load".to_string())?;

    let width = img.natural_width() as u32;
    let height = img.natural_height() as u32;
    if width == 0 || height == 0 {
        let _ = body.remove_child(&img);
        return Err("Image has zero dimensions".to_string());
    }
    let _ = body.remove_child(&img);

    // Draw to offscreen canvas and extract pixels
    let canvas = document
        .create_element("canvas")
        .map_err(|_| "Failed to create canvas".to_string())?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| "Failed to cast to canvas".to_string())?;
    canvas.set_width(width);
    canvas.set_height(height);

    let ctx = canvas
        .get_context("2d")
        .map_err(|_| "Failed to get 2D context".to_string())?
        .ok_or("2D context is null".to_string())?
        .unchecked_into::<web_sys::CanvasRenderingContext2d>();

    ctx.draw_image_with_html_image_element(&img, 0.0, 0.0)
        .map_err(|_| "Failed to draw image".to_string())?;

    let image_data = ctx
        .get_image_data(0.0, 0.0, width as f64, height as f64)
        .map_err(|_| "Failed to get image data".to_string())?;

    let clamped = image_data.data();
    let pixels = js_sys::Uint8Array::from(clamped.to_vec().as_slice());

    Ok((pixels, width, height))
}
