use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use modern_colorthief_wasm::{get_palette_from_pixels, get_color_from_pixels};

fn solid_pixels(r: u8, g: u8, b: u8, w: u32, h: u32) -> js_sys::Uint8Array {
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for i in 0..(w * h) as usize {
        buf[i * 4] = r;
        buf[i * 4 + 1] = g;
        buf[i * 4 + 2] = b;
        buf[i * 4 + 3] = 255;
    }
    js_sys::Uint8Array::from(buf.as_slice())
}

fn two_color_pixels(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8, w: u32, h: u32) -> js_sys::Uint8Array {
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize * 4;
            if x < w / 2 {
                buf[idx] = r1;
                buf[idx + 1] = g1;
                buf[idx + 2] = b1;
            } else {
                buf[idx] = r2;
                buf[idx + 1] = g2;
                buf[idx + 2] = b2;
            }
            buf[idx + 3] = 255;
        }
    }
    js_sys::Uint8Array::from(buf.as_slice())
}

fn palette_to_vec(palette: &js_sys::Array) -> Vec<(u8, u8, u8)> {
    let mut colors = Vec::new();
    for i in 0..palette.length() {
        let tuple = palette.get(i).dyn_into::<js_sys::Array>().unwrap();
        let r = tuple.get(0).as_f64().unwrap() as u8;
        let g = tuple.get(1).as_f64().unwrap() as u8;
        let b = tuple.get(2).as_f64().unwrap() as u8;
        colors.push((r, g, b));
    }
    colors
}

fn color_to_tuple(color: &js_sys::Array) -> (u8, u8, u8) {
    let r = color.get(0).as_f64().unwrap() as u8;
    let g = color.get(1).as_f64().unwrap() as u8;
    let b = color.get(2).as_f64().unwrap() as u8;
    (r, g, b)
}

// ---------------------------------------------------------------------------
// Browser tests (require Canvas API via web-sys)
// ---------------------------------------------------------------------------

#[cfg(feature = "browser-tests")]
mod browser {
    use super::*;

    #[wasm_bindgen_test]
    fn api_exports() {
        use modern_colorthief_wasm::get_palette_promise;
        let p = get_palette_promise(&wasm_bindgen::JsValue::NULL, 5, 10);
        assert!(js_sys::Promise::instanceof(&p));
    }

    #[wasm_bindgen_test]
    async fn decode_image_via_canvas() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.create_element("canvas").unwrap();
        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        canvas.set_width(100);
        canvas.set_height(100);
        let ctx = canvas.get_context("2d").unwrap().unwrap().unchecked_into::<web_sys::CanvasRenderingContext2d>();
        ctx.fill_style = &wasm_bindgen::JsValue::from_str("rgb(255,0,0)");
        ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
        let image_data = ctx.get_image_data(0.0, 0.0, 100.0, 100.0).unwrap();
        let pixels = image_data.data();
        assert_eq!(pixels.index(0), 255);
        assert_eq!(pixels.index(1), 0);
        assert_eq!(pixels.index(2), 0);
        assert_eq!(pixels.index(3), 255);
        assert_eq!(pixels.length(), 100 * 100 * 4);
    }

    #[wasm_bindgen_test]
    async fn decode_image_dimensions() {
        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.create_element("canvas").unwrap();
        let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
        canvas.set_width(200);
        canvas.set_height(150);
        let ctx = canvas.get_context("2d").unwrap().unwrap().unchecked_into::<web_sys::CanvasRenderingContext2d>();
        ctx.fill_style = &wasm_bindgen::JsValue::from_str("rgb(0,255,0)");
        ctx.fill_rect(0.0, 0.0, 200.0, 150.0);
        let image_data = ctx.get_image_data(0.0, 0.0, 200.0, 150.0).unwrap();
        assert_eq!(image_data.data().length(), 200 * 150 * 4);
    }

    #[wasm_bindgen_test]
    async fn canvas_solid_colors() {
        for (r, g, b) in [(255u8, 0, 0), (0, 255, 0), (0, 0, 255), (255, 255, 255), (0, 0, 0)] {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            let canvas = document.create_element("canvas").unwrap();
            let canvas = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
            canvas.set_width(10);
            canvas.set_height(10);
            let ctx = canvas.get_context("2d").unwrap().unwrap().unchecked_into::<web_sys::CanvasRenderingContext2d>();
            ctx.fill_style = &format!("rgb({},{},{})", r, g, b).into();
            ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
            let data = ctx.get_image_data(0.0, 0.0, 10.0, 10.0).unwrap().data();
            assert_eq!(data.index(0), r);
            assert_eq!(data.index(1), g);
            assert_eq!(data.index(2), b);
            assert_eq!(data.index(3), 255);
        }
    }
}

// ---------------------------------------------------------------------------
// Raw pixel tests (run in any JS environment: browser, Node, Deno)
// ---------------------------------------------------------------------------

mod pixels {
    use super::*;

    #[wasm_bindgen_test]
    async fn solid_red() {
        let pixels = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels, 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        let (r, g, b) = colors[0];
        assert!(r > 200 && g < 55 && b < 55, "expected red, got ({},{},{})", r, g, b);
    }

