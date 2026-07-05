use napi::bindgen_prelude::*;

/// Extract a palette of dominant colors from raw RGBA pixel data.
///
/// @param pixels - Raw RGBA pixel buffer (4 bytes per pixel)
/// @param width - Image width in pixels
/// @param height - Image height in pixels
/// @param colorCount - Number of colors to extract (default: 10)
/// @param quality - Sampling quality, higher is faster but less accurate (default: 10)
/// @returns Array of [R, G, B] color tuples
#[napi_derive::napi]
pub fn get_palette(
    pixels: Uint8Array,
    width: u32,
    height: u32,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> napi::Result<Vec<Vec<u8>>> {
    let pixels = pixels.to_vec();
    let color_count = color_count.unwrap_or(10);
    let quality = quality.unwrap_or(10);

    modern_colorthief_core_cpu::extract_palette_from_buffer(
        &pixels,
        width,
        height,
        color_count,
        quality,
    )
    .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
    .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e))
}

/// Extract the dominant color from raw RGBA pixel data.
///
/// @param pixels - Raw RGBA pixel buffer (4 bytes per pixel)
/// @param width - Image width in pixels
/// @param height - Image height in pixels
/// @param quality - Sampling quality, higher is faster but less accurate (default: 10)
/// @returns [R, G, B] color tuple
#[napi_derive::napi]
pub fn get_color(
    pixels: Uint8Array,
    width: u32,
    height: u32,
    quality: Option<u8>,
) -> napi::Result<Vec<u8>> {
    let pixels = pixels.to_vec();
    let quality = quality.unwrap_or(10);

    let palette =
        modern_colorthief_core_cpu::extract_palette_from_buffer(&pixels, width, height, 5, quality)
            .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e))?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| napi::Error::new(napi::Status::GenericFailure, "Image contains no colors"))
}
