use ext_php_rs::prelude::*;

#[php_function]
fn get_palette(
    pixels: String,
    width: u32,
    height: u32,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> PhpResult<Vec<Vec<u8>>> {
    let pixels = pixels.into_bytes();
    let color_count = color_count.unwrap_or(10);
    let quality = quality.unwrap_or(10);

    modern_colorthief_core_gpu::extract_palette_from_buffer(
        &pixels,
        width,
        height,
        color_count,
        quality,
    )
    .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
    .map_err(|e| PhpException::default(e.to_string()))
}

#[php_function]
fn get_color(pixels: String, width: u32, height: u32, quality: Option<u8>) -> PhpResult<Vec<u8>> {
    let pixels = pixels.into_bytes();
    let quality = quality.unwrap_or(10);

    let palette =
        modern_colorthief_core_gpu::extract_palette_from_buffer(&pixels, width, height, 5, quality)
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