    #[wasm_bindgen_test]
    async fn solid_green() {
        let pixels = solid_pixels(0, 255, 0, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels, 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette_to_vec(&palette)[0].1 > 200);
    }

    #[wasm_bindgen_test]
    async fn solid_white() {
        let pixels = solid_pixels(255, 255, 255, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels, 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = palette_to_vec(&palette)[0];
        assert!(r > 200 && g > 200 && b > 200);
    }

    #[wasm_bindgen_test]
    async fn solid_black() {
        let pixels = solid_pixels(0, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels, 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = palette_to_vec(&palette)[0];
        assert!(r < 55 && g < 55 && b < 55);
    }

    #[wasm_bindgen_test]
    async fn valid_rgb_values() {
        let pixels = solid_pixels(100, 150, 200, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels, 100, 100, 10, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        for i in 0..palette.length() {
            let tuple = palette.get(i).dyn_into::<js_sys::Array>().unwrap();
            assert_eq!(tuple.length(), 3);
            for j in 0..3 {
                assert!(tuple.get(j).as_f64().unwrap() >= 0.0);
                assert!(tuple.get(j).as_f64().unwrap() <= 255.0);
            }
        }
    }

    #[wasm_bindgen_test]
    async fn respects_color_count() {
        let pixels = solid_pixels(128, 64, 32, 100, 100);
        for count in [3u8, 5u8] {
            let result = JsFuture::from(get_palette_from_pixels(pixels, 100, 100, count, 10)).await.unwrap();
            let palette: js_sys::Array = result.dyn_into().unwrap();
            assert!(palette.length() <= count as u32);
        }
    }

    #[wasm_bindgen_test]
    async fn no_duplicates() {
        let pixels = solid_pixels(100, 150, 200, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels, 100, 100, 20, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        let unique: std::collections::HashSet<_> = colors.into_iter().collect();
        assert_eq!(unique.len(), colors.len());
    }

    #[wasm_bindgen_test]
    async fn color_from_pixels_red() {
        let pixels = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_color_from_pixels(pixels, 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = color_to_tuple(&color);
        assert!(r > 200 && g < 55 && b < 55);
    }

    #[wasm_bindgen_test]
    async fn color_from_pixels_green() {
        let pixels = solid_pixels(0, 255, 0, 100, 100);
        let result = JsFuture::from(get_color_from_pixels(pixels, 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = color_to_tuple(&color);
        assert!(r < 55 && g > 200 && b < 55);
    }

    #[wasm_bindgen_test]
    async fn color_rgb_length() {
        let pixels = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_color_from_pixels(pixels, 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
    }

    #[wasm_bindgen_test]
    async fn two_color_detection() {
        let pixels = two_color_pixels(255, 0, 0, 0, 0, 255, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels, 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        assert!(colors.len() >= 2);
        let has_red = colors.iter().any(|(r, g, b)| *r > 200 && *g < 55 && *b < 55);
        let has_blue = colors.iter().any(|(r, g, b)| *r < 55 && *g < 55 && *b > 200);
        assert!(has_red || has_blue);
    }

    #[wasm_bindgen_test]
    async fn determinism() {
        let pixels = solid_pixels(180, 90, 45, 100, 100);
        let p1 = JsFuture::from(get_palette_from_pixels(pixels, 100, 100, 5, 10)).await.unwrap();
        let p2 = JsFuture::from(get_palette_from_pixels(pixels, 100, 100, 5, 10)).await.unwrap();
        let p1_arr: js_sys::Array = p1.dyn_into().unwrap();
        let p2_arr: js_sys::Array = p2.dyn_into().unwrap();
        assert_eq!(palette_to_vec(&p1_arr), palette_to_vec(&p2_arr));
    }

    #[wasm_bindgen_test]
    async fn quality_bounds() {
        let pixels = solid_pixels(200, 100, 50, 100, 100);
        for q in [1u8, 5u8, 10u8] {
            let result = JsFuture::from(get_palette_from_pixels(pixels, 100, 100, 5, q)).await.unwrap();
            let palette: js_sys::Array = result.dyn_into().unwrap();
            assert!(palette.length() > 0);
        }
    }

    #[wasm_bindgen_test]
    async fn edge_1x1() {
        let pixels = solid_pixels(255, 128, 64, 1, 1);
        let result = JsFuture::from(get_palette_from_pixels(pixels, 1, 1, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn edge_large_image() {
        let pixels = solid_pixels(100, 200, 150, 500, 500);
        let result = JsFuture::from(get_palette_from_pixels(pixels, 500, 500, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn rejects_empty_pixels() {
        let empty = js_sys::Uint8Array::new(0);
        let result = JsFuture::from(get_palette_from_pixels(empty, 0, 0, 5, 10)).await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn rejects_empty_color() {
        let empty = js_sys::Uint8Array::new(0);
        let result = JsFuture::from(get_color_from_pixels(empty, 0, 0, 10)).await;
        assert!(result.is_err());
    }
}
