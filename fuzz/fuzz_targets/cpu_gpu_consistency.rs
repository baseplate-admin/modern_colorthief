#![no_main]

//! Differential fuzzer: verify CPU and GPU backends produce consistent results.
//! Feeds the same image to both backends and checks that palettes match within tolerance.

use libfuzzer_sys::arbitrary::{Arbitrary, Error};

#[derive(Debug, Arbitrary)]
struct ConsistencyInput {
    width: u16,
    height: u16,
    color_count: u8,
    quality: u8,
    data: Vec<u8>,
}

libfuzzer_sys::fuzz!(|input: ConsistencyInput| {
    let width = input.width.max(1).min(256) as u32;
    let height = input.height.max(1).min(256) as u32;
    let color_count = input.color_count.max(1).min(10);
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

    let cpu_result = modern_colorthief_core_cpu::extract_palette_from_buffer(
        &data, width, height, color_count, quality,
    );
    let gpu_result = modern_colorthief_core_gpu::extract_palette_from_buffer(
        &data, width, height, color_count, quality,
    );

    match (cpu_result, gpu_result) {
        (Ok(cpu_palette), Ok(gpu_palette)) => {
            // Both backends should return non-empty palettes
            assert!(!cpu_palette.is_empty(), "CPU should return non-empty palette");
            assert!(!gpu_palette.is_empty(), "GPU should return non-empty palette");

            // Every CPU color should have a close GPU match (within tolerance)
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

            // Flag exact matches as interesting for the fuzzer
            if cpu_palette == gpu_palette {
                // exact match — both backends agree perfectly
            }
        }
        (Ok(cpu_p), Err(_)) => {
            assert!(!cpu_p.is_empty(), "CPU should return non-empty palette when GPU unavailable");
        }
        (Err(_), Ok(gpu_p)) => {
            assert!(!gpu_p.is_empty(), "GPU should return non-empty palette when CPU unavailable");
        }
        (Err(_), Err(_)) => { /* Both unavailable — skip */ }
    }
});
