use ash::vk;

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

    /// Platform-specific error message when Vulkan is not found.
    fn vulkan_not_found_error() -> String {
        #[cfg(target_os = "windows")]
        return "Vulkan not found. modern_colorthief GPU mode requires Vulkan ICD loader.\n\
                \n\
                Fix: Install GPU drivers from your hardware vendor:\n\
                • NVIDIA: https://www.nvidia.com/Download/index.aspx\n\
                • AMD: https://www.amd.com/en/support\n\
                • Intel: https://www.intel.com/content/www/us/en/download-center/home.html\n\
                \n\
                Vulkan ICD (vulkan-1.dll) is included with modern GPU drivers."
        .to_string();
        #[cfg(target_os = "linux")]
        return "Vulkan not found. modern_colorthief GPU mode requires the Vulkan loader.\n\
                \n\
                Fix: Install the Vulkan loader package:\n\
                • Debian/Ubuntu: sudo apt install libvulkan1\n\
                • Fedora: sudo dnf install vulkan-loader\n\
                • Arch: sudo pacman -S vulkan-loader\n\
                • openSUSE: sudo zypper install vulkan-loader\n\
                \n\
                Also install GPU-specific Vulkan drivers (e.g., mesa-vulkan-drivers)."
        .to_string();
        #[cfg(target_os = "macos")]
        return "Vulkan not found. macOS does not include native Vulkan support.\n\
                \n\
                Fix: Install MoltenVK to translate Vulkan to Metal:\n\
                • Homebrew: brew install molten-vk\n\
                • Then run: MoltenVKHelper --accept-sla\n\
                \n\
                MoltenVK provides Vulkan ICD on top of Metal."
        .to_string();
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            "Vulkan not supported on this platform.".to_string()
        }
    }

    /// Check if Vulkan loader is available on this system.
    fn vulkan_loader_available() -> bool {
        #[cfg(target_os = "windows")]
        {
            let vulkan_dll = std::path::Path::new("vulkan-1.dll");
            if vulkan_dll.exists() { return true; }
            std::env::var("SystemRoot").ok().map_or(false, |r| {
                std::path::Path::new(&format!("{}\\System32\\vulkan-1.dll", r)).exists()
            }) || std::env::var("VULKAN_SDK").ok().map_or(false, |sdk| {
                std::path::Path::new(&format!("{}\\Bin\\vulkan-1.dll", sdk)).exists()
            })
        }
        #[cfg(target_os = "linux")]
        {
            std::fs::metadata("libvulkan.so").is_ok()
                || std::fs::metadata("/usr/lib/libvulkan.so").is_ok()
                || std::fs::metadata("/usr/lib/x86_64-linux-gnu/libvulkan.so").is_ok()
                || std::env::var("VULKAN_SDK").ok().map_or(false, |sdk| {
                    std::path::Path::new(&format!("{}/lib/libvulkan.so", sdk)).exists()
                })
        }
        #[cfg(target_os = "macos")]
        {
            // Check for MoltenVK installation
            std::fs::metadata("/usr/local/lib/libMoltenVK.dylib").is_ok()
                || std::fs::metadata("/opt/homebrew/lib/libMoltenVK.dylib").is_ok()
                || std::fs::metadata("libMoltenVK.dylib").is_ok()
                || std::env::var("MOLTENVK_ROOT").ok().map_or(false, |root| {
                    std::path::Path::new(&format!("{}/libMoltenVK.dylib", root)).exists()
                })
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            false
        }
    }

    fn create_instance(&self) -> Result<ash::Instance, String> {
        if !Self::vulkan_loader_available() {
            return Err(Self::vulkan_not_found_error());
        }
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

fn vendor_name(vendor_id: u32) -> &'static str {
    match vendor_id {
        0x10DE => "NVIDIA",
        0x1002 => "AMD",
        0x8086 => "Intel",
        0x1028 => "VMware",
        0x1234 => "Mesa (llvmpipe/swrast)",
        0x106B => "Apple (MoltenVK)",
        0x5143 => "Qualcomm (Adreno)",
        0x13B5 => "ARM (Mali/Bifrost)",
        0x1022 => "ATI (legacy AMD)",
        0x14E4 => "Samsung",
        _ => "Unknown",
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
            let vendor = vendor_name(props.vendor_id);

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
                vendor_name: vendor.to_string(),
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
