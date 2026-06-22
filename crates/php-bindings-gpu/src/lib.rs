#[cfg(windows)]
#![feature(abi_vectorcall)]
use ext_php_rs::prelude::*;

fn pixels_to_bytes(pixels: Vec<i64>) -> PhpResult<Vec<u8>> {
    pixels
        .into_iter()
        .map(|b| {
            if (0..=255).contains(&b) {
                Ok(b as u8)
            } else {
                Err(PhpException::default(format!(
                    "Pixel byte value {} out of range [0, 255]",
                    b
                )))
            }
        })
        .collect()
}

#[php_function]
fn get_palette(
    pixels: Vec<i64>,
    width: u32,
    height: u32,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> PhpResult<Vec<Vec<u8>>> {
    let color_count = color_count.unwrap_or(10);
    let quality = quality.unwrap_or(10);
    let bytes = pixels_to_bytes(pixels)?;

    modern_colorthief_core_gpu::extract_palette_from_buffer(
        &bytes,
        width,
        height,
        color_count,
        quality,
    )
    .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
    .map_err(|e| PhpException::default(e.to_string()))
}

#[php_function]
fn get_color(pixels: Vec<i64>, width: u32, height: u32, quality: Option<u8>) -> PhpResult<Vec<u8>> {
    let quality = quality.unwrap_or(10);
    let bytes = pixels_to_bytes(pixels)?;

    let palette =
        modern_colorthief_core_gpu::extract_palette_from_buffer(&bytes, width, height, 5, quality)
            .map_err(|e| PhpException::default(e.to_string()))?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| PhpException::default("No color extracted".into()))
}

#[php_module]
fn php_module(module: ModuleBuilder) -> ModuleBuilder {
    module
        .function(wrap_function!(get_palette))
        .function(wrap_function!(get_color))
}
