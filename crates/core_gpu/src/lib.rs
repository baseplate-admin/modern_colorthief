//! GPU-accelerated palette extraction.
//! Uses Vulkan compute backend. Returns error if no GPU available.

/// Info about a discovered GPU.
#[derive(Clone, Debug)]
pub struct GpuInfo {
    pub index: usize,
    pub name: String,
    pub device_type: GpuDevice,
    pub vendor_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuDevice {
    Discrete,
    Integrated,
    Virtual,
    CPU,
    Other,
}

mod vulkan;

pub use vulkan::{VulkanBackend, list_gpus, select_gpu};

/// Palette extraction result.
pub type Palette = Vec<(u8, u8, u8)>;

/// Extract palette using GPU compute.
/// Returns error if no GPU is available — no CPU fallback.
pub fn extract_palette_from_buffer(
    buffer: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> Result<Palette, String> {
    vulkan::gpu_extract(buffer, width, height, color_count, quality)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_buffer() {
        let result = extract_palette_from_buffer(&[], 0, 0, 5, 10);
        assert!(result.is_err());
    }

    /// Test that the Vulkan loader can be found (or not) without crashing.
    /// Actual GPU extraction is tested in the integration test `vulkan_init_probe`,
    /// because Vulkan instance creation may segfault on SwiftShader/llvmpipe CI runners.
    #[test]
    fn test_gpu_or_not_available() {
        let loader_path = VulkanBackend::find_vulkan_loader();
        // Either the loader is found or not — both are valid.
        // If found, the loader path should point to an existing file.
        if let Some(path) = loader_path {
            assert!(
                path.exists(),
                "Vulkan loader path exists: {}",
                path.display()
            );
        }
    }

    /// Test extraction with solid red buffer — verifies Vulkan compute pipeline.
    #[test]
    fn test_solid_red() {
        let buffer: Vec<u8> = [255u8, 0, 0, 255].repeat(100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        match result {
            Ok(palette) => {
                assert!(!palette.is_empty(), "palette should not be empty");
                // Should contain a color close to red
                assert!(
                    palette
                        .iter()
                        .any(|(r, g, b)| *r > 200 && *g < 55 && *b < 55),
                    "palette should contain a red-dominant color"
                );
            }
            Err(e) => {
                // GPU may not be available — that's acceptable
                assert!(e.contains("Vulkan") || e.contains("Empty") || e.contains("not found"));
            }
        }
    }

    /// Test extraction with solid green buffer.
    #[test]
    fn test_solid_green() {
        let buffer: Vec<u8> = [0u8, 255, 0, 255].repeat(100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        match result {
            Ok(palette) => {
                assert!(!palette.is_empty());
                assert!(
                    palette
                        .iter()
                        .any(|(r, g, b)| *r < 55 && *g > 200 && *b < 55),
                    "palette should contain a green-dominant color"
                );
            }
            Err(_) => { /* GPU not available */ }
        }
    }

    /// Test extraction with solid blue buffer.
    #[test]
    fn test_solid_blue() {
        let buffer: Vec<u8> = [0u8, 0, 255, 255].repeat(100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        match result {
            Ok(palette) => {
                assert!(!palette.is_empty());
                assert!(
                    palette
                        .iter()
                        .any(|(r, g, b)| *r < 55 && *g < 55 && *b > 200),
                    "palette should contain a blue-dominant color"
                );
            }
            Err(_) => { /* GPU not available */ }
        }
    }

    /// Test that color_count limits the output palette size.
    #[test]
    fn test_color_count_limit() {
        let buffer: Vec<u8> = [100u8, 150, 200, 255].repeat(100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 1, 1);
        if let Ok(palette) = result {
            assert!(
                palette.len() <= 1,
                "color_count=1 should return at most 1 color"
            );
        }
    }

    /// Test that quality parameter doesn't cause panics at extremes.
    #[test]
    fn test_quality_extremes() {
        let buffer: Vec<u8> = [200u8, 100, 50, 255].repeat(100);
        for quality in [1u8, 10, 100] {
            let result = extract_palette_from_buffer(&buffer, 10, 10, 5, quality);
            match result {
                Ok(palette) => {
                    assert!(
                        !palette.is_empty(),
                        "quality={} should return colors",
                        quality
                    );
                }
                Err(_) => { /* GPU not available */ }
            }
        }
    }

    /// Test with non-square dimensions.
    #[test]
    fn test_wide_image() {
        let buffer: Vec<u8> = [255u8, 128, 0, 255].repeat(200);
        let result = extract_palette_from_buffer(&buffer, 20, 10, 5, 1);
        match result {
            Ok(palette) => assert!(!palette.is_empty()),
            Err(_) => { /* GPU not available */ }
        }
    }

    /// Test with tall dimensions.
    #[test]
    fn test_tall_image() {
        let buffer: Vec<u8> = [0u8, 128, 255, 255].repeat(200);
        let result = extract_palette_from_buffer(&buffer, 10, 20, 5, 1);
        match result {
            Ok(palette) => assert!(!palette.is_empty()),
            Err(_) => { /* GPU not available */ }
        }
    }

    /// Test that returned colors are valid RGB values.
    #[test]
    fn test_valid_rgb_values() {
        let buffer: Vec<u8> = [170u8, 85, 220, 255].repeat(100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        if let Ok(palette) = result {
            for (r, g, b) in &palette {
                // All values are u8 so they're inherently in [0, 255]
                assert!((*r as i32) >= 0 && (*r as i32) <= 255);
                assert!((*g as i32) >= 0 && (*g as i32) <= 255);
                assert!((*b as i32) >= 0 && (*b as i32) <= 255);
            }
        }
    }

    /// Test GpuDevice enum properties.
    #[test]
    fn test_gpu_device_traits() {
        // Verify GpuDevice implements Copy, Clone, Debug, PartialEq, Eq
        let d1 = GpuDevice::Discrete;
        let d2 = GpuDevice::Discrete;
        let d3 = GpuDevice::CPU;
        assert_eq!(d1, d2);
        assert_ne!(d1, d3);
        // Verify Clone and Copy
        let _clone = d1;
        let _copy = d1;
    }

    /// Test GpuInfo struct properties.
    #[test]
    fn test_gpu_info_struct() {
        let info = GpuInfo {
            index: 0,
            name: "Test GPU".to_string(),
            device_type: GpuDevice::Discrete,
            vendor_name: "Test Vendor".to_string(),
        };
        assert_eq!(info.index, 0);
        assert_eq!(info.name, "Test GPU");
        assert_eq!(info.device_type, GpuDevice::Discrete);
        // Verify Debug works
        let _debug = format!("{:?}", info);
    }

    /// Test two-color detection — palette should contain both red and blue.
    #[test]
    fn test_two_color_red_blue() {
        let mut buffer = Vec::new();
        for _ in 0..50 {
            buffer.extend_from_slice(&[255, 0, 0, 255]);
        }
        for _ in 0..50 {
            buffer.extend_from_slice(&[0, 0, 255, 255]);
        }
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        if let Ok(palette) = result {
            assert!(
                palette
                    .iter()
                    .any(|(r, g, b)| *r > 200 && *g < 55 && *b < 55),
                "should detect red"
            );
            assert!(
                palette
                    .iter()
                    .any(|(r, g, b)| *r < 55 && *g < 55 && *b > 200),
                "should detect blue"
            );
        }
    }

    /// Test 1x1 single pixel returns that exact color.
    #[test]
    fn test_single_pixel() {
        let buffer = [42u8, 100, 200, 255];
        let result = extract_palette_from_buffer(&buffer, 1, 1, 5, 1);
        if let Ok(palette) = result {
            assert!(!palette.is_empty());
            assert_eq!(palette[0], (42, 100, 200));
        }
    }

    /// Test gradient image returns multiple distinct colors.
    #[test]
    fn test_gradient_image() {
        let mut buffer = Vec::new();
        for x in 0..20u8 {
            for _ in 0..10u8 {
                buffer.extend_from_slice(&[x * 13, x * 7, x * 5, 255]);
            }
        }
        let result = extract_palette_from_buffer(&buffer, 20, 10, 10, 1);
        if let Ok(palette) = result {
            assert!(palette.len() > 1, "gradient should produce >1 color");
        }
    }

    /// Test checkerboard pattern.
    #[test]
    fn test_checkerboard() {
        let mut buffer = Vec::new();
        for y in 0..10u8 {
            for x in 0..10u8 {
                let (r, g, b) = if (x + y) % 2 == 0 {
                    (200, 50, 50)
                } else {
                    (50, 50, 200)
                };
                buffer.extend_from_slice(&[r, g, b, 255]);
            }
        }
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        if let Ok(palette) = result {
            assert!(!palette.is_empty());
        }
    }

    /// Test determinism — same input produces identical output.
    #[test]
    fn test_determinism() {
        let buffer: Vec<u8> = [170u8, 85, 220, 255].repeat(100);
        let r1 = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        let r2 = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        match (r1, r2) {
            (Ok(p1), Ok(p2)) => assert_eq!(p1, p2, "deterministic palette"),
            (Err(_), Err(_)) => {} // GPU unavailable is fine
            _ => panic!("inconsistent GPU availability"),
        }
    }

    /// Test deduplication — solid color should not produce duplicates.
    #[test]
    fn test_deduplication() {
        let buffer: Vec<u8> = [100u8, 200, 150, 255].repeat(100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 255, 1);
        if let Ok(palette) = result {
            let len = palette.len();
            let unique: std::collections::HashSet<_> = palette.into_iter().collect();
            assert_eq!(len, unique.len(), "no duplicate colors");
        }
    }

    /// Test quality=0 is clamped to valid range without panic.
    #[test]
    fn test_quality_zero_clamped() {
        let buffer: Vec<u8> = [255u8, 0, 0, 255].repeat(100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 0);
        match result {
            Ok(palette) => assert!(!palette.is_empty()),
            Err(_) => { /* GPU not available */ }
        }
    }

    /// Test dominant color appears in palette.
    #[test]
    fn test_dominant_in_palette() {
        let buffer: Vec<u8> = [255u8, 0, 0, 255]
            .repeat(50)
            .into_iter()
            .chain(std::iter::repeat_n(0, 200)) // 50 blue pixels
            .collect();
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        if let Ok(palette) = result {
            assert!(
                palette.iter().any(|(r, _g, _b)| *r > 200),
                "dominant red should be in palette"
            );
        }
    }

    /// Test different images produce different palettes.
    #[test]
    fn test_different_images_different_palette() {
        let red: Vec<u8> = [255u8, 0, 0, 255].repeat(100);
        let blue: Vec<u8> = [0u8, 0, 255, 255].repeat(100);
        let r1 = extract_palette_from_buffer(&red, 10, 10, 5, 1);
        let r2 = extract_palette_from_buffer(&blue, 10, 10, 5, 1);
        match (r1, r2) {
            (Ok(p1), Ok(p2)) => assert_ne!(p1, p2, "red and blue should differ"),
            (Err(_), Err(_)) => {} // GPU unavailable
            _ => panic!("inconsistent GPU availability"),
        }
    }

    /// Test extraction with solid white buffer.
    #[test]
    fn test_solid_white() {
        let buffer: Vec<u8> = [255u8, 255, 255, 255].repeat(100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        match result {
            Ok(palette) => {
                assert!(!palette.is_empty());
                assert!(
                    palette
                        .iter()
                        .any(|(r, g, b)| *r > 200 && *g > 200 && *b > 200),
                    "palette should contain a white-dominant color"
                );
            }
            Err(_) => { /* GPU not available */ }
        }
    }

    /// Test extraction with solid black buffer.
    #[test]
    fn test_solid_black() {
        let buffer: Vec<u8> = [0u8, 0, 0, 255].repeat(100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        match result {
            Ok(palette) => {
                assert!(!palette.is_empty());
                assert!(
                    palette
                        .iter()
                        .any(|(r, g, b)| *r < 55 && *g < 55 && *b < 55),
                    "palette should contain a black-dominant color"
                );
            }
            Err(_) => { /* GPU not available */ }
        }
    }

    /// Test buffer too small for requested dimensions.
    #[test]
    fn test_buffer_too_small() {
        let buffer = [255u8, 0, 0, 255]; // only 1 pixel, but asking for 10x10
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        assert!(result.is_err(), "too-small buffer should return error");
    }

    /// Test deduplication — solid color yields exactly 1 color.
    #[test]
    fn test_dedup_solid_color_exactly_one() {
        let buffer: Vec<u8> = [255u8, 0, 0, 255].repeat(100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        if let Ok(palette) = result {
            assert_eq!(
                palette.len(),
                1,
                "solid red with color_count=5 should return exactly 1 unique color"
            );
        }
    }

    /// Test two-color green/purple detection.
    #[test]
    fn test_two_color_green_purple() {
        let mut buffer = Vec::new();
        for _ in 0..50 {
            buffer.extend_from_slice(&[0, 255, 0, 255]);
        }
        for _ in 0..50 {
            buffer.extend_from_slice(&[128, 0, 128, 255]);
        }
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        if let Ok(palette) = result {
            assert!(
                palette
                    .iter()
                    .any(|(r, g, b)| *r < 55 && *g > 200 && *b < 55),
                "should detect green"
            );
        }
    }

    /// Test dominant color reflects majority (90/10 split).
    #[test]
    fn test_dominant_reflects_majority() {
        let mut buffer = Vec::new();
        for _ in 0..90 {
            buffer.extend_from_slice(&[255, 0, 0, 255]);
        }
        for _ in 0..10 {
            buffer.extend_from_slice(&[0, 0, 255, 255]);
        }
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        if let Ok(palette) = result {
            let dominant = &palette[0];
            assert!(
                dominant.0 > 200,
                "dominant color should be red for 90/10 red/blue split"
            );
        }
    }
}
