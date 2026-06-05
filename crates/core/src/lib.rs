use color_thief::ColorFormat;
use itertools::Itertools;

/// Extract a deduplicated color palette from raw RGBA pixel data.
///
/// Expects a flat RGBA buffer (4 bytes per pixel, `width * height * 4`).
pub fn extract_palette_from_buffer(
    buffer: &[u8],
    _width: u32,
    _height: u32,
    color_count: u8,
    quality: u8,
) -> Result<Vec<(u8, u8, u8)>, String> {
    let colors = color_thief::get_palette(buffer, ColorFormat::Rgba, quality, color_count)
        .map_err(|e| format!("color_thief failed: {e}"))?;

    let color_vec: Vec<(u8, u8, u8)> = colors
        .iter()
        .map(|c| (c.r, c.g, c.b))
        .unique()
        .collect();

    Ok(color_vec)
}
