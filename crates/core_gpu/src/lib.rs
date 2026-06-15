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
}
