#![no_main]

//! Fuzz dominant color extraction from raw RGBA buffers.
//! Exercises both CPU and GPU backends to find crashes in the dominant-color path.

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    let width = ((data[0] as u32).max(1)).min(64);
    let height = ((data[1] as u32).max(1)).min(64);
    let quality = ((data[2] as u8).max(1)).min(10);

    let pixel_count = (width * height) as usize;
    let expected = pixel_count * 4;

    let pixels: Vec<u8> = if data.len() >= expected + 3 {
        data[3..3 + expected].to_vec()
    } else {
        // Pad with the available bytes (or zeros) up to the expected size.
        let available = data.len().saturating_sub(3);
        let mut buf = vec![0u8; expected];
        buf[..available.min(expected)].copy_from_slice(&data[3..3 + available.min(expected)]);
        buf
    };

    let cpu_result =
        modern_colorthief_core_cpu::extract_palette_from_buffer(&pixels, width, height, 1, quality);
    let gpu_result =
        modern_colorthief_core_gpu::extract_palette_from_buffer(&pixels, width, height, 1, quality);

    // The GPU backend is a CPU fallback (chunk means) that uses a different
    // algorithm than the CPU median-cut, so cross-backend color similarity
    // is not asserted. Only invariants (non-empty, valid RGB) are checked.
    for (label, result) in [("cpu", &cpu_result), ("gpu", &gpu_result)] {
        if let Ok(palette) = result {
            assert!(!palette.is_empty(), "{label} palette should not be empty");
            assert_eq!(palette.len(), 1, "{label} color_count=1 should return 1 color");
            let (r, g, b) = palette[0];
            // All channels are u8 by type, so this is a no-op safety check
            // that the value round-trips through i32 without overflow.
            let _ = (r as i32).abs() + (g as i32).abs() + (b as i32).abs();
        }
    }
});
