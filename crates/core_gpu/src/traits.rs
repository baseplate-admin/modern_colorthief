use crate::GpuInfo;

/// Trait for GPU compute backends. Implement this for new backends (e.g. DX12, Metal, WebGPU).
pub trait ComputeBackend: Send + Sync {
    /// Check if this backend is available.
    fn is_available(&self) -> bool;

    /// Extract palette using GPU compute.
    fn extract_palette(
        &self,
        buffer: &[u8],
        width: u32,
        height: u32,
        color_count: u8,
        quality: u8,
    ) -> Result<Vec<(u8, u8, u8)>, String>;

    /// List available devices for this backend.
    fn list_devices(&self) -> Result<Vec<GpuInfo>, String>;
}
