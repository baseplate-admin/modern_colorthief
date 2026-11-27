use ash::vk;

use crate::{GpuDevice, GpuInfo};

/// Map a Vulkan physical-device type to the shared `GpuDevice` enum.
/// (Free function rather than `impl From<...>` because `GpuDevice` is
/// defined in `modern-colorthief-core`.)
fn device_type_from_vk(dt: vk::PhysicalDeviceType) -> GpuDevice {
    match dt {
        vk::PhysicalDeviceType::DISCRETE_GPU => GpuDevice::Discrete,
        vk::PhysicalDeviceType::INTEGRATED_GPU => GpuDevice::Integrated,
        vk::PhysicalDeviceType::VIRTUAL_GPU => GpuDevice::Virtual,
        vk::PhysicalDeviceType::CPU => GpuDevice::CPU,
        vk::PhysicalDeviceType::OTHER => GpuDevice::Other,
        _ => GpuDevice::Other,
    }
}

const VK_API_V1_0: u32 = 0x00400000;
const APP_VERSION_0_1_0: u32 = 0x00000100;

static SELECTED_GPUS: std::sync::OnceLock<Option<Vec<usize>>> = std::sync::OnceLock::new();

/// Vulkan compute backend implementation.
pub struct VulkanBackend;

impl Default for VulkanBackend {
    fn default() -> Self {
        VulkanBackend
    }
}

impl VulkanBackend {
    pub fn new() -> Self {
        VulkanBackend
    }

    /// Platform-specific error when Vulkan is not found.
    fn vulkan_not_found_error() -> modern_colorthief_core::ColorthiefError {
        modern_colorthief_core::ColorthiefError::Backend(Self::vulkan_not_found_msg())
    }

    /// Platform-specific error message when Vulkan is not found.
    fn vulkan_not_found_msg() -> String {
        #[cfg(target_os = "windows")]
        return "Vulkan not found. modern_colorthief GPU mode requires Vulkan ICD loader.\n\
                \n\
                Fix: Install GPU drivers from your hardware vendor:\n\
                • NVIDIA: https://www.nvidia.com/Download/index.aspx\n\
                • AMD: https://www.amd.com/en/support\n\
                • Intel: https://www.intel.com/content/www/us/en/download-center/home.html\n\
                \n\
                Or install the Vulkan SDK with SwiftShader for software rendering:\n\
                https://vulkan.lunarg.com/sdk\n\
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
                • openSUSE: sudo zypper in vulkan-loader\n\
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

    /// Find the Vulkan loader library path on this system.
    /// Returns the explicit path if found, or None if not available.
    pub(crate) fn find_vulkan_loader() -> Option<std::path::PathBuf> {
        #[cfg(target_os = "windows")]
        {
            use std::path::PathBuf;

            // Check current directory
            let vulkan_dll = PathBuf::from("vulkan-1.dll");
            if vulkan_dll.exists() {
                return Some(vulkan_dll);
            }

            // Check System32
            if let Ok(system_root) = std::env::var("SystemRoot") {
                let sys_dll = std::path::Path::new(&system_root)
                    .join("System32")
                    .join("vulkan-1.dll");
                if sys_dll.exists() {
                    return Some(sys_dll);
                }
            }

            // Check Vulkan SDK Bin directory
            if let Ok(sdk) = std::env::var("VULKAN_SDK") {
                let sdk_bin_dll = std::path::Path::new(&sdk).join("Bin").join("vulkan-1.dll");
                if sdk_bin_dll.exists() {
                    return Some(sdk_bin_dll);
                }
            }

            // Check PATH directories for vulkan-1.dll
            if let Ok(path_var) = std::env::var("PATH") {
                for dir in std::env::split_paths(&path_var) {
                    let dll_path = dir.join("vulkan-1.dll");
                    if dll_path.exists() {
                        return Some(dll_path);
                    }
                }
            }
            None
        }
        #[cfg(target_os = "linux")]
        {
            let paths = [
                "libvulkan.so",
                "libvulkan.so.1",
                "/usr/lib/libvulkan.so",
                "/usr/lib/libvulkan.so.1",
                "/usr/lib/x86_64-linux-gnu/libvulkan.so",
                "/usr/lib/x86_64-linux-gnu/libvulkan.so.1",
                "/usr/lib/i386-linux-gnu/libvulkan.so",
                "/usr/lib/i386-linux-gnu/libvulkan.so.1",
                "/usr/local/lib/libvulkan.so",
                "/usr/local/lib/libvulkan.so.1",
            ];
            for p in &paths {
                if std::path::Path::new(p).exists() {
                    return Some(std::path::PathBuf::from(p));
                }
            }
            if let Ok(sdk) = std::env::var("VULKAN_SDK") {
                for sub in &[
                    "lib/libvulkan.so",
                    "lib/libvulkan.so.1",
                    "lib/x86_64-linux-gnu/libvulkan.so",
                ] {
                    let sdk_path = format!("{}/{}", sdk, sub);
                    if std::path::Path::new(&sdk_path).exists() {
                        return Some(std::path::PathBuf::from(&sdk_path));
                    }
                }
            }
            // Check LD_LIBRARY_PATH / standard library paths
            if let Ok(ld_path) = std::env::var("LD_LIBRARY_PATH") {
                for dir in std::env::split_paths(&ld_path) {
                    for name in &["libvulkan.so.1", "libvulkan.so"] {
                        let lib = dir.join(name);
                        if lib.exists() {
                            return Some(lib);
                        }
                    }
                }
            }
            None
        }
        #[cfg(target_os = "macos")]
        {
            let paths = [
                "/usr/local/lib/libMoltenVK.dylib",
                "/opt/homebrew/lib/libMoltenVK.dylib",
                "libMoltenVK.dylib",
            ];
            for p in &paths {
                if std::path::Path::new(p).exists() {
                    return Some(std::path::PathBuf::from(p));
                }
            }
            if let Ok(root) = std::env::var("MOLTENVK_ROOT") {
                let moltenvk_path = format!("{}/libMoltenVK.dylib", root);
                if std::path::Path::new(&moltenvk_path).exists() {
                    return Some(std::path::PathBuf::from(&moltenvk_path));
                }
            }
            None
        }
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            None
        }
    }

