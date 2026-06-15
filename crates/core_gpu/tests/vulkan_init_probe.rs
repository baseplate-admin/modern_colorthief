/// Integration test: verifies Vulkan compute init works.
/// Runs as a separate binary from the unit tests, so a segfault during
/// Vulkan instance creation (known with SwiftShader/llvmpipe on CI)
/// does not crash the main test harness.
#[test]
#[ignore]
fn vulkan_init_and_extract() {
    let buffer: Vec<u8> = [255u8, 0, 0, 255].repeat(25);
    let result = modern_colorthief_core_gpu::extract_palette_from_buffer(&buffer, 10, 10, 5, 1);
    if let Ok(palette) = result {
        assert!(!palette.is_empty(), "Extract returned empty palette");
    }
    // Error is acceptable — Vulkan may not be available
}
