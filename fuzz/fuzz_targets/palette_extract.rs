#![no_main]

//! Fuzz palette extraction from raw RGBA buffers.
//! Exercises both CPU and GPU backends with arbitrary image data to find crashes
//! and panics in the quantization pipeline.

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    let width = ((data[0] as u32).max(1)).min(512);
    let height = ((data[1] as u32).max(1)).min(512);
    let color_count = ((data[2] as u8).max(1)).min(16);
    let quality = ((data[3] as u8).max(1)).min(10);

    let pixel_count = (width * height) as usize;
    let expected = pixel_count * 4;

    let pixels: Vec<u8> = if data.len() >= expected + 4 {
        data[4..4 + expected].to_vec()
    } else if data.len() > 4 {
        let mut buf = Vec::with_capacity(expected);
        let slice = &data[4..];
        let mut i = 0;
        while buf.len() < expected {
            let end = (i + slice.len()).min(expected);
            buf.extend_from_slice(&slice[i..end]);
            i = (i + 1).min(slice.len());
        }
        buf
    } else {
        return;
    };

    let cpu_result = modern_colorthief_core_cpu::extract_palette_from_buffer(
        &pixels, width, height, color_count, quality,
    );
    if let Ok(palette) = &cpu_result {
        assert!(!palette.is_empty(), "CPU palette should not be empty");
        assert!(
            palette.len() <= color_count as usize,
            "CPU palette should respect color_count limit"
        );
    }

    let gpu_result = modern_colorthief_core_gpu::extract_palette_from_buffer(
        &pixels, width, height, color_count, quality,
    );
    if let Ok(palette) = &gpu_result {
        assert!(!palette.is_empty(), "GPU palette should not be empty");
        assert!(
            palette.len() <= color_count as usize,
            "GPU palette should respect color_count limit"
        );
    }
});
