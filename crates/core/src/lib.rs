//! Core types and traits shared by CPU and GPU backends.

/// Info about a discovered GPU device.
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

/// Trait for palette extraction backends (CPU, GPU, WebGPU, etc.).
pub trait PaletteExtractor: Send + Sync {
    /// Extract a deduplicated color palette from raw RGBA pixel data.
    fn extract_palette(
        &self,
        buffer: &[u8],
        width: u32,
        height: u32,
        color_count: u8,
        quality: u8,
    ) -> Result<Vec<(u8, u8, u8)>, String>;

    /// Extract the dominant color from raw RGBA pixel data.
    fn extract_color(
        &self,
        buffer: &[u8],
        width: u32,
        height: u32,
        quality: u8,
    ) -> Result<(u8, u8, u8), String> {
        let palette = self.extract_palette(buffer, width, height, 5, quality)?;
        palette
            .first()
            .copied()
            .ok_or("No colors found".to_string())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_palette() {
        // Trait is abstract — backends implement it.
        // Integration tests live in core_cpu and core_gpu.
    }
}
