use magnus::{function, prelude::*, module, Error};

#[magnus::wrap]
pub fn get_palette(
    pixels: &[u8],
    width: u32,
    height: u32,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> Result<Vec<Vec<u8>>, Error> {
    let color_count = color_count.unwrap_or(10);
    let quality = quality.unwrap_or(10);

    modern_colorthief_core_gpu::extract_palette_from_buffer(pixels, width, height, color_count, quality)
        .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
        .map_err(|e| Error::new(magnus::exception::runtime_error(), e))
}

#[magnus::wrap]
pub fn get_color(
    pixels: &[u8],
    width: u32,
    height: u32,
    quality: Option<u8>,
) -> Result<Vec<u8>, Error> {
    let quality = quality.unwrap_or(10);

    let palette = modern_colorthief_core_gpu::extract_palette_from_buffer(pixels, width, height, 5, quality)
        .map_err(|e| Error::new(magnus::exception::runtime_error(), e))?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| Error::new(magnus::exception::runtime_error(), "Image contains no colors"))
}

#[magnus::init]
fn init_colorthief_gpu_ruby() -> Result<(), Error> {
    let mod_colorthief_gpu = module("ColorthiefGpu")?;
    mod_colorthief_gpu.define_singleton_method("get_palette", function!(get_palette, 4))?;
    mod_colorthief_gpu.define_singleton_method("get_color", function!(get_color, 3))?;
    Ok(())
}
