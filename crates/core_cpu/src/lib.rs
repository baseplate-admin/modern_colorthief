use modern_colorthief_core::PaletteExtractor;
use rayon::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
struct Color {
    r: u8,
    g: u8,
    b: u8,
}

#[derive(Clone, Debug)]
struct Bucket {
    pixels: Vec<Color>,
}

impl Bucket {
    fn avg(&self) -> Color {
        let count = self.pixels.len() as u32;
        let sum: (u32, u32, u32) = self
            .pixels
            .par_iter()
            .map(|p| (p.r as u32, p.g as u32, p.b as u32))
            .reduce(|| (0, 0, 0), |a, b| (a.0 + b.0, a.1 + b.1, a.2 + b.2));
        Color {
            r: (sum.0 / count) as u8,
            g: (sum.1 / count) as u8,
            b: (sum.2 / count) as u8,
        }
    }

    fn volume(&self) -> u64 {
        let min_r = self.pixels.iter().map(|p| p.r).min().unwrap_or(0);
        let max_r = self.pixels.iter().map(|p| p.r).max().unwrap_or(0);
        let min_g = self.pixels.iter().map(|p| p.g).min().unwrap_or(0);
        let max_g = self.pixels.iter().map(|p| p.g).max().unwrap_or(0);
        let min_b = self.pixels.iter().map(|p| p.b).min().unwrap_or(0);
        let max_b = self.pixels.iter().map(|p| p.b).max().unwrap_or(0);
        let spread_r = (max_r - min_r) as u32;
        let spread_g = (max_g - min_g) as u32;
        let spread_b = (max_b - min_b) as u32;
        (spread_r as u64 * spread_g as u64 * spread_b as u64).max(1) * self.pixels.len() as u64
    }
}

fn sample_pixels(buffer: &[u8], quality: u8) -> Vec<Color> {
    let step = quality.max(1) as usize;
    let total_pixels = buffer.len() / 4;

    (0..total_pixels)
        .step_by(step)
        .filter_map(|i| {
            let offset = i * 4;
            if offset + 2 < buffer.len() {
                Some(Color {
                    r: buffer[offset],
                    g: buffer[offset + 1],
                    b: buffer[offset + 2],
                })
            } else {
                None
            }
        })
        .collect()
}

fn split_bucket(bucket: &Bucket, axis: usize) -> (Bucket, Bucket) {
    let mut sorted = bucket.pixels.clone();
    sorted.sort_by(match axis {
        0 => |a: &Color, b: &Color| a.r.cmp(&b.r),
        1 => |a: &Color, b: &Color| a.g.cmp(&b.g),
        2 => |a: &Color, b: &Color| a.b.cmp(&b.b),
        _ => panic!("Invalid axis"),
    });

    let mid = sorted.len() / 2;
    let (left, right) = sorted.split_at(mid);

    (
        Bucket {
            pixels: left.to_vec(),
        },
        Bucket {
            pixels: right.to_vec(),
        },
    )
}

fn longest_axis(bucket: &Bucket) -> usize {
    let min_r = bucket.pixels.iter().map(|p| p.r as i32).min().unwrap_or(0);
    let max_r = bucket.pixels.iter().map(|p| p.r as i32).max().unwrap_or(0);
    let min_g = bucket.pixels.iter().map(|p| p.g as i32).min().unwrap_or(0);
    let max_g = bucket.pixels.iter().map(|p| p.g as i32).max().unwrap_or(0);
    let min_b = bucket.pixels.iter().map(|p| p.b as i32).min().unwrap_or(0);
    let max_b = bucket.pixels.iter().map(|p| p.b as i32).max().unwrap_or(0);

    let range_r = (max_r - min_r) as usize;
    let range_g = (max_g - min_g) as usize;
    let range_b = (max_b - min_b) as usize;

    if range_r >= range_g && range_r >= range_b {
        0
    } else if range_g >= range_r && range_g >= range_b {
        1
    } else {
        2
    }
}

