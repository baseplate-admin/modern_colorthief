use color_thief::ColorFormat;
use image::DynamicImage;
use itertools::Itertools;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Convert a DynamicImage to a flat byte buffer and its corresponding color format.
fn get_image_buffer(img: DynamicImage) -> (Vec<u8>, ColorFormat) {
    if img.color().has_alpha() {
        (img.to_rgba8().into_raw(), ColorFormat::Rgba)
    } else {
        (img.to_rgb8().into_raw(), ColorFormat::Rgb)
    }
}

/// Extract a deduplicated color palette from an already-loaded image.
fn extract_palette(
    image: DynamicImage,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> Result<Vec<(u8, u8, u8)>, String> {
    let (buffer, color_format) = get_image_buffer(image);

    let colors = color_thief::get_palette(
        &buffer,
        color_format,
        quality.unwrap_or(10),
        color_count.unwrap_or(10),
    )
    .map_err(|e| format!("color_thief failed: {e}"))?;

    Ok(colors.iter().map(|c| (c.r, c.g, c.b)).unique().collect())
}

/// Extract a color palette from raw image bytes.
///
/// Args:
///     image (bytes): Raw image data in a supported format.
///     color_count (int, optional): Number of colors to extract. Defaults to 10.
///     quality (int, optional): Downsample factor. Higher is faster but less accurate. Defaults to 10.
///
/// Returns:
///     list[tuple[int, int, int]]: Deduplicated list of RGB color tuples.
///
/// Raises:
///     ValueError: If the image data is invalid or the algorithm fails.
#[pyfunction]
#[pyo3(signature = (image, color_count=None, quality=None))]
fn _get_palette_given_bytes(
    image: &[u8],
    color_count: Option<u8>,
    quality: Option<u8>,
) -> PyResult<Vec<(u8, u8, u8)>> {
    let img = image::load_from_memory(image)
        .map_err(|e| PyValueError::new_err(format!("Failed to load image from memory: {e}")))?;
    extract_palette(img, color_count, quality).map_err(PyValueError::new_err)
}

/// Extract a color palette from an image file.
///
/// Args:
///     image (str): Path to the image file.
///     color_count (int, optional): Number of colors to extract. Defaults to 10.
///     quality (int, optional): Downsample factor. Higher is faster but less accurate. Defaults to 10.
///
/// Returns:
///     list[tuple[int, int, int]]: Deduplicated list of RGB color tuples.
///
/// Raises:
///     ValueError: If the file cannot be opened or the algorithm fails.
#[pyfunction]
#[pyo3(signature = (image, color_count=None, quality=None))]
fn _get_palette_given_location(
    image: &str,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> PyResult<Vec<(u8, u8, u8)>> {
    let img = image::open(image)
        .map_err(|e| PyValueError::new_err(format!("Failed to open image at {image}: {e}")))?;
    extract_palette(img, color_count, quality).map_err(PyValueError::new_err)
}

/// Extract the dominant color from an image file.
///
/// Args:
///     image (str): Path to the image file.
///     quality (int, optional): Downsample factor. Higher is faster but less accurate. Defaults to 10.
///
/// Returns:
///     tuple[int, int, int]: The dominant RGB color.
///
/// Raises:
///     ValueError: If the file cannot be opened, the algorithm fails, or the image contains no colors.
#[pyfunction]
#[pyo3(signature = (image, quality=None))]
fn _get_color_given_location(image: &str, quality: Option<u8>) -> PyResult<(u8, u8, u8)> {
    let img = image::open(image)
        .map_err(|e| PyValueError::new_err(format!("Failed to open image at {image}: {e}")))?;
    let palette = extract_palette(img, Some(5), quality).map_err(PyValueError::new_err)?;
    palette
        .first()
        .copied()
        .ok_or_else(|| PyValueError::new_err("Image contains no colors"))
}

/// Extract the dominant color from raw image bytes.
///
/// Args:
///     image (bytes): Raw image data in a supported format.
///     quality (int, optional): Downsample factor. Higher is faster but less accurate. Defaults to 10.
///
/// Returns:
///     tuple[int, int, int]: The dominant RGB color.
///
/// Raises:
///     ValueError: If the image data is invalid, the algorithm fails, or the image contains no colors.
#[pyfunction]
#[pyo3(signature = (image, quality=None))]
fn _get_color_given_bytes(image: &[u8], quality: Option<u8>) -> PyResult<(u8, u8, u8)> {
    let img = image::load_from_memory(image)
        .map_err(|e| PyValueError::new_err(format!("Failed to load image from memory: {e}")))?;
    let palette = extract_palette(img, Some(5), quality).map_err(PyValueError::new_err)?;
    palette
        .first()
        .copied()
        .ok_or_else(|| PyValueError::new_err("Image contains no colors"))
}

/// A Python module implemented in Rust.
#[pymodule(gil_used = false)]
fn _modern_colorthief(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    m.add_function(wrap_pyfunction!(_get_palette_given_location, m)?)?;
    m.add_function(wrap_pyfunction!(_get_palette_given_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(_get_color_given_bytes, m)?)?;
    m.add_function(wrap_pyfunction!(_get_color_given_location, m)?)?;
    Ok(())
}