    /// Create a Vulkan Entry + Instance pair where the Entry outlives the Instance.
    ///
    /// CRITICAL: ash::Entry loaded via `load_from()` must NOT be dropped while
    /// the Instance is in use — the Instance resolves function pointers through
    /// the Entry. Using the Instance after Entry drops is UB (segfaults on
    /// Windows SwiftShader CI).
    fn create_instance_pair(
        &self,
    ) -> Result<(ash::Entry, ash::Instance), modern_colorthief_core::ColorthiefError> {
        let loader_path = Self::find_vulkan_loader().ok_or_else(Self::vulkan_not_found_error)?;

        // Use explicit path loading to avoid DLL search path issues on Windows.
        // This is critical for SwiftShader on CI where vulkan-1.dll is not in PATH.
        let entry = unsafe { ash::Entry::load_from(&loader_path) }
            .map_err(|e| modern_colorthief_core::ColorthiefError::Backend(e.to_string()))?;

        let app_name = c"modern_colorthief";
        let app_info = vk::ApplicationInfo {
            p_application_name: app_name.as_ptr() as *const _,
            application_version: APP_VERSION_0_1_0,
            p_engine_name: app_name.as_ptr() as *const _,
            engine_version: APP_VERSION_0_1_0,
            api_version: VK_API_V1_0,
            ..Default::default()
        };

        let create_info = vk::InstanceCreateInfo {
            p_application_info: &app_info,
            ..Default::default()
        };

        let instance = unsafe { entry.create_instance(&create_info, None) }
            .map_err(|e| modern_colorthief_core::ColorthiefError::Backend(e.to_string()))?;

        Ok((entry, instance))
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

/// Check if a queue family can dispatch compute work.
/// Accepts COMPUTE queues (ideal) or GRAPHICS queues (software Vulkan fallback like lavapipe/SwiftShader).
fn has_compute_queue(queues: &[vk::QueueFamilyProperties]) -> bool {
    queues.iter().any(|q| {
        q.queue_flags.contains(vk::QueueFlags::COMPUTE)
            || q.queue_flags.contains(vk::QueueFlags::GRAPHICS)
    })
}

pub fn list_gpus() -> Result<Vec<crate::GpuInfo>, modern_colorthief_core::ColorthiefError> {
    // Keep Entry alive while Instance is used — Entry must NOT be dropped
    // before Instance operations complete (UB per ash docs).
    let (entry, instance) = VulkanBackend::new().create_instance_pair()?;

    let physical_devices = unsafe { instance.enumerate_physical_devices() }
        .map_err(|e| modern_colorthief_core::ColorthiefError::Backend(e.to_string()))?;

    let mut gpus = Vec::new();
    for (i, pd) in physical_devices.into_iter().enumerate() {
        let props = unsafe { instance.get_physical_device_properties(pd) };
        let queues = unsafe { instance.get_physical_device_queue_family_properties(pd) };
        let has_compute = has_compute_queue(&queues);
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
            let device_type = device_type_from_vk(props.device_type);
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

    // Entry is dropped here, after all Instance operations complete.
    drop(entry);
    Ok(gpus)
}

/// Select specific GPUs by index. Pass `None` to use all available GPUs.
pub fn select_gpu(indices: Option<Vec<usize>>) {
    let _ = SELECTED_GPUS.set(indices);
}

// ---------------------------------------------------------------------------
// Compute dispatch (F1): WGSL → SPIR-V via naga, then a real Vulkan compute pass.
// Mirrors `core_webgpu/src/helper.ts` + `shader.wgsl`.
// ---------------------------------------------------------------------------

/// WGSL source for the chunk-mean compute shader. Compiled to SPIR-V at
/// startup via naga (see `shader_spirv`). Identical to `core_webgpu/src/shader.wgsl`.
const SHADER_WGSL: &str = include_str!("shader.wgsl");

fn backend_err(msg: String) -> modern_colorthief_core::ColorthiefError {
    modern_colorthief_core::ColorthiefError::Backend(msg)
}

/// Compile the WGSL shader to a SPIR-V word vector once per process.
fn shader_spirv() -> Result<Vec<u32>, modern_colorthief_core::ColorthiefError> {
    static CODE: std::sync::OnceLock<Result<Vec<u32>, String>> = std::sync::OnceLock::new();
    let result = CODE.get_or_init(|| {
        let module = naga::front::wgsl::parse_str(SHADER_WGSL)
            .map_err(|e| format!("WGSL parse error: {e}"))?;
        let mut validator = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::default(),
        );
        let info = validator
            .validate(&module)
            .map_err(|e| format!("WGSL validation error: {e}"))?;
        let options = naga::back::spv::Options::default();
        let spv = naga::back::spv::write_vec(&module, &info, &options, None)
            .map_err(|e| format!("SPIR-V codegen error: {e}"))?;
        Ok(spv)
    });
    result.clone().map_err(backend_err)
}

/// Pick a compute-capable queue family on `physical_device`.
fn pick_queue_family(instance: &ash::Instance, physical_device: vk::PhysicalDevice) -> Option<u32> {
    let queues = unsafe { instance.get_physical_device_queue_family_properties(physical_device) };
    // Prefer a combined GRAPHICS|COMPUTE (general-purpose) family: on some
    // drivers (observed on RTX 5090) the dedicated compute-only family
    // returns stale/zero results for compute dispatches, while the general
    // family is reliable. Fall back to any compute-capable family.
    for (i, q) in queues.iter().enumerate() {
        if q.queue_flags.contains(vk::QueueFlags::COMPUTE)
            && q.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        {
            return Some(i as u32);
        }
    }
    for (i, q) in queues.iter().enumerate() {
        if has_compute_queue(std::slice::from_ref(q)) {
            return Some(i as u32);
        }
    }
    None
}

/// Create a logical device with one queue on the chosen family.
fn create_logical_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    family_index: u32,
) -> Result<ash::Device, modern_colorthief_core::ColorthiefError> {
    let priority = 1.0f32;
    let queue_info = vk::DeviceQueueCreateInfo {
        queue_family_index: family_index,
        queue_count: 1,
        p_queue_priorities: &priority,
        ..Default::default()
    };

    let create_info = vk::DeviceCreateInfo {
        s_type: vk::StructureType::DEVICE_CREATE_INFO,
        p_next: std::ptr::null(),
        flags: vk::DeviceCreateFlags::empty(),
        queue_create_info_count: 1,
        p_queue_create_infos: &queue_info,
        ..Default::default()
    };

    unsafe { instance.create_device(physical_device, &create_info, None) }
        .map_err(|e| backend_err(e.to_string()))
}

/// Allocate and bind device memory for a buffer. Returns the buffer and its memory handle.
fn create_buffer_with_memory(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: &ash::Device,
    size_bytes: u64,
    usage: vk::BufferUsageFlags,
) -> Result<(vk::Buffer, vk::DeviceMemory), modern_colorthief_core::ColorthiefError> {
    let buffer_info = vk::BufferCreateInfo {
        size: size_bytes.max(1),
        usage,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };
    let buffer = unsafe { device.create_buffer(&buffer_info, None) }
        .map_err(|e| backend_err(e.to_string()))?;

    let reqs = unsafe { device.get_buffer_memory_requirements(buffer) };
    let mem_props = unsafe { instance.get_physical_device_memory_properties(physical_device) };

    // Choose the most local memory type that both supports the buffer's
    // required bits and satisfies the property constraints we need:
    //   - staging buffer (TRANSFER_DST): HOST_VISIBLE|HOST_COHERENT required
    //     (CPU readback) — any host-visible type works.
    //   - all other buffers (STORAGE_BUFFER, UNIFORM_BUFFER): prefer
    //     DEVICE_LOCAL|HOST_VISIBLE|HOST_COHERENT if available (most local,
    //     AND mappable — needed for the map-based uniform buffer upload that
    //     is the only path verified GPU-visible on this platform), falling
    //     back to plain DEVICE_LOCAL (best GPU locality), and finally to any
    //     host-visible type if the device offers no local+visible combination.
    let is_staging = is_host_visible_usage(usage);
    // UNIFORM_BUFFER is uploaded via map/write/unmap → needs HOST_VISIBLE.
    // STORAGE_BUFFER (pixels, chunks) is written by cmd_copy_buffer and
    // read/written by the compute shader → needs pure DEVICE_LOCAL on the
    // RTX 5090; DL|HV|HC left the buffer partially stale for the shader.
    let needs_mappable = usage.contains(vk::BufferUsageFlags::UNIFORM_BUFFER);

    let mut memory_type_index: Option<u32> = None;

    if is_staging || needs_mappable {
        // Host-visible path: prefer DL|HV|HC (most local AND mappable),
        // fall back to any HV|HC.
        for (i, mt) in mem_props.memory_types.iter().enumerate() {
            if ((reqs.memory_type_bits >> i as u32) & 1 == 1)
                && mt.property_flags.contains(
                    vk::MemoryPropertyFlags::DEVICE_LOCAL
                        | vk::MemoryPropertyFlags::HOST_VISIBLE
                        | vk::MemoryPropertyFlags::HOST_COHERENT,
                )
            {
                memory_type_index = Some(i as u32);
                break;
            }
        }
        if memory_type_index.is_none() {
            for (i, mt) in mem_props.memory_types.iter().enumerate() {
                if ((reqs.memory_type_bits >> i as u32) & 1 == 1)
                    && mt.property_flags.contains(
                        vk::MemoryPropertyFlags::HOST_VISIBLE
                            | vk::MemoryPropertyFlags::HOST_COHERENT,
                    )
                {
                    memory_type_index = Some(i as u32);
                    break;
                }
            }
        }
    } else {
        // Pure DEVICE_LOCAL (no HOST_VISIBLE) — reliable for shader I/O.
        for (i, mt) in mem_props.memory_types.iter().enumerate() {
            if ((reqs.memory_type_bits >> i as u32) & 1 == 1)
                && mt
                    .property_flags
                    .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
                && !mt
                    .property_flags
                    .contains(vk::MemoryPropertyFlags::HOST_VISIBLE)
            {
                memory_type_index = Some(i as u32);
                break;
            }
        }
        // Fall back: any DEVICE_LOCAL, then any HV|HC.
        if memory_type_index.is_none() {
            for (i, mt) in mem_props.memory_types.iter().enumerate() {
                if ((reqs.memory_type_bits >> i as u32) & 1 == 1)
                    && mt
                        .property_flags
                        .contains(vk::MemoryPropertyFlags::DEVICE_LOCAL)
                {
                    memory_type_index = Some(i as u32);
                    break;
                }
            }
        }
        if memory_type_index.is_none() {
            for (i, mt) in mem_props.memory_types.iter().enumerate() {
                if ((reqs.memory_type_bits >> i as u32) & 1 == 1)
                    && mt.property_flags.contains(
                        vk::MemoryPropertyFlags::HOST_VISIBLE
                            | vk::MemoryPropertyFlags::HOST_COHERENT,
                    )
                {
                    memory_type_index = Some(i as u32);
                    break;
                }
            }
        }
    }
    let memory_type_index = memory_type_index
        .ok_or_else(|| backend_err("No suitable Vulkan memory type found".to_string()))?;

    let alloc_info = vk::MemoryAllocateInfo {
        allocation_size: reqs.size,
        memory_type_index,
        ..Default::default()
    };
    let memory = unsafe { device.allocate_memory(&alloc_info, None) }
        .map_err(|e| backend_err(e.to_string()))?;

    unsafe { device.bind_buffer_memory(buffer, memory, 0) }
        .map_err(|e| backend_err(e.to_string()))?;

    Ok((buffer, memory))
}

/// Whether this buffer must live in host-visible (CPU-mappable) memory.
fn is_host_visible_usage(usage: vk::BufferUsageFlags) -> bool {
    usage.contains(vk::BufferUsageFlags::TRANSFER_DST)
}

/// Run the chunk-mean compute shader on `buffer` and return per-chunk RGB colors.
fn dispatch_chunk_means(
    buffer: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> Result<Vec<(u8, u8, u8)>, modern_colorthief_core::ColorthiefError> {
    let (entry, instance) = VulkanBackend::new().create_instance_pair()?;

    let result = dispatch_chunk_means_inner(&instance, buffer, width, height, color_count, quality);

    unsafe {
        instance.destroy_instance(None);
    }
    drop(entry);
    result
}

fn dispatch_chunk_means_inner(
    instance: &ash::Instance,
    buffer: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> Result<Vec<(u8, u8, u8)>, modern_colorthief_core::ColorthiefError> {
    let physical_devices =
        unsafe { instance.enumerate_physical_devices() }.map_err(|e| backend_err(e.to_string()))?;

    // Prefer a DISCRETE_GPU (e.g. RTX 5090); fall back to the first
    // compute-capable device (e.g. AMD iGPU).
    let mut chosen: Option<(vk::PhysicalDevice, u32)> = None;
    let mut fallback: Option<(vk::PhysicalDevice, u32)> = None;
    for pd in &physical_devices {
        let props = unsafe { instance.get_physical_device_properties(*pd) };
        if let Some(family) = pick_queue_family(instance, *pd) {
            if props.device_type == vk::PhysicalDeviceType::DISCRETE_GPU {
                chosen = Some((*pd, family));
                break;
            }
            if fallback.is_none() {
                fallback = Some((*pd, family));
            }
        }
    }
    let (physical_device, family_index) = match chosen.or(fallback) {
        Some(c) => c,
        None => {
            return Err(backend_err(
                "No Vulkan GPU with compute support found".to_string(),
            ));
        }
    };

    // Warn if the only available device is CPU-type (matches previous behavior).
    let props = unsafe { instance.get_physical_device_properties(physical_device) };
    if device_type_from_vk(props.device_type) == GpuDevice::CPU {
        eprintln!(
            "WARNING: All available Vulkan compute devices are CPU-type. Consider using the CPU-only crate for better performance."
        );
    }

    let device = create_logical_device(instance, physical_device, family_index)?;
    let queue = unsafe { device.get_device_queue(family_index, 0) };

    // Vulkan handles are Copy; track everything for unconditional cleanup.
    let mut shader_module = vk::ShaderModule::null();
    let mut set_layout = vk::DescriptorSetLayout::null();
    let mut pipeline_layout = vk::PipelineLayout::null();
    let mut pipeline = vk::Pipeline::null();
    let mut pixel_buffer = vk::Buffer::null();
    let mut pixel_memory = vk::DeviceMemory::null();
    let mut uniform_buffer = vk::Buffer::null();
    let mut uniform_memory = vk::DeviceMemory::null();
    let mut chunk_buffer = vk::Buffer::null();
    let mut chunk_memory = vk::DeviceMemory::null();
    let mut staging_buffer = vk::Buffer::null();
    let mut staging_memory = vk::DeviceMemory::null();
    let mut descriptor_pool = vk::DescriptorPool::null();
    let mut command_pool = vk::CommandPool::null();
    let mut fence = vk::Fence::null();
    let mut pixel_src_buffer = vk::Buffer::null();
    let mut pixel_src_memory = vk::DeviceMemory::null();

    let result = (|| -> Result<Vec<(u8, u8, u8)>, modern_colorthief_core::ColorthiefError> {
        // --- Shader module + pipeline layout (with descriptor set) + pipeline ---
        let spirv = shader_spirv()?;
        let shader_info = vk::ShaderModuleCreateInfo {
            code_size: spirv.len() * 4,
            p_code: spirv.as_ptr(),
            ..Default::default()
        };
        shader_module = unsafe { device.create_shader_module(&shader_info, None) }
            .map_err(|e| backend_err(e.to_string()))?;

        // Descriptor set layout matching the WGSL bindings:
        //   0: storage buffer (read)       — pixels
        //   1: uniform buffer              — params
        //   2: storage buffer (read_write) — chunk_colors
        let binding_layouts = [
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 2,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 1,
                stage_flags: vk::ShaderStageFlags::COMPUTE,
                ..Default::default()
            },
        ];
        let set_layout_info = vk::DescriptorSetLayoutCreateInfo {
            binding_count: binding_layouts.len() as u32,
            p_bindings: binding_layouts.as_ptr(),
            ..Default::default()
        };
        set_layout = unsafe { device.create_descriptor_set_layout(&set_layout_info, None) }
            .map_err(|e| backend_err(e.to_string()))?;

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo {
            set_layout_count: 1,
            p_set_layouts: &set_layout,
            ..Default::default()
        };
        pipeline_layout = unsafe { device.create_pipeline_layout(&pipeline_layout_info, None) }
            .map_err(|e| backend_err(e.to_string()))?;

        let stage_info = vk::PipelineShaderStageCreateInfo {
            stage: vk::ShaderStageFlags::COMPUTE,
            module: shader_module,
            p_name: c"main".as_ptr() as *const _,
            ..Default::default()
        };
        let compute_info = vk::ComputePipelineCreateInfo {
            stage: stage_info,
            layout: pipeline_layout,
            ..Default::default()
        };
        let pipelines = unsafe {
            device.create_compute_pipelines(vk::PipelineCache::default(), &[compute_info], None)
        }
        .map_err(|(_pipelines, err)| {
            backend_err(format!("Compute pipeline creation failed: {err:?}"))
        })?;
        pipeline = match pipelines.into_iter().next() {
            Some(p) => p,
            None => {
                return Err(backend_err(
                    "Compute pipeline creation returned no pipeline".to_string(),
                ));
            }
        };

        // --- Buffers ---
        let total_pixels = (buffer.len() / 4) as u32;
        let num_chunks = ((total_pixels as f64).sqrt().ceil() as u32).max(1) * 4;
        // Mirrors core_webgpu/src/helper.ts: Math.ceil(Math.sqrt(totalPixels)) * 4

        // Convert RGBA u8 to vec4<f32> in [0,1] for the shader.
        #[allow(clippy::chunks_exact_to_as_chunks)] // `array_chunks` is not yet stable
        let float_pixels: Vec<f32> = buffer
            .chunks_exact(4)
            .flat_map(|px| [px[0], px[1], px[2], px[3]].map(|v| v as f32 / 255.0))
            .collect();

        let params: [u32; 6] = [
            width,
            height,
            color_count as u32,
            quality.max(1) as u32,
            total_pixels,
            num_chunks,
        ];

        let pixel_size = (float_pixels.len() * 4).max(1) as u64;
        // In a Vulkan storage buffer, vec3<f32> has a stride of 16 bytes
        // (4 × 4-byte slots, one padding slot), not 12.
        let chunk_size = ((num_chunks as usize) * 16).max(1) as u64;
        let params_size = (params.len() * 4) as u64;

        // Pixel buffer: device-local + vkCmdUpdateBuffer (STORAGE_BUFFER-only
        // upload path, verified working on the RTX 5090).
        let (pb, pm) = create_buffer_with_memory(
            instance,
            physical_device,
            &device,
            pixel_size,
            vk::BufferUsageFlags::STORAGE_BUFFER,
        )?;
        pixel_buffer = pb;
        pixel_memory = pm;
        // Uniform buffer: host-visible so the params are uploaded via
        // map/write/unmap. vkCmdUpdateBuffer into host-visible memory left the
        // buffer stale from the GPU's perspective (verified: CPU readback correct,
        // shader echo all zeros), while map-based writes are GPU-visible.
        // (create_buffer_with_memory picks a HOST_VISIBLE type for any usage that
        // is not TRANSFER_DST; UNIFORM_BUFFER is host-visible by that rule.)
        let (ub, um) = create_buffer_with_memory(
            instance,
            physical_device,
            &device,
            params_size,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
        )?;
        uniform_buffer = ub;
        uniform_memory = um;
        let (cb, cm) = create_buffer_with_memory(
            instance,
            physical_device,
            &device,
            chunk_size,
            vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_SRC,
        )?;
        chunk_buffer = cb;
        chunk_memory = cm;
        let (sb, sm) = create_buffer_with_memory(
            instance,
            physical_device,
            &device,
            chunk_size,
            vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST,
        )?;
        staging_buffer = sb;
        staging_memory = sm;

        // --- Descriptor set ---
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 2,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 1,
            },
        ];
        let pool_info = vk::DescriptorPoolCreateInfo {
            max_sets: 1,
            pool_size_count: pool_sizes.len() as u32,
            p_pool_sizes: pool_sizes.as_ptr(),
            ..Default::default()
        };
        descriptor_pool = unsafe { device.create_descriptor_pool(&pool_info, None) }
            .map_err(|e| backend_err(e.to_string()))?;

        let alloc_sets_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool,
            descriptor_set_count: 1,
            p_set_layouts: &set_layout,
            ..Default::default()
        };
        let descriptor_sets = unsafe { device.allocate_descriptor_sets(&alloc_sets_info) }
            .map_err(|e| backend_err(e.to_string()))?;
        let descriptor_set = match descriptor_sets.into_iter().next() {
            Some(s) => s,
            None => {
                return Err(backend_err(
                    "Descriptor set allocation returned no sets".to_string(),
                ));
            }
        };

        let buffer_infos = [
            vk::DescriptorBufferInfo {
                buffer: pixel_buffer,
                offset: 0,
                range: vk::WHOLE_SIZE,
            },
            vk::DescriptorBufferInfo {
                buffer: uniform_buffer,
                offset: 0,
                range: vk::WHOLE_SIZE,
            },
            vk::DescriptorBufferInfo {
                buffer: chunk_buffer,
                offset: 0,
                range: vk::WHOLE_SIZE,
            },
        ];
        let write_descs = [
            vk::WriteDescriptorSet {
                dst_set: descriptor_set,
                dst_binding: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                p_buffer_info: &buffer_infos[0],
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_set,
                dst_binding: 1,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                p_buffer_info: &buffer_infos[1],
                ..Default::default()
            },
            vk::WriteDescriptorSet {
                dst_set: descriptor_set,
                dst_binding: 2,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::STORAGE_BUFFER,
                p_buffer_info: &buffer_infos[2],
                ..Default::default()
            },
        ];
        unsafe { device.update_descriptor_sets(&write_descs, &[]) };

        // --- Command pool + fence ---
        let cmd_pool_info = vk::CommandPoolCreateInfo {
            flags: vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER,
            queue_family_index: family_index,
            ..Default::default()
        };
        command_pool = unsafe { device.create_command_pool(&cmd_pool_info, None) }
            .map_err(|e| backend_err(e.to_string()))?;

        // Fence is passed to `queue_submit` for GPU-side completion tracking.
        // We synchronise on the CPU side with `device_wait_idle()` (which blocks
        // until all previously submitted commands finish) rather than
        // `wait_for_fences`, which has been observed to time out on this
        // platform even for a fence that should have been signalled.
        let fence_info = vk::FenceCreateInfo {
            flags: vk::FenceCreateFlags::empty(),
            ..Default::default()
        };
        fence = unsafe { device.create_fence(&fence_info, None) }
            .map_err(|e| backend_err(e.to_string()))?;

        let alloc_info = vk::CommandBufferAllocateInfo {
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
            command_buffer_count: 1,
            ..Default::default()
        };
        let begin_info = vk::CommandBufferBeginInfo {
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
            ..Default::default()
        };

        let pixel_bytes: Vec<u8> = float_pixels.iter().flat_map(|f| f.to_le_bytes()).collect();

        // --- Dispatch the compute pass (upload + dispatch in one command buffer) ---
        let cmd_buffers2 = unsafe { device.allocate_command_buffers(&alloc_info) }
            .map_err(|e| backend_err(e.to_string()))?;
        let cmd2 = match cmd_buffers2.into_iter().next() {
            Some(c) => c,
            None => return Err(backend_err("Command buffer allocation failed".to_string())),
        };

        // Upload the 24-byte params via map/write/unmap into the host-visible
        // uniform buffer BEFORE recording the command buffer (see comment at
        // buffer creation for why vkCmdUpdateBuffer is not used here).
        {
            let params_bytes: Vec<u8> = params.iter().flat_map(|v| v.to_le_bytes()).collect();
            let ptr = unsafe {
                device.map_memory(uniform_memory, 0, params_size, vk::MemoryMapFlags::empty())
            }
            .map_err(|e| backend_err(format!("map_memory (uniform) failed: {e:?}")))?;
            unsafe {
                std::ptr::copy_nonoverlapping(
                    params_bytes.as_ptr(),
                    ptr as *mut u8,
                    params_bytes.len(),
                );
            }
            unsafe { device.unmap_memory(uniform_memory) };
        }

        // Upload the pixels into a host-visible source buffer via map/write,
        // then transfer to the device-local pixel buffer with vkCmdCopyBuffer
        // (in the same command buffer, before dispatch, so ordering is
        // guaranteed). This replaces vkCmdUpdateBuffer, which is flaky on this
        // platform — some dispatches read stale (all-zero) pixel data, which
        // is the root cause of the flaky all-zero palettes observed in tests.
        let (psb, psm) = create_buffer_with_memory(
            instance,
            physical_device,
            &device,
            pixel_size,
            vk::BufferUsageFlags::TRANSFER_SRC | vk::BufferUsageFlags::TRANSFER_DST,
        )?;
        pixel_src_buffer = psb;
        pixel_src_memory = psm;
        {
            let ptr = unsafe {
                device.map_memory(pixel_src_memory, 0, pixel_size, vk::MemoryMapFlags::empty())
            }
            .map_err(|e| backend_err(format!("map_memory (pixel src) failed: {e:?}")))?;
            unsafe {
                std::ptr::copy_nonoverlapping(
                    pixel_bytes.as_ptr(),
                    ptr as *mut u8,
                    pixel_bytes.len(),
                );
            }
            unsafe { device.unmap_memory(pixel_src_memory) };
        }

        unsafe { device.begin_command_buffer(cmd2, &begin_info) }
            .map_err(|e| backend_err(e.to_string()))?;

        unsafe {
            // Transfer uploaded pixels into the device-local pixel buffer, then
            // dispatch — all in the same command buffer so the GPU sees the data
            // in-order on the same queue.
            let pixel_copy = vk::BufferCopy {
                src_offset: 0,
                dst_offset: 0,
                size: pixel_bytes.len() as u64,
            };
            device.cmd_copy_buffer(cmd2, pixel_src_buffer, pixel_buffer, &[pixel_copy]);
            // Barrier: TRANSFER write → SHADER read of the pixel buffer.
            // Without this the compute shader may read stale (un-copied) pixels.
            device.cmd_pipeline_barrier(
                cmd2,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::DependencyFlags::empty(),
                &[vk::MemoryBarrier {
                    src_access_mask: vk::AccessFlags::TRANSFER_WRITE,
                    dst_access_mask: vk::AccessFlags::SHADER_READ,
                    ..Default::default()
                }],
                &[],
                &[],
            );
            // Zero the output buffer so chunks whose workgroup never runs read
            // back as black; the black entries are filtered on CPU before dedup.
            device.cmd_fill_buffer(cmd2, chunk_buffer, 0, vk::WHOLE_SIZE, 0);
            device.cmd_bind_pipeline(cmd2, vk::PipelineBindPoint::COMPUTE, pipeline);
            device.cmd_bind_descriptor_sets(
                cmd2,
                vk::PipelineBindPoint::COMPUTE,
                pipeline_layout,
                0,
                &[descriptor_set],
                &[],
            );
            device.cmd_dispatch(cmd2, num_chunks.div_ceil(64), 1, 1);
            // Barrier: SHADER write → TRANSFER read of the chunk-color results.
            device.cmd_pipeline_barrier(
                cmd2,
                vk::PipelineStageFlags::COMPUTE_SHADER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[vk::MemoryBarrier {
                    src_access_mask: vk::AccessFlags::SHADER_WRITE,
                    dst_access_mask: vk::AccessFlags::TRANSFER_READ,
                    ..Default::default()
                }],
                &[],
                &[],
            );
        }

        // Copy the chunk-color results to the host-visible staging buffer so the
        // CPU can map it back after the compute pass.
        let copy_region = vk::BufferCopy {
            src_offset: 0,
            dst_offset: 0,
            size: (num_chunks as u64) * 16,
        };
        unsafe { device.cmd_copy_buffer(cmd2, chunk_buffer, staging_buffer, &[copy_region]) };

        unsafe { device.end_command_buffer(cmd2) }.map_err(|e| backend_err(e.to_string()))?;

        let submit_info2 = vk::SubmitInfo {
            command_buffer_count: 1,
            p_command_buffers: &cmd2,
            ..Default::default()
        };
        unsafe { device.queue_submit(queue, &[submit_info2], fence) }
            .map_err(|e| backend_err(e.to_string()))?;
        // Block until all previously submitted commands complete.
        unsafe { device.device_wait_idle() }
            .map_err(|e| backend_err(format!("Dispatch wait_idle failed: {e:?}")))?;

        // --- Read back chunk colors from the staging buffer (memory handle from creation) ---
        let read_size = (num_chunks as u64) * 16;
        let mapped_ptr =
            unsafe { device.map_memory(staging_memory, 0, read_size, vk::MemoryMapFlags::empty()) }
                .map_err(|e| backend_err(format!("map_memory failed: {e:?}")))?;

        let floats = unsafe {
            std::slice::from_raw_parts(mapped_ptr.cast::<f32>(), (num_chunks as usize) * 4)
        };

        let mut colors = Vec::with_capacity(num_chunks as usize);
        for i in 0..num_chunks as usize {
            let r = (floats[i * 4] * 255.0).round().clamp(0.0, 255.0) as u8;
            let g = (floats[i * 4 + 1] * 255.0).round().clamp(0.0, 255.0) as u8;
            let b = (floats[i * 4 + 2] * 255.0).round().clamp(0.0, 255.0) as u8;
            colors.push((r, g, b));
        }

        unsafe { device.unmap_memory(staging_memory) };
        Ok(colors)
    })();

    unsafe {
        device.device_wait_idle().ok();
        if pixel_src_buffer != vk::Buffer::null() {
            device.destroy_buffer(pixel_src_buffer, None);
        }
        if pixel_src_memory != vk::DeviceMemory::null() {
            device.free_memory(pixel_src_memory, None);
        }
        if fence != vk::Fence::null() {
            device.destroy_fence(fence, None);
        }
        if command_pool != vk::CommandPool::null() {
            device.destroy_command_pool(command_pool, None);
        }
        if descriptor_pool != vk::DescriptorPool::null() {
            device.destroy_descriptor_pool(descriptor_pool, None);
        }
        if staging_buffer != vk::Buffer::null() {
            device.destroy_buffer(staging_buffer, None);
        }
        if staging_memory != vk::DeviceMemory::null() {
            device.free_memory(staging_memory, None);
        }
        if chunk_buffer != vk::Buffer::null() {
            device.destroy_buffer(chunk_buffer, None);
        }
        if chunk_memory != vk::DeviceMemory::null() {
            device.free_memory(chunk_memory, None);
        }
        if uniform_buffer != vk::Buffer::null() {
            device.destroy_buffer(uniform_buffer, None);
        }
        if uniform_memory != vk::DeviceMemory::null() {
            device.free_memory(uniform_memory, None);
        }
        if pixel_buffer != vk::Buffer::null() {
            device.destroy_buffer(pixel_buffer, None);
        }
        if pixel_memory != vk::DeviceMemory::null() {
            device.free_memory(pixel_memory, None);
        }
        if pipeline != vk::Pipeline::null() {
            device.destroy_pipeline(pipeline, None);
        }
        if pipeline_layout != vk::PipelineLayout::null() {
            device.destroy_pipeline_layout(pipeline_layout, None);
        }
        if set_layout != vk::DescriptorSetLayout::null() {
            device.destroy_descriptor_set_layout(set_layout, None);
        }
        if shader_module != vk::ShaderModule::null() {
            device.destroy_shader_module(shader_module, None);
        }
        device.destroy_device(None);
    }

    result
}

