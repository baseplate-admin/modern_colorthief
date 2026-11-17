mod webgpu;

use modern_colorthief_core::{ColorthiefError, PaletteExtractor, Rgb};

/// WebGPU compute backend that implements the shared PaletteExtractor trait.
#[derive(Default)]
pub struct WebGpuExtractor;

/// Deduplicate colors preserving first-seen order, capped at `max` entries.
fn dedup_colors(colors: Vec<Rgb>, max: usize) -> Vec<Rgb> {
    let mut seen = std::collections::HashSet::with_capacity(colors.len());
    let mut out = Vec::with_capacity(colors.len());
    for c in colors {
        if seen.insert(c) && out.len() < max {
            out.push(c);
        }
    }
    out
}

impl PaletteExtractor for WebGpuExtractor {
    fn extract_palette(
        &self,
        buffer: &[u8],
        width: u32,
        height: u32,
        color_count: u8,
        quality: u8,
    ) -> Result<Vec<Rgb>, ColorthiefError> {
        // WebGPU is async-only — use extract_palette_from_buffer_webgpu from the wasm bindings.
        // This sync path is a placeholder for trait compatibility.
        let _ = (buffer, width, height, color_count, quality);
        Err(ColorthiefError::Backend(
            "WebGPU requires async — use extract_palette_from_buffer_webgpu() instead".to_string(),
        ))
    }
}

fn string_err_to_colorthief_err(msg: String) -> ColorthiefError {
    ColorthiefError::Backend(msg)
}

/// Extract palette using WebGPU compute shaders.
pub fn extract_palette_from_buffer_webgpu(
    buffer: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> impl std::future::Future<Output = Result<Vec<Rgb>, ColorthiefError>> {
    extract_palette_async(buffer, width, height, color_count, quality)
}

/// Extract dominant color using WebGPU compute shaders.
pub fn extract_color_from_buffer_webgpu(
    buffer: &[u8],
    width: u32,
    height: u32,
    quality: u8,
) -> impl std::future::Future<Output = Result<Rgb, ColorthiefError>> {
    extract_color_async(buffer, width, height, quality)
}

async fn extract_palette_async(
    buffer: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> Result<Vec<Rgb>, ColorthiefError> {
    if buffer.is_empty() {
        return Err(ColorthiefError::EmptyBuffer);
    }

    let raw = webgpu::extract_palette_webgpu(buffer, width, height, color_count, quality)
        .await
        .map_err(string_err_to_colorthief_err)?;

    let mut colors = Vec::with_capacity(raw.len() / 3);
    let mut i = 0;
    while i + 2 < raw.len() {
        colors.push((raw[i], raw[i + 1], raw[i + 2]));
        i += 3;
    }
    Ok(dedup_colors(colors, color_count as usize))
}

async fn extract_color_async(
    buffer: &[u8],
    width: u32,
    height: u32,
    quality: u8,
) -> Result<Rgb, ColorthiefError> {
    let palette = extract_palette_async(buffer, width, height, 5, quality).await?;
    modern_colorthief_core::bindings::dominant_color(&palette)
}
