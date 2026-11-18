use napi::bindgen_prelude::{Error, Status, Uint8Array};

use modern_colorthief_core::ColorthiefError;

/// Map a core error to the right `napi::Status`: invalid input
/// (buffer/shape problems) is `InvalidArg`; everything else is a core
/// failure.
fn invalid_input_or_core_error(err: ColorthiefError) -> Error {
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

/// Extract a palette of dominant colors from raw RGBA pixel data.
///
/// The heavy work runs off the JS event loop (on the tokio blocking pool,
/// via napi's `tokio_rt` runtime) — this returns a `Promise` that resolves
/// to `number[][]` (each inner array is `[R, G, B]`).
///
/// @param pixels - Raw RGBA pixel buffer (4 bytes per pixel)
/// @param width - Image width in pixels
/// @param height - Image height in pixels
/// @param colorCount - Number of colors to extract (default: 10)
/// @param quality - Sampling quality, higher is faster but less accurate (default: 10)
/// @returns Promise resolving to `number[][]`
#[napi_derive::napi]
pub async fn get_palette(
    pixels: Uint8Array,
    width: u32,
    height: u32,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> napi::Result<Vec<Vec<u8>>> {
    let pixels = pixels.to_vec();
    let color_count = color_count.unwrap_or(10);
    let quality = quality.unwrap_or(10);

    // Off-thread: the JS event loop stays responsive during the median cut.
    let colors = napi::tokio::task::spawn_blocking(move || {
        modern_colorthief_core_cpu::extract_palette_from_buffer(
            &pixels,
            width,
            height,
            color_count,
            quality,
        )
    })
    .await
    .map_err(|e| Error::new(Status::GenericFailure, format!("Task panicked: {e}")))?
    .map_err(invalid_input_or_core_error)?;

    Ok(modern_colorthief_core::bindings::colors_to_rows_u8(&colors))
}

/// Extract the dominant color from raw RGBA pixel data.
///
/// The dominant color is the **first** entry of a 5-color palette — the
/// single canonical rule for all bindings (the old `.pop()`-based "last
/// color" behavior in the WASM surfaces was a bug).
///
/// @param pixels - Raw RGBA pixel buffer (4 bytes per pixel)
/// @param width - Image width in pixels
/// @param height - Image height in pixels
/// @param quality - Sampling quality (default: 10)
/// @returns Promise resolving to `number[]` — `[R, G, B]`
#[napi_derive::napi]
pub async fn get_color(
    pixels: Uint8Array,
    width: u32,
    height: u32,
    quality: Option<u8>,
) -> napi::Result<Vec<u8>> {
    let pixels = pixels.to_vec();
    let quality = quality.unwrap_or(10);

    // Off-thread: the JS event loop stays responsive during the median cut.
    napi::tokio::task::spawn_blocking(move || {
        let palette = modern_colorthief_core_cpu::extract_palette_from_buffer(
            &pixels, width, height, 5, quality,
        )?;
        modern_colorthief_core::bindings::dominant_color(&palette).map(|(r, g, b)| vec![r, g, b])
    })
    .await
    .map_err(|e| Error::new(Status::GenericFailure, format!("Task panicked: {e}")))?
    .map_err(invalid_input_or_core_error)
}

/// Synchronous variants for callers that don't want a Promise (e.g.
/// `node:worker_threads`, REPL, or environments without an event loop).
///
/// These block the calling thread — prefer `get_palette`/`get_color` on the
/// main thread.
#[napi_derive::napi]
pub fn get_palette_sync(
    pixels: Uint8Array,
    width: u32,
    height: u32,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> napi::Result<Vec<Vec<u8>>> {
    let pixels = pixels.to_vec();
    let colors = modern_colorthief_core_cpu::extract_palette_from_buffer(
        &pixels,
        width,
        height,
        color_count.unwrap_or(10),
        quality.unwrap_or(10),
    )
    .map_err(invalid_input_or_core_error)?;
    Ok(modern_colorthief_core::bindings::colors_to_rows_u8(&colors))
}

#[napi_derive::napi]
pub fn get_color_sync(
    pixels: Uint8Array,
    width: u32,
    height: u32,
    quality: Option<u8>,
) -> napi::Result<Vec<u8>> {
    let pixels = pixels.to_vec();
    let palette = modern_colorthief_core_cpu::extract_palette_from_buffer(
        &pixels,
        width,
        height,
        5,
        quality.unwrap_or(10),
    )
    .map_err(invalid_input_or_core_error)?;
    let (r, g, b) = modern_colorthief_core::bindings::dominant_color(&palette)
        .map_err(invalid_input_or_core_error)?;
    Ok(vec![r, g, b])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_status_for_invalid_input() {
        let e = invalid_input_or_core_error(ColorthiefError::EmptyBuffer);
        assert_eq!(e.status, Status::InvalidArg);
        let e = invalid_input_or_core_error(ColorthiefError::InvalidDimensions);
        assert_eq!(e.status, Status::InvalidArg);
    }

    #[test]
    fn error_status_for_core_failure() {
        let e = invalid_input_or_core_error(ColorthiefError::Backend("Vulkan not found".into()));
        assert_eq!(e.status, Status::GenericFailure);
    }

    #[test]
    fn sync_solid_red() {
        let colors = get_palette_sync(
            [255u8, 0, 0, 255].repeat(100).into(),
            10,
            10,
            Some(5),
            Some(1),
        )
        .unwrap();
        assert_eq!(colors.first(), Some(&vec![255, 0, 0]));
    }

    #[test]
    fn sync_rejects_truncated_buffer() {
        let err = get_palette_sync(
            [255u8, 0, 0, 255].repeat(2).into(),
            10,
            10,
            Some(5),
            Some(1),
        )
        .unwrap_err();
        assert_eq!(err.status, Status::InvalidArg);
    }

    #[test]
    fn sync_color_returns_dominant_first_entry() {
        // 90 red / 10 blue: dominant must be red, NOT the last bucket color.
        let mut pixels = vec![];
        pixels.extend(std::iter::repeat_n([255u8, 0, 0, 255], 90).flatten());
        pixels.extend(std::iter::repeat_n([0u8, 0, 255, 255], 10).flatten());
        let color = get_color_sync(pixels.into(), 10, 10, Some(1)).unwrap();
        assert!(
            color[0] > 200,
            "dominant color should be red, got {color:?}"
        );
    }

    #[test]
    fn sync_color_rejects_empty_buffer() {
        let err = get_color_sync(Vec::<u8>::new().into(), 0, 0, Some(1)).unwrap_err();
        assert_eq!(err.status, Status::InvalidArg);
    }
}
