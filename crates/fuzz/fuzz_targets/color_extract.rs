#![no_main]

//! Fuzz dominant color extraction from raw RGBA buffers.
//! Exercises both CPU and GPU backends to find crashes in the dominant-color path.

use libfuzzer_sys::arbitrary::{Arbitrary, Error};

#[derive(Debug, Arbitrary)]
struct ColorInput {
    width: u16,
    height: u16,
    quality: u8,
    data: Vec<u8>,
}

libfuzzer_sys::fuzz!(|input: ColorInput| {
    let width = input.width.max(1).min(512) as u32;
    let height = input.height.max(1).min(512) as u32;
    let quality = input.quality.max(1).min(10);

    let pixel_count = (width * height) as usize;
    let expected = pixel_count * 4;

    let data: Vec<u8> = if input.data.len() >= expected {
        input.data[..expected].to_vec()
    } else if input.data.len() > 0 {
        let mut buf = Vec::with_capacity(expected);
        let mut i = 0;
        while buf.len() < expected {
            let end = (i + input.data.len()).min(expected);
            buf.extend_from_slice(&input.data[i..end]);
            i = (i + 1).min(input.data.len());
        }
        buf
    } else {
        return;
    };

    // CPU dominant color
    let cpu_result =
        modern_colorthief_core_cpu::extract_dominant_color_from_buffer(&data, width, height, quality);
    // GPU dominant color
    let gpu_result =
        modern_colorthief_core_gpu::extract_dominant_color_from_buffer(&data, width, height, quality);

    match (cpu_result, gpu_result) {
        (Ok(cpu_color), Ok(gpu_color)) => {
            // Both should produce colors within reasonable range
            let dr = (cpu_color.0 as i32 - gpu_color.0 as i32).unsigned_abs();
            let dg = (cpu_color.1 as i32 - gpu_color.1 as i32).unsigned_abs();
            let db = (cpu_color.2 as i32 - gpu_color.2 as i32).unsigned_abs();
            // Allow wide tolerance — different sampling strategies produce different results
            assert!(
                dr + dg + db <= 120,
                "CPU and GPU dominant colors should be roughly similar.\n\
                 CPU: ({},{},{}) GPU: ({},{},{})",
                cpu_color.0, cpu_color.1, cpu_color.2,
                gpu_color.0, gpu_color.1, gpu_color.2,
            );
        }
        (Ok(_), Err(_)) | (Err(_), Ok(_)) => { /* One backend unavailable */ }
        (Err(_), Err(_)) => { /* Both unavailable */ }
    }
});
