use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

use modern_colorthief_wasm_gpu::{get_palette_gpu_promise, get_color_gpu_promise};

fn solid_pixels(r: u8, g: u8, b: u8, w: u32, h: u32) -> js_sys::Uint8Array {
    let len = (w * h * 4) as usize;
    let buf = js_sys::Uint8Array::new(len as u32);
    for i in 0..(w * h) as usize {
        buf.indexed_set(i * 4, r);
        buf.indexed_set(i * 4 + 1, g);
        buf.indexed_set(i * 4 + 2, b);
        buf.indexed_set(i * 4 + 3, 255);
    }
    buf
}

fn two_color_pixels(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8, w: u32, h: u32) -> js_sys::Uint8Array {
    let buf = js_sys::Uint8Array::new((w * h * 4) as u32);
    for y in 0..h {
        for x in 0..w {
            let idx = (y * w + x) as usize * 4;
            if x < w / 2 {
                buf.indexed_set(idx, r1);
                buf.indexed_set(idx + 1, g1);
                buf.indexed_set(idx + 2, b1);
            } else {
                buf.indexed_set(idx, r2);
                buf.indexed_set(idx + 1, g2);
                buf.indexed_set(idx + 2, b2);
            }
            buf.indexed_set(idx + 3, 255);
        }
    }
    buf
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
// GPU tests (require WebGPU — only available in browser)
// ---------------------------------------------------------------------------

#[cfg(feature = "browser-tests")]
mod gpu {
    use super::*;

    #[wasm_bindgen_test]
    async fn palette_solid_red() {
        let pixels = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(&pixels, 100, 100, 10, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        assert!(!colors.is_empty());
        let (r, g, b) = colors[0];
        assert!(r > 200 && g < 55 && b < 55, "expected red, got ({},{},{})", r, g, b);
    }

    #[wasm_bindgen_test]
    async fn palette_solid_green() {
        let pixels = solid_pixels(0, 255, 0, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(&pixels, 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette_to_vec(&palette)[0].1 > 200);
    }

    #[wasm_bindgen_test]
    async fn palette_white() {
        let pixels = solid_pixels(255, 255, 255, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(&pixels, 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = palette_to_vec(&palette)[0];
        assert!(r > 200 && g > 200 && b > 200);
    }

    #[wasm_bindgen_test]
    async fn palette_black() {
        let pixels = solid_pixels(0, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(&pixels, 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = palette_to_vec(&palette)[0];
        assert!(r < 55 && g < 55 && b < 55);
    }

    #[wasm_bindgen_test]
    async fn palette_valid_rgb() {
        let pixels = solid_pixels(100, 150, 200, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(&pixels, 100, 100, 10, 10)).await.unwrap();
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
    async fn palette_respects_color_count() {
        let pixels = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(&pixels, 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() <= 5);
    }

    #[wasm_bindgen_test]
    async fn palette_no_duplicates() {
        let pixels = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(&pixels, 100, 100, 10, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        let unique: std::collections::HashSet<_> = colors.into_iter().collect();
        assert_eq!(unique.len(), colors.len());
    }

    #[wasm_bindgen_test]
    async fn palette_deterministic() {
        let pixels = solid_pixels(200, 100, 50, 100, 100);
        let p1 = JsFuture::from(get_palette_gpu_promise(&pixels, 100, 100, 10, 10)).await.unwrap();
        let p2 = JsFuture::from(get_palette_gpu_promise(&pixels, 100, 100, 10, 10)).await.unwrap();
        let p1_arr: js_sys::Array = p1.dyn_into().unwrap();
        let p2_arr: js_sys::Array = p2.dyn_into().unwrap();
        assert_eq!(palette_to_vec(&p1_arr), palette_to_vec(&p2_arr));
    }

    #[wasm_bindgen_test]
    async fn palette_quality_variation() {
        let pixels = solid_pixels(100, 150, 200, 200, 200);
        for q in [1u8, 5u8, 10u8] {
            let result = JsFuture::from(get_palette_gpu_promise(&pixels, 200, 200, 10, q)).await.unwrap();
            let palette: js_sys::Array = result.dyn_into().unwrap();
            assert!(palette.length() > 0);
        }
    }

    #[wasm_bindgen_test]
    async fn color_solid_red() {
        let pixels = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_color_gpu_promise(&pixels, 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        let (r, g, b) = color_to_tuple(&color);
        assert!(r > 200 && g < 55 && b < 55);
    }

    #[wasm_bindgen_test]
    async fn color_valid_rgb() {
        let pixels = solid_pixels(50, 100, 150, 100, 100);
        let result = JsFuture::from(get_color_gpu_promise(&pixels, 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
    }

    #[wasm_bindgen_test]
    async fn color_deterministic() {
        let pixels = solid_pixels(200, 100, 50, 100, 100);
        let c1 = JsFuture::from(get_color_gpu_promise(&pixels, 100, 100, 10)).await.unwrap();
        let c2 = JsFuture::from(get_color_gpu_promise(&pixels, 100, 100, 10)).await.unwrap();
        let c1_arr: js_sys::Array = c1.dyn_into().unwrap();
        let c2_arr: js_sys::Array = c2.dyn_into().unwrap();
        assert_eq!(color_to_tuple(&c1_arr), color_to_tuple(&c2_arr));
    }

    #[wasm_bindgen_test]
    async fn two_color_detection() {
        let pixels = two_color_pixels(255, 0, 0, 0, 0, 255, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(&pixels, 100, 100, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        let colors = palette_to_vec(&palette);
        let has_red = colors.iter().any(|(r, g, b)| *r > 200 && *g < 55 && *b < 55);
        let has_blue = colors.iter().any(|(r, g, b)| *r < 55 && *g < 55 && *b > 200);
        assert!(has_red || has_blue);
    }

    #[wasm_bindgen_test]
    async fn edge_1x1() {
        let pixels = solid_pixels(255, 128, 64, 1, 1);
        let result = JsFuture::from(get_palette_gpu_promise(&pixels, 1, 1, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn edge_large_image() {
        let pixels = solid_pixels(100, 200, 150, 500, 500);
        let result = JsFuture::from(get_palette_gpu_promise(&pixels, 500, 500, 5, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn rejects_empty_pixels() {
        let empty = js_sys::Uint8Array::new(0);
        let result = JsFuture::from(get_palette_gpu_promise(&empty, 0, 0, 5, 10)).await;
        assert!(result.is_err());
    }
}

// ---------------------------------------------------------------------------
// Raw pixel tests (run in any JS environment — GPU errors in Node)
// ---------------------------------------------------------------------------

mod pixels {
    use super::*;

    #[wasm_bindgen_test]
    async fn gpu_palette_solid_red() {
        let pixels = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_palette_gpu_promise(&pixels, 100, 100, 10, 10)).await.unwrap();
        let palette: js_sys::Array = result.dyn_into().unwrap();
        assert!(palette.length() > 0);
    }

    #[wasm_bindgen_test]
    async fn gpu_color_solid_red() {
        let pixels = solid_pixels(255, 0, 0, 100, 100);
        let result = JsFuture::from(get_color_gpu_promise(&pixels, 100, 100, 10)).await.unwrap();
        let color: js_sys::Array = result.dyn_into().unwrap();
        assert_eq!(color.length(), 3);
    }
}
