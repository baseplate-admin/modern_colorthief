use magnus::{Error, RString, prelude::*};

fn get_palette(pixels: RString, width: u32, height: u32, color_count: u8, quality: u8) -> Result<Vec<Vec<u8>>, Error> {
    let pixels = pixels.as_slice();
    modern_colorthief_core_gpu::extract_palette_from_buffer(pixels, width, height, color_count, quality)
        .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
        .map_err(|e| Error::new(magnus::exception::error(), e))
}

fn get_color(pixels: RString, width: u32, height: u32, quality: u8) -> Result<Vec<u8>, Error> {
    let pixels = pixels.as_slice();
    let palette = modern_colorthief_core_gpu::extract_palette_from_buffer(pixels, width, height, 5, quality)
        .map_err(|e| Error::new(magnus::exception::error(), e))?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| Error::new(magnus::exception::error(), "Image contains no colors"))
}

#[magnus::init]
fn init_colorthief_gpu_ruby() -> Result<(), Error> {
    let mod_colorthief_gpu = magnus::ruby().define_module("ColorthiefGpu")?;
    mod_colorthief_gpu.define_singleton_method("get_palette", function!(get_palette, 5))?;
    mod_colorthief_gpu.define_singleton_method("get_color", function!(get_color, 4))?;
    Ok(())
}
