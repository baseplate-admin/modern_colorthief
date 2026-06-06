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

    fn volume(&self) -> u32 {
        let min_r = self.pixels.iter().map(|p| p.r).min().unwrap_or(0);
        let max_r = self.pixels.iter().map(|p| p.r).max().unwrap_or(0);
        let min_g = self.pixels.iter().map(|p| p.g).min().unwrap_or(0);
        let max_g = self.pixels.iter().map(|p| p.g).max().unwrap_or(0);
        let min_b = self.pixels.iter().map(|p| p.b).min().unwrap_or(0);
        let max_b = self.pixels.iter().map(|p| p.b).max().unwrap_or(0);
        let spread_r = (max_r - min_r) as u32;
        let spread_g = (max_g - min_g) as u32;
        let spread_b = (max_b - min_b) as u32;
        (spread_r * spread_g * spread_b).max(1) * self.pixels.len() as u32
    }
}

fn sample_pixels(buffer: &[u8], quality: u8) -> Vec<Color> {
    let step = quality.max(1) as usize;
    let total_pixels = buffer.len() / 4;

    (0..total_pixels)
        .step_by(step)
        .par_bridge()
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

    let unique: Vec<(u8, u8, u8)> = colors
        .into_iter()
        .fold(Vec::new(), |mut acc, c| {
            if !acc.contains(&c) {
                acc.push(c);
            }
            acc
        });

    Ok(unique)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solid_red() {
        let buffer: Vec<u8> = [255u8, 0, 0, 255].repeat(25);
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1).unwrap();
        assert!(!result.is_empty());
        assert_eq!(result[0], (255, 0, 0));
    }

    #[test]
    fn test_two_colors() {
        let mut buffer = Vec::new();
        for _ in 0..50 {
            buffer.extend_from_slice(&[255, 0, 0, 255]);
        }
        for _ in 0..50 {
            buffer.extend_from_slice(&[0, 0, 255, 255]);
        }
        let result = extract_palette_from_buffer(&buffer, 10, 10, 5, 1).unwrap();
        assert!(result.contains(&(255, 0, 0)));
        assert!(result.contains(&(0, 0, 255)));
    }

    #[test]
    fn test_empty_buffer() {
        let result = extract_palette_from_buffer(&[], 0, 0, 5, 10);
        assert!(result.is_err());
    }
}
