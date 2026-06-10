use std::path::PathBuf;

use modern_colorthief_core_gpu::{extract_palette_from_buffer, list_gpus};
use image::ImageReader;
use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (buffer, width, height, color_count=5, quality=1))]
fn extract_palette_from_buffer_py(
    py: Python,
    buffer: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> PyResult<Vec<(u8, u8, u8)>> {
    py.detach(move || {
        extract_palette_from_buffer(buffer, width, height, color_count, quality)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))
    })
}

#[pyfunction]
#[pyo3(signature = (buffer, width, height, quality=1))]
fn extract_dominant_color_from_buffer_py(
    py: Python,
    buffer: &[u8],
    width: u32,
    height: u32,
    quality: u8,
) -> PyResult<(u8, u8, u8)> {
    py.detach(move || {
        extract_palette_from_buffer(buffer, width, height, 1, quality)
            .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))?
            .into_iter()
            .next()
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("No color extracted"))
    })
}

fn load_image_info(path: &str) -> PyResult<(Vec<u8>, u32, u32)> {
    let img = ImageReader::open(PathBuf::from(path))
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Failed to open image: {e}")))?
        .decode()
        .map_err(|e| pyo3::exceptions::PyValueError::new_err(format!("Failed to decode image: {e}")))?;
    let (w, h) = (img.width(), img.height());
    Ok((img.into_rgba8().into_raw(), w, h))
}

#[pyfunction]
#[pyo3(signature = (path, color_count=5, quality=1))]
fn extract_palette_py(
    py: Python,
    path: String,
    color_count: u8,
    quality: u8,
) -> PyResult<Vec<(u8, u8, u8)>> {
    let (buffer, width, height) = py.detach(move || -> PyResult<(Vec<u8>, u32, u32)> {
        load_image_info(&path)
    })?;
    extract_palette_from_buffer_py(py, &buffer, width, height, color_count, quality)
}

#[pyfunction]
#[pyo3(signature = (path, quality=1))]
fn extract_dominant_color_py(
    py: Python,
    path: String,
    quality: u8,
) -> PyResult<(u8, u8, u8)> {
    let (buffer, width, height) = py.detach(move || -> PyResult<(Vec<u8>, u32, u32)> {
        load_image_info(&path)
    })?;
    extract_dominant_color_from_buffer_py(py, &buffer, width, height, quality)
}

#[pyfunction]
fn list_gpus_py(py: Python) -> PyResult<Vec<pyo3::Bound<pyo3::types::PyDict>>> {
    use pyo3::types::PyDict;
    let gpus = list_gpus().map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(e))?;
    let mut result = Vec::new();
    for gpu in gpus {
        let d = PyDict::new(py);
        d.set_item("index", gpu.index)?;
        d.set_item("name", gpu.name)?;
        d.set_item("device_type", format!("{:?}", gpu.device_type))?;
        d.set_item("vendor_name", gpu.vendor_name)?;
        result.push(d);
    }
    Ok(result)
}

#[pymodule]
fn gpu(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction![extract_palette_from_buffer_py](m)?)?;
    m.add_function(wrap_pyfunction![extract_dominant_color_from_buffer_py](m)?)?;
    m.add_function(wrap_pyfunction![extract_palette_py](m)?)?;
    m.add_function(wrap_pyfunction![extract_dominant_color_py](m)?)?;
    m.add_function(wrap_pyfunction![list_gpus_py](m)?)?;
    Ok(())
}
