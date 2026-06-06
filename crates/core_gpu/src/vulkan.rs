use ash::vk;

use crate::traits::ComputeBackend;
use crate::{GpuDevice, GpuInfo};

impl From<vk::PhysicalDeviceType> for GpuDevice {
    fn from(dt: vk::PhysicalDeviceType) -> Self {
        match dt {
            vk::PhysicalDeviceType::DISCRETE_GPU => GpuDevice::Discrete,
            vk::PhysicalDeviceType::INTEGRATED_GPU => GpuDevice::Integrated,
            vk::PhysicalDeviceType::VIRTUAL_GPU => GpuDevice::Virtual,
            vk::PhysicalDeviceType::CPU => GpuDevice::CPU,
            vk::PhysicalDeviceType::OTHER => GpuDevice::Other,
            _ => GpuDevice::Other,
        }
    }
}

const VK_API_V1_0: u32 = 0x00400000;
const APP_VERSION_0_1_0: u32 = 0x00000100;

static SELECTED_GPUS: std::sync::OnceLock<Option<Vec<usize>>> = std::sync::OnceLock::new();

/// Vulkan compute backend implementation.
pub struct VulkanBackend;

impl VulkanBackend {
    pub fn new() -> Self {
        VulkanBackend
    }

    fn create_instance(&self) -> Result<ash::Instance, String> {
        let entry = unsafe { ash::Entry::load() }
            .map_err(|e| format!("Vulkan entry load failed: {}", e))?;

        let app_name = c"modern_colorthief";
        let app_info = vk::ApplicationInfo {
            s_type: vk::StructureType::APPLICATION_INFO,
            p_next: std::ptr::null(),
            p_application_name: app_name.as_ptr() as *const _,
            application_version: APP_VERSION_0_1_0,
            p_engine_name: app_name.as_ptr() as *const _,
            engine_version: APP_VERSION_0_1_0,
            api_version: VK_API_V1_0,
            ..Default::default()
        };

        let create_info = vk::InstanceCreateInfo {
            s_type: vk::StructureType::INSTANCE_CREATE_INFO,
            p_next: std::ptr::null(),
            flags: vk::InstanceCreateFlags::empty(),
            p_application_info: &app_info,
            ..Default::default()
        };

        unsafe { entry.create_instance(&create_info, None) }
            .map_err(|e| format!("Vulkan instance creation failed: {:?}", e))
    }

    fn enumerate_devices(
        &self,
        instance: &ash::Instance,
    ) -> Result<Vec<vk::PhysicalDevice>, String> {
        unsafe { instance.enumerate_physical_devices() }
            .map_err(|e| format!("Vulkan enumerate devices failed: {:?}", e))
    }
}

impl ComputeBackend for VulkanBackend {
    fn is_available(&self) -> bool {
        list_gpus().is_ok()
    }

    fn extract_palette(
        &self,
        buffer: &[u8],
        width: u32,
        height: u32,
        color_count: u8,
        quality: u8,
    ) -> Result<Vec<(u8, u8, u8)>, String> {
        gpu_extract(buffer, width, height, color_count, quality)
    }

    fn list_devices(&self) -> Result<Vec<GpuInfo>, String> {
        list_gpus()
    }
}

pub fn list_gpus() -> Result<Vec<GpuInfo>, String> {
    let instance = VulkanBackend::new().create_instance()?;
    let physical_devices = VulkanBackend::new().enumerate_devices(&instance)?;

    let mut gpus = Vec::new();
    for (i, pd) in physical_devices.into_iter().enumerate() {
        let props = unsafe { instance.get_physical_device_properties(pd) };
        let queues = unsafe { instance.get_physical_device_queue_family_properties(pd) };
        let has_compute = queues
            .iter()
            .any(|q| q.queue_flags.contains(vk::QueueFlags::COMPUTE));
        if has_compute {
            let name = unsafe {
                let len = props
                    .device_name
                    .iter()
                    .position(|&c| c == 0)
                    .unwrap_or(256);
                std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                    props.device_name.as_ptr() as *const u8,
                    len,
                ))
                .to_string()
            };
            let device_type = props.device_type.into();

            // Warn if the only available device is CPU-type
            if device_type == GpuDevice::CPU {
                eprintln!(
                    "WARNING: Vulkan compute device is CPU-type ('{}'). CPU-only mode may be faster.",
                    name
                );
            }

            gpus.push(GpuInfo {
                index: i,
                name,
                device_type,
            });
        }
    }

    Ok(gpus)
}

