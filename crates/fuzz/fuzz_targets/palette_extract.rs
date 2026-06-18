#![no_main]

//! Fuzz palette extraction from raw RGBA buffers.
//! Exercises both CPU and GPU backends with arbitrary image data to find crashes
//! and panics in the quantization pipeline.

use libfuzzer_sys::arbitrary::{Arbitrary, Result, Unstructured};

#[derive(Debug, Arbitrary)]
struct PaletteInput {
    width: u16,
    height: u16,
    color_count: u8,
    quality: u8,
    data: Vec<u8>,
}

libfuzzer_sys::fuzz!(|input: PaletteInput| {
    // Clamp dimensions to avoid OOM on fuzzers that generate large values
    let width = input.width.max(1).min(512) as u32;
    let height = input.height.max(1).min(512) as u32;
    let color_count = input.color_count.max(1).min(16);
    let quality = input.quality.max(1).min(10);

    let pixel_count = (width * height) as usize;
    let expected = pixel_count * 4;

    // Reshape data to match the requested dimensions
    let data: Vec<u8> = if input.data.len() >= expected {
        input.data[..expected].to_vec()
    } else if input.data.len() > 0 {
        // Tile available data to fill the buffer
        let mut buf = Vec::with_capacity(expected);
        let mut i = 0;
        while buf.len() < expected {
            let end = (i + input.data.len()).min(expected);
            buf.extend_from_slice(&input.data[i..end]);
            i = (i + 1).min(input.data.len());
        }
        buf
    } else {
        // Empty input — just return, both backends should handle gracefully
        return;
    };

    // CPU path
    let cpu_result = modern_colorthief_core_cpu::extract_palette_from_buffer(
        &data, width, height, color_count, quality,
    );
    if let Ok(palette) = &cpu_result {
        assert!(!palette.is_empty(), "CPU palette should not be empty");
        assert!(
            palette.len() <= color_count as usize,
            "CPU palette should respect color_count limit"
        );
    }

    // GPU path
    let gpu_result = modern_colorthief_core_gpu::extract_palette_from_buffer(
        &data, width, height, color_count, quality,
    );
    if let Ok(palette) = &gpu_result {
        assert!(!palette.is_empty(), "GPU palette should not be empty");
        assert!(
            palette.len() <= color_count as usize,
            "GPU palette should respect color_count limit"
        );
    }

    // Hint the fuzzer about interesting inputs
    if let Ok(p) = &cpu_result {
        if p.len() == color_count as usize {
            // Full palette — good coverage (no-op marker for potential future fuzz_hint)
        }
    }
});
