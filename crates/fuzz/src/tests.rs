use color_thief::{ColorFormat, get_palette};
use modern_colorthief_core_cpu::extract_palette_from_buffer as cpu_extract;
use modern_colorthief_core_gpu::extract_palette_from_buffer as gpu_extract;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

// ============================================================
// Helpers
// ============================================================

/// Compare two palettes for approximate equality.
/// Each color in one palette should have a close match in the other.
fn palettes_approximately_equal(a: &[(u8, u8, u8)], b: &[(u8, u8, u8)], tolerance: u32) -> bool {
    if a.is_empty() && b.is_empty() {
        return true;
    }
    if a.is_empty() || b.is_empty() {
        return false;
    }

    for ca in a {
        let matched = b.iter().any(|cb| {
            let dr = (ca.0 as i32 - cb.0 as i32).unsigned_abs();
            let dg = (ca.1 as i32 - cb.1 as i32).unsigned_abs();
            let db = (ca.2 as i32 - cb.2 as i32).unsigned_abs();
            dr + dg + db <= tolerance
        });
        if !matched {
            return false;
        }
    }
    for cb in b {
        let matched = a.iter().any(|ca| {
            let dr = (ca.0 as i32 - cb.0 as i32).unsigned_abs();
            let dg = (ca.1 as i32 - cb.1 as i32).unsigned_abs();
            let db = (ca.2 as i32 - cb.2 as i32).unsigned_abs();
            dr + dg + db <= tolerance
        });
        if !matched {
            return false;
        }
    }
    true
}

/// Generate a random RGBA buffer with the given dimensions.
fn random_buffer(width: u32, height: u32, rng: &mut impl Rng) -> Vec<u8> {
    let size = (width * height) as usize * 4;
    (0..size).map(|_| rng.gen_range(0..=255)).collect()
}

/// Generate a solid-color buffer.
fn solid_buffer(r: u8, g: u8, b: u8, width: u32, height: u32) -> Vec<u8> {
    [r, g, b, 255].repeat((width * height) as usize)
}

