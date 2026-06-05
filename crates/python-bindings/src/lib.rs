use colorthief_core::extract_palette_from_buffer;
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;

/// Extract a color palette from raw RGBA pixel bytes.
///
/// Args:
///     pixels (bytes): Raw RGBA pixel data (4 bytes per pixel).
///     width (int): Image width in pixels.
///     height (int): Image height in pixels.
///     color_count (int, optional): Number of colors to extract. Defaults to 10.
///     quality (int, optional): Downsample factor. Higher is faster but less accurate. Defaults to 10.
///
/// Returns:
///     list[tuple[int, int, int]]: Deduplicated list of RGB color tuples.
///
/// Raises:
///     ValueError: If the pixel data is invalid or the algorithm fails.
#[pyfunction]
#[pyo3(signature = (pixels, width, height, color_count=None, quality=None))]
fn _get_palette_given_pixels(
    py: Python<'_>,
    pixels: &[u8],
    width: u32,
    height: u32,
    color_count: Option<u8>,
    quality: Option<u8>,
) -> PyResult<Vec<(u8, u8, u8)>> {
    py.detach(move || {
        extract_palette_from_buffer(
            pixels,
            width,
            height,
            color_count.unwrap_or(10),
            quality.unwrap_or(10),
        )
    })
    .map_err(PyValueError::new_err)
}

/// Extract the dominant color from raw RGBA pixel bytes.
///
/// Args:
///     pixels (bytes): Raw RGBA pixel data (4 bytes per pixel).
///     width (int): Image width in pixels.
///     height (int): Image height in pixels.
///     quality (int, optional): Downsample factor. Higher is faster but less accurate. Defaults to 10.
///
/// Returns:
///     tuple[int, int, int]: The dominant RGB color.
///
/// Raises:
///     ValueError: If the pixel data is invalid, the algorithm fails, or the image contains no colors.
#[pyfunction]
#[pyo3(signature = (pixels, width, height, quality=None))]
fn _get_color_given_pixels(
    py: Python<'_>,
    pixels: &[u8],
    width: u32,
    height: u32,
    quality: Option<u8>,
) -> PyResult<(u8, u8, u8)> {
    let palette = py
        .detach(move || {
            extract_palette_from_buffer(pixels, width, height, 5, quality.unwrap_or(10))
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
    m.add_function(wrap_pyfunction!(_get_palette_given_pixels, m)?)?;
    m.add_function(wrap_pyfunction!(_get_color_given_pixels, m)?)?;
    Ok(())
}
