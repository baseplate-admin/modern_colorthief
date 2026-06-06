#[cfg(feature = "cpu")]
mod backend {
    pub use modern_colorthief_core_cpu::*;
}

#[cfg(feature = "gpu")]
mod backend {
    pub use modern_colorthief_core_gpu::*;
}

pub use backend::extract_palette_from_buffer;
