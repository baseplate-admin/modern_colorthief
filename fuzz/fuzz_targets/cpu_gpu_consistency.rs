#![no_main]

//! Fuzz both CPU and GPU backends for crash-freedom and basic invariants.
//!
//! The GPU backend is a CPU fallback (chunk means) that uses a different
//! algorithm than the CPU median-cut, so cross-backend color similarity is
//! not asserted. Only structural invariants are checked.

libfuzzer_sys::fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    let width = ((data[0] as u32).max(1)).min(256);
    let height = ((data[1] as u32).max(1)).min(256);
    let color_count = ((data[2] as u8).max(1)).min(10);
    let quality = ((data[3] as u8).max(1)).min(10);

    let pixel_count = (width * height) as usize;
    let expected = pixel_count * 4;

    let pixels: Vec<u8> = if data.len() >= expected + 4 {
        data[4..4 + expected].to_vec()
    } else {
        let available = data.len().saturating_sub(4);
        let mut buf = vec![0u8; expected];
        let n = available.min(expected);
        buf[..n].copy_from_slice(&data[4..4 + n]);
        buf
    };

    let cpu_result = modern_colorthief_core_cpu::extract_palette_from_buffer(
        &pixels, width, height, color_count, quality,
    );
    let gpu_result = modern_colorthief_core_gpu::extract_palette_from_buffer(
        &pixels, width, height, color_count, quality,
    );

    for (label, result) in [("cpu", &cpu_result), ("gpu", &gpu_result)] {
        if let Ok(palette) = result {
            assert!(!palette.is_empty(), "{label} palette should not be empty");
            assert!(
                palette.len() <= color_count as usize,
                "{label} palette should respect color_count"
            );
        }
    }
});
