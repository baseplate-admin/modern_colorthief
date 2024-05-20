use color_thief::ColorFormat;
use image::DynamicImage;
use pyo3::prelude::*;

fn get_image_buffer(img: image::DynamicImage) -> (Vec<u8>, ColorFormat) {
    match img {
        image::DynamicImage::ImageRgb8(buffer) => (buffer.to_vec(), color_thief::ColorFormat::Rgb),
        image::DynamicImage::ImageRgba8(buffer) => {
            (buffer.to_vec(), color_thief::ColorFormat::Rgba)
        }
        _ => unreachable!(),
    }
}
/// Returns the pallette given an bytes object

#[pyfunction]
fn _get_palette_given_bytes(
    image: Vec<u8>,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> PyResult<Vec<(u8, u8, u8)>> {
    let mut _image = image;
    let img = image::load_from_memory(&_image).unwrap();

    Ok(get_palette(img, color_count, quality).unwrap())
}

/// Returns the pallette given an image path
#[pyfunction]
fn _get_palette_given_location(
    image: String,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> PyResult<Vec<(u8, u8, u8)>> {
    let img = image::open(&std::path::Path::new(&image)).unwrap();

    Ok(get_palette(img, color_count, quality).unwrap())
}

// Gets the dominant color given an image
#[pyfunction]
fn _get_color_given_location(image: String, quality: Option<u8>) -> PyResult<(u8, u8, u8)> {
    let palette = _get_palette_given_location(image, Some(5), Some(quality.unwrap_or(10))).unwrap();
    Ok(palette[0])
}

#[pyfunction]
fn _get_color_given_bytes(image: Vec<u8>, quality: Option<u8>) -> PyResult<(u8, u8, u8)> {
    let palette =
        _get_palette_given_bytes(image.to_owned(), Some(5), Some(quality.unwrap_or(10))).unwrap();
    Ok(palette[0])
}

fn get_palette(
    image: DynamicImage,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> Result<Vec<(u8, u8, u8)>, String> {
    let (buffer, color_type) = get_image_buffer(image);

    let colors = color_thief::get_palette(
        &buffer,
        color_type,
        quality.unwrap_or(10),
        color_count.unwrap_or(10),
    )
    .unwrap();

    let color_vec = colors
        .iter()
        .map(|color| (color.r, color.g, color.b))
        .collect();

    Ok(color_vec)
}

fn get_version() -> &'static str {
    static VERSION: std::sync::OnceLock<String> = std::sync::OnceLock::new();

    VERSION.get_or_init(|| env!("CARGO_PKG_VERSION").to_owned())
}
/// A Python module implemented in Rust.
#[pymodule]
fn modern_colorthief(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", get_version())?;

    // Palette utils
    m.add_function(wrap_pyfunction!(_get_palette_given_location, m)?)?;
    m.add_function(wrap_pyfunction!(_get_palette_given_bytes, m)?)?;

    // Color utils
    m.add_function(wrap_pyfunction!(_get_color_given_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(_get_color_given_location, m)?)?;

    Ok(())
}
