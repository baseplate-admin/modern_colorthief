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

    #[test]
    #[ignore]
    fn test_gpu_or_not_available() {
        let buffer: Vec<u8> = [255u8, 0, 0, 255].repeat(25);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        if let Ok(palette) = result {
            assert!(!palette.is_empty());
        }
    }
}
