/// Trait for palette extraction backends (CPU, GPU, WebGPU, etc.).
pub trait PaletteExtractor: Send + Sync {
    fn extract(
        &self,
        buffer: &[u8],
        width: u32,
        height: u32,
        color_count: u8,
        quality: u8,
    ) -> Result<Vec<(u8, u8, u8)>, String>;
}

#[cfg(feature = "cpu")]
mod backend {
    pub use modern_colorthief_core_cpu::*;
}

#[cfg(feature = "gpu")]
mod backend {
    pub use modern_colorthief_core_gpu::*;
}

pub use backend::extract_palette_from_buffer;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_palette() {
        let buffer: Vec<u8> = [255u8, 0, 0, 255].repeat(100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
        assert!(result.is_ok());
        let palette = result.unwrap();
        assert!(!palette.is_empty());
        assert!(palette[0].0 == 255 && palette[0].1 == 0 && palette[0].2 == 0);
    }
}
