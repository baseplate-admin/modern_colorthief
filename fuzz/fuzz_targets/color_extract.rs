#![no_main]

//! Fuzz dominant color extraction from raw RGBA buffers.
//! Exercises both CPU and GPU backends to find crashes in the dominant-color path.

use libfuzzer_sys::fuzz;

fuzz!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    let width = ((data[0] as u32).max(1)).min(512);
    let height = ((data[1] as u32).max(1)).min(512);
    let quality = ((data[2] as u8).max(1)).min(10);

    let pixel_count = (width * height) as usize;
    let expected = pixel_count * 4;

    let pixels: Vec<u8> = if data.len() >= expected + 3 {
        data[3..3 + expected].to_vec()
    } else if data.len() > 3 {
        let mut buf = Vec::with_capacity(expected);
        let slice = &data[3..];
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

    let cpu_result =
        modern_colorthief_core_cpu::extract_dominant_color_from_buffer(&pixels, width, height, quality);
    let gpu_result =
        modern_colorthief_core_gpu::extract_dominant_color_from_buffer(&pixels, width, height, quality);

    match (cpu_result, gpu_result) {
        (Ok(cpu_color), Ok(gpu_color)) => {
            let dr = (cpu_color.0 as i32 - gpu_color.0 as i32).unsigned_abs();
            let dg = (cpu_color.1 as i32 - gpu_color.1 as i32).unsigned_abs();
            let db = (cpu_color.2 as i32 - gpu_color.2 as i32).unsigned_abs();
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
