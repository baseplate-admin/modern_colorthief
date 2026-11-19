//! Core types and traits shared by CPU and GPU backends.

pub mod bindings;

/// A single RGB color in the 0-255 range.
pub type Rgb = (u8, u8, u8);

/// Error type for palette extraction across all backends and bindings.
///
/// `Display` output is the stable contract — bindings surface these strings
/// verbatim to the host language.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColorthiefError {
    /// The input pixel buffer was empty.
    EmptyBuffer,
    /// The buffer is too short to contain even a single pixel.
    NoPixels,
    /// The buffer length does not match `width * height * 4`.
    InvalidDimensions,
    /// A backend-specific failure (decode, Vulkan, etc.).
    Backend(String),
}

impl std::fmt::Display for ColorthiefError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ColorthiefError::EmptyBuffer => write!(f, "Empty pixel buffer"),
            ColorthiefError::NoPixels => write!(f, "No pixels to process"),
            ColorthiefError::InvalidDimensions => write!(
                f,
                "Buffer length does not match declared dimensions (expected at least width * height * 4 bytes)"
            ),
            ColorthiefError::Backend(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for ColorthiefError {}

/// Validate a pixel buffer against its declared dimensions.
///
/// This is the single validation point for all backends (CPU, Vulkan GPU,
/// WebGPU): the buffer must be non-empty, at least one pixel, and long
/// enough to hold `width * height` RGBA pixels (trailing padding is
/// allowed, truncation is not). Declared dimensions of 0 mean "unknown"
/// and skip the length check.
pub fn validate_buffer(buffer: &[u8], width: u32, height: u32) -> Result<(), ColorthiefError> {
    if buffer.is_empty() {
        return Err(ColorthiefError::EmptyBuffer);
    }
    if buffer.len() < 4 {
        return Err(ColorthiefError::NoPixels);
    }
    let required = (width as usize)
        .saturating_mul(height as usize)
        .saturating_mul(4);
    if required > 0 && buffer.len() < required {
        return Err(ColorthiefError::InvalidDimensions);
    }
    Ok(())
}

/// Info about a discovered GPU device.
#[derive(Clone, Debug)]
pub struct GpuInfo {
    pub index: usize,
    pub name: String,
    pub device_type: GpuDevice,
    pub vendor_name: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GpuDevice {
    Discrete,
    Integrated,
    Virtual,
    CPU,
    Other,
}

/// Trait for palette extraction backends (CPU, GPU, WebGPU, etc.).
pub trait PaletteExtractor: Send + Sync {
    /// Extract a deduplicated color palette from raw RGBA pixel data.
    fn extract_palette(
        &self,
        buffer: &[u8],
        width: u32,
        height: u32,
        color_count: u8,
        quality: u8,
    ) -> Result<Vec<Rgb>, ColorthiefError>;

    /// Extract the dominant color from raw RGBA pixel data.
    ///
    /// The default implementation extracts a 5-color palette and returns the
    /// first (dominant) entry. This is the single canonical rule for all
    /// backends — per-binding copies of this logic (e.g. `.pop()`-based
    /// "last color" variants) are bugs and must not be reintroduced.
    fn extract_color(
        &self,
        buffer: &[u8],
        width: u32,
        height: u32,
        quality: u8,
    ) -> Result<Rgb, ColorthiefError> {
        let palette = self.extract_palette(buffer, width, height, 5, quality)?;
        palette
            .into_iter()
            .next()
            .ok_or_else(|| ColorthiefError::Backend("No colors found".to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockExtractor {
        palette: Vec<Rgb>,
        err: Option<ColorthiefError>,
    }

    impl PaletteExtractor for MockExtractor {
        fn extract_palette(
            &self,
            _buffer: &[u8],
            _width: u32,
            _height: u32,
            _color_count: u8,
            _quality: u8,
        ) -> Result<Vec<Rgb>, ColorthiefError> {
            match &self.err {
                Some(e) => Err(e.clone()),
                None => Ok(self.palette.clone()),
            }
        }
    }

    #[test]
    fn error_display_empty_buffer() {
        assert_eq!(
            ColorthiefError::EmptyBuffer.to_string(),
            "Empty pixel buffer"
        );
    }

    #[test]
    fn error_display_no_pixels() {
        assert_eq!(
            ColorthiefError::NoPixels.to_string(),
            "No pixels to process"
        );
    }

    #[test]
    fn error_display_invalid_dimensions() {
        let s = ColorthiefError::InvalidDimensions.to_string();
        assert!(s.contains("width * height * 4"));
    }

    #[test]
    fn error_display_backend() {
        assert_eq!(
            ColorthiefError::Backend("Vulkan not found".to_string()).to_string(),
            "Vulkan not found"
        );
    }

    #[test]
    fn error_is_std_error() {
        let e: Box<dyn std::error::Error> = Box::new(ColorthiefError::EmptyBuffer);
        assert_eq!(e.to_string(), "Empty pixel buffer");
    }

    #[test]
    fn validate_buffer_rules() {
        assert_eq!(
            validate_buffer(&[], 10, 10).unwrap_err(),
            ColorthiefError::EmptyBuffer
        );
        assert_eq!(
            validate_buffer(&[1, 2, 3], 0, 0).unwrap_err(),
            ColorthiefError::NoPixels
        );
        assert_eq!(
            validate_buffer(&[1, 2, 3, 4], 10, 10).unwrap_err(),
            ColorthiefError::InvalidDimensions
        );
        // exact fit
        assert!(validate_buffer(&[1, 2, 3, 4], 1, 1).is_ok());
        // trailing padding allowed
        assert!(validate_buffer(&[0u8; 32], 2, 2).is_ok());
        // unknown dimensions skip the length check
        assert!(validate_buffer(&[1, 2, 3, 4], 0, 0).is_ok());
    }

    #[test]
    fn extract_color_returns_first_entry() {
        let e = MockExtractor {
            palette: vec![(255, 0, 0), (0, 255, 0)],
            err: None,
        };
        assert_eq!(e.extract_color(&[0; 8], 2, 1, 1).unwrap(), (255, 0, 0));
    }

    #[test]
    fn extract_color_empty_palette_errors() {
        let e = MockExtractor {
            palette: vec![],
            err: None,
        };
        let err = e.extract_color(&[0; 8], 1, 1, 1).unwrap_err();
        assert_eq!(err, ColorthiefError::Backend("No colors found".to_string()));
    }

    #[test]
    fn extract_color_propagates_backend_error() {
        let e = MockExtractor {
            palette: vec![],
            err: Some(ColorthiefError::EmptyBuffer),
        };
        assert_eq!(
            e.extract_color(&[], 0, 0, 1).unwrap_err(),
            ColorthiefError::EmptyBuffer
        );
    }
}
