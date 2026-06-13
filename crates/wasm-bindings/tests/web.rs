use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

#[cfg(feature = "browser-tests")]
wasm_bindgen_test_configure!(run_in_browser);

use modern_colorthief_wasm::{
    get_palette_from_pixels, get_color_from_pixels,
    get_palette_promise, get_color_promise, version,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn solid_pixels(r: u8, g: u8, b: u8, w: u32, h: u32) -> Vec<u8> {
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for i in 0..(w * h) as usize {
        buf[i * 4] = r;
        buf[i * 4 + 1] = g;
        buf[i * 4 + 2] = b;
        buf[i * 4 + 3] = 255;
    }
    buf
}

fn two_color_pixels(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8, w: u32, h: u32) -> Vec<u8> {
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
    buf
}

fn gradient_pixels(w: u32, h: u32) -> Vec<u8> {
    let mut buf = vec![0u8; (w * h * 4) as usize];
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize * 4;
            buf[idx] = (x as f64 / w as f64 * 255.0) as u8;
            buf[idx + 1] = (y as f64 / h as f64 * 255.0) as u8;
            buf[idx + 2] = ((x + y) as f64 / (w + h) as f64 * 255.0) as u8;
            buf[idx + 3] = 255;
        }
    }
    buf
}

/// Create a Uint8Array view over a Vec<u8>. The vec must outlive the view.
fn pixels_view(buf: &[u8]) -> js_sys::Uint8Array {
    js_sys::Uint8Array::from(&buf[..])
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
    use std::ops::Index;

    #[allow(deprecated)]
    fn set_fill_style(ctx: &web_sys::CanvasRenderingContext2d, value: &str) {
        ctx.set_fill_style(&wasm_bindgen::JsValue::from_str(value));
    }

    #[wasm_bindgen_test]
    fn api_exports() {
        let p = get_palette_promise(&wasm_bindgen::JsValue::NULL, 5, 10);
        assert!(p.is_instance_of::<js_sys::Promise>());
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
        set_fill_style(&ctx, "rgb(255,0,0)");
        ctx.fill_rect(0.0, 0.0, 100.0, 100.0);
        let image_data = ctx.get_image_data(0.0, 0.0, 100.0, 100.0).unwrap();
        let pixels = image_data.data();
        assert_eq!(*pixels.index(0), 255);
        assert_eq!(*pixels.index(1), 0);
        assert_eq!(*pixels.index(2), 0);
        assert_eq!(*pixels.index(3), 255);
        assert_eq!(pixels.len(), (100 * 100 * 4) as usize);
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
        set_fill_style(&ctx, "rgb(0,255,0)");
        ctx.fill_rect(0.0, 0.0, 200.0, 150.0);
        let image_data = ctx.get_image_data(0.0, 0.0, 200.0, 150.0).unwrap();
        assert_eq!(image_data.data().len(), (200 * 150 * 4) as usize);
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
            set_fill_style(&ctx, &format!("rgb({},{},{})", r, g, b));
            ctx.fill_rect(0.0, 0.0, 10.0, 10.0);
            let data = ctx.get_image_data(0.0, 0.0, 10.0, 10.0).unwrap().data();
            assert_eq!(*data.index(0), r);
            assert_eq!(*data.index(1), g);
            assert_eq!(*data.index(2), b);
            assert_eq!(*data.index(3), 255);
        }
    }
}

// ---------------------------------------------------------------------------
// Raw pixel tests (run in any JS environment: browser, Node, Deno)
// Ported from Python test suite: test_properties.py, test_edge_cases.py,
// test_errors.py, test_input_types.py, test_deduplication.py, test_concurrent.py,
// test_version.py, test_api.py, test_path.py
// ---------------------------------------------------------------------------

mod pixels {
    use super::*;

    // ---- Solid color palette tests ----

    #[wasm_bindgen_test]
    async fn solid_red() {
        let buf = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        let (r, g, b) = colors[0];
        assert!(r > 200 && g < 55 && b < 55, "expected red, got ({},{},{})", r, g, b);
    }

