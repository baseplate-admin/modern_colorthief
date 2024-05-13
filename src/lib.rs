use color_thief::ColorFormat;
use pyo3::prelude::*;

// get_color
// get_palette

fn get_image_buffer(img: image::DynamicImage) -> (Vec<u8>, ColorFormat) {
    match img {
        image::DynamicImage::ImageRgb8(buffer) => (buffer.to_vec(), color_thief::ColorFormat::Rgb),
        image::DynamicImage::ImageRgba8(buffer) => {
            (buffer.to_vec(), color_thief::ColorFormat::Rgba)
        }
        _ => unreachable!(),
    }
}
#[pyfunction]
fn get_color(location: String, quality: Option<u8>) -> PyResult<(u8, u8, u8)> {
    let img = image::open(&std::path::Path::new(&location)).unwrap();
    let (buffer, color_type) = get_image_buffer(img);

    let colors = color_thief::get_palette(&buffer, color_type, quality.unwrap_or(10), 10).unwrap();

    Ok((colors[0].r, colors[0].g, colors[0].b))
}

/// Formats the sum of two numbers as string.
#[pyfunction]
fn get_palette(location: String) -> PyResult<Vec<(u8, u8, u8)>> {
    let img = image::open(&std::path::Path::new(&location)).unwrap();
    let (buffer, color_type) = get_image_buffer(img);

    let colors = color_thief::get_palette(&buffer, color_type, 10, 10).unwrap();

    let color_vec = colors
        .iter()
        .map(|color| (color.r, color.g, color.b))
        .collect();

    Ok(color_vec)
}

/// A Python module implemented in Rust.
#[pymodule]
fn modern_colorthief(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(get_palette, m)?)?;
    m.add_function(wrap_pyfunction!(get_color, m)?)?;
    Ok(())
}
