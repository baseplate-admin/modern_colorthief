//! GPU-accelerated palette extraction using Vulkan.
//! Returns an error if no Vulkan device is available.

#[cfg(not(target_arch = "wasm32"))]
mod vulkan;

#[cfg(not(target_arch = "wasm32"))]
pub use vulkan::{GpuInfo, select_gpu, list_gpus};

/// Palette extraction result.
pub type Palette = Vec<(u8, u8, u8)>;

/// Extract palette using the selected Vulkan GPU.
/// Returns error if Vulkan is unavailable.
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
    Err("Vulkan not available on WASM — use CPU version".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_buffer() {
        let result = extract_palette_from_buffer(&[], 0, 0, 5, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_gpu_or_not_available() {
        let buffer: Vec<u8> = [255u8, 0, 0, 255].repeat(25);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        if result.is_ok() {
            let palette = result.unwrap();
            assert!(!palette.is_empty());
        }
    }
}
