use colorthief_core::extract_palette_from_buffer;
use image::DynamicImage;
use image::GenericImageView;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use rayon::iter::{IntoParallelIterator, ParallelIterator};

/// Convert a DynamicImage to a flat RGBA byte buffer using parallel pixel iteration.
fn to_rgba_buffer(img: &DynamicImage) -> Vec<u8> {
    let (width, height) = img.dimensions();
    let count = (width * height) as usize;
    let rgba = img.to_rgba8();

    (0..count)
        .into_par_iter()
        .flat_map(|i| {
            let x = (i % width as usize) as u32;
            let y = (i / width as usize) as u32;
            let p = &rgba[(x, y)];
            [p[0], p[1], p[2], p[3]]
        })
        .collect()
}

/// Extract a deduplicated color palette from a DynamicImage.
fn extract_palette(
    img: DynamicImage,
    color_count: u8,
    quality: u8,
) -> Result<Vec<(u8, u8, u8)>, String> {
    let buffer = to_rgba_buffer(&img);
    let (width, height) = img.dimensions();
    extract_palette_from_buffer(&buffer, width, height, color_count, quality)
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
    py: Python<'_>,
    image: &[u8],
    color_count: Option<u8>,
    quality: Option<u8>,
) -> PyResult<Vec<(u8, u8, u8)>> {
    py.detach(move || {
        let img = image::load_from_memory(image)
            .map_err(|e| format!("Failed to load image from memory: {e}"))?;
        extract_palette(img, color_count.unwrap_or(10), quality.unwrap_or(10))
    })
    .map_err(PyValueError::new_err)
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
    py: Python<'_>,
    image: &str,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> PyResult<Vec<(u8, u8, u8)>> {
    let path = image.to_string();
    py.detach(move || {
        let img = image::open(&path)
            .map_err(|e| format!("Failed to open image at {path}: {e}"))?;
        extract_palette(img, color_count.unwrap_or(10), quality.unwrap_or(10))
    })
    .map_err(PyValueError::new_err)
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
fn _get_color_given_location(
    py: Python<'_>,
    image: &str,
    quality: Option<u8>,
) -> PyResult<(u8, u8, u8)> {
    let path = image.to_string();
    let palette = py
        .detach(move || {
            let img = image::open(&path)
                .map_err(|e| format!("Failed to open image at {path}: {e}"))?;
            extract_palette(img, 5, quality.unwrap_or(10))
        })
        .map_err(PyValueError::new_err)?;

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
fn _get_color_given_bytes(
    py: Python<'_>,
    image: &[u8],
    quality: Option<u8>,
) -> PyResult<(u8, u8, u8)> {
    let palette = py
        .detach(move || {
            let img =
                image::load_from_memory(image).map_err(|e| format!("Failed to load image: {e}"))?;
            extract_palette(img, 5, quality.unwrap_or(10))
        })
        .map_err(PyValueError::new_err)?;

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
