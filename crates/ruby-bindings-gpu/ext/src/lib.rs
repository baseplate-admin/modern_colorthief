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

    fn solid_pixels(r: u8, g: u8, b: u8) -> Vec<u8> {
        let mut buf = Vec::with_capacity(400);
        for _ in 0..100 {
            buf.extend_from_slice(&[r, g, b, 255]);
        }
        buf
    }

    #[ruby_test]
    fn test_get_palette_solid_red(_vm: &magnus::Vm) {
        let pixels = solid_pixels(255, 0, 0);
        let rs = RString::from_str(&String::from_utf8_lossy(&pixels)).unwrap();
        let palette = get_palette(rs, 10, 10, 5, 1).unwrap();
        assert!(!palette.is_empty());
        assert!(palette.len() <= 5);
    }

    #[ruby_test]
    fn test_get_color_solid_red(_vm: &magnus::Vm) {
        let pixels = solid_pixels(255, 0, 0);
        let rs = RString::from_str(&String::from_utf8_lossy(&pixels)).unwrap();
        let color = get_color(rs, 10, 10, 1).unwrap();
        assert_eq!(color, vec![255, 0, 0]);
    }

    #[ruby_test]
    fn test_get_color_solid_green(_vm: &magnus::Vm) {
        let pixels = solid_pixels(0, 255, 0);
        let rs = RString::from_str(&String::from_utf8_lossy(&pixels)).unwrap();
        let color = get_color(rs, 10, 10, 1).unwrap();
        assert_eq!(color, vec![0, 255, 0]);
    }

    #[ruby_test]
    fn test_palette_determinism(_vm: &magnus::Vm) {
        let pixels = solid_pixels(100, 150, 200);
        let rs1 = RString::from_str(&String::from_utf8_lossy(&pixels)).unwrap();
        let rs2 = RString::from_str(&String::from_utf8_lossy(&pixels)).unwrap();
        let p1 = get_palette(rs1, 10, 10, 5, 1).unwrap();
        let p2 = get_palette(rs2, 10, 10, 5, 1).unwrap();
        assert_eq!(p1, p2);
    }

    #[ruby_test]
    fn test_color_count_bound(_vm: &magnus::Vm) {
        let pixels = solid_pixels(255, 128, 64);
        let rs = RString::from_str(&String::from_utf8_lossy(&pixels)).unwrap();
        let palette = get_palette(rs, 10, 10, 3, 1).unwrap();
        assert!(palette.len() <= 3);
    }

    #[ruby_test]
    fn test_empty_pixels_error(_vm: &magnus::Vm) {
        let rs = RString::from_str("").unwrap();
        let result = get_palette(rs, 0, 0, 5, 1);
        assert!(result.is_err());
    }
}
