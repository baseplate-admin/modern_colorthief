mod webgpu;

use modern_colorthief_core::PaletteExtractor;

/// WebGPU compute backend that implements the shared PaletteExtractor trait.
#[derive(Default)]
pub struct WebGpuExtractor;

impl PaletteExtractor for WebGpuExtractor {
    fn extract_palette(
        &self,
        buffer: &[u8],
        width: u32,
        height: u32,
        color_count: u8,
        quality: u8,
    ) -> Result<Vec<(u8, u8, u8)>, String> {
        // WebGPU is async-only — use extract_palette_async / extract_palette_from_buffer_webgpu
        // from the wasm bindings instead. This sync path is a placeholder for trait
        // compatibility; calling it will return an error.
        let _ = (buffer, width, height, color_count, quality);
        Err("WebGPU requires async — use extract_palette_from_buffer_webgpu() instead".to_string())
    }
}

/// Extract palette using WebGPU compute shaders.
pub fn extract_palette_from_buffer_webgpu(
    buffer: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> impl std::future::Future<Output = Result<Vec<(u8, u8, u8)>, String>> {
    extract_palette_async(buffer, width, height, color_count, quality)
}

/// Extract dominant color using WebGPU compute shaders.
pub fn extract_color_from_buffer_webgpu(
    buffer: &[u8],
    width: u32,
    height: u32,
    quality: u8,
) -> impl std::future::Future<Output = Result<(u8, u8, u8), String>> {
    extract_color_async(buffer, width, height, quality)
}

async fn extract_palette_async(
    buffer: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> Result<Vec<(u8, u8, u8)>, String> {
    if buffer.is_empty() {
        return Err("Empty pixel buffer".to_string());
    }

    let raw = webgpu::extract_palette_webgpu(buffer, width, height, color_count, quality).await?;

    let mut colors = Vec::with_capacity(raw.len() / 3);
    let mut i = 0;
    while i + 2 < raw.len() {
        colors.push((raw[i], raw[i + 1], raw[i + 2]));
        i += 3;
    }
    Ok(colors)
}

async fn extract_color_async(
    buffer: &[u8],
    width: u32,
    height: u32,
    quality: u8,
) -> Result<(u8, u8, u8), String> {
    let palette = extract_palette_async(buffer, width, height, 5, quality).await?;
    palette
        .into_iter()
        .next()
        .ok_or("No colors extracted".to_string())
}
