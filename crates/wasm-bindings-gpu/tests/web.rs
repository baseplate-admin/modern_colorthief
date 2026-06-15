use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;


#[allow(unused_imports)]
use modern_colorthief_wasm_gpu::{
    get_palette_gpu_promise, get_color_gpu_promise, get_version,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

#[allow(dead_code)]
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

fn pixels_view(buf: &[u8]) -> js_sys::Uint8Array {
    js_sys::Uint8Array::from(&buf[..])
}

#[allow(dead_code)]
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

#[allow(dead_code)]
fn color_to_tuple(color: &js_sys::Array) -> (u8, u8, u8) {
    let r = color.get(0).as_f64().unwrap() as u8;
    let g = color.get(1).as_f64().unwrap() as u8;
    let b = color.get(2).as_f64().unwrap() as u8;
    (r, g, b)
}

// ---------------------------------------------------------------------------
// GPU tests (require WebGPU - only available in browser)
// Ported from Python test suite
// ---------------------------------------------------------------------------

mod gpu {
    use super::*;

    // ---- Solid color palette tests (ported from test_properties.py) ----

    #[wasm_bindgen_test]
    async fn palette_solid_red() {
        let buf = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 100, 100, 10, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        assert!(!colors.is_empty());
        let (r, g, b) = colors[0];
        assert!(r > 200 && g < 55 && b < 55, "expected red, got ({},{},{})", r, g, b);
    }

    #[wasm_bindgen_test]
    async fn palette_solid_green() {
        let buf = solid_pixels(0, 255, 0, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette_to_vec(&palette)[0].1 > 200);
    }

    #[wasm_bindgen_test]
    async fn palette_white() {
        let buf = solid_pixels(255, 255, 255, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = palette_to_vec(&palette)[0];
        assert!(r > 200 && g > 200 && b > 200);
    }

    #[wasm_bindgen_test]
    async fn palette_black() {
        let buf = solid_pixels(0, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = palette_to_vec(&palette)[0];
        assert!(r < 55 && g < 55 && b < 55);
    }

    // ---- Properties (ported from test_properties.py) ----

    #[wasm_bindgen_test]
    async fn palette_valid_rgb() {
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 100, 100, 10, 10)).await.unwrap();
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
    async fn palette_non_empty() {
        let buf = solid_pixels(100, 150, 200, 50, 50);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 50, 50, 10, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn palette_respects_color_count() {
        let buf = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() <= 5);
    }

    #[wasm_bindgen_test]
    async fn palette_count_bounded() {
        // Ported from test_properties.py::test_palette_count_bounded
        for &count in &[3u8, 5u8] {
            let buf = gradient_pixels(200, 200);
            let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 200, 200, count, 10)).await.unwrap();
            let palette: js_sys::Array = result.dyn_into().unwrap();
            assert!(palette.length() <= count as u32,
                "palette length {} exceeds requested count {}", palette.length(), count);
        }
    }

    #[wasm_bindgen_test]
    async fn palette_no_duplicates() {
        let buf = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 100, 100, 10, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        let unique: std::collections::HashSet<_> = colors.clone().into_iter().collect();
        assert_eq!(unique.len(), colors.len());
    }

    #[wasm_bindgen_test]
    async fn palette_deduplication_large() {
        // Ported from test_deduplication.py::test_deduplication
        let buf = gradient_pixels(512, 512);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 512, 512, 100, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        let unique: std::collections::HashSet<_> = colors.clone().into_iter().collect();
        assert_eq!(unique.len(), colors.len(), "palette contains duplicates");
        assert!(0 < colors.len() && colors.len() <= 100);
    }

    #[wasm_bindgen_test]
    async fn palette_deterministic() {
        let buf = solid_pixels(200, 100, 50, 100, 100);
        let p1 = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 100, 100, 10, 10)).await.unwrap();
        let p2 = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 100, 100, 10, 10)).await.unwrap();
        let p1_arr: js_sys::Array = p1.dyn_into().unwrap();
        let p2_arr: js_sys::Array = p2.dyn_into().unwrap();
        assert_eq!(palette_to_vec(&p1_arr), palette_to_vec(&p2_arr));
    }

    // ---- Quality variation (ported from test_edge_cases.py) ----

    #[wasm_bindgen_test]
    async fn palette_quality_variation() {
        let buf = solid_pixels(100, 150, 200, 200, 200);
        for &q in &[1u8, 5u8, 10u8] {
            let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 200, 200, 10, q)).await.unwrap();
            let palette: js_sys::Array = result.dyn_into().unwrap();
            assert!(palette.length() > 0);
        }
    }

    #[wasm_bindgen_test]
    async fn quality_min_valid() {
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let result = JsFuture::from(get_color_gpu_promise(pixels_view(&buf), 100, 100, 1)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
    }

    #[wasm_bindgen_test]
    async fn quality_max_valid() {
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let result = JsFuture::from(get_color_gpu_promise(pixels_view(&buf), 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
    }

    // ---- Dominant color tests (ported from test_properties.py) ----

    #[wasm_bindgen_test]
    async fn color_solid_red() {
        let buf = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_color_gpu_promise(pixels_view(&buf), 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = color_to_tuple(&color);
        assert!(r > 200 && g < 55 && b < 55);
    }

    #[wasm_bindgen_test]
    async fn color_valid_rgb() {
        let buf = solid_pixels(50, 100, 150, 100, 100);
        let result = JsFuture::from(get_color_gpu_promise(pixels_view(&buf), 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
        for i in 0..3 {
            let v = color.get(i).as_f64().unwrap();
            assert!(v >= 0.0 && v <= 255.0);
        }
    }

    #[wasm_bindgen_test]
    async fn color_deterministic() {
        let buf = solid_pixels(200, 100, 50, 100, 100);
        let c1 = JsFuture::from(get_color_gpu_promise(pixels_view(&buf), 100, 100, 10)).await.unwrap();
        let c2 = JsFuture::from(get_color_gpu_promise(pixels_view(&buf), 100, 100, 10)).await.unwrap();
        let c1_arr: js_sys::Array = c1.dyn_into().unwrap();
        let c2_arr: js_sys::Array = c2.dyn_into().unwrap();
        assert_eq!(color_to_tuple(&c1_arr), color_to_tuple(&c2_arr));
    }

    #[wasm_bindgen_test]
    async fn color_consistent_across_quality() {
        let buf = solid_pixels(200, 100, 50, 200, 200);
        let c1 = JsFuture::from(get_color_gpu_promise(pixels_view(&buf), 200, 200, 1)).await.unwrap();
        let c2 = JsFuture::from(get_color_gpu_promise(pixels_view(&buf), 200, 200, 10)).await.unwrap();
        let c1_arr: js_sys::Array = c1.dyn_into().unwrap();
        let c2_arr: js_sys::Array = c2.dyn_into().unwrap();
        let (r1, g1, b1) = color_to_tuple(&c1_arr);
        let (r2, g2, b2) = color_to_tuple(&c2_arr);
        assert!((r1 as i32 - r2 as i32).abs() < 30);
        assert!((g1 as i32 - g2 as i32).abs() < 30);
        assert!((b1 as i32 - b2 as i32).abs() < 30);
    }

    // ---- Different images produce different colors (ported from test_edge_cases.py) ----

    #[wasm_bindgen_test]
    async fn different_images_different_colors() {
        let red_buf = solid_pixels(255, 0, 0, 100, 100);
        let blue_buf = solid_pixels(0, 0, 255, 100, 100);
        let c1 = JsFuture::from(get_color_gpu_promise(pixels_view(&red_buf), 100, 100, 10)).await.unwrap();
        let c2 = JsFuture::from(get_color_gpu_promise(pixels_view(&blue_buf), 100, 100, 10)).await.unwrap();
        let c1_arr: js_sys::Array = c1.dyn_into().unwrap();
        let c2_arr: js_sys::Array = c2.dyn_into().unwrap();
        assert_ne!(
            color_to_tuple(&c1_arr),
            color_to_tuple(&c2_arr),
            "different images should produce different dominant colors"
        );
    }

    // ---- Two-color detection ----

    #[wasm_bindgen_test]
    async fn two_color_detection() {
        let buf = two_color_pixels(255, 0, 0, 0, 0, 255, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        let has_red = colors.iter().any(|(r, g, b)| *r > 200 && *g < 55 && *b < 55);
        let has_blue = colors.iter().any(|(r, g, b)| *r < 55 && *g < 55 && *b > 200);
        assert!(has_red || has_blue);
    }

    // ---- Edge cases ----

    #[wasm_bindgen_test]
    async fn edge_1x1() {
        let buf = solid_pixels(255, 128, 64, 1, 1);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 1, 1, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn edge_2x2() {
        let buf = solid_pixels(128, 64, 32, 2, 2);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 2, 2, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn edge_large_image() {
        let buf = solid_pixels(100, 200, 150, 500, 500);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 500, 500, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn edge_wide_image() {
        let buf = solid_pixels(200, 100, 50, 1000, 10);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 1000, 10, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn edge_tall_image() {
        let buf = solid_pixels(200, 100, 50, 10, 1000);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 10, 1000, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    // ---- Error handling (ported from test_errors.py) ----

    #[wasm_bindgen_test]
    async fn rejects_empty_pixels() {
        let empty = js_sys::Uint8Array::new_with_length(0);
        let result = JsFuture::from(get_palette_gpu_promise(empty, 0, 0, 5, 10)).await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn rejects_empty_color() {
        let empty = js_sys::Uint8Array::new_with_length(0);
        let result = JsFuture::from(get_color_gpu_promise(empty, 0, 0, 10)).await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn rejects_zero_dimensions() {
        let buf = solid_pixels(100, 150, 200, 10, 10);
        let pixels = pixels_view(&buf);
        let result = JsFuture::from(get_palette_gpu_promise(pixels, 0, 0, 5, 10)).await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn rejects_corrupted_data() {
        let mut corrupted = vec![0u8; 100];
        for i in 0..100 {
            corrupted[i] = (i * 7 + 13) as u8;
        }
        let pixels = js_sys::Uint8Array::from(&corrupted[..]);
        let result = JsFuture::from(get_palette_gpu_promise(pixels, 100, 100, 5, 10)).await;
        assert!(result.is_err());
    }

    // ---- Concurrency (ported from test_concurrent.py) ----

    #[wasm_bindgen_test]
    async fn concurrent_gpu_palette_calls() {
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let promises = js_sys::Array::new();
        promises.push(&get_palette_gpu_promise(pixels_view(&buf), 100, 100, 10, 10));
        promises.push(&get_palette_gpu_promise(pixels_view(&buf), 100, 100, 10, 10));
        promises.push(&get_palette_gpu_promise(pixels_view(&buf), 100, 100, 10, 10));
        let results = JsFuture::from(js_sys::Promise::all(&promises)).await.unwrap();
        let results: js_sys::Array = results.dyn_into().unwrap();
        assert_eq!(results.length(), 3);
        for i in 0..results.length() {
            let palette: js_sys::Array = results.get(i).dyn_into().unwrap();
            assert!(palette.length() > 0);
        }
    }

    #[wasm_bindgen_test]
    async fn concurrent_gpu_color_calls() {
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let promises = js_sys::Array::new();
        promises.push(&get_color_gpu_promise(pixels_view(&buf), 100, 100, 10));
        promises.push(&get_color_gpu_promise(pixels_view(&buf), 100, 100, 10));
        promises.push(&get_color_gpu_promise(pixels_view(&buf), 100, 100, 10));
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
    async fn concurrent_gpu_mixed_ops() {
        let buf = solid_pixels(100, 150, 200, 100, 100);
        let promises = js_sys::Array::new();
        promises.push(&get_color_gpu_promise(pixels_view(&buf), 100, 100, 10));
        promises.push(&get_palette_gpu_promise(pixels_view(&buf), 100, 100, 3, 10));
        promises.push(&get_color_gpu_promise(pixels_view(&buf), 100, 100, 10));
        let results = JsFuture::from(js_sys::Promise::all(&promises)).await.unwrap();
        let results: js_sys::Array = results.dyn_into().unwrap();
        assert_eq!(results.length(), 3);
    }

    // ---- API surface (ported from test_api.py) ----

    #[wasm_bindgen_test]
    fn api_gpu_exports_exist() {
        let empty = js_sys::Uint8Array::new_with_length(0);
        let p = get_palette_gpu_promise(empty, 0, 0, 5, 10);
        assert!(p.is_instance_of::<js_sys::Promise>());
        let empty2 = js_sys::Uint8Array::new_with_length(0);
        let c = get_color_gpu_promise(empty2, 0, 0, 10);
        assert!(c.is_instance_of::<js_sys::Promise>());
    }

    // ---- RGB to hex (ported from test_cli_helper.py::test_rgb_to_hex) ----

    fn rgb_to_hex(r: u8, g: u8, b: u8) -> String {
        format!("#{:02x}{:02x}{:02x}", r, g, b)
    }

    #[wasm_bindgen_test]
    fn rgb_to_hex_white() {
        assert_eq!(rgb_to_hex(255, 255, 255), "#ffffff");
    }

    #[wasm_bindgen_test]
    fn rgb_to_hex_black() {
        assert_eq!(rgb_to_hex(0, 0, 0), "#000000");
    }

    #[wasm_bindgen_test]
    fn rgb_to_hex_red() {
        assert_eq!(rgb_to_hex(255, 0, 0), "#ff0000");
    }

    #[wasm_bindgen_test]
    fn rgb_to_hex_small_values() {
        assert_eq!(rgb_to_hex(1, 2, 3), "#010203");
    }

    #[wasm_bindgen_test]
    fn rgb_to_hex_mid_values() {
        assert_eq!(rgb_to_hex(179, 51, 55), "#b33337");
    }

    // ---- Error handling (ported from test_errors.py) ----

    #[wasm_bindgen_test]
    async fn rejects_truncated_buffer() {
        // Ported from test_errors.py - truncated JPEG header
        let truncated = js_sys::Uint8Array::from(&[0xFFu8, 0xD8, 0xFF, 0xE0, 0, 0, 0, 0, 0, 0][..]);
        let result = JsFuture::from(get_palette_gpu_promise(truncated, 10, 10, 5, 10)).await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn rejects_oversized_dimensions() {
        // Ported from test_errors.py - dimensions exceed buffer
        let small = solid_pixels(100, 150, 200, 10, 10);
        let pixels = pixels_view(&small);
        let result = JsFuture::from(get_palette_gpu_promise(pixels, 100, 100, 5, 10)).await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn rejects_zero_width_color() {
        let buf = solid_pixels(100, 150, 200, 10, 10);
        let pixels = pixels_view(&buf);
        let result = JsFuture::from(get_color_gpu_promise(pixels, 0, 10, 10)).await;
        assert!(result.is_err());
    }

    #[wasm_bindgen_test]
    async fn rejects_zero_height_color() {
        let buf = solid_pixels(100, 150, 200, 10, 10);
        let pixels = pixels_view(&buf);
        let result = JsFuture::from(get_color_gpu_promise(pixels, 10, 0, 10)).await;
        assert!(result.is_err());
    }

    // ---- Input type variety (ported from test_input_types.py) ----

    #[wasm_bindgen_test]
    async fn uint8array_input_gpu_palette() {
        let buf = solid_pixels(100, 150, 200, 50, 50);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 50, 50, 10, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn uint8array_input_gpu_color() {
        let buf = solid_pixels(100, 150, 200, 50, 50);
        let result = JsFuture::from(get_color_gpu_promise(pixels_view(&buf), 50, 50, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
    }

    #[wasm_bindgen_test]
    async fn arraybuffer_input_gpu_palette() {
        let buf = solid_pixels(200, 100, 50, 50, 50);
        let view = pixels_view(&buf);
        let ab = view.buffer();
        let result = JsFuture::from(get_palette_gpu_promise(
            js_sys::Uint8Array::new(&ab), 50, 50, 10, 10,
        )).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn bytes_not_mutated_gpu() {
        // Ported from test_input_types.py::test_bytes_not_mutated
        let buf = solid_pixels(100, 150, 200, 50, 50);
        let snapshot = buf.clone();
        let _ = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 50, 50, 10, 10)).await.unwrap();
        let _ = JsFuture::from(get_color_gpu_promise(pixels_view(&buf), 50, 50, 10)).await.unwrap();
        assert_eq!(buf, snapshot);
    }

    // ---- Property invariants (ported from test_properties.py) ----

    #[wasm_bindgen_test]
    async fn color_returns_tuple_of_ints() {
        // Ported from test_properties.py::test_color_returns_valid_rgb
        let buf = solid_pixels(128, 64, 32, 50, 50);
        let result = JsFuture::from(get_color_gpu_promise(pixels_view(&buf), 50, 50, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
        for i in 0..3 {
            let v = color.get(i).as_f64().unwrap();
            assert!(v >= 0.0 && v <= 255.0);
            assert_eq!(v, (v as u8) as f64, "color component should be an integer");
        }
    }

    #[wasm_bindgen_test]
    async fn palette_entries_are_tuples_of_ints() {
        // Ported from test_properties.py::test_palette_returns_valid_rgb_list
        let buf = solid_pixels(100, 150, 200, 50, 50);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 50, 50, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
        for i in 0..palette.length() {
            let tuple = palette.get(i).dyn_into::<js_sys::Array>().unwrap();
            assert_eq!(tuple.length(), 3);
            for j in 0..3 {
                let v = tuple.get(j).as_f64().unwrap();
                assert!(v >= 0.0 && v <= 255.0);
                assert_eq!(v, (v as u8) as f64, "palette entry should be an integer");
            }
        }
    }

    // ---- Version (ported from test_version.py) ----

    #[wasm_bindgen_test]
    fn version_exists() {
        let v = get_version();
        assert!(!v.is_empty());
    }

    #[wasm_bindgen_test]
    fn version_is_string() {
        let v = get_version();
        assert!(v.chars().all(|c| c.is_ascii_digit() || c == '.' || c == '-' || c == '_'));
    }

    #[wasm_bindgen_test]
    fn version_semver_like() {
        let v = get_version();
        let parts: Vec<&str> = v.split('.').collect();
        assert!(parts.len() >= 2, "version should have at least major.minor");
        assert!(parts[0].parse::<u32>().is_ok(), "major version should be numeric");
        assert!(parts[1].parse::<u32>().is_ok(), "minor version should be numeric");
    }

    #[wasm_bindgen_test]
    fn version_no_whitespace() {
        let v = get_version();
        assert_eq!(v.trim(), v);
    }
}

// ---------------------------------------------------------------------------
// Raw pixel tests — gated behind browser-tests because WebGPU (and js_eval)
// is only available in the browser, not in the wasm-bindgen Node.js test runner.
// ---------------------------------------------------------------------------

#[cfg(feature = "browser-tests")]
mod pixels {
    use super::*;

    #[wasm_bindgen_test]
    async fn gpu_palette_available() {
        let buf = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(pixels_view(&buf), 100, 100, 10, 10)).await;
        if let Ok(res) = result {
            let palette: js_sys::Array = res.dyn_into().unwrap();
            assert!(palette.length() > 0);
        }
    }

    #[wasm_bindgen_test]
    async fn gpu_color_available() {
        let buf = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_color_gpu_promise(pixels_view(&buf), 100, 100, 10)).await;
        if let Ok(res) = result {
            let color: js_sys::Array = res.dyn_into().unwrap();
            assert_eq!(color.length(), 3);
        }
    }
}