pub fn gpu_extract(
    buffer: &[u8],
    width: u32,
    height: u32,
    color_count: u8,
    quality: u8,
) -> Result<Vec<crate::Rgb>, modern_colorthief_core::ColorthiefError> {
    modern_colorthief_core::validate_buffer(buffer, width, height)?;

    let gpus = list_gpus()?;
    if gpus.is_empty() {
        return Err(backend_err(
            "No Vulkan GPU with compute support found".to_string(),
        ));
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
        .cloned()
        .unwrap_or_else(|| (0..gpus.len()).collect());

    if gpu_indices.is_empty() {
        return Err(backend_err("No GPUs selected".to_string()));
    }

    let chunk_colors = dispatch_chunk_means(buffer, width, height, color_count, quality)?;

    if chunk_colors.is_empty() {
        return Err(backend_err("No colors extracted".to_string()));
    }

    // Drop zero-valued entries: the chunk buffer is zero-filled before
    // dispatch, so a (0,0,0) entry means the chunk's workgroup never wrote
    // (stale/uninitialized tail chunks on some drivers). A genuinely black
    // image is never all-zero because every chunk covers real pixels and
    // writes its mean — except when the buffer is truly black, in which case
    // we keep the first black entry to preserve the palette.
    let mut seen: Vec<(u8, u8, u8)> = Vec::new();
    let all_black = chunk_colors.iter().all(|c| *c == (0, 0, 0));
    for c in chunk_colors {
        if (c == (0, 0, 0)) && !all_black {
            continue;
        }
        if !seen.contains(&c) {
            seen.push(c);
        }
    }

    Ok(seen.into_iter().take(color_count as usize).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- vendor_name ---

    #[test]
    fn test_vendor_nvidia() {
        assert_eq!(vendor_name(0x10DE), "NVIDIA");
    }

    #[test]
    fn test_vendor_amd() {
        assert_eq!(vendor_name(0x1002), "AMD");
    }

    #[test]
    fn test_vendor_intel() {
        assert_eq!(vendor_name(0x8086), "Intel");
    }

    #[test]
    fn test_vendor_apple_moltenvk() {
        assert_eq!(vendor_name(0x106B), "Apple (MoltenVK)");
    }

    #[test]
    fn test_vendor_mesa() {
        assert_eq!(vendor_name(0x1234), "Mesa (llvmpipe/swrast)");
    }

    #[test]
    fn test_vendor_unknown() {
        assert_eq!(vendor_name(0xDEAD), "Unknown");
    }

    // --- has_compute_queue ---

    #[test]
    fn test_has_compute_queue_compute_flag() {
        let queues = vec![vk::QueueFamilyProperties {
            queue_flags: vk::QueueFlags::COMPUTE,
            ..Default::default()
        }];
        assert!(has_compute_queue(&queues));
    }

    #[test]
    fn test_has_compute_queue_graphics_flag() {
        let queues = vec![vk::QueueFamilyProperties {
            queue_flags: vk::QueueFlags::GRAPHICS,
            ..Default::default()
        }];
        assert!(has_compute_queue(&queues));
    }

    #[test]
    fn test_has_compute_queue_combined() {
        let queues = vec![vk::QueueFamilyProperties {
            queue_flags: vk::QueueFlags::GRAPHICS | vk::QueueFlags::COMPUTE,
            ..Default::default()
        }];
        assert!(has_compute_queue(&queues));
    }

    #[test]
    fn test_has_compute_queue_transfer_only() {
        let queues = vec![vk::QueueFamilyProperties {
            queue_flags: vk::QueueFlags::TRANSFER,
            ..Default::default()
        }];
        assert!(!has_compute_queue(&queues));
    }

    #[test]
    fn test_has_compute_queue_empty_flags() {
        let queues = vec![vk::QueueFamilyProperties {
            queue_flags: vk::QueueFlags::empty(),
            ..Default::default()
        }];
        assert!(!has_compute_queue(&queues));
    }

    #[test]
    fn test_has_compute_queue_empty_list() {
        let queues: Vec<vk::QueueFamilyProperties> = vec![];
        assert!(!has_compute_queue(&queues));
    }

    // --- GpuDevice mapping ---

    #[test]
    fn test_gpu_device_discrete() {
        assert_eq!(
            device_type_from_vk(vk::PhysicalDeviceType::DISCRETE_GPU),
            GpuDevice::Discrete
        );
    }

    #[test]
    fn test_gpu_device_integrated() {
        assert_eq!(
            device_type_from_vk(vk::PhysicalDeviceType::INTEGRATED_GPU),
            GpuDevice::Integrated
        );
    }

    #[test]
    fn test_gpu_device_virtual() {
        assert_eq!(
            device_type_from_vk(vk::PhysicalDeviceType::VIRTUAL_GPU),
            GpuDevice::Virtual
        );
    }

    #[test]
    fn test_gpu_device_cpu() {
        assert_eq!(
            device_type_from_vk(vk::PhysicalDeviceType::CPU),
            GpuDevice::CPU
        );
    }

    #[test]
    fn test_gpu_device_other() {
        assert_eq!(
            device_type_from_vk(vk::PhysicalDeviceType::OTHER),
            GpuDevice::Other
        );
    }

    // --- shader compilation (no GPU required) ---

    #[test]
    fn test_shader_compiles_to_spirv() {
        let spv = shader_spirv().expect("WGSL should compile to SPIR-V");
        assert!(!spv.is_empty());
        // SPIR-V magic number is 0x07230203.
        assert_eq!(spv[0], 0x0723_0203);
    }

    // --- gpu_extract edge cases ---

    #[test]
    fn test_extract_empty_buffer() {
        let result = gpu_extract(&[], 0, 0, 5, 1);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Empty"));
    }

    #[test]
    fn test_extract_insufficient_pixels() {
        // 1 pixel worth of data (4 bytes)
        let result = gpu_extract(&[255, 0, 0, 255], 1, 1, 5, 1);
        // May succeed or fail depending on GPU availability — both are valid.
        // Just verify it doesn't panic.
        let _ = result;
    }

    #[test]
    fn test_extract_solid_color() {
        // 4x4 solid red image (64 bytes). With a GPU present this should return
        // red as the dominant color; without one it errors — both are acceptable.
        let mut buf = vec![0u8; 16 * 4];
        for px in buf.chunks_mut(4) {
            px[0] = 255;
            px[3] = 255;
        }
        let result = gpu_extract(&buf, 4, 4, 5, 1);
        if let Ok(colors) = result {
            assert!(!colors.is_empty());
            // The dominant color of a solid red image must be (close to) red.
            let (r, g, b) = colors[0];
            assert!(r > 200);
            assert!(g < 60);
            assert!(b < 60);
        }
    }
}
