use magnus::{error, function, Object, RString, Ruby};

fn get_palette(
    pixels: RString,
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> Result<Vec<Vec<u8>>, error::Error> {
    let pixels = unsafe { pixels.as_slice() };
    modern_colorthief_core_cpu::extract_palette_from_buffer(
        pixels,
        width,
        height,
        color_count,
        quality,
    )
    .map(|colors| colors.into_iter().map(|(r, g, b)| vec![r, g, b]).collect())
    .map_err(|e| error::Error::new(Ruby::exception_runtime_error(), e.to_string()))
}

fn get_color(
    pixels: RString,
    width: u32,
    height: u32,
    quality: u8,
) -> Result<Vec<u8>, error::Error> {
    let pixels = unsafe { pixels.as_slice() };
    let palette =
        modern_colorthief_core_cpu::extract_palette_from_buffer(pixels, width, height, 5, quality)
            .map_err(|e| error::Error::new(Ruby::exception_runtime_error(), e.to_string()))?;

    palette
        .first()
        .copied()
        .map(|(r, g, b)| vec![r, g, b])
        .ok_or_else(|| {
            error::Error::new(Ruby::exception_runtime_error(), "Image contains no colors")
        })
}

#[magnus::init]
fn init_colorthief_ruby() {
    let mod_colorthief = Ruby::define_module("Colorthief").unwrap();
    mod_colorthief
        .define_singleton_method("get_palette", function!(get_palette, 5))
        .unwrap();
    mod_colorthief
        .define_singleton_method("get_color", function!(get_color, 4))
        .unwrap();
}
