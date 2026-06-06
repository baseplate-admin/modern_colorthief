use napi::bindgen_prelude::*;

/// GPU-accelerated palette extraction (Vulkan Compute backend).
/// Falls back to CPU if no Vulkan device is available.
#[napi_derive::napi]
pub fn get_palette_gpu(
    pixels: Uint8Array,
    width: u32,
    height: u32,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> napi::Result<Vec<Vec<u8>>> {
    let pixels = pixels.to_vec();
    modern_colorthief_core_gpu::extract_palette_from_buffer(
        &pixels,
        width,
        height,
        color_count.unwrap_or(10),
        quality.unwrap_or(10),
    )
    .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
    .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e))
}

/// GPU-accelerated dominant color extraction.
#[napi_derive::napi]
pub fn get_color_gpu(
    pixels: Uint8Array,
    width: u32,
    height: u32,
    quality: Option<u8>,
) -> napi::Result<Vec<u8>> {
    let pixels = pixels.to_vec();
    let palette = modern_colorthief_core_gpu::extract_palette_from_buffer(
        &pixels,
        width,
        height,
        5,
        quality.unwrap_or(10),
    )
    .map_err(|e| napi::Error::new(napi::Status::GenericFailure, e))?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| napi::Error::new(napi::Status::GenericFailure, "Image contains no colors"))
}
