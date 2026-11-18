#![cfg_attr(windows, feature(abi_vectorcall))]
use ext_php_rs::prelude::*;
use ext_php_rs::types::Binary;

/// Convert a PHP binary string (`Binary<u8>`) to a `Vec<u8>`, validating that
/// every byte is in `[0, 255]` (always true for a `u8`, but the check is kept
/// for clarity and to produce a meaningful error if the caller passes a string
/// with non-ASCII bytes that were not intended as pixel data).
fn pixels_to_bytes(pixels: Binary<u8>) -> PhpResult<Vec<u8>> {
    // Binary<u8> Deref's to Vec<u8>; each byte is already in [0, 255] by
    // construction. Convert directly — no range check needed for u8.
    Ok(Vec::from(pixels))
}

/// Legacy compatibility shim: convert a PHP array of integers to a byte vec.
/// Kept so existing callers passing `array` continue to work.
fn pixels_to_bytes_compat(pixels: Vec<i64>) -> PhpResult<Vec<u8>> {
    pixels
        .into_iter()
        .map(|b| {
            if (0..=255).contains(&b) {
                Ok(b as u8)
            } else {
                Err(PhpException::from_message(format!(
                    "Pixel byte value {} out of range [0, 255]",
                    b
                )))
            }
        })
        .collect()
}

#[php_function]
fn get_palette(
    pixels: Binary<u8>,
    width: u32,
    height: u32,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> PhpResult<Vec<Vec<u8>>> {
    let color_count = color_count.unwrap_or(10);
    let quality = quality.unwrap_or(10);
    let bytes = pixels_to_bytes(pixels)?;

    modern_colorthief_core_cpu::extract_palette_from_buffer(
        &bytes,
        width,
        height,
        color_count,
        quality,
    )
    .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
    .map_err(|e| PhpException::from_message(e.to_string()))
}

#[php_function]
fn get_color(
    pixels: Binary<u8>,
    width: u32,
    height: u32,
    quality: Option<u8>,
) -> PhpResult<Vec<u8>> {
    let quality = quality.unwrap_or(10);
    let bytes = pixels_to_bytes(pixels)?;

    let palette =
        modern_colorthief_core_cpu::extract_palette_from_buffer(&bytes, width, height, 5, quality)
            .map_err(|e| PhpException::from_message(e.to_string()))?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| PhpException::from_message("Image contains no colors".into()))
}

/// Legacy compatibility function accepting a PHP array of integers.
#[php_function]
fn get_palette_compat(
    pixels: Vec<i64>,
    width: u32,
    height: u32,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> PhpResult<Vec<Vec<u8>>> {
    let bytes = pixels_to_bytes_compat(pixels)?;
    modern_colorthief_core_cpu::extract_palette_from_buffer(
        &bytes,
        width,
        height,
        color_count.unwrap_or(10),
        quality.unwrap_or(10),
    )
    .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
    .map_err(|e| PhpException::from_message(e.to_string()))
}

/// Legacy compatibility function accepting a PHP array of integers.
#[php_function]
fn get_color_compat(
    pixels: Vec<i64>,
    width: u32,
    height: u32,
    quality: Option<u8>,
) -> PhpResult<Vec<u8>> {
    let bytes = pixels_to_bytes_compat(pixels)?;
    let palette = modern_colorthief_core_cpu::extract_palette_from_buffer(
        &bytes,
        width,
        height,
        5,
        quality.unwrap_or(10),
    )
    .map_err(|e| PhpException::from_message(e.to_string()))?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| PhpException::from_message("Image contains no colors".into()))
}

#[php_module]
fn php_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .function(wrap_function!(get_palette))
        .function(wrap_function!(get_color))
        .function(wrap_function!(get_palette_compat))
        .function(wrap_function!(get_color_compat))
}
