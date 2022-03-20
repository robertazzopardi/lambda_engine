use crate::{debug, device::Devices, memory};
use ash::{extensions::ext::DebugUtils, vk, Entry, Instance};
use std::ffi::CString;
use winit::window::Window;

#[derive(new)]
pub struct Image {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    #[new(value = "1")]
    pub mip_levels: u32,
}

impl Image {
    pub fn mip_levels(mut self, mip_levels: u32) -> Self {
        self.mip_levels = mip_levels;
        self
    }
}

#[derive(new)]
pub(crate) struct ImageInfo {
    dimensions: (u32, u32),
    mip_levels: u32,
    samples: vk::SampleCountFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
}

#[derive(new)]
pub struct InstanceDevices<'a> {
    pub instance: &'a Instance,
    pub devices: &'a Devices,
}

pub(crate) struct EntryInstance {
    pub entry: Entry,
    pub instance: Instance,
}

impl EntryInstance {
    pub(crate) fn new(window: &Window) -> Self {
        if debug::enable_validation_layers() && !debug::check_validation_layer_support(window) {
            panic!("Validation layers requested, but not available!")
        }

        let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
        let layers_names_raw: Vec<*const i8> = layer_names
            .iter()
            .map(|raw_name| raw_name.as_ptr())
            .collect();

        let surface_extensions = ash_window::enumerate_required_extensions(window).unwrap();
        let mut extension_names_raw = surface_extensions
            .iter()
            .map(|ext| ext.as_ptr())
            .collect::<Vec<_>>();
        extension_names_raw.push(DebugUtils::name().as_ptr());

        let app_name = CString::new("Vulkan").unwrap();
        let engine_name = CString::new("No Engine").unwrap();

        let app_info = vk::ApplicationInfo::builder()
            .application_name(&app_name)
            .application_version(0)
            .engine_name(&engine_name)
            .engine_version(0)
            .api_version(vk::API_VERSION_1_1);

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_layer_names(&layers_names_raw)
            .enabled_extension_names(&extension_names_raw);

        unsafe {
            let entry = Entry::load().unwrap();
            let instance: Instance = entry
                .create_instance(&create_info, None)
                .expect("Instance creation error");

            Self { instance, entry }
        }
    }
}

pub(crate) fn create_image(info: ImageInfo, instance_devices: &InstanceDevices) -> Image {
    let InstanceDevices { devices, .. } = instance_devices;

    let image_info = vk::ImageCreateInfo::builder()
        .image_type(vk::ImageType::TYPE_2D)
        .extent(vk::Extent3D {
            width: info.dimensions.0,
            height: info.dimensions.1,
            depth: 1,
        })
        .mip_levels(info.mip_levels)
        .array_layers(1)
        .format(info.format)
        .tiling(info.tiling)
        .initial_layout(vk::ImageLayout::UNDEFINED)
        .usage(info.usage)
        .samples(info.samples)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    unsafe {
        let image = devices
            .logical
            .device
            .create_image(&image_info, None)
            .expect("Failed to create image!");

        let memory_requirements = devices.logical.device.get_image_memory_requirements(image);

        let alloc_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.size,
            memory_type_index: memory::find_memory_type(
                memory_requirements.memory_type_bits,
                info.properties,
                instance_devices,
            ),
            ..Default::default()
        };

        let image_memory = devices
            .logical
            .device
            .allocate_memory(&alloc_info, None)
            .expect("Failed to allocate image memory!");

        devices
            .logical
            .device
            .bind_image_memory(image, image_memory, 0)
            .expect("Failed to bind image memory");

        Image::new(image, image_memory)
    }
}

pub(crate) fn create_image_view(
    image: &Image,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
    devices: &Devices,
) -> vk::ImageView {
    let sub_resource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(aspect_mask)
        .base_mip_level(0)
        .level_count(image.mip_levels)
        .base_array_layer(0)
        .layer_count(1)
        .build();

    let image_view_info = vk::ImageViewCreateInfo::builder()
        .image(image.image)
        .view_type(vk::ImageViewType::TYPE_2D)
        .format(format)
        .subresource_range(sub_resource_range);

    unsafe {
        devices
            .logical
            .device
            .create_image_view(&image_view_info, None)
            .expect("Failed to create textured image view!")
    }
}
