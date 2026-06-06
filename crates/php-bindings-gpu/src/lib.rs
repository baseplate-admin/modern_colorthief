use ext_php_rs::prelude::*;

#[allow(clippy::missing_safety_doc)]
#[php_function]
fn get_palette(pixels: &[u8], width: u32, height: u32, color_count: Option<u8>, quality: Option<u8>) -> Result<Vec<Vec<u8>>, Exception> {
    let color_count = color_count.unwrap_or(10);
    let quality = quality.unwrap_or(10);

    modern_colorthief_core_gpu::extract_palette_from_buffer(pixels, width, height, color_count, quality)
        .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
        .map_err(|e| Exception::new_fn("Exception", e))
}

#[allow(clippy::missing_safety_doc)]
#[php_function]
fn get_color(pixels: &[u8], width: u32, height: u32, quality: Option<u8>) -> Result<Vec<u8>, Exception> {
    let quality = quality.unwrap_or(10);

    let palette = modern_colorthief_core_gpu::extract_palette_from_buffer(pixels, width, height, 5, quality)
        .map_err(|e| Exception::new_fn("Exception", e))?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| Exception::new_fn("Exception", "Image contains no colors"))
}

#[php_module]
fn php_module() -> Module {
    let module = Module::new("modern_colorthief_gpu");
    module.add_function(get_palette());
    module.add_function(get_color());
    module
}
