//! Corpus test: run the extraction pipeline over a directory of real images.
//!
//! Reads images from `CORPUS_DIR`, decodes them, and asserts the invariants the
//! library guarantees for every backend. The corpus itself is never committed;
//! `fetch_open_images.py` downloads a deterministic sample at test time.
//!
//! Images are processed in parallel with rayon; `RAYON_NUM_THREADS` limits the
//! worker count.
//!
//! Environment:
//! - `CORPUS_DIR`    directory containing the images (required)
//! - `CORPUS_LIMIT`  process at most this many images
//! - `CORPUS_GPU`    set to `1`/`true` to also exercise the Vulkan backend

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use modern_colorthief_core::{ColorthiefError, Rgb};
use rayon::prelude::*;

/// Color counts swept for every image on the CPU backend.
const COLOR_COUNTS: [u8; 4] = [1, 5, 10, 16];
/// Fast sampling quality for the per-image sweep.
const SWEEP_QUALITY: u8 = 10;
/// One full-resolution sampling pass per image, where the cost is highest.
const HEAVY_QUALITY: u8 = 1;
/// GPU dispatch is comparatively expensive, so check one representative config.
const GPU_CONFIG: (u8, u8) = (10, 10);

struct Failure {
    image: PathBuf,
    context: String,
    message: String,
}

#[derive(Default)]
struct Stats {
    images: usize,
    cpu_extractions: usize,
    gpu_extractions: usize,
    gpu_skipped: usize,
    pixels: u64,
}

#[derive(Default)]
struct ImageOutcome {
    stats: Stats,
    failures: Vec<Failure>,
    gpu_error: Option<String>,
}

fn main() -> ExitCode {
    let corpus_dir = match env::var("CORPUS_DIR") {
        Ok(dir) if !dir.trim().is_empty() => PathBuf::from(dir),
        _ => {
            eprintln!("CORPUS_DIR is not set; point it at a directory of images.");
            return ExitCode::FAILURE;
        }
    };

    let limit = env::var("CORPUS_LIMIT")
        .ok()
        .and_then(|v| v.parse::<usize>().ok())
        .filter(|v| *v > 0);
    let run_gpu = env::var("CORPUS_GPU").is_ok_and(|v| v == "1" || v == "true");

    let images = match collect_images(&corpus_dir, limit) {
        Ok(images) if images.is_empty() => {
            eprintln!("No images found in {}", corpus_dir.display());
            return ExitCode::FAILURE;
        }
        Ok(images) => images,
        Err(err) => {
            eprintln!("Failed to read {}: {err}", corpus_dir.display());
            return ExitCode::FAILURE;
        }
    };

    println!(
        "Running corpus test over {} image(s) from {} with {} worker(s)",
        images.len(),
        corpus_dir.display(),
        rayon::current_num_threads()
    );

    let gpu_unavailable = AtomicBool::new(false);
    let processed = AtomicUsize::new(0);
    let total = images.len();

    let outcomes: Vec<ImageOutcome> = images
        .par_iter()
        .map(|path| run_image(path, run_gpu, &gpu_unavailable, &processed, total))
        .collect();

    let mut stats = Stats::default();
    let mut failures = Vec::new();
    let mut gpu_error: Option<String> = None;
    for outcome in outcomes {
        stats.images += outcome.stats.images;
        stats.cpu_extractions += outcome.stats.cpu_extractions;
        stats.gpu_extractions += outcome.stats.gpu_extractions;
        stats.gpu_skipped += outcome.stats.gpu_skipped;
        stats.pixels += outcome.stats.pixels;
        failures.extend(outcome.failures);
        if gpu_error.is_none() {
            gpu_error = outcome.gpu_error;
        }
    }
    failures.sort_by(|a, b| {
        a.image
            .cmp(&b.image)
            .then_with(|| a.context.cmp(&b.context))
    });

    failures.iter().take(50).for_each(|failure| {
        eprintln!(
            "FAIL {} [{}]: {}",
            failure.image.display(),
            failure.context,
            failure.message
        );
    });
    if failures.len() > 50 {
        eprintln!("... and {} more failure(s)", failures.len() - 50);
    }

    println!(
        "Corpus: {} images, {} megapixels, {} CPU extractions, {} GPU extractions, {} GPU skips",
        stats.images,
        stats.pixels / 1_000_000,
        stats.cpu_extractions,
        stats.gpu_extractions,
        stats.gpu_skipped
    );
    if let Some(err) = &gpu_error {
        println!("GPU backend unavailable, remaining GPU checks skipped: {err}");
    }

    if failures.is_empty() {
        ExitCode::SUCCESS
    } else {
        eprintln!("{} corpus failure(s)", failures.len());
        ExitCode::FAILURE
    }
}

fn collect_images(dir: &Path, limit: Option<usize>) -> Result<Vec<PathBuf>, std::io::Error> {
    let mut images = Vec::new();
    for entry in fs::read_dir(dir)? {
        let path = entry?.path();
        if !path.is_file() {
            continue;
        }
        let is_image = path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| matches!(ext.to_ascii_lowercase().as_str(), "jpg" | "jpeg" | "png"));
        if is_image {
            images.push(path);
        }
    }
    images.sort();
    if let Some(limit) = limit {
        images.truncate(limit);
    }
    Ok(images)
}

