use magnus::{Error, exception, function, prelude::*};

fn get_palette(pixels: &[u8], width: u32, height: u32, color_count: u8, quality: u8) -> Result<Vec<Vec<u8>>, Error> {
    modern_colorthief_core_cpu::extract_palette_from_buffer(pixels, width, height, color_count as usize, quality)
        .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
        .map_err(|e| Error::new(exception::runtime_error(), e))
}

fn get_color(pixels: &[u8], width: u32, height: u32, quality: u8) -> Result<Vec<u8>, Error> {
    let palette = modern_colorthief_core_cpu::extract_palette_from_buffer(pixels, width, height, 5, quality)
        .map_err(|e| Error::new(exception::runtime_error(), e))?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| Error::new(exception::runtime_error(), "Image contains no colors"))
}

#[magnus::init]
fn init_colorthief_ruby(ruby: &magnus::Ruby) -> Result<(), Error> {
    let mod_colorthief = ruby.define_module("Colorthief")?;
    mod_colorthief.define_singleton_method("get_palette", function!(get_palette, 5))?;
    mod_colorthief.define_singleton_method("get_color", function!(get_color, 4))?;
    Ok(())
}
