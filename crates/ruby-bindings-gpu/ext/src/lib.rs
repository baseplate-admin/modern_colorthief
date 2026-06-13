use magnus::{error, function, Object, RString, Ruby};

fn get_palette(
    pixels: RString,
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> Result<Vec<Vec<u8>>, error::Error> {
    let pixels = unsafe { pixels.as_slice() };
    modern_colorthief_core_gpu::extract_palette_from_buffer(
        pixels,
        width,
        height,
        color_count,
        quality,
    )
    .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
    .map_err(|e| {
        error::Error::new(
            Ruby::get().unwrap().exception_runtime_error(),
            e.to_string(),
        )
    })
}

fn get_color(
    pixels: RString,
    width: u32,
    height: u32,
    quality: u8,
) -> Result<Vec<u8>, error::Error> {
    let pixels = unsafe { pixels.as_slice() };
    let palette =
        modern_colorthief_core_gpu::extract_palette_from_buffer(pixels, width, height, 5, quality)
            .map_err(|e| {
                error::Error::new(
                    Ruby::get().unwrap().exception_runtime_error(),
                    e.to_string(),
                )
            })?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| {
            error::Error::new(
                Ruby::get().unwrap().exception_runtime_error(),
                "No color extracted",
            )
        })
}

#[magnus::init]
fn init_colorthief_gpu_ruby(ruby: &Ruby) {
    let mod_colorthief_gpu = ruby.define_module("ColorthiefGpu").unwrap();
    mod_colorthief_gpu
        .define_singleton_method("get_palette", function!(get_palette, 5))
        .unwrap();
    mod_colorthief_gpu
        .define_singleton_method("get_color", function!(get_color, 4))
        .unwrap();
}

// See: https://oxidize-rb.org/docs/api-reference/test-helpers
#[cfg(test)]
mod tests {
    use super::*;
    use rb_sys_test_helpers::ruby_test;

    // ---------------------------------------------------------------------------
    // Pixel helpers — build RGBA buffers for synthetic test images
    // ---------------------------------------------------------------------------

    fn solid_pixels(r: u8, g: u8, b: u8, count: usize) -> Vec<u8> {
        let mut buf = Vec::with_capacity(count * 4);
        for _ in 0..count {
            buf.extend_from_slice(&[r, g, b, 255]);
        }
        buf
    }

    fn two_color_pixels(r1: u8, g1: u8, b1: u8, r2: u8, g2: u8, b2: u8) -> Vec<u8> {
        let mut buf = solid_pixels(r1, g1, b1, 50);
        buf.append(&mut solid_pixels(r2, g2, b2, 50));
        buf
    }

    fn build_image<W: Into<u32>, H: Into<u32>>(
        width: W,
        height: H,
        pixel_fn: impl Fn(u32, u32) -> (u8, u8, u8),
    ) -> Vec<u8> {
        let (w, h) = (width.into(), height.into());
        let mut buf = Vec::with_capacity((w * h) as usize * 4);
        for y in 0..h {
            for x in 0..w {
                let (r, g, b) = pixel_fn(x, y);
                buf.extend_from_slice(&[r, g, b, 255]);
            }
        }
        buf
    }

    fn rstring(data: &[u8]) -> RString {
        RString::from_slice(data)
    }

    // ---------------------------------------------------------------------------
    // Solid color detection
    // ---------------------------------------------------------------------------