fn run_image(
    path: &Path,
    run_gpu: bool,
    gpu_unavailable: &AtomicBool,
    processed: &AtomicUsize,
    total: usize,
) -> ImageOutcome {
    let mut outcome = ImageOutcome::default();

    let decoded = match image::open(path) {
        Ok(decoded) => decoded,
        Err(err) => {
            outcome.failures.push(Failure {
                image: path.to_path_buf(),
                context: "decode".to_string(),
                message: err.to_string(),
            });
            return outcome;
        }
    };
    let rgba = decoded.to_rgba8();
    let (width, height) = rgba.dimensions();
    let buffer = rgba.as_raw();
    outcome.stats.images = 1;
    outcome.stats.pixels += u64::from(width) * u64::from(height);

    check_determinism(path, buffer, width, height, &mut outcome.failures);

    for &color_count in &COLOR_COUNTS {
        check_cpu(
            path,
            buffer,
            width,
            height,
            color_count,
            SWEEP_QUALITY,
            &mut outcome.stats,
            &mut outcome.failures,
        );
    }
    // One pass at full sampling resolution: this is the slowest, most
    // allocation-heavy path and must survive real photographs.
    check_cpu(
        path,
        buffer,
        width,
        height,
        10,
        HEAVY_QUALITY,
        &mut outcome.stats,
        &mut outcome.failures,
    );

    if run_gpu && !gpu_unavailable.load(Ordering::Relaxed) {
        let (color_count, quality) = GPU_CONFIG;
        let context = format!("gpu color_count={color_count} quality={quality}");
        match modern_colorthief_core_gpu::extract_palette_from_buffer(
            buffer,
            width,
            height,
            color_count,
            quality,
        ) {
            Ok(palette) => {
                outcome.stats.gpu_extractions += 1;
                check_palette(path, &context, &palette, color_count, &mut outcome.failures);
            }
            Err(ColorthiefError::Backend(err)) => {
                outcome.stats.gpu_skipped += 1;
                outcome.gpu_error = Some(err);
                gpu_unavailable.store(true, Ordering::Relaxed);
            }
            Err(err) => outcome.failures.push(Failure {
                image: path.to_path_buf(),
                context,
                message: err.to_string(),
            }),
        }
    }

    let done = processed.fetch_add(1, Ordering::Relaxed) + 1;
    if done.is_multiple_of(100) {
        println!("  processed {done}/{total} images");
    }

    outcome
}

fn check_determinism(
    path: &Path,
    buffer: &[u8],
    width: u32,
    height: u32,
    failures: &mut Vec<Failure>,
) {
    let first = modern_colorthief_core_cpu::extract_palette_from_buffer(
        buffer,
        width,
        height,
        5,
        SWEEP_QUALITY,
    );
    let second = modern_colorthief_core_cpu::extract_palette_from_buffer(
        buffer,
        width,
        height,
        5,
        SWEEP_QUALITY,
    );
    if first != second {
        failures.push(Failure {
            image: path.to_path_buf(),
            context: "cpu determinism".to_string(),
            message: "two identical calls returned different palettes".to_string(),
        });
        return;
    }

    let dominant =
        modern_colorthief_core_cpu::extract_color_from_buffer(buffer, width, height, SWEEP_QUALITY);
    match (first, dominant) {
        (Ok(palette), Ok(color)) if palette.first() != Some(&color) => {
            failures.push(Failure {
                image: path.to_path_buf(),
                context: "dominant color".to_string(),
                message: format!(
                    "extract_color returned {color:?}, palette head is {:?}",
                    palette.first()
                ),
            });
        }
        (Err(err), _) | (_, Err(err)) => failures.push(Failure {
            image: path.to_path_buf(),
            context: "cpu determinism".to_string(),
            message: err.to_string(),
        }),
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn check_cpu(
    path: &Path,
    buffer: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
    stats: &mut Stats,
    failures: &mut Vec<Failure>,
) {
    let context = format!("cpu color_count={color_count} quality={quality}");
    match modern_colorthief_core_cpu::extract_palette_from_buffer(
        buffer,
        width,
        height,
        color_count,
        quality,
    ) {
        Ok(palette) => {
            stats.cpu_extractions += 1;
            check_palette(path, &context, &palette, color_count, failures);
        }
        Err(err) => failures.push(Failure {
            image: path.to_path_buf(),
            context,
            message: err.to_string(),
        }),
    }
}

fn check_palette(
    path: &Path,
    context: &str,
    palette: &[Rgb],
    color_count: u8,
    failures: &mut Vec<Failure>,
) {
    if palette.is_empty() {
        failures.push(Failure {
            image: path.to_path_buf(),
            context: context.to_string(),
            message: "palette is empty".to_string(),
        });
    }
    if palette.len() > color_count as usize {
        failures.push(Failure {
            image: path.to_path_buf(),
            context: context.to_string(),
            message: format!(
                "palette has {} colors, more than color_count={color_count}",
                palette.len()
            ),
        });
    }
}