    #[wasm_bindgen_test]
    async fn solid_green() {
        let buf = solid_pixels(0, 255, 0, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette_to_vec(&palette)[0].1 > 200);
    }

    #[wasm_bindgen_test]
    async fn solid_white() {
        let buf = solid_pixels(255, 255, 255, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = palette_to_vec(&palette)[0];
        assert!(r > 200 && g > 200 && b > 200);
    }

    #[wasm_bindgen_test]
    async fn solid_black() {
        let buf = solid_pixels(0, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = palette_to_vec(&palette)[0];
        assert!(r < 55 && g < 55 && b < 55);
    }

    // ---- Solid color dominant color tests ----

    #[wasm_bindgen_test]
    async fn color_from_pixels_red() {
        let buf = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_color_from_pixels(pixels_view(&buf), 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = color_to_tuple(&color);
        assert!(r > 200 && g < 55 && b < 55);
    }

    #[wasm_bindgen_test]
    async fn color_from_pixels_green() {
        let buf = solid_pixels(0, 255, 0, 100, 100);
        let result = JsFuture::from(get_color_from_pixels(pixels_view(&buf), 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = color_to_tuple(&color);
        assert!(r < 55 && g > 200 && b < 55);
    }

    #[wasm_bindgen_test]
    async fn color_rgb_length() {
        let buf = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_color_from_pixels(pixels_view(&buf), 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
    }

    // ---- Two-color detection (from test_path.py::test_consistent_across_inputs) ----

    #[wasm_bindgen_test]
    async fn two_color_detection() {
        let buf = two_color_pixels(255, 0, 0, 0, 0, 255, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        assert!(colors.len() >= 2);
        let has_red = colors.iter().any(|(r, g, b)| *r > 200 && *g < 55 && *b < 55);
        let has_blue = colors.iter().any(|(r, g, b)| *r < 55 && *g < 55 && *b > 200);
        assert!(has_red || has_blue);
    }

    // ---- Properties (ported from test_properties.py) ----

    #[wasm_bindgen_test]
    async fn valid_rgb_values() {
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 100, 100, 10, 10)).await.unwrap();
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
    async fn color_returns_valid_rgb() {
        let buf = solid_pixels(128, 64, 32, 50, 50);
        let result = JsFuture::from(get_color_from_pixels(pixels_view(&buf), 50, 50, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
        for i in 0..3 {
            let v = color.get(i).as_f64().unwrap();
            assert!(v >= 0.0 && v <= 255.0);
        }
    }

    #[wasm_bindgen_test]
    async fn palette_non_empty() {
        let buf = solid_pixels(100, 150, 200, 50, 50);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 50, 50, 10, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn respects_color_count() {
        for &count in &[3u8, 5u8] {
            let buf = solid_pixels(128, 64, 32, 100, 100);
            let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 100, 100, count, 10)).await.unwrap();
            let palette: js_sys::Array = result.dyn_into().unwrap();
            assert!(palette.length() <= count as u32);
        }
    }

    #[wasm_bindgen_test]
    async fn palette_count_bounded() {
        // Ported from test_properties.py::test_palette_count_bounded
        for &count in &[3u8, 5u8] {
            let buf = gradient_pixels(200, 200);
            let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 200, 200, count, 10)).await.unwrap();
            let palette: js_sys::Array = result.dyn_into().unwrap();
            assert!(palette.length() <= count as u32,
                "palette length {} exceeds requested count {}", palette.length(), count);
        }
    }

    // ---- Deduplication (ported from test_deduplication.py, test_properties.py) ----

    #[wasm_bindgen_test]
    async fn no_duplicates() {
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 100, 100, 20, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        let unique: std::collections::HashSet<_> = colors.clone().into_iter().collect();
        assert_eq!(unique.len(), colors.len());
    }

    #[wasm_bindgen_test]
    async fn deduplication_large_palette() {
        // Ported from test_deduplication.py::test_deduplication
        // Request 255 colors and verify no duplicates
        let buf = gradient_pixels(1024, 1024);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 1024, 1024, 255, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        let unique: std::collections::HashSet<_> = colors.clone().into_iter().collect();
        assert_eq!(unique.len(), colors.len(), "palette contains duplicates");
        assert!(0 < colors.len() && colors.len() <= 255);
    }

    // ---- Determinism (ported from test_edge_cases.py) ----

    // ---- Different images produce different colors (ported from test_edge_cases.py) ----

    #[wasm_bindgen_test]
    async fn different_images_different_colors() {
        let red_buf = solid_pixels(255, 0, 0, 100, 100);
        let blue_buf = solid_pixels(0, 0, 255, 100, 100);
        let c1 = JsFuture::from(get_color_from_pixels(pixels_view(&red_buf), 100, 100, 10)).await.unwrap();
        let c2 = JsFuture::from(get_color_from_pixels(pixels_view(&blue_buf), 100, 100, 10)).await.unwrap();
        let c1_arr: js_sys::Array = c1.dyn_into().unwrap();
        let c2_arr: js_sys::Array = c2.dyn_into().unwrap();
        assert_ne!(
            color_to_tuple(&c1_arr),
            color_to_tuple(&c2_arr),
            "different images should produce different dominant colors"
        );
    }

    // ---- Determinism (ported from test_edge_cases.py) ----

    #[wasm_bindgen_test]
    async fn determinism() {
        let buf = solid_pixels(180, 90, 45, 100, 100);
        let p1 = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 100, 100, 5, 10)).await.unwrap();
        let p2 = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 100, 100, 5, 10)).await.unwrap();
        let p1_arr: js_sys::Array = p1.dyn_into().unwrap();
        let p2_arr: js_sys::Array = p2.dyn_into().unwrap();
        assert_eq!(palette_to_vec(&p1_arr), palette_to_vec(&p2_arr));
    }

    #[wasm_bindgen_test]
    async fn color_determinism() {
        let buf = solid_pixels(180, 90, 45, 100, 100);
        let c1 = JsFuture::from(get_color_from_pixels(pixels_view(&buf), 100, 100, 10)).await.unwrap();
        let c2 = JsFuture::from(get_color_from_pixels(pixels_view(&buf), 100, 100, 10)).await.unwrap();
        let c1_arr: js_sys::Array = c1.dyn_into().unwrap();
        let c2_arr: js_sys::Array = c2.dyn_into().unwrap();
        assert_eq!(color_to_tuple(&c1_arr), color_to_tuple(&c2_arr));
    }

    #[wasm_bindgen_test]
    async fn consistent_across_quality_settings() {
        // Different quality settings should return the same dominant color for solid images
        let buf = solid_pixels(200, 100, 50, 200, 200);
        let c1 = JsFuture::from(get_color_from_pixels(pixels_view(&buf), 200, 200, 1)).await.unwrap();
        let c2 = JsFuture::from(get_color_from_pixels(pixels_view(&buf), 200, 200, 10)).await.unwrap();
        let c1_arr: js_sys::Array = c1.dyn_into().unwrap();
        let c2_arr: js_sys::Array = c2.dyn_into().unwrap();
        let (r1, g1, b1) = color_to_tuple(&c1_arr);
        let (r2, g2, b2) = color_to_tuple(&c2_arr);
        // Colors should be close even with different quality
        assert!((r1 as i32 - r2 as i32).abs() < 30);
        assert!((g1 as i32 - g2 as i32).abs() < 30);
        assert!((b1 as i32 - b2 as i32).abs() < 30);
    }

    // ---- Quality bounds (ported from test_edge_cases.py, test_errors.py) ----

    #[wasm_bindgen_test]
    async fn quality_bounds() {
        let buf = solid_pixels(200, 100, 50, 100, 100);
        for &q in &[1u8, 5u8, 10u8] {
            let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 100, 100, 5, q)).await.unwrap();
            let palette: js_sys::Array = result.dyn_into().unwrap();
            assert!(palette.length() > 0, "quality={} returned empty palette", q);
        }
    }

    #[wasm_bindgen_test]
    async fn quality_min_valid() {
        // Ported from test_edge_cases.py::test_quality_min_valid
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let result = JsFuture::from(get_color_from_pixels(pixels_view(&buf), 100, 100, 1)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
    }

    #[wasm_bindgen_test]
    async fn quality_ten_fastest() {
        // Ported from test_edge_cases.py::test_quality_ten_fastest
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let result = JsFuture::from(get_color_from_pixels(pixels_view(&buf), 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
    }

    // ---- Edge cases (ported from test_edge_cases.py) ----

    #[wasm_bindgen_test]
    async fn edge_1x1() {
        let buf = solid_pixels(255, 128, 64, 1, 1);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 1, 1, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn edge_2x2() {
        let buf = solid_pixels(128, 64, 32, 2, 2);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 2, 2, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn edge_large_image() {
        let buf = solid_pixels(100, 200, 150, 500, 500);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 500, 500, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn edge_wide_image() {
        let buf = solid_pixels(200, 100, 50, 1000, 10);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 1000, 10, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn edge_tall_image() {
        let buf = solid_pixels(200, 100, 50, 10, 1000);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 10, 1000, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    // ---- Error handling (ported from test_errors.py) ----

    #[wasm_bindgen_test]
    async fn rejects_empty_pixels() {
        let empty = js_sys::Uint8Array::new_with_length(0);
        let result = JsFuture::from(get_palette_from_pixels(empty, 0, 0, 5, 10)).await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn rejects_empty_color() {
        let empty = js_sys::Uint8Array::new_with_length(0);
        let result = JsFuture::from(get_color_from_pixels(empty, 0, 0, 10)).await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn rejects_zero_width() {
        let pixels = js_sys::Uint8Array::new_with_length(0);
        let result = JsFuture::from(get_palette_from_pixels(pixels, 0, 100, 5, 10)).await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn rejects_zero_height() {
        let pixels = js_sys::Uint8Array::new_with_length(0);
        let result = JsFuture::from(get_palette_from_pixels(pixels, 100, 0, 5, 10)).await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn accepts_partial_pixel_data() {
        // The raw pixel API processes whatever bytes are given without validation.
        let short = js_sys::Uint8Array::from(&[0xFFu8, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0, 0, 0][..]);
        let result = JsFuture::from(get_palette_from_pixels(short, 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
  let _ = &palette; // result is a valid JS Array
    }

    #[wasm_bindgen_test]
    async fn accepts_arbitrary_pixel_data() {
        // The raw pixel API processes arbitrary byte patterns as valid RGBA data
        let mut data = vec![0u8; 1600]; // 20x20 image
        for i in 0..1600 {
            data[i] = (i * 7 + 13) as u8;
        }
        let pixels = js_sys::Uint8Array::from(&data[..]);
        let result = JsFuture::from(get_palette_from_pixels(pixels, 20, 20, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    // ---- Input type tests (ported from test_input_types.py) ----

    #[wasm_bindgen_test]
    async fn uint8array_input_palette() {
        let buf = solid_pixels(100, 150, 200, 50, 50);
        let result = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 50, 50, 10, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn uint8array_input_color() {
        let buf = solid_pixels(100, 150, 200, 50, 50);
        let result = JsFuture::from(get_color_from_pixels(pixels_view(&buf), 50, 50, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
    }

    #[wasm_bindgen_test]
    async fn arraybuffer_input_palette() {
        // Ported from test_input_types.py - ArrayBuffer input
        let buf = solid_pixels(200, 100, 50, 50, 50);
        let view = pixels_view(&buf);
        let ab = view.buffer();
        let result = JsFuture::from(get_palette_from_pixels(
            js_sys::Uint8Array::new(&ab), 50, 50, 10, 10,
        )).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn arraybuffer_input_color() {
        let buf = solid_pixels(200, 100, 50, 50, 50);
        let view = pixels_view(&buf);
        let ab = view.buffer();
        let result = JsFuture::from(get_color_from_pixels(
            js_sys::Uint8Array::new(&ab), 50, 50, 10,
        )).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
    }

    #[wasm_bindgen_test]
    async fn bytes_not_mutated() {
        // Ported from test_input_types.py::test_bytes_not_mutated
        let buf = solid_pixels(100, 150, 200, 50, 50);
        let snapshot = buf.clone();
        let _ = JsFuture::from(get_palette_from_pixels(pixels_view(&buf), 50, 50, 10, 10)).await.unwrap();
        let _ = JsFuture::from(get_color_from_pixels(pixels_view(&buf), 50, 50, 10)).await.unwrap();
        assert_eq!(buf, snapshot);
    }

    // ---- Concurrency (ported from test_concurrent.py, test_multithread.py) ----

    #[wasm_bindgen_test]
    async fn concurrent_palette_calls() {
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let promises = js_sys::Array::new();
        for _ in 0..3 {
            promises.push(&get_palette_from_pixels(pixels_view(&buf), 100, 100, 10, 10));
        }
        let results = JsFuture::from(js_sys::Promise::all(&promises)).await.unwrap();
        let results: js_sys::Array = results.dyn_into().unwrap();
        assert_eq!(results.length(), 3);
        for i in 0..results.length() {
            let palette: js_sys::Array = results.get(i).dyn_into().unwrap();
            assert!(palette.length() > 0);
        }
    }

    #[wasm_bindgen_test]
    async fn concurrent_color_calls() {
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let promises = js_sys::Array::new();
        for _ in 0..3 {
            promises.push(&get_color_from_pixels(pixels_view(&buf), 100, 100, 10));
        }
        let results = JsFuture::from(js_sys::Promise::all(&promises)).await.unwrap();
        let results: js_sys::Array = results.dyn_into().unwrap();
        assert_eq!(results.length(), 3);
        let first = results.get(0).dyn_into::<js_sys::Array>().unwrap();
        for i in 1..results.length() {
            let color = results.get(i).dyn_into::<js_sys::Array>().unwrap();
            assert_eq!(color_to_tuple(&color), color_to_tuple(&first));
        }
    }

    #[wasm_bindgen_test]
    async fn concurrent_mixed_ops() {
        // Ported from test_concurrent.py::test_concurrent_mixed_ops
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let promises = js_sys::Array::new();
        promises.push(&get_color_from_pixels(pixels_view(&buf), 100, 100, 10));
        promises.push(&get_palette_from_pixels(pixels_view(&buf), 100, 100, 3, 10));
        promises.push(&get_color_from_pixels(pixels_view(&buf), 100, 100, 10));
        let results = JsFuture::from(js_sys::Promise::all(&promises)).await.unwrap();
        let results: js_sys::Array = results.dyn_into().unwrap();
        assert_eq!(results.length(), 3);
    }

    // ---- API surface (ported from test_api.py) ----

    #[wasm_bindgen_test]
    fn api_get_palette_exists() {
        let p = get_palette_promise(&wasm_bindgen::JsValue::NULL, 5, 10);
        assert!(p.is_instance_of::<js_sys::Promise>());
    }

    #[wasm_bindgen_test]
    fn api_get_color_exists() {
        let p = get_color_promise(&wasm_bindgen::JsValue::NULL, 10);
        assert!(p.is_instance_of::<js_sys::Promise>());
    }

    // ---- Version (ported from test_version.py, test_api.py) ----

    #[wasm_bindgen_test]
    fn version_exists() {
        let v = version();
        assert!(!v.is_empty());
    }

    #[wasm_bindgen_test]
    fn version_is_string() {
        let v = version();
        assert!(v.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == '_'));
    }

    #[wasm_bindgen_test]
    fn version_semver_like() {
        let v = version();
        let parts: Vec<&str> = v.split('.').collect();
        assert!(parts.len() >= 2, "version should have at least major.minor");
        assert!(parts[0].parse::<u32>().is_ok(), "major version should be numeric");
        assert!(parts[1].parse::<u32>().is_ok(), "minor version should be numeric");
    }

    #[wasm_bindgen_test]
    fn version_no_whitespace() {
        let v = version();
        assert_eq!(v.trim(), v);
    }
}
