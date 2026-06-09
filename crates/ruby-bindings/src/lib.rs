use magnus::{Error, rfn, prelude::*};

fn get_palette(ruby: &magnus::Ruby, pixels: &[u8], width: u32, height: u32, color_count: u8, quality: u8) -> Result<Vec<Vec<u8>>, Error> {
    modern_colorthief_core_cpu::extract_palette_from_buffer(pixels, width, height, color_count, quality)
        .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
        .map_err(|e| Error::new(ruby.exception_runtime_error(), e))
}

fn get_color(ruby: &magnus::Ruby, pixels: &[u8], width: u32, height: u32, quality: u8) -> Result<Vec<u8>, Error> {
    let palette = modern_colorthief_core_cpu::extract_palette_from_buffer(pixels, width, height, 5, quality)
        .map_err(|e| Error::new(ruby.exception_runtime_error(), e))?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| Error::new(ruby.exception_runtime_error(), "Image contains no colors"))
}

#[magnus::init]
fn init_colorthief_ruby(ruby: &magnus::Ruby) -> Result<(), Error> {
    let mod_colorthief = ruby.define_module("Colorthief")?;
    mod_colorthief.define_singleton_method("get_palette", rfn!(get_palette, 5))?;
    mod_colorthief.define_singleton_method("get_color", rfn!(get_color, 4))?;
    Ok(())
}
