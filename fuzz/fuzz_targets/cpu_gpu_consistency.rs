#![no_main]

//! Differential fuzzer: verify CPU and GPU backends produce consistent results.
//! Feeds the same image to both backends and checks that palettes match within tolerance.

use libfuzzer_sys::fuzz;

fuzz!(|data: &[u8]| {
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
    let gpu_result = modern_colorthief_core_gpu::extract_palette_from_buffer(
        &pixels, width, height, color_count, quality,
    );

    match (cpu_result, gpu_result) {
        (Ok(cpu_palette), Ok(gpu_palette)) => {
            assert!(!cpu_palette.is_empty(), "CPU should return non-empty palette");
            assert!(!gpu_palette.is_empty(), "GPU should return non-empty palette");

            let tolerance = 40;
            for ca in &cpu_palette {
                let matched = gpu_palette.iter().any(|cb| {
                    let dr = (ca.0 as i32 - cb.0 as i32).unsigned_abs();
                    let dg = (ca.1 as i32 - cb.1 as i32).unsigned_abs();
                    let db = (ca.2 as i32 - cb.2 as i32).unsigned_abs();
                    dr + dg + db <= tolerance
                });
                assert!(
                    matched,
                    "CPU color ({},{},{}) has no close GPU match.\n\
                     CPU palette: {:?}\nGPU palette: {:?}",
                    ca.0, ca.1, ca.2, cpu_palette, gpu_palette,
                );
            }
        }
        (Ok(cpu_p), Err(_)) => {
            assert!(!cpu_p.is_empty(), "CPU should return non-empty palette when GPU unavailable");
        }
        (Err(_), Ok(gpu_p)) => {
            assert!(!gpu_p.is_empty(), "GPU should return non-empty palette when CPU unavailable");
        }
        (Err(_), Err(_)) => { /* Both unavailable */ }
    }
});
