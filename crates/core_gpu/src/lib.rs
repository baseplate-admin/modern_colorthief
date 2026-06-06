//! GPU-accelerated palette extraction.
//! Uses Vulkan compute backend. Returns error if no GPU available.

/// Info about a discovered GPU.
#[derive(Clone, Debug)]
pub struct GpuInfo {
    pub index: usize,
    pub name: String,
    pub device_type: GpuDevice,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuDevice {
    Discrete,
    Integrated,
    Virtual,
    CPU,
    Other,
}
mod traits;

pub use traits::ComputeBackend;

#[cfg(not(target_arch = "wasm32"))]
mod vulkan;

#[cfg(not(target_arch = "wasm32"))]
pub use vulkan::{VulkanBackend, list_gpus, select_gpu};

/// WebGPU compute backend stub (for WASM targets).
/// TODO: Implement WebGPU compute shaders for palette extraction.
#[cfg(target_arch = "wasm32")]
pub mod webgpu {
    use super::{ComputeBackend, GpuInfo};

    pub struct WebGpuBackend;

    impl WebGpuBackend {
        pub fn new() -> Self {
            WebGpuBackend
        }
    }

    impl ComputeBackend for WebGpuBackend {
        fn is_available(&self) -> bool {
            false
        }
        fn extract_palette(
            &self,
            _buffer: &[u8],
            _width: u32,
            _height: u32,
            _color_count: u8,
            _quality: u8,
        ) -> Result<Vec<(u8, u8, u8)>, String> {
            Err("WebGPU backend not yet implemented".to_string())
        }
        fn list_devices(&self) -> Result<Vec<GpuInfo>, String> {
            Ok(Vec::new())
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