fn median_cut(buckets: &mut Vec<Bucket>, count: usize) {
    while buckets.len() < count {
        let mut max_vol = 0;
        let mut max_idx = 0;
        for (i, bucket) in buckets.iter().enumerate() {
            if bucket.pixels.len() > 1 {
                let vol = bucket.volume();
                if vol > max_vol {
                    max_vol = vol;
                    max_idx = i;
                }
            }
        }

        if buckets[max_idx].pixels.len() <= 1 {
            break;
        }

        let bucket = buckets.remove(max_idx);
        let axis = longest_axis(&bucket);
        let (left, right) = split_bucket(&bucket, axis);

        if left.pixels.is_empty() || right.pixels.is_empty() {
            buckets.push(bucket);
            break;
        }

        buckets.push(left);
        buckets.push(right);
    }
}

/// Extract a deduplicated color palette from raw RGBA pixel data.
///
/// Expects a flat RGBA buffer (4 bytes per pixel, `width * height * 4`).
pub fn extract_palette_from_buffer(
    buffer: &[u8],
    _width: u32,
    _height: u32,
    color_count: u8,
    quality: u8,
) -> Result<Vec<(u8, u8, u8)>, String> {
    if buffer.is_empty() {
        return Err("Empty pixel buffer".to_string());
    }

    let pixels = sample_pixels(buffer, quality);
    if pixels.is_empty() {
        return Err("No pixels to process".to_string());
    }

    let mut buckets = vec![Bucket { pixels }];
    median_cut(&mut buckets, color_count as usize);

    let colors: Vec<(u8, u8, u8)> = buckets
        .iter()
        .map(|b| {
            let avg = b.avg();
            (avg.r, avg.g, avg.b)
        })
        .collect();

    let unique: Vec<(u8, u8, u8)> = colors.into_iter().fold(Vec::new(), |mut acc, c| {
        if !acc.contains(&c) {
            acc.push(c);
        }
        acc
    });

    Ok(unique)
}

/// CPU backend that implements the shared PaletteExtractor trait.
pub struct CpuExtractor;

impl PaletteExtractor for CpuExtractor {
    fn extract_palette(
        &self,
        buffer: &[u8],
        width: u32,
        height: u32,
        color_count: u8,
        quality: u8,
    ) -> Result<Vec<(u8, u8, u8)>, String> {
        extract_palette_from_buffer(buffer, width, height, color_count, quality)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Helpers ---

    fn solid_buffer(r: u8, g: u8, b: u8, count: usize) -> Vec<u8> {
        [r, g, b, 255].repeat(count)
    }

    fn two_color_buffer(
        r1: u8,
        g1: u8,
        b1: u8,
        r2: u8,
        g2: u8,
        b2: u8,
        count: usize,
    ) -> Vec<u8> {
        let mut buf = solid_buffer(r1, g1, b1, count);
        buf.extend(solid_buffer(r2, g2, b2, count));
        buf
    }

    fn image_buffer<F>(width: u32, height: u32, pixel_fn: F) -> Vec<u8>
    where
        F: Fn(u32, u32) -> (u8, u8, u8),
    {
        let mut buf = Vec::with_capacity((width * height) as usize * 4);
        for y in 0..height {
            for x in 0..width {
                let (r, g, b) = pixel_fn(x, y);
                buf.extend_from_slice(&[r, g, b, 255]);
            }
        }
        buf
    }

    // --- Solid color detection ---

    #[test]
    fn test_solid_red() {
        let buffer = solid_buffer(255, 0, 0, 100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result[0], (255, 0, 0));
    }

    #[test]
    fn test_solid_green() {
        let buffer = solid_buffer(0, 255, 0, 100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result[0], (0, 255, 0));
    }

    #[test]
    fn test_solid_blue() {
        let buffer = solid_buffer(0, 0, 255, 100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result[0], (0, 0, 255));
    }

    #[test]
    fn test_solid_white() {
        let buffer = solid_buffer(255, 255, 255, 9);
        let result = extract_palette_from_buffer(&buffer, 3, 3, 5, 1).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result[0], (255, 255, 255));
    }

    #[test]
    fn test_solid_black() {
        let buffer = solid_buffer(0, 0, 0, 100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result[0], (0, 0, 0));
    }

    // --- Two-color detection ---

