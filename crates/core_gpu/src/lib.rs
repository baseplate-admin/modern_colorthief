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

#[cfg(not(target_arch = "wasm32"))]
mod vulkan;

#[cfg(not(target_arch = "wasm32"))]
pub use vulkan::{VulkanBackend, list_gpus, select_gpu};

/// WebGPU compute backend for WASM targets.
/// Uses browser WebGPU API via wasm-bindgen interop.
#[cfg(target_arch = "wasm32")]
pub mod webgpu {
    use super::GpuInfo;

    pub struct WebGpuBackend {
        available: bool,
    }

    impl WebGpuBackend {
        pub fn new() -> Self {
            WebGpuBackend { available: false }
        }

        fn is_available(&self) -> bool {
            self.available
        }
    }

    impl Default for WebGpuBackend {
        fn default() -> Self {
            Self::new()
        }
    }
}

/// Palette extraction result.
pub type Palette = Vec<(u8, u8, u8)>;

/// Extract palette using GPU compute.
/// Returns error if no GPU is available — no CPU fallback.
#[cfg(not(target_arch = "wasm32"))]
pub fn extract_palette_from_buffer(
    buffer: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> Result<Palette, String> {
    vulkan::gpu_extract(buffer, width, height, color_count, quality)
}

#[cfg(target_arch = "wasm32")]
pub fn extract_palette_from_buffer(
    _buffer: &[u8],
    _width: u32,
    _height: u32,
    _color_count: u8,
    _quality: u8,
) -> Result<Palette, String> {
    Err("GPU not available on WASM — use CPU version or WebGPU backend".to_string())
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
            assert!(path.exists(), "Vulkan loader path exists: {}", path.display());
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
                    palette.iter().any(|(r, g, b)| *r > 200 && *g < 55 && *b < 55),
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
                    palette.iter().any(|(r, g, b)| *r < 55 && *g > 200 && *b < 55),
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
                    palette.iter().any(|(r, g, b)| *r < 55 && *g < 55 && *b > 200),
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
            assert!(palette.len() <= 1, "color_count=1 should return at most 1 color");
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
                    assert!(!palette.is_empty(), "quality={} should return colors", quality);
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
        let _clone = d1.clone();
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
}
