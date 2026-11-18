use napi::bindgen_prelude::{Error, Status, Uint8Array};

use modern_colorthief_core::ColorthiefError;

fn err_to_napi(err: ColorthiefError) -> Error {
    let msg = err.to_string();
    let lower = msg.to_lowercase();
    let status =
        if lower.contains("empty") || lower.contains("dimensions") || lower.contains("pixel") {
            Status::InvalidArg
        } else {
            Status::GenericFailure
        };
    Error::new(status, msg)
}

/// GPU-accelerated palette extraction (Vulkan Compute backend).
/// Falls back to CPU if no Vulkan device is available.
///
/// @param pixels - Raw RGBA pixel buffer (4 bytes per pixel)
/// @param width - Image width in pixels
/// @param height - Image height in pixels
/// @param colorCount - Number of colors to extract (default: 10)
/// @param quality - Sampling quality (default: 10)
/// @returns Promise resolving to `number[][]`
#[napi_derive::napi]
pub async fn get_palette_gpu(
    pixels: Uint8Array,
    width: u32,
    height: u32,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> napi::Result<Vec<Vec<u8>>> {
    let pixels = pixels.to_vec();
    let color_count = color_count.unwrap_or(10);
    let quality = quality.unwrap_or(10);

    let colors = napi::tokio::task::spawn_blocking(move || {
        modern_colorthief_core_gpu::extract_palette_from_buffer(
            &pixels,
            width,
            height,
            color_count,
            quality,
        )
    })
    .await
    .map_err(|e| Error::new(Status::GenericFailure, format!("Task panicked: {e}")))?
    .map_err(err_to_napi)?;

    Ok(modern_colorthief_core::bindings::colors_to_rows_u8(&colors))
}

/// GPU-accelerated dominant color extraction.
///
/// @param pixels - Raw RGBA pixel buffer (4 bytes per pixel)
/// @param width - Image width in pixels
/// @param height - Image height in pixels
/// @param quality - Sampling quality (default: 10)
/// @returns Promise resolving to `number[]` — `[R, G, B]`
#[napi_derive::napi]
pub async fn get_color_gpu(
    pixels: Uint8Array,
    width: u32,
    height: u32,
    quality: Option<u8>,
) -> napi::Result<Vec<u8>> {
    let pixels = pixels.to_vec();
    let quality = quality.unwrap_or(10);

    let result = napi::tokio::task::spawn_blocking(move || {
        let palette = modern_colorthief_core_gpu::extract_palette_from_buffer(
            &pixels, width, height, 5, quality,
        )
        .map_err(err_to_napi)?;
        modern_colorthief_core::bindings::dominant_color(&palette)
            .map(|(r, g, b)| vec![r, g, b])
            .map_err(err_to_napi)
    })
    .await
    .map_err(|e| Error::new(Status::GenericFailure, format!("Task panicked: {e}")))?;

    result
}
