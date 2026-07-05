//! GPU-specific fuzz / property-based tests.
//!
//! Exercises the GPU backend with random and edge-case images to catch
//! panics, assertion failures, and Vulkan compute pipeline bugs.

use modern_colorthief_core_gpu::extract_palette_from_buffer;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Generate a random RGBA buffer.
fn random_buffer(width: u32, height: u32, rng: &mut impl Rng) -> Vec<u8> {
    let size = (width * height) as usize * 4;
    (0..size).map(|_| rng.gen_range(0..=255)).collect()
}

/// Generate a solid-color buffer.
fn solid_buffer(r: u8, g: u8, b: u8, width: u32, height: u32) -> Vec<u8> {
    [r, g, b, 255].repeat((width * height) as usize)
}

/// Generate a horizontal gradient buffer.
fn gradient_buffer(width: u32, height: u32, r_start: u8, r_end: u8, g: u8, b: u8) -> Vec<u8> {
    let mut buf = Vec::with_capacity((width as usize * height as usize) * 4);
    for x in 0..width {
        let r_val = {
            let diff = r_end as i32 - r_start as i32;
            let step = diff * x as i32 / width.max(1) as i32;
            (r_start as i32 + step).clamp(0, 255) as u8
        };
        for _y in 0..height {
            buf.extend_from_slice(&[r_val, g, b, 255]);
        }
    }
    buf
}

/// Assert the GPU either succeeds with valid output or returns an error (GPU unavailable).
fn assert_gpu_valid(buf: &[u8], width: u32, height: u32, color_count: u8, quality: u8, label: &str) {
    let result = extract_palette_from_buffer(buf, width, height, color_count, quality);
    match result {
        Ok(palette) => {
            assert!(
                !palette.is_empty(),
                "{}: GPU returned non-empty palette",
                label
            );
            assert!(
                palette.len() <= color_count as usize,
                "{}: GPU returned at most color_count colors",
                label
            );
        }
        Err(e) => {
            // GPU unavailable is acceptable — error should mention Vulkan or similar
            assert!(
                e.contains("Vulkan") || e.contains("not found") || e.contains("Empty"),
                "{}: unexpected GPU error: {}",
                label,
                e
            );
        }
    }
}

// ============================================================
// GPU: Solid colors
// ============================================================

#[test]
fn fuzz_gpu_solid_red() {
    assert_gpu_valid(&solid_buffer(255, 0, 0, 50, 50), 50, 50, 5, 1, "solid_red");
}

#[test]
fn fuzz_gpu_solid_green() {
    assert_gpu_valid(&solid_buffer(0, 255, 0, 50, 50), 50, 50, 5, 1, "solid_green");
}

#[test]
fn fuzz_gpu_solid_blue() {
    assert_gpu_valid(&solid_buffer(0, 0, 255, 50, 50), 50, 50, 5, 1, "solid_blue");
}

#[test]
fn fuzz_gpu_solid_white() {
    assert_gpu_valid(&solid_buffer(240, 240, 240, 50, 50), 50, 50, 5, 1, "solid_white");
}

#[test]
fn fuzz_gpu_solid_black() {
    assert_gpu_valid(&solid_buffer(0, 0, 0, 50, 50), 50, 50, 5, 1, "solid_black");
}

#[test]
fn fuzz_gpu_solid_purple() {
    assert_gpu_valid(&solid_buffer(128, 0, 128, 50, 50), 50, 50, 5, 1, "solid_purple");
}

#[test]
fn fuzz_gpu_solid_yellow() {
    assert_gpu_valid(&solid_buffer(255, 255, 0, 50, 50), 50, 50, 5, 1, "solid_yellow");
}

#[test]
fn fuzz_gpu_solid_cyan() {
    assert_gpu_valid(&solid_buffer(0, 255, 255, 50, 50), 50, 50, 5, 1, "solid_cyan");
}

#[test]
fn fuzz_gpu_solid_mid_gray() {
    assert_gpu_valid(&solid_buffer(128, 128, 128, 50, 50), 50, 50, 5, 1, "solid_mid_gray");
}

// ============================================================
// GPU: Gradient / checkerboard
// ============================================================

#[test]
fn fuzz_gpu_horizontal_gradient() {
    assert_gpu_valid(&gradient_buffer(60, 60, 0, 255, 128, 64), 60, 60, 5, 1, "gradient");
}

#[test]
fn fuzz_gpu_gradient_wide() {
    assert_gpu_valid(&gradient_buffer(100, 50, 50, 200, 100, 150), 100, 50, 5, 2, "wide_gradient");
}

// ============================================================
// GPU: Random images with multiple seeds
// ============================================================

#[test]
fn fuzz_gpu_random_seed_42() {
    let mut rng = StdRng::seed_from_u64(42);
    assert_gpu_valid(&random_buffer(64, 64, &mut rng), 64, 64, 5, 1, "random_42");
}

#[test]
fn fuzz_gpu_random_seed_123() {
    let mut rng = StdRng::seed_from_u64(123);
    assert_gpu_valid(&random_buffer(64, 64, &mut rng), 64, 64, 5, 1, "random_123");
}

#[test]
fn fuzz_gpu_random_seed_999() {
    let mut rng = StdRng::seed_from_u64(999);
    assert_gpu_valid(&random_buffer(64, 64, &mut rng), 64, 64, 5, 1, "random_999");
}

