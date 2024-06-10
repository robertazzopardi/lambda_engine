use crate::{
    debug::{self, vulkan_debug_callback, Debugger, ENABLE_VALIDATION_LAYERS, VALIDATION_LAYERS},
    device::Devices,
    memory,
};
use ash::{
    vk::{self, EXT_DEBUG_UTILS_NAME},
    Device, Entry, Instance,
};
use std::ffi::CString;
use winit::{raw_window_handle::HasDisplayHandle, window::Window};

#[derive(Default, Debug, Clone)]
pub struct Image {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub mip_levels: u32,
}

impl Image {
    pub fn new(image: vk::Image, memory: vk::DeviceMemory) -> Self {
        Self {
            image,
            memory,
            mip_levels: 1,
        }
    }

    pub fn mip_levels(mut self, mip_levels: u32) -> Self {
        self.mip_levels = mip_levels;
        self
    }
}

#[derive(Debug)]
pub struct ImageInfo {
    dimensions: (u32, u32),
    mip_levels: u32,
    samples: vk::SampleCountFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
}

impl ImageInfo {
    pub fn new(
        dimensions: (u32, u32),
        mip_levels: u32,
        samples: vk::SampleCountFlags,
        format: vk::Format,
        tiling: vk::ImageTiling,
        usage: vk::ImageUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> Self {
        Self {
            dimensions,
            mip_levels,
            samples,
            format,
            tiling,
            usage,
            properties,
        }
    }
}

#[derive(Clone)]
pub struct InstanceDevices {
    pub instance: Instance,
    pub devices: Devices,
}

pub struct EntryInstance {
    pub entry: Entry,
    pub instance: Instance,
}

impl EntryInstance {
    pub fn new(window: &Window, debugging: Option<Debugger>) -> Self {
        let layer_names = VALIDATION_LAYERS
            .iter()
            .map(|layer_name| CString::new(*layer_name).unwrap())
            .collect::<Vec<CString>>();
        let layers_names_raw: Vec<*const i8> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let surface_extensions =
            ash_window::enumerate_required_extensions(window.display_handle().unwrap().into())
                .unwrap();
        let mut extension_names_raw = surface_extensions.to_vec();

        if ENABLE_VALIDATION_LAYERS {
            extension_names_raw.push(EXT_DEBUG_UTILS_NAME.as_ptr());
        }
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        {
            extension_names_raw.push(vk::KHR_PORTABILITY_ENUMERATION_NAME.as_ptr());
            // Enabling this extension is a requirement when using `VK_KHR_portability_subset`
            extension_names_raw.push(vk::KHR_GET_PHYSICAL_DEVICE_PROPERTIES2_NAME.as_ptr());
        }

        let app_name = CString::new("Vulkan").unwrap();
        let engine_name = CString::new("No Engine").unwrap();

        let app_info = vk::ApplicationInfo::default()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&engine_name)
            .engine_version(0)
            .api_version(vk::API_VERSION_1_3);

        unsafe {
            let entry = Entry::load().unwrap();

            if ENABLE_VALIDATION_LAYERS && !debug::check_validation_layer_support(&entry) {
                panic!("Validation layers requested, but not available!")
            }

            let mut debug_create_info: vk::DebugUtilsMessengerCreateInfoEXT;

            let create_flags = if cfg!(any(target_os = "macos", target_os = "ios")) {
                vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR
            } else {
                vk::InstanceCreateFlags::default()
            };

            let mut create_info = vk::InstanceCreateInfo::default()
                .application_info(&app_info)
                .flags(create_flags)
                .enabled_extension_names(extension_names_raw.as_slice());

            if ENABLE_VALIDATION_LAYERS {
                if let Some(debugging) = debugging {
                    debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::default()
                        .message_severity(debugging.message_level.flags)
                        .message_type(debugging.message_type.flags)
                        .pfn_user_callback(Some(vulkan_debug_callback));
                    create_info = create_info
                        .enabled_layer_names(&layers_names_raw)
                        .push_next(&mut debug_create_info);
                }
            }

            let instance = entry
                .create_instance(&create_info, None)
                .expect("Instance creation error");

            Self { instance, entry }
        }
    }
}

pub(crate) fn create_image(info: ImageInfo, instance_devices: &InstanceDevices) -> Image {
    let device = &instance_devices.devices.logical.device;

    let image_info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(
            vk::Extent3D::default()
                .width(info.dimensions.0)
                .height(info.dimensions.1)
                .depth(1),
        )
        .mip_levels(info.mip_levels)
        .array_layers(1)
        .format(info.format)
        .tiling(info.tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(info.usage)
        .samples(info.samples)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    unsafe {
        let image = device
            .create_image(&image_info, None)
            .expect("Failed to create image!");

        let memory_requirements = device.get_image_memory_requirements(image);

        let memory_type_index = memory::find_memory_type(
            memory_requirements.memory_type_bits,
            info.properties,
            instance_devices,
        );
        let alloc_info = vk::MemoryAllocateInfo::default()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index);

        let image_memory = device
            .allocate_memory(&alloc_info, None)
            .expect("Failed to allocate image memory!");

        device
            .bind_image_memory(image, image_memory, 0)
            .expect("Failed to bind image memory");

        Image::new(image, image_memory)
    }
}

pub(crate) fn create_image_view(
    image: &Image,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
    device: &Device,
) -> vk::ImageView {
    let sub_resource_range = vk::ImageSubresourceRange::default()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(image.mip_levels)
        .base_array_layer(0)
        .layer_count(1);

    let image_view_info = vk::ImageViewCreateInfo::default()
        .image(image.image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(sub_resource_range);

    unsafe {
        device
            .create_image_view(&image_view_info, None)
            .expect("Failed to create textured image view!")
    }
}