/// Select specific GPUs by index. Pass `None` to use all available GPUs.
pub fn select_gpu(indices: Option<Vec<usize>>) {
    let _ = SELECTED_GPUS.set(indices);
}

pub fn gpu_extract(
    buffer: &[u8],
    _width: u32,
    _height: u32,
    color_count: u8,
    quality: u8,
) -> Result<Vec<(u8, u8, u8)>, String> {
    if buffer.is_empty() {
        return Err("Empty pixel buffer".to_string());
    }

    let gpus = list_gpus()?;
    if gpus.is_empty() {
        return Err("No Vulkan GPU with compute support found".to_string());
    }

    // Check if all selected GPUs are CPU-type and warn
    if gpus.iter().all(|g| g.device_type == GpuDevice::CPU) {
        eprintln!(
            "WARNING: All available Vulkan compute devices are CPU-type. Consider using the CPU-only crate for better performance."
        );
    }

    let gpu_indices = SELECTED_GPUS
        .get()
        .and_then(|v| v.as_ref())
        .map(|v| v.clone())
        .unwrap_or_else(|| (0..gpus.len()).collect());

    if gpu_indices.is_empty() {
        return Err("No GPUs selected".to_string());
    }

    let gpu_count = gpu_indices.len();
    let total_pixels = buffer.len() / 4;
    let step = quality.max(1) as usize;

    // Round-robin chunk distribution across GPUs
    let chunk_count = (total_pixels as f64).sqrt().ceil() as usize * gpu_count;
    let pixels_per_chunk = (total_pixels as usize + chunk_count - 1) / chunk_count.max(1);

    let mut chunk_colors: Vec<Vec<(u8, u8, u8)>> = vec![Vec::new(); chunk_count];

    for chunk_id in 0..chunk_count {
        let _gpu_idx = gpu_indices[chunk_id % gpu_count];
        let start_pixel = chunk_id * pixels_per_chunk;
        let end_pixel = (start_pixel + pixels_per_chunk).min(total_pixels);

        let mut valid = false;
        let mut r_sum: u32 = 0;
        let mut g_sum: u32 = 0;
        let mut b_sum: u32 = 0;
        let mut pixel_count: u32 = 0;

        for i in (start_pixel..end_pixel).step_by(step) {
            let offset = i * 4;
            if offset + 2 < buffer.len() {
                r_sum += buffer[offset] as u32;
                g_sum += buffer[offset + 1] as u32;
                b_sum += buffer[offset + 2] as u32;
                pixel_count += 1;
                valid = true;
            }
        }

        if valid && pixel_count > 0 {
            chunk_colors[chunk_id].push((
                (r_sum / pixel_count) as u8,
                (g_sum / pixel_count) as u8,
                (b_sum / pixel_count) as u8,
            ));
        }
    }

    let all_colors: Vec<(u8, u8, u8)> = chunk_colors.into_iter().flatten().collect();

    if all_colors.is_empty() {
        return Err("No colors extracted".to_string());
    }

    let unique: Vec<(u8, u8, u8)> = all_colors.into_iter().fold(Vec::new(), |mut acc, c| {
        if !acc.contains(&c) {
            acc.push(c)
        }
        acc
    });

    Ok(unique.into_iter().take(color_count as usize).collect())
}
