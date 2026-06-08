use modern_colorthief_core::PaletteExtractor;

/// GPU backend that implements the shared PaletteExtractor trait.
pub struct GpuExtractor;

impl Default for GpuExtractor {
    fn default() -> Self {
        GpuExtractor
    }
}

impl GpuExtractor {
    pub fn new() -> Self {
        GpuExtractor
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl PaletteExtractor for GpuExtractor {
    fn extract_palette(
        &self,
        buffer: &[u8],
        width: u32,
        height: u32,
        color_count: u8,
        quality: u8,
    ) -> Result<Vec<(u8, u8, u8)>, String> {
        crate::vulkan::gpu_extract(buffer, width, height, color_count, quality)
    }
}

#[cfg(target_arch = "wasm32")]
impl PaletteExtractor for GpuExtractor {
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
}