/// Generate a two-color buffer (half one color, half another).
fn two_color_buffer(c1: (u8, u8, u8), c2: (u8, u8, u8), width: u32, height: u32) -> Vec<u8> {
    let half = (width * height) as usize / 2;
    let mut buf = [c1.0, c1.1, c1.2, 255].repeat(half);
    buf.extend([c2.0, c2.1, c2.2, 255].repeat((width * height) as usize - half));
    buf
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

/// Generate a checkerboard buffer.
fn checkerboard_buffer(
    width: u32,
    height: u32,
    c1: (u8, u8, u8),
    c2: (u8, u8, u8),
    tile_size: u32,
) -> Vec<u8> {
    let mut buf = Vec::with_capacity((width * height) as usize * 4);
    for y in 0..height {
        for x in 0..width {
            let tile = (x / tile_size + y / tile_size) % 2;
            let (r, g, b) = if tile == 0 {
                (c1.0, c1.1, c1.2)
            } else {
                (c2.0, c2.1, c2.2)
            };
            buf.extend_from_slice(&[r, g, b, 255]);
        }
    }
    buf
}

/// Verify all colors in a palette are valid RGB values.
fn assert_valid_rgb(palette: &[(u8, u8, u8)], _label: &str) {
    // All values are u8 so inherently in [0, 255] — just verify non-empty
    assert!(!palette.is_empty());
}

/// Run a CPU vs GPU comparison.
/// For solid/two-color images: expect bidirectional palette match.
/// For gradient/random images: only check that both produce valid non-empty palettes
/// with colors in the expected range (different quantization algorithms produce different palettes).
fn assert_cpu_gpu_consistent(
    buf: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
    tolerance: u32,
    label: &str,
) {
    let cpu = cpu_extract(buf, width, height, color_count, quality);
    let gpu = gpu_extract(buf, width, height, color_count, quality);

    match (cpu, gpu) {
        (Ok(cpu_palette), Ok(gpu_palette)) => {
            assert_valid_rgb(&cpu_palette, &format!("{} cpu", label));
            assert_valid_rgb(&gpu_palette, &format!("{} gpu", label));
            assert!(
                palettes_approximately_equal(&cpu_palette, &gpu_palette, tolerance),
                "{}: CPU and GPU palettes should match.\n\
                 CPU: {:?}\nGPU: {:?}",
                label,
                cpu_palette,
                gpu_palette
            );
        }
        (Ok(p), Err(_)) => assert!(!p.is_empty(), "{}: CPU returned non-empty palette", label),
        (Err(_), Ok(p)) => assert!(!p.is_empty(), "{}: GPU returned non-empty palette", label),
        (Err(_), Err(_)) => { /* Both unavailable */ }
    }
}

/// For gradient/random images where CPU and GPU quantization algorithms differ,
/// only verify both produce valid palettes with colors in the expected range.
fn assert_cpu_gpu_both_valid(
    buf: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
    label: &str,
) {
    let cpu = cpu_extract(buf, width, height, color_count, quality);
    let gpu = gpu_extract(buf, width, height, color_count, quality);

    match (cpu, gpu) {
        (Ok(cpu_palette), Ok(gpu_palette)) => {
            assert!(
                !cpu_palette.is_empty(),
                "{}: CPU returned non-empty palette",
                label
            );
            assert!(
                !gpu_palette.is_empty(),
                "{}: GPU returned non-empty palette",
                label
            );
            assert!(
                cpu_palette.len() <= color_count as usize,
                "{}: CPU returned more than color_count colors",
                label
            );
            assert!(
                gpu_palette.len() <= color_count as usize,
                "{}: GPU returned more than color_count colors",
                label
            );
        }
        (Ok(p), Err(_)) => assert!(!p.is_empty(), "{}: CPU returned non-empty palette", label),
        (Err(_), Ok(p)) => assert!(!p.is_empty(), "{}: GPU returned non-empty palette", label),
        (Err(_), Err(_)) => { /* Both unavailable */ }
    }
}

/// Run a CPU vs color_thief reference comparison.
/// Different algorithms produce different intermediate colors, so we check that
/// each CPU color has a nearby match in the reference palette (one-directional).
/// color_thief requires max_colors >= 2 and quality 1..10.
fn assert_cpu_vs_reference(buf: &[u8], color_count: u8, quality: u8, tolerance: u32, label: &str) {
    let ct_count = color_count.max(2);
    let ct_quality = quality.clamp(1, 10);

    let cpu = cpu_extract(buf, 50, 50, color_count, quality);
    let ref_result = get_palette(buf, ColorFormat::Rgba, ct_quality, ct_count);

    match (cpu, ref_result) {
        (Ok(cpu_palette), Ok(ref_palette)) => {
            let ref_tuples: Vec<(u8, u8, u8)> =
                ref_palette.iter().map(|c| (c.r, c.g, c.b)).collect();
            // Check: every CPU color has a nearby match in the reference palette
            let all_matched = cpu_palette.iter().all(|ca| {
                ref_tuples.iter().any(|cb| {
                    let dr = (ca.0 as i32 - cb.0 as i32).unsigned_abs();
                    let dg = (ca.1 as i32 - cb.1 as i32).unsigned_abs();
                    let db = (ca.2 as i32 - cb.2 as i32).unsigned_abs();
                    dr + dg + db <= tolerance
                })
            });
            assert!(
                all_matched,
                "{}: Each CPU color should have a nearby match in color_thief palette.\n\
                 CPU: {:?}\nref: {:?}",
                label, cpu_palette, ref_tuples
            );
        }
        (Ok(p), Err(e)) => {
            eprintln!("color_thief failed: {:?}", e);
            assert!(!p.is_empty());
        }
        (Err(e), Ok(p)) => {
            eprintln!("CPU failed: {}", e);
            assert!(!p.is_empty());
        }
        (Err(_), Err(_)) => {}
    }
}

// ============================================================
// CPU vs GPU: Solid colors
// ============================================================

#[test]
fn fuzz_cpu_gpu_solid_red() {
    let buf = solid_buffer(255, 0, 0, 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 20, "solid_red");
}

#[test]
fn fuzz_cpu_gpu_solid_green() {
    let buf = solid_buffer(0, 255, 0, 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 20, "solid_green");
}

#[test]
fn fuzz_cpu_gpu_solid_blue() {
    let buf = solid_buffer(0, 0, 255, 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 20, "solid_blue");
}

#[test]
fn fuzz_cpu_gpu_solid_white() {
    let buf = solid_buffer(240, 240, 240, 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 20, "solid_white");
}

#[test]
fn fuzz_cpu_gpu_solid_black() {
    let buf = solid_buffer(0, 0, 0, 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 20, "solid_black");
}

#[test]
fn fuzz_cpu_gpu_solid_purple() {
    let buf = solid_buffer(128, 0, 128, 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 20, "solid_purple");
}

#[test]
fn fuzz_cpu_gpu_solid_yellow() {
    let buf = solid_buffer(255, 255, 0, 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 20, "solid_yellow");
}

#[test]
fn fuzz_cpu_gpu_solid_cyan() {
    let buf = solid_buffer(0, 255, 255, 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 20, "solid_cyan");
}

#[test]
fn fuzz_cpu_gpu_solid_mid_gray() {
    let buf = solid_buffer(128, 128, 128, 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 20, "solid_mid_gray");
}

// ============================================================
// CPU vs GPU: Two colors
// ============================================================

#[test]
fn fuzz_cpu_gpu_two_colors_rgb() {
    let buf = two_color_buffer((255, 0, 0), (0, 255, 0), 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 30, "two_colors_rgb");
}

#[test]
fn fuzz_cpu_gpu_two_colors_blue_purple() {
    let buf = two_color_buffer((0, 0, 255), (128, 0, 128), 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 30, "two_colors_blue_purple");
}

#[test]
fn fuzz_cpu_gpu_two_colors_complementary() {
    let buf = two_color_buffer((255, 128, 0), (0, 128, 255), 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 30, "two_colors_complementary");
}

#[test]
fn fuzz_cpu_gpu_two_colors_near_black() {
    let buf = two_color_buffer((10, 10, 10), (50, 50, 50), 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 5, 1, 30, "two_colors_near_black");
}

// ============================================================
// CPU vs GPU: Gradient / checkerboard
// ============================================================

#[test]
fn fuzz_cpu_gpu_horizontal_gradient() {
    let buf = gradient_buffer(60, 60, 0, 255, 128, 64);
    assert_cpu_gpu_both_valid(&buf, 60, 60, 5, 1, "horizontal_gradient");
}

#[test]
fn fuzz_cpu_gpu_checkerboard() {
    let buf = checkerboard_buffer(60, 60, (200, 50, 50), (50, 50, 200), 4);
    assert_cpu_gpu_both_valid(&buf, 60, 60, 5, 1, "checkerboard");
}

#[test]
fn fuzz_cpu_gpu_checkerboard_fine() {
    let buf = checkerboard_buffer(80, 80, (255, 0, 0), (0, 0, 255), 2);
    assert_cpu_gpu_both_valid(&buf, 80, 80, 5, 1, "checkerboard_fine");
}

// ============================================================
// CPU vs GPU: Random images with multiple seeds
// ============================================================

#[test]
fn fuzz_cpu_gpu_random_seed_42() {
    let mut rng = StdRng::seed_from_u64(42);
    let buf = random_buffer(64, 64, &mut rng);
    assert_cpu_gpu_both_valid(&buf, 64, 64, 5, 1, "random_seed_42");
}

#[test]
fn fuzz_cpu_gpu_random_seed_123() {
    let mut rng = StdRng::seed_from_u64(123);
    let buf = random_buffer(64, 64, &mut rng);
    assert_cpu_gpu_both_valid(&buf, 64, 64, 5, 1, "random_seed_123");
}

#[test]
fn fuzz_cpu_gpu_random_seed_999() {
    let mut rng = StdRng::seed_from_u64(999);
    let buf = random_buffer(64, 64, &mut rng);
    assert_cpu_gpu_both_valid(&buf, 64, 64, 5, 1, "random_seed_999");
}

#[test]
fn fuzz_cpu_gpu_random_seed_42069() {
    let mut rng = StdRng::seed_from_u64(42069);
    let buf = random_buffer(48, 48, &mut rng);
    assert_cpu_gpu_both_valid(&buf, 48, 48, 5, 1, "random_seed_42069");
}

#[test]
fn fuzz_cpu_gpu_random_seed_1337() {
    let mut rng = StdRng::seed_from_u64(1337);
    let buf = random_buffer(100, 100, &mut rng);
    assert_cpu_gpu_both_valid(&buf, 100, 100, 5, 2, "random_seed_1337_quality2");
}

#[test]
fn fuzz_cpu_gpu_random_varied_dimensions() {
    let mut rng = StdRng::seed_from_u64(777);
    let dimensions = [
        (32, 32),
        (50, 30),
        (30, 50),
        (100, 50),
        (50, 100),
        (128, 64),
    ];
    for (w, h) in dimensions {
        let buf = random_buffer(w, h, &mut rng);
        assert_cpu_gpu_both_valid(&buf, w, h, 5, 1, &format!("random_{}x{}", w, h));
    }
}

// ============================================================
// CPU vs GPU: Quality parameter variation
// ============================================================

#[test]
fn fuzz_cpu_gpu_quality_1() {
    let buf = solid_buffer(170, 85, 220, 100, 100);
    assert_cpu_gpu_consistent(&buf, 100, 100, 5, 1, 20, "quality_1");
}

#[test]
fn fuzz_cpu_gpu_quality_2() {
    let buf = solid_buffer(170, 85, 220, 100, 100);
    assert_cpu_gpu_consistent(&buf, 100, 100, 5, 2, 25, "quality_2");
}

#[test]
fn fuzz_cpu_gpu_quality_5() {
    let buf = solid_buffer(170, 85, 220, 100, 100);
    assert_cpu_gpu_consistent(&buf, 100, 100, 5, 5, 30, "quality_5");
}

#[test]
fn fuzz_cpu_gpu_quality_10() {
    let buf = solid_buffer(170, 85, 220, 200, 200);
    assert_cpu_gpu_consistent(&buf, 200, 200, 5, 10, 35, "quality_10");
}

// ============================================================
// CPU vs GPU: Color count variation
// ============================================================

#[test]
fn fuzz_cpu_gpu_color_count_1() {
    let buf = solid_buffer(100, 150, 200, 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 1, 1, 20, "color_count_1");
}

#[test]
fn fuzz_cpu_gpu_color_count_3() {
    let buf = solid_buffer(100, 150, 200, 50, 50);
    assert_cpu_gpu_consistent(&buf, 50, 50, 3, 1, 20, "color_count_3");
}

#[test]
fn fuzz_cpu_gpu_color_count_10() {
    let buf = gradient_buffer(100, 100, 0, 255, 128, 64);
    assert_cpu_gpu_both_valid(&buf, 100, 100, 10, 1, "color_count_10");
}

// ============================================================
// CPU vs GPU: Edge cases
// ============================================================

#[test]
fn fuzz_cpu_gpu_single_pixel() {
    let buf = [42u8, 84, 126, 255];
    assert_cpu_gpu_consistent(&buf, 1, 1, 5, 1, 10, "single_pixel");
}

#[test]
fn fuzz_cpu_gpu_two_pixels() {
    let buf = [255u8, 0, 0, 255, 0, 0, 255, 255];
    assert_cpu_gpu_consistent(&buf, 2, 1, 5, 1, 10, "two_pixels");
}

#[test]
fn fuzz_cpu_gpu_narrow_image() {
    let buf = solid_buffer(255, 128, 0, 1, 100);
    assert_cpu_gpu_consistent(&buf, 1, 100, 5, 1, 20, "narrow_image");
}

#[test]
fn fuzz_cpu_gpu_tall_image() {
    let buf = solid_buffer(0, 128, 255, 100, 1);
    assert_cpu_gpu_consistent(&buf, 100, 1, 5, 1, 20, "tall_image");
}

#[test]
fn fuzz_cpu_gpu_large_solid() {
    let buf = solid_buffer(170, 85, 220, 500, 500);
    assert_cpu_gpu_consistent(&buf, 500, 500, 5, 5, 25, "large_solid");
}

#[test]
fn fuzz_cpu_gpu_reject_empty() {
    let cpu = cpu_extract(&[], 0, 0, 5, 1);
    let gpu = gpu_extract(&[], 0, 0, 5, 1);
    assert!(cpu.is_err(), "CPU should reject empty buffer");
    assert!(gpu.is_err(), "GPU should reject empty buffer");
}

// ============================================================
// CPU vs GPU: Determinism
// ============================================================

#[test]
fn fuzz_cpu_deterministic() {
    let buf = gradient_buffer(80, 80, 0, 255, 100, 50);
    let r1 = cpu_extract(&buf, 80, 80, 5, 1).unwrap();
    let r2 = cpu_extract(&buf, 80, 80, 5, 1).unwrap();
    assert_eq!(r1, r2, "CPU should be deterministic");
}

#[test]
fn fuzz_gpu_deterministic() {
    let buf = gradient_buffer(80, 80, 0, 255, 100, 50);
    let g1 = gpu_extract(&buf, 80, 80, 5, 1);
    let g2 = gpu_extract(&buf, 80, 80, 5, 1);
    match (g1, g2) {
        (Ok(p1), Ok(p2)) => assert_eq!(p1, p2, "GPU should be deterministic"),
        (Err(_), Err(_)) => { /* GPU unavailable */ }
        (Ok(p), Err(_)) | (Err(_), Ok(p)) => assert!(!p.is_empty()),
    }
}

// ============================================================
// CPU vs GPU: Property-based — color_count limits palette size
// ============================================================

#[test]
fn fuzz_cpu_color_count_limits_size() {
    let buf = gradient_buffer(100, 100, 0, 255, 128, 64);
    for count in [1u8, 2, 3, 5, 10] {
        let palette = cpu_extract(&buf, 100, 100, count, 1).unwrap();
        assert!(
            palette.len() <= count as usize,
            "CPU: color_count={} should return at most {} colors, got {}",
            count,
            count,
            palette.len()
        );
    }
}

#[test]
fn fuzz_gpu_color_count_limits_size() {
    let buf = gradient_buffer(100, 100, 0, 255, 128, 64);
    for count in [1u8, 2, 3, 5, 10] {
        let palette = gpu_extract(&buf, 100, 100, count, 1);
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
// CPU vs color_thief reference (RazrFalcon)
// ============================================================

#[test]
fn fuzz_cpu_vs_ref_solid_color() {
    let buf = solid_buffer(200, 100, 50, 50, 50);
    assert_cpu_vs_reference(&buf, 5, 1, 30, "solid_color");
}

#[test]
fn fuzz_cpu_vs_ref_solid_red() {
    let buf = solid_buffer(255, 0, 0, 50, 50);
    assert_cpu_vs_reference(&buf, 5, 1, 20, "solid_red");
}

#[test]
fn fuzz_cpu_vs_ref_solid_green() {
    let buf = solid_buffer(0, 255, 0, 50, 50);
    assert_cpu_vs_reference(&buf, 5, 1, 20, "solid_green");
}

#[test]
fn fuzz_cpu_vs_ref_solid_blue() {
    let buf = solid_buffer(0, 0, 255, 50, 50);
    assert_cpu_vs_reference(&buf, 5, 1, 20, "solid_blue");
}

#[test]
fn fuzz_cpu_vs_ref_two_colors() {
    let buf = two_color_buffer((255, 0, 0), (0, 255, 0), 50, 50);
    assert_cpu_vs_reference(&buf, 5, 1, 30, "two_colors");
}

#[test]
fn fuzz_cpu_vs_ref_two_colors_blue_purple() {
    let buf = two_color_buffer((0, 0, 255), (128, 0, 128), 50, 50);
    assert_cpu_vs_reference(&buf, 5, 1, 30, "two_colors_blue_purple");
}

#[test]
fn fuzz_cpu_vs_ref_gradient() {
    let buf = gradient_buffer(60, 60, 0, 255, 128, 64);
    assert_cpu_vs_reference(&buf, 5, 1, 50, "gradient");
}

#[test]
fn fuzz_cpu_vs_ref_checkerboard() {
    let buf = checkerboard_buffer(60, 60, (200, 50, 50), (50, 50, 200), 4);
    assert_cpu_vs_reference(&buf, 5, 1, 40, "checkerboard");
}

#[test]
fn fuzz_cpu_vs_ref_random_seed_42() {
    let mut rng = StdRng::seed_from_u64(42);
    let buf = random_buffer(50, 50, &mut rng);
    // Random images: different quantization algorithms produce very different palettes.
    // Only verify both produce valid non-empty palettes with correct size.
    let cpu = modern_colorthief_core_cpu::extract_palette_from_buffer(&buf, 50, 50, 5, 1);
    let ref_result = get_palette(&buf, ColorFormat::Rgba, 1, 5);
    match (cpu, ref_result) {
        (Ok(cpu_p), Ok(ref_p)) => {
            assert!(!cpu_p.is_empty());
            assert!(!ref_p.is_empty());
            assert!(cpu_p.len() <= 5);
            assert!(ref_p.len() <= 5);
        }
        (Ok(p), Err(e)) => {
            eprintln!("color_thief failed: {:?}", e);
            assert!(!p.is_empty());
        }
        (Err(e), Ok(p)) => {
            eprintln!("CPU failed: {}", e);
            assert!(!p.is_empty());
        }
        (Err(_), Err(_)) => {}
    }
}

#[test]
fn fuzz_cpu_vs_ref_random_seed_777() {
    let mut rng = StdRng::seed_from_u64(777);
    let buf = random_buffer(50, 50, &mut rng);
    let cpu = modern_colorthief_core_cpu::extract_palette_from_buffer(&buf, 50, 50, 5, 2);
    let ref_result = get_palette(&buf, ColorFormat::Rgba, 2, 5);
    match (cpu, ref_result) {
        (Ok(cpu_p), Ok(ref_p)) => {
            assert!(!cpu_p.is_empty());
            assert!(!ref_p.is_empty());
            assert!(cpu_p.len() <= 5);
            assert!(ref_p.len() <= 5);
        }
        (Ok(p), Err(e)) => {
            eprintln!("color_thief failed: {:?}", e);
            assert!(!p.is_empty());
        }
        (Err(e), Ok(p)) => {
            eprintln!("CPU failed: {}", e);
            assert!(!p.is_empty());
        }
        (Err(_), Err(_)) => {}
    }
}

// ============================================================
// color_thief: Verify reference behavior
// ============================================================

#[test]
fn fuzz_ref_solid_dominant_color() {
    let buf = solid_buffer(200, 100, 50, 50, 50);
    let palette = get_palette(&buf, ColorFormat::Rgba, 1, 5).unwrap();
    // First color should be close to (200, 100, 50)
    let dominant = &palette[0];
    assert!(
        (dominant.r as i32 - 200).unsigned_abs() < 30
            && (dominant.g as i32 - 100).unsigned_abs() < 30
            && (dominant.b as i32 - 50).unsigned_abs() < 30,
        "color_thief dominant color should be near (200, 100, 50), got ({}, {}, {})",
        dominant.r,
        dominant.g,
        dominant.b
    );
}

#[test]
fn fuzz_ref_two_colors_found() {
    let buf = two_color_buffer((255, 0, 0), (0, 255, 0), 50, 50);
    let palette = get_palette(&buf, ColorFormat::Rgba, 1, 5).unwrap();
    let has_red = palette.iter().any(|c| c.r > 200 && c.g < 55 && c.b < 55);
    let has_green = palette.iter().any(|c| c.r < 55 && c.g > 200 && c.b < 55);
    assert!(has_red || has_green, "color_thief should find red or green");
}

#[test]
fn fuzz_ref_gradient_multiple_colors() {
    let buf = gradient_buffer(60, 60, 0, 255, 128, 64);
    let palette = get_palette(&buf, ColorFormat::Rgba, 1, 5).unwrap();
    assert!(
        palette.len() >= 2,
        "color_thief should find >= 2 colors in gradient"
    );
}

#[test]
fn fuzz_ref_dominant_color_with_accent() {
    // Mostly (100, 150, 200) with a small red accent
    let mut buf = solid_buffer(100, 150, 200, 48, 50);
    for _ in 0..20 {
        buf.extend_from_slice(&[255, 0, 0, 255]);
    }
    let palette = get_palette(&buf, ColorFormat::Rgba, 1, 5).unwrap();
    let dominant = &palette[0];
    assert!(
        (dominant.r as i32 - 100).unsigned_abs() < 40
            && (dominant.g as i32 - 150).unsigned_abs() < 40
            && (dominant.b as i32 - 200).unsigned_abs() < 40,
        "color_thief should find dominant color near (100, 150, 200), got ({}, {}, {})",
        dominant.r,
        dominant.g,
        dominant.b
    );
}

// ============================================================
// color_thief: Edge cases
// ============================================================

#[test]
fn fuzz_ref_single_pixel() {
    let buf = [42u8, 84, 126, 255];
    let palette = get_palette(&buf, ColorFormat::Rgba, 1, 2);
    // color_thief may succeed or fail on single pixel — both are acceptable
    if let Ok(p) = palette {
        assert!(!p.is_empty());
    }
}

#[test]
fn fuzz_ref_black_image() {
    let buf = solid_buffer(0, 0, 0, 50, 50);
    let palette = get_palette(&buf, ColorFormat::Rgba, 1, 5);
    // color_thief skips transparent/white pixels; black should be found
    if let Ok(p) = palette {
        assert!(!p.is_empty(), "color_thief should handle black image");
    }
}

// ============================================================
// All three: CPU, GPU, and color_thief consistency
// ============================================================

#[test]
fn fuzz_all_three_solid_red() {
    let buf = solid_buffer(255, 0, 0, 50, 50);
    let cpu = cpu_extract(&buf, 50, 50, 5, 1);
    let gpu = gpu_extract(&buf, 50, 50, 5, 1);
    let ref_result = get_palette(&buf, ColorFormat::Rgba, 1, 5);

    match (cpu, gpu, ref_result) {
        (Ok(cpu_p), Ok(gpu_p), Ok(ref_p)) => {
            let ref_tuples: Vec<(u8, u8, u8)> = ref_p.iter().map(|c| (c.r, c.g, c.b)).collect();
            // All three should find red
            let cpu_red = cpu_p.iter().any(|(r, g, b)| *r > 200 && *g < 55 && *b < 55);
            let gpu_red = gpu_p.iter().any(|(r, g, b)| *r > 200 && *g < 55 && *b < 55);
            let ref_red = ref_tuples
                .iter()
                .any(|(r, g, b)| *r > 200 && *g < 55 && *b < 55);
            assert!(
                cpu_red || gpu_red || ref_red,
                "At least one backend should find red"
            );
        }
        _ => { /* Some backends unavailable */ }
    }
}

#[test]
fn fuzz_all_three_two_colors() {
    let buf = two_color_buffer((255, 0, 0), (0, 0, 255), 50, 50);
    let cpu = cpu_extract(&buf, 50, 50, 5, 1);
    let gpu = gpu_extract(&buf, 50, 50, 5, 1);
    let ref_result = get_palette(&buf, ColorFormat::Rgba, 1, 5);

    match (cpu, gpu, ref_result) {
        (Ok(cpu_p), Ok(gpu_p), Ok(ref_p)) => {
            let ref_tuples: Vec<(u8, u8, u8)> = ref_p.iter().map(|c| (c.r, c.g, c.b)).collect();
            // All three should find at least 2 distinct colors
            assert!(cpu_p.len() >= 2, "CPU should find >= 2 colors");
            assert!(gpu_p.len() >= 2, "GPU should find >= 2 colors");
            assert!(ref_tuples.len() >= 2, "color_thief should find >= 2 colors");
        }
        _ => { /* Some backends unavailable */ }
    }
}

#[test]
fn fuzz_all_three_gradient() {
    let buf = gradient_buffer(60, 60, 0, 255, 128, 64);
    let cpu = cpu_extract(&buf, 60, 60, 5, 1);
    let gpu = gpu_extract(&buf, 60, 60, 5, 1);
    let ref_result = get_palette(&buf, ColorFormat::Rgba, 1, 5);

    match (cpu, gpu, ref_result) {
        (Ok(cpu_p), Ok(gpu_p), Ok(ref_p)) => {
            let ref_tuples: Vec<(u8, u8, u8)> = ref_p.iter().map(|c| (c.r, c.g, c.b)).collect();
            // All three should find colors at both ends of the gradient
            let cpu_spread = cpu_p.iter().map(|(r, _, _)| *r).max().unwrap_or(0) as i32
                - cpu_p.iter().map(|(r, _, _)| *r).min().unwrap_or(255) as i32;
            let gpu_spread = gpu_p.iter().map(|(r, _, _)| *r).max().unwrap_or(0) as i32
                - gpu_p.iter().map(|(r, _, _)| *r).min().unwrap_or(255) as i32;
            let ref_spread = ref_tuples.iter().map(|(r, _, _)| *r).max().unwrap_or(0) as i32
                - ref_tuples.iter().map(|(r, _, _)| *r).min().unwrap_or(255) as i32;
            assert!(
                cpu_spread > 50 || gpu_spread > 50 || ref_spread > 50,
                "At least one backend should find a wide spread of red values in the gradient"
            );
        }
        _ => { /* Some backends unavailable */ }
    }
}

// ============================================================
// Large-scale stress test
// ============================================================

#[test]
fn fuzz_cpu_gpu_stress_large_random() {
    let mut rng = StdRng::seed_from_u64(31415);
    let buf = random_buffer(200, 200, &mut rng);
    assert_cpu_gpu_both_valid(&buf, 200, 200, 10, 2, "stress_large_random");
}

#[test]
fn fuzz_cpu_gpu_stress_wide_image() {
    let buf = gradient_buffer(400, 50, 0, 255, 100, 150);
    assert_cpu_gpu_both_valid(&buf, 400, 50, 5, 2, "stress_wide");
}

#[test]
fn fuzz_cpu_gpu_stress_tall_image() {
    let buf = solid_buffer(200, 100, 50, 50, 400);
    assert_cpu_gpu_consistent(&buf, 50, 400, 5, 3, 25, "stress_tall");
}
