use colorthief_core::extract_palette_from_buffer;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn resolve(p: &js_sys::Function, val: &JsValue) {
    let _ = p.call1(&JsValue::UNDEFINED, val);
}

fn reject(p: &js_sys::Function, msg: &str) {
    let _ = p.call1(&JsValue::UNDEFINED, &JsValue::from_str(msg));
}

/// Extract a palette of dominant colors from an image.
///
/// Accepts a string URL, `Uint8Array`, or `ArrayBuffer` of raw image bytes.
/// Uses the browser's Canvas API to decode, then runs Median Cut in Rust.
///
/// # Returns
/// Promise resolving to `number[][]` — each inner array is `[R, G, B]`.
///
/// # Example
/// ```js
/// const palette = await getPalette('photo.jpg', 10, 10);
/// console.log(palette); // [[139, 69, 19], [220, 20, 60], ...]
/// ```
#[wasm_bindgen(js_name = "getPalette")]
pub fn get_palette_promise(image: &JsValue, color_count: u8, quality: u8) -> js_sys::Promise {
    js_sys::Promise::new(&mut |res_fn, rej_fn| {
        match decode_image_sync(image) {
            Ok((pixels, width, height)) => {
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
                    Err(e) => reject(&rej_fn, &e),
                }
            }
            Err(e) => reject(&rej_fn, &e),
        }
    })
}

/// Extract the dominant color from an image.
///
/// Accepts a string URL, `Uint8Array`, or `ArrayBuffer` of raw image bytes.
/// Internally extracts a small palette and returns the top color.
///
/// # Returns
/// Promise resolving to `number[]` — `[R, G, B]`.
///
/// # Example
/// ```js
/// const color = await getColor('photo.jpg', 10);
/// console.log(color); // [139, 69, 19]
/// ```
#[wasm_bindgen(js_name = "getColor")]
pub fn get_color_promise(image: &JsValue, quality: u8) -> js_sys::Promise {
    js_sys::Promise::new(&mut |res_fn, rej_fn| {
        match decode_image_sync(image) {
            Ok((pixels, width, height)) => {
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
                    Err(e) => reject(&rej_fn, &e),
                }
            }
            Err(e) => reject(&rej_fn, &e),
        }
    })
}

/// Decode an image via the browser Canvas API.
///
/// Returns a Promise resolving to `{ pixels: Uint8Array, width: number, height: number }`.
/// Useful when you need the raw pixel data for custom processing.
///
/// # Example
/// ```js
/// const { pixels, width, height } = await decodeImage('photo.jpg');
/// ```
#[wasm_bindgen(js_name = "decodeImage")]
pub fn decode_image_promise(image: &JsValue) -> js_sys::Promise {
    js_sys::Promise::new(&mut |res_fn, rej_fn| {
        match decode_image_sync(image) {
            Ok((pixels, width, height)) => {
                let obj = js_sys::Object::new();
                let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("pixels"), &pixels.into());
                let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("width"), &JsValue::from(width));
                let _ = js_sys::Reflect::set(&obj, &JsValue::from_str("height"), &JsValue::from(height));
                resolve(&res_fn, &obj);
            }
            Err(e) => reject(&rej_fn, &e),
        }
    })
}

/// Synchronously decode an image via the browser Canvas API.
fn decode_image_sync(image: &JsValue) -> Result<(js_sys::Uint8Array, u32, u32), String> {
    let window = web_sys::window().ok_or("No window available")?;
    let document = window.document().ok_or("No document available")?;
    let body = document.body().ok_or("No document body")?;

    let img = document
        .create_element("img")
        .map_err(|_| "Failed to create img element")?
        .dyn_into::<web_sys::HtmlImageElement>()
        .map_err(|_| "Failed to cast to img element")?;

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

        let blob = web_sys::Blob::new_with_u8_array_sequence(&js_sys::Uint8Array::from(bytes.as_slice()))
            .map_err(|_| "Failed to create blob")?;
        let url = web_sys::Url::create_object_url_with_blob(&blob)
            .map_err(|_| "Failed to create blob URL")?;
        img.set_src(&url);
    }

    body.append_child(&img)
        .map_err(|_| "Failed to append image")?;

    // Wait for load (synchronous via polling)
    loop {
        if img.complete() && img.natural_width() > 0 {
            break;
        }
        js_sys::Date::now();
    }

    let width = img.natural_width() as u32;
    let height = img.natural_height() as u32;
    let _ = body.remove_child(&img);

    // Draw to offscreen canvas and extract pixels
    let canvas = document
        .create_element("canvas")
        .map_err(|_| "Failed to create canvas")?
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| "Failed to cast to canvas")?;
    canvas.set_width(width);
    canvas.set_height(height);

    let ctx = canvas
        .get_context("2d")
        .map_err(|_| "Failed to get 2D context")?
        .ok_or("2D context is null")?
        .unchecked_into::<web_sys::CanvasRenderingContext2d>();

    ctx.draw_image_with_html_image_element(&img, 0.0, 0.0)
        .map_err(|_| "Failed to draw image")?;

    let image_data = ctx
        .get_image_data(0.0, 0.0, width as f64, height as f64)
        .map_err(|_| "Failed to get image data")?;

    let clamped = image_data.data();
    let pixels = js_sys::Uint8Array::from(clamped.to_vec().as_slice());

    Ok((pixels, width, height))
}
