use magnus::{function, prelude::*, module, Error, Rational};

/// Extract a palette of dominant colors from raw RGBA pixel data.
///
/// @param pixels [String] Raw RGBA pixel buffer (4 bytes per pixel)
/// @param width [Integer] Image width in pixels
/// @param height [Integer] Image height in pixels
/// @param color_count [Integer] Number of colors to extract (default: 10)
/// @param quality [Integer] Sampling quality (default: 10)
/// @return [Array] Array of [R, G, B] color arrays
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

    colorthief_core::extract_palette_from_buffer(pixels, width, height, color_count, quality)
        .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
        .map_err(|e| Error::new(magnus::exception::runtime_error(), e))
}

/// Extract the dominant color from raw RGBA pixel data.
///
/// @param pixels [String] Raw RGBA pixel buffer (4 bytes per pixel)
/// @param width [Integer] Image width in pixels
/// @param height [Integer] Image height in pixels
/// @param quality [Integer] Sampling quality (default: 10)
/// @return [Array] [R, G, B] color array
#[magnus::wrap]
pub fn get_color(
    pixels: &[u8],
    width: u32,
    height: u32,
    quality: Option<u8>,
) -> Result<Vec<u8>, Error> {
    let quality = quality.unwrap_or(10);

    let palette = colorthief_core::extract_palette_from_buffer(pixels, width, height, 5, quality)
        .map_err(|e| Error::new(magnus::exception::runtime_error(), e))?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| Error::new(magnus::exception::runtime_error(), "Image contains no colors"))
}

#[magnus::init]
fn init_colorthief_ruby() -> Result<(), Error> {
    let mod_colorthief = module("Colorthief")?;
    mod_colorthief.define_singleton_method("get_palette", function!(get_palette, 4))?;
    mod_colorthief.define_singleton_method("get_color", function!(get_color, 3))?;
    Ok(())
}
