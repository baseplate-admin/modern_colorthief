use color_thief::ColorFormat;
use colors_transform::Rgb;
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

/// Formats the sum of two numbers as string.
#[pyfunction]
fn _get_palette(location: String) -> PyResult<Vec<String>> {
    let img = image::open(&std::path::Path::new(&location)).unwrap();
    let (buffer, color_type) = get_image_buffer(img);
    let colors = color_thief::get_palette(&buffer, color_type, 10, 10).unwrap();

    let mut color_vec: Vec<String> = vec![];

    for color in colors {
        let rgb = Rgb::from(color.r as f32, color.g as f32, color.b as f32);
        let hex = rgb.to_css_hex_string().to_owned();
        color_vec.push(hex);
    }
    Ok(color_vec)
}

/// A Python module implemented in Rust.
#[pymodule]
fn modern_colorthief(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(_get_palette, m)?)?;
    Ok(())
}