    #[test]
    fn test_two_colors() {
        let buffer = two_color_buffer(255, 0, 0, 0, 0, 255, 50);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1).unwrap();
        assert!(result.contains(&(255, 0, 0)));
        assert!(result.contains(&(0, 0, 255)));
    }

    #[test]
    fn test_two_colors_green_purple() {
        let buffer = two_color_buffer(0, 255, 0, 128, 0, 128, 50);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1).unwrap();
        assert!(result.contains(&(0, 255, 0)));
        assert!(result.contains(&(128, 0, 128)));
    }

    // --- Empty / error ---

    #[test]
    fn test_empty_buffer() {
        let result = extract_palette_from_buffer(&[], 0, 0, 5, 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_buffer_too_small() {
        // 3 bytes — not enough for one RGBA pixel
        let result = extract_palette_from_buffer(&[255, 0, 0], 1, 1, 5, 1);
        // Should either succeed with 0 colors or error
        match result {
            Ok(palette) => assert!(palette.is_empty()),
            Err(_) => {}
        }
    }

    // --- color_count limits ---

    #[test]
    fn test_color_count_one() {
        let buffer = solid_buffer(200, 100, 50, 400);
        let result = extract_palette_from_buffer(&buffer, 20, 20, 1, 1).unwrap();
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_color_count_capped() {
        // Requesting more colors than available unique colors
        let buffer = solid_buffer(100, 150, 200, 100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 10, 1).unwrap();
        assert!(result.len() <= 10);
    }

    // --- Quality parameter ---

    #[test]
    fn test_quality_1() {
        let buffer = solid_buffer(255, 0, 0, 100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_quality_10() {
        let buffer = solid_buffer(255, 0, 0, 100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 10).unwrap();
        // May return fewer colors due to sampling — but should still find red
        assert!(result.iter().any(|(r, _, _)| *r > 200));
    }

    #[test]
    fn test_quality_100() {
        // Quality=100 means sampling 1 in 100 pixels
        let buffer = solid_buffer(255, 0, 0, 10_000);
        let result = extract_palette_from_buffer(&buffer, 100, 100, 5, 100).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_quality_0() {
        // quality=0 should be treated as quality=1 (max quality)
        let buffer = solid_buffer(255, 0, 0, 100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 0).unwrap();
        assert!(!result.is_empty());
    }

    // --- Determinism ---

    #[test]
    fn test_deterministic() {
        let buffer = solid_buffer(170, 85, 220, 100);
        let r1 = extract_palette_from_buffer(&buffer, 10, 10, 5, 1).unwrap();
        let r2 = extract_palette_from_buffer(&buffer, 10, 10, 5, 1).unwrap();
        assert_eq!(r1, r2, "same input should produce same output");
    }

    // --- Non-square images ---

    #[test]
    fn test_wide_image() {
        let buffer = solid_buffer(255, 128, 0, 200);
        let result = extract_palette_from_buffer(&buffer, 20, 10, 5, 1).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_tall_image() {
        let buffer = solid_buffer(0, 128, 255, 200);
        let result = extract_palette_from_buffer(&buffer, 10, 20, 5, 1).unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_1x1_image() {
        let buffer = solid_buffer(42, 84, 126, 1);
        let result = extract_palette_from_buffer(&buffer, 1, 1, 5, 1).unwrap();
        assert_eq!(result[0], (42, 84, 126));
    }

    // --- Gradient / checkerboard ---

    #[test]
    fn test_horizontal_gradient() {
        let buffer = image_buffer(30, 30, |x, _| ((x * 8) as u8, 128, 64));
        let result = extract_palette_from_buffer(&buffer, 30, 30, 5, 1).unwrap();
        assert!(!result.is_empty());
        // Should contain multiple distinct colors from the gradient
        assert!(result.len() >= 2);
    }

    #[test]
    fn test_checkerboard() {
        let buffer = image_buffer(50, 50, |x, y| {
            if (x + y) % 2 == 0 {
                (200, 50, 50)
            } else {
                (50, 50, 200)
            }
        });
        let result = extract_palette_from_buffer(&buffer, 50, 50, 5, 1).unwrap();
        assert!(!result.is_empty());
    }

    // --- Valid RGB range ---

    #[test]
    fn test_valid_rgb_values() {
        let buffer = solid_buffer(100, 150, 200, 100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1).unwrap();
        for (r, g, b) in &result {
            // u8 is inherently in [0, 255], but verify explicitly
            assert!((*r as i32) >= 0 && (*r as i32) <= 255);
            assert!((*g as i32) >= 0 && (*g as i32) <= 255);
            assert!((*b as i32) >= 0 && (*b as i32) <= 255);
        }
    }

    // --- Deduplication ---

    #[test]
    fn test_dedup_solid_color() {
        let buffer = solid_buffer(170, 85, 220, 100);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 10, 1).unwrap();
        // Even though color_count=10, solid color should produce 1 unique color
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], (170, 85, 220));
    }

    // --- CpuExtractor trait ---

    #[test]
    fn test_cpu_extractor_trait() {
        let extractor = CpuExtractor;
        let buffer = solid_buffer(255, 0, 0, 100);
        let result = extractor.extract_palette(&buffer, 10, 10, 5, 1).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result[0], (255, 0, 0));
    }

    // --- Bucket / median_cut internals ---

    #[test]
    fn test_bucket_avg() {
        let bucket = Bucket {
            pixels: vec![
                Color { r: 255, g: 0, b: 0 },
                Color { r: 0, g: 0, b: 255 },
            ],
        };
        let avg = bucket.avg();
        assert_eq!(avg.r, 127); // (255 + 0) / 2 = 127
        assert_eq!(avg.g, 0);
        assert_eq!(avg.b, 127);
    }

    #[test]
    fn test_bucket_volume() {
        let bucket = Bucket {
            pixels: vec![
                Color { r: 0, g: 0, b: 0 },
                Color { r: 255, g: 255, b: 255 },
            ],
        };
        let vol = bucket.volume();
        // spread: 255*255*255 * 2 pixels
        assert_eq!(vol, 255 * 255 * 255 * 2);
    }

    #[test]
    fn test_bucket_single_pixel_volume() {
        let bucket = Bucket {
            pixels: vec![Color { r: 100, g: 100, b: 100 }],
        };
        let vol = bucket.volume();
        // spread: 0*0*0 = 0, but max(1) * 1 = 1
        assert_eq!(vol, 1);
    }

    #[test]
    fn test_longest_axis_red_spread() {
        let bucket = Bucket {
            pixels: vec![
                Color { r: 0, g: 10, b: 10 },
                Color { r: 255, g: 20, b: 20 },
            ],
        };
        assert_eq!(longest_axis(&bucket), 0); // red axis has largest spread
    }

    #[test]
    fn test_longest_axis_green_spread() {
        let bucket = Bucket {
            pixels: vec![
                Color { r: 10, g: 0, b: 10 },
                Color { r: 20, g: 255, b: 20 },
            ],
        };
        assert_eq!(longest_axis(&bucket), 1); // green axis has largest spread
    }

    #[test]
    fn test_longest_axis_blue_spread() {
        let bucket = Bucket {
            pixels: vec![
                Color { r: 10, g: 10, b: 0 },
                Color { r: 20, g: 20, b: 255 },
            ],
        };
        assert_eq!(longest_axis(&bucket), 2); // blue axis has largest spread
    }

    #[test]
    fn test_split_bucket() {
        let bucket = Bucket {
            pixels: vec![
                Color { r: 0, g: 0, b: 0 },
                Color { r: 128, g: 0, b: 0 },
                Color { r: 255, g: 0, b: 0 },
            ],
        };
        let (left, right) = split_bucket(&bucket, 0); // split on red axis
        assert!(!left.pixels.is_empty());
        assert!(!right.pixels.is_empty());
        assert_eq!(left.pixels.len() + right.pixels.len(), 3);
    }

    #[test]
    fn test_sample_pixels() {
        let buffer = solid_buffer(255, 0, 0, 100);
        let pixels = sample_pixels(&buffer, 1);
        assert_eq!(pixels.len(), 100);
    }

    #[test]
    fn test_sample_pixels_quality_10() {
        let buffer = solid_buffer(255, 0, 0, 100);
        let pixels = sample_pixels(&buffer, 10);
        assert_eq!(pixels.len(), 10);
    }
}