#[test]
fn fuzz_gpu_random_seed_42069() {
    let mut rng = StdRng::seed_from_u64(42069);
    assert_gpu_valid(&random_buffer(48, 48, &mut rng), 48, 48, 5, 1, "random_42069");
}

#[test]
fn fuzz_gpu_random_seed_1337() {
    let mut rng = StdRng::seed_from_u64(1337);
    assert_gpu_valid(&random_buffer(100, 100, &mut rng), 100, 100, 5, 2, "random_1337");
}

#[test]
fn fuzz_gpu_random_varied_dimensions() {
    let mut rng = StdRng::seed_from_u64(777);
    for &(w, h) in &[(32, 32), (50, 30), (30, 50), (100, 50), (50, 100)] {
        assert_gpu_valid(
            &random_buffer(w, h, &mut rng),
            w, h, 5, 1,
            &format!("random_{}x{}", w, h),
        );
    }
}

// ============================================================
// GPU: Quality parameter variation
// ============================================================

#[test]
fn fuzz_gpu_quality_1() {
    assert_gpu_valid(&solid_buffer(170, 85, 220, 100, 100), 100, 100, 5, 1, "quality_1");
}

#[test]
fn fuzz_gpu_quality_5() {
    assert_gpu_valid(&solid_buffer(170, 85, 220, 100, 100), 100, 100, 5, 5, "quality_5");
}

#[test]
fn fuzz_gpu_quality_10() {
    assert_gpu_valid(&solid_buffer(170, 85, 220, 200, 200), 200, 200, 5, 10, "quality_10");
}

// ============================================================
// GPU: Color count variation
// ============================================================

#[test]
fn fuzz_gpu_color_count_1() {
    assert_gpu_valid(&solid_buffer(100, 150, 200, 50, 50), 50, 50, 1, 1, "count_1");
}

#[test]
fn fuzz_gpu_color_count_3() {
    assert_gpu_valid(&solid_buffer(100, 150, 200, 50, 50), 50, 50, 3, 1, "count_3");
}

#[test]
fn fuzz_gpu_color_count_10() {
    assert_gpu_valid(&gradient_buffer(100, 100, 0, 255, 128, 64), 100, 100, 10, 1, "count_10");
}

// ============================================================
// GPU: Edge cases
// ============================================================

#[test]
fn fuzz_gpu_single_pixel() {
    assert_gpu_valid(&[42u8, 84, 126, 255], 1, 1, 5, 1, "single_pixel");
}

#[test]
fn fuzz_gpu_narrow_image() {
    assert_gpu_valid(&solid_buffer(255, 128, 0, 1, 100), 1, 100, 5, 1, "narrow");
}

#[test]
fn fuzz_gpu_tall_image() {
    assert_gpu_valid(&solid_buffer(0, 128, 255, 100, 1), 100, 1, 5, 1, "tall");
}

#[test]
fn fuzz_gpu_large_solid() {
    assert_gpu_valid(
        &solid_buffer(170, 85, 220, 500, 500),
        500,
        500,
        5,
        5,
        "large_solid",
    );
}

#[test]
fn fuzz_gpu_reject_empty() {
    let result = extract_palette_from_buffer(&[], 0, 0, 5, 1);
    assert!(result.is_err(), "GPU should reject empty buffer");
}

// ============================================================
// GPU: Determinism
// ============================================================

#[test]
fn fuzz_gpu_deterministic() {
    let buf = gradient_buffer(80, 80, 0, 255, 100, 50);
    let g1 = extract_palette_from_buffer(&buf, 80, 80, 5, 1);
    let g2 = extract_palette_from_buffer(&buf, 80, 80, 5, 1);
    match (g1, g2) {
        (Ok(p1), Ok(p2)) => assert_eq!(p1, p2, "GPU should be deterministic"),
        (Err(_), Err(_)) => {} // GPU unavailable
        _ => panic!("inconsistent GPU availability"),
    }
}

// ============================================================
// GPU: Property-based — color_count limits palette size
// ============================================================

#[test]
fn fuzz_gpu_color_count_limits_size() {
    let buf = gradient_buffer(100, 100, 0, 255, 128, 64);
    for count in [1u8, 2, 3, 5, 10] {
        let palette = extract_palette_from_buffer(&buf, 100, 100, count, 1);
        if let Ok(p) = palette {
            assert!(
                p.len() <= count as usize,
                "GPU: color_count={} should return at most {} colors, got {}",
                count,
                count,
                p.len()
            );
        }
    }
}

// ============================================================
// GPU: Stress test
// ============================================================

#[test]
fn fuzz_gpu_stress_large_random() {
    let mut rng = StdRng::seed_from_u64(31415);
    assert_gpu_valid(
        &random_buffer(200, 200, &mut rng),
        200,
        200,
        10,
        2,
        "stress_large",
    );
}

#[test]
fn fuzz_gpu_stress_wide() {
    assert_gpu_valid(
        &gradient_buffer(400, 50, 0, 255, 100, 150),
        400,
        50,
        5,
        2,
        "stress_wide",
    );
}

#[test]
fn fuzz_gpu_stress_tall() {
    assert_gpu_valid(
        &solid_buffer(200, 100, 50, 50, 400),
        50,
        400,
        5,
        3,
        "stress_tall",
    );
}
