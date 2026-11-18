//! Shared marshalling and color rules for modern_colorthief language bindings.
//!
//! Centralises the two pieces of logic that used to be copied per-binding
//! (and drifted):
//!
//! * converting an `RGB` palette to the flat numeric shapes each host
//!   language wants (JS `number[]`, `Vec<u8>`, `Vec<i64>`, etc.)
//! * the `get_color` rule: dominant color = **first** entry of a 5-color
//!   palette. The `.pop()`-based "last color" variants that shipped in two
//!   WASM surfaces were bugs; this is now the single canonical rule.

use crate::{ColorthiefError, Rgb};

/// The canonical dominant-color rule for all bindings: the first entry of a
/// 5-color palette. Returns an error if the palette is empty.
pub fn dominant_color(palette: &[Rgb]) -> Result<Rgb, ColorthiefError> {
    palette
        .first()
        .copied()
        .ok_or_else(|| ColorthiefError::Backend("Image contains no colors".to_string()))
}

/// Flat `[r0, g0, b0, r1, g1, b1, ...]` as `u8` — JS/TS, Node, WASM shapes.
pub fn colors_to_flat_u8(colors: &[Rgb]) -> Vec<u8> {
    let mut out = Vec::with_capacity(colors.len() * 3);
    for (r, g, b) in colors {
        out.extend_from_slice(&[*r, *g, *b]);
    }
    out
}

/// Flat `[r0, g0, b0, ...]` as `i64` — the PHP binding's current wire shape
/// (kept for compatibility; new code should prefer `colors_to_flat_u8`).
pub fn colors_to_flat_i64(colors: &[Rgb]) -> Vec<i64> {
    let mut out = Vec::with_capacity(colors.len() * 3);
    for (r, g, b) in colors {
        out.extend_from_slice(&[i64::from(*r), i64::from(*g), i64::from(*b)]);
    }
    out
}

/// `[ [r, g, b], [r, g, b], ... ]` — Node's `Vec<Vec<u8>>` shape.
pub fn colors_to_rows_u8(colors: &[Rgb]) -> Vec<Vec<u8>> {
    colors.iter().map(|(r, g, b)| vec![*r, *g, *b]).collect()
}

/// Flat `[r0, g0, b0, ...]` as `u32` — a shape convenient for numeric
/// bindings that pass colors as 32-bit integers.
pub fn colors_to_flat_u32(colors: &[Rgb]) -> Vec<u32> {
    let mut out = Vec::with_capacity(colors.len() * 3);
    for (r, g, b) in colors {
        out.extend_from_slice(&[u32::from(*r), u32::from(*g), u32::from(*b)]);
    }
    out
}

/// Pack a color into a `u32` RGB word (0xRRGGBB).
pub fn color_to_u32(color: Rgb) -> u32 {
    let (r, g, b) = color;
    u32::from(r) << 16 | u32::from(g) << 8 | u32::from(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dominant_color_first_entry() {
        let palette = vec![(255, 0, 0), (0, 255, 0), (0, 0, 255)];
        assert_eq!(dominant_color(&palette).unwrap(), (255, 0, 0));
    }

    #[test]
    fn dominant_color_single_entry() {
        assert_eq!(dominant_color(&[(10, 20, 30)]).unwrap(), (10, 20, 30));
    }

    #[test]
    fn dominant_color_empty_errors() {
        let err = dominant_color(&[]).unwrap_err();
        assert!(matches!(err, ColorthiefError::Backend(_)));
        assert_eq!(err.to_string(), "Image contains no colors");
    }

    #[test]
    fn flat_u8_ordering() {
        let colors = vec![(1, 2, 3), (4, 5, 6)];
        assert_eq!(colors_to_flat_u8(&colors), vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn flat_u8_empty() {
        assert!(colors_to_flat_u8(&[]).is_empty());
    }

    #[test]
    fn flat_i64_ordering() {
        let colors = vec![(255, 0, 128)];
        assert_eq!(colors_to_flat_i64(&colors), vec![255, 0, 128]);
    }

    #[test]
    fn flat_i64_empty() {
        assert!(colors_to_flat_i64(&[]).is_empty());
    }

    #[test]
    fn rows_u8_shape() {
        let colors = vec![(1, 2, 3), (4, 5, 6)];
        assert_eq!(
            colors_to_rows_u8(&colors),
            vec![vec![1, 2, 3], vec![4, 5, 6]]
        );
    }

    #[test]
    fn rows_u8_empty() {
        assert!(colors_to_rows_u8(&[]).is_empty());
    }

    #[test]
    fn flat_u32_ordering() {
        let colors = vec![(255, 1, 2)];
        assert_eq!(colors_to_flat_u32(&colors), vec![255, 1, 2]);
    }

    #[test]
    fn flat_u32_empty() {
        assert!(colors_to_flat_u32(&[]).is_empty());
    }

    #[test]
    fn color_to_u32_packs_rrggbb() {
        assert_eq!(color_to_u32((0xFF, 0x80, 0x40)), 0xFF8040);
        assert_eq!(color_to_u32((0, 0, 0)), 0);
        assert_eq!(color_to_u32((255, 255, 255)), 0xFFFFFF);
    }
}