    #[ruby_test]
    fn test_palette_solid_red() {
        let pixels = solid_pixels(255, 0, 0, 100);
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 10, 10, 5, 1).unwrap();
        assert!(!palette.is_empty(), "palette should not be empty");
        assert!(palette.len() <= 5, "palette should respect color_count");
    }

    #[ruby_test]
    fn test_color_solid_red() {
        let pixels = solid_pixels(255, 0, 0, 100);
        let rs = rstring(&pixels);
        let color = get_color(rs, 10, 10, 1).unwrap();
        assert_eq!(color.len(), 3);
        assert!(color[0] > 200, "red channel should be dominant");
    }

    #[ruby_test]
    fn test_color_solid_green() {
        let pixels = solid_pixels(0, 255, 0, 100);
        let rs = rstring(&pixels);
        let color = get_color(rs, 10, 10, 1).unwrap();
        assert_eq!(color.len(), 3);
        assert!(color[1] > 200, "green channel should be dominant");
    }

    #[ruby_test]
    fn test_color_solid_blue() {
        let pixels = solid_pixels(0, 0, 255, 100);
        let rs = rstring(&pixels);
        let color = get_color(rs, 10, 10, 1).unwrap();
        assert_eq!(color.len(), 3);
        assert!(color[2] > 200, "blue channel should be dominant");
    }

    #[ruby_test]
    fn test_palette_solid_white() {
        let pixels = solid_pixels(255, 255, 255, 9);
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 3, 3, 5, 1).unwrap();
        assert!(!palette.is_empty());
        assert!(palette.iter().any(|c| c[0] > 200 && c[1] > 200 && c[2] > 200));
    }

    // ---------------------------------------------------------------------------
    // Two-color detection
    // ---------------------------------------------------------------------------

    #[ruby_test]
    fn test_two_colors_red_blue() {
        let pixels = two_color_pixels(255, 0, 0, 0, 0, 255);
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 10, 10, 5, 1).unwrap();
        let has_red = palette.iter().any(|c| c[0] > 200 && c[1] < 55 && c[2] < 55);
        let has_blue = palette.iter().any(|c| c[0] < 55 && c[1] < 55 && c[2] > 200);
        assert!(has_red, "palette should contain red");
        assert!(has_blue, "palette should contain blue");
    }

    // ---------------------------------------------------------------------------
    // Return value structure — valid RGB in [0, 255]
    // ---------------------------------------------------------------------------

    #[ruby_test]
    fn test_palette_valid_rgb() {
        let pixels = solid_pixels(100, 150, 200, 100);
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 10, 10, 5, 1).unwrap();
        for color in &palette {
            assert_eq!(color.len(), 3, "each color should have 3 channels");
            for &v in color {
                assert!((v as i32) >= 0 && (v as i32) <= 255, "channel values in [0, 255]");
            }
        }
    }

    #[ruby_test]
    fn test_color_valid_rgb() {
        let pixels = solid_pixels(50, 100, 150, 100);
        let rs = rstring(&pixels);
        let color = get_color(rs, 10, 10, 1).unwrap();
        assert_eq!(color.len(), 3);
        for &v in &color {
            assert!((v as i32) >= 0 && (v as i32) <= 255);
        }
    }

    // ---------------------------------------------------------------------------
    // Palette length respects color_count
    // ---------------------------------------------------------------------------

    #[ruby_test]
    fn test_color_count_one() {
        let pixels = solid_pixels(200, 100, 50, 400);
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 20, 20, 1, 1).unwrap();
        assert_eq!(palette.len(), 1, "color_count=1 should return exactly 1 color");
    }

    #[ruby_test]
    fn test_color_count_three() {
        let pixels = solid_pixels(200, 100, 50, 400);
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 20, 20, 3, 1).unwrap();
        assert!(palette.len() <= 3);
    }

    #[ruby_test]
    fn test_color_count_fifty() {
        let pixels = build_image(20, 20, |_, y| {
            let band = y / 2;
            ((band * 25) as u8, (band * 20) as u8, (band * 15) as u8)
        });
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 20, 20, 50, 1).unwrap();
        assert!(palette.len() <= 50);
    }

    #[ruby_test]
    fn test_color_count_max() {
        let pixels = solid_pixels(100, 150, 200, 100);
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 10, 10, 255, 1).unwrap();
        assert!(palette.len() <= 255);
        assert!(palette.len() >= 1, "solid image returns at least 1 color");
    }

    // ---------------------------------------------------------------------------
    // Deduplication — no duplicate colors in palette
    // ---------------------------------------------------------------------------

    #[ruby_test]
    fn test_deduplicated() {
        let pixels = solid_pixels(255, 0, 0, 100);
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 10, 10, 255, 1).unwrap();
        let unique: std::collections::HashSet<_> = palette.iter().collect();
        assert_eq!(palette.len(), unique.len(), "no duplicate colors");
    }

    // ---------------------------------------------------------------------------
    // Quality parameter variation
    // ---------------------------------------------------------------------------

    #[ruby_test]
    fn test_quality_one() {
        let pixels = solid_pixels(170, 85, 220, 100);
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 10, 10, 5, 1).unwrap();
        assert!(!palette.is_empty());
    }

    #[ruby_test]
    fn test_quality_ten() {
        let pixels = solid_pixels(170, 85, 220, 100);
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 10, 10, 5, 10).unwrap();
        assert!(!palette.is_empty());
    }

    #[ruby_test]
    fn test_quality_middle() {
        let pixels = solid_pixels(170, 85, 220, 100);
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 10, 10, 5, 5).unwrap();
        assert!(!palette.is_empty());
    }

    #[ruby_test]
    fn test_color_quality_one() {
        let pixels = solid_pixels(100, 150, 200, 100);
        let rs = rstring(&pixels);
        let color = get_color(rs, 10, 10, 1).unwrap();
        assert_eq!(color.len(), 3);
    }

    #[ruby_test]
    fn test_color_quality_ten() {
        let pixels = solid_pixels(100, 150, 200, 100);
        let rs = rstring(&pixels);
        let color = get_color(rs, 10, 10, 10).unwrap();
        assert_eq!(color.len(), 3);
    }

    // ---------------------------------------------------------------------------
    // Determinism — same input always produces same output
    // ---------------------------------------------------------------------------

    #[ruby_test]
    fn test_palette_determinism() {
        let pixels = solid_pixels(100, 150, 200, 100);
        let rs1 = rstring(&pixels);
        let rs2 = rstring(&pixels);
        let p1 = get_palette(rs1, 10, 10, 5, 1).unwrap();
        let p2 = get_palette(rs2, 10, 10, 5, 1).unwrap();
        assert_eq!(p1, p2, "palette should be deterministic");
    }

    #[ruby_test]
    fn test_color_determinism() {
        let pixels = solid_pixels(50, 100, 150, 100);
        let rs1 = rstring(&pixels);
        let rs2 = rstring(&pixels);
        let c1 = get_color(rs1, 10, 10, 1).unwrap();
        let c2 = get_color(rs2, 10, 10, 1).unwrap();
        assert_eq!(c1, c2, "color should be deterministic");
    }

    // ---------------------------------------------------------------------------
    // Different images produce different results
    // ---------------------------------------------------------------------------

    #[ruby_test]
    fn test_different_images_different_colors() {
        let red = solid_pixels(255, 0, 0, 100);
        let green = solid_pixels(0, 255, 0, 100);
        let rs1 = rstring(&red);
        let rs2 = rstring(&green);
        let c1 = get_color(rs1, 10, 10, 1).unwrap();
        let c2 = get_color(rs2, 10, 10, 1).unwrap();
        assert_ne!(c1, c2, "different images should yield different dominant colors");
    }

    // ---------------------------------------------------------------------------
    // Consistency — get_color result appears in palette
    // ---------------------------------------------------------------------------

    #[ruby_test]
    fn test_color_in_palette() {
        let pixels = two_color_pixels(255, 0, 0, 0, 0, 255);
        let rs1 = rstring(&pixels);
        let rs2 = rstring(&pixels);
        let color = get_color(rs1, 10, 10, 1).unwrap();
        let palette = get_palette(rs2, 10, 10, 5, 1).unwrap();
        assert!(
            palette.contains(&color),
            "dominant color should appear in palette"
        );
    }

    // ---------------------------------------------------------------------------
    // Edge cases
    // ---------------------------------------------------------------------------

    #[ruby_test]
    fn test_single_pixel() {
        let pixels = vec![42, 128, 200, 255];
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 1, 1, 5, 1).unwrap();
        assert!(!palette.is_empty());
        assert_eq!(palette[0], vec![42, 128, 200]);
    }

    #[ruby_test]
    fn test_single_pixel_color() {
        let pixels = vec![200, 100, 50, 255];
        let rs = rstring(&pixels);
        let color = get_color(rs, 1, 1, 1).unwrap();
        assert_eq!(color, vec![200, 100, 50]);
    }

    #[ruby_test]
    fn test_non_square_wide() {
        let pixels = build_image(10, 2, |x, _| {
            if x < 5 { (255, 0, 0) } else { (0, 0, 255) }
        });
        let rs = rstring(&pixels);
        let color = get_color(rs, 10, 2, 1).unwrap();
        assert_eq!(color.len(), 3);
    }

    #[ruby_test]
    fn test_non_square_tall() {
        let pixels = build_image(2, 10, |_, y| {
            if y < 5 { (200, 100, 50) } else { (50, 100, 200) }
        });
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 2, 10, 5, 1).unwrap();
        assert!(!palette.is_empty());
        assert!(palette.len() <= 5);
    }

    #[ruby_test]
    fn test_large_solid_image() {
        let pixels = solid_pixels(170, 85, 220, 10_000);
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 100, 100, 10, 1).unwrap();
        assert!(palette.iter().any(|c| c[0] == 170 && c[1] == 85 && c[2] == 220));
    }

    #[ruby_test]
    fn test_gradient_image() {
        let pixels = build_image(30, 30, |x, _, _| {
            ((x * 8) as u8, 128, 64)
        });
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 30, 30, 5, 1).unwrap();
        assert!(!palette.is_empty());
    }

    #[ruby_test]
    fn test_checkerboard() {
        let pixels = build_image(50, 50, |x, y| {
            if (x + y) % 2 == 0 { (200, 50, 50) } else { (50, 50, 200) }
        });
        let rs = rstring(&pixels);
        let palette = get_palette(rs, 50, 50, 5, 1).unwrap();
        assert!(!palette.is_empty());
    }

    // ---------------------------------------------------------------------------
    // Error handling
    // ---------------------------------------------------------------------------

    #[ruby_test]
    fn test_empty_pixels_error() {
        let rs = RString::new("");
        let result = get_palette(rs, 0, 0, 5, 1);
        assert!(result.is_err(), "empty pixels should error");
    }

    #[ruby_test]
    fn test_empty_pixels_color_error() {
        let rs = RString::new("");
        let result = get_color(rs, 0, 0, 1);
        assert!(result.is_err(), "empty pixels should error for get_color");
    }

    #[ruby_test]
    fn test_zero_dimensions_error() {
        let rs = RString::new("");
        let result = get_palette(rs, 0, 0, 5, 1);
        assert!(result.is_err());
    }

    // ---------------------------------------------------------------------------
    // GC stress — repeated calls should not leak or crash
    // ---------------------------------------------------------------------------

    #[ruby_test]
    fn test_gc_stress_palette() {
        let pixels = solid_pixels(100, 150, 200, 400);
        for _ in 0..50 {
            let rs = rstring(&pixels);
            let palette = get_palette(rs, 20, 20, 5, 1).unwrap();
            assert!(!palette.is_empty());
        }
    }

    #[ruby_test]
    fn test_gc_stress_color() {
        let pixels = solid_pixels(100, 150, 200, 400);
        for _ in 0..50 {
            let rs = rstring(&pixels);
            let color = get_color(rs, 20, 20, 1).unwrap();
            assert_eq!(color.len(), 3);
        }
    }

    #[ruby_test]
    fn test_gc_stress_mixed() {
        let pixels = two_color_pixels(255, 0, 0, 0, 255, 0);
        for _ in 0..25 {
            let rs1 = rstring(&pixels);
            let rs2 = rstring(&pixels);
            let palette = get_palette(rs1, 10, 10, 5, 1).unwrap();
            let color = get_color(rs2, 10, 10, 1).unwrap();
            assert!(!palette.is_empty());
            assert_eq!(color.len(), 3);
        }
    }
}