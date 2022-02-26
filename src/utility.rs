use crate::{debug, device::Devices, memory, swapchain::SwapChain, types::Tuple};
use ash::{extensions::ext::DebugUtils, vk, Device, Entry, Instance};
use std::ffi::CString;
use winit::window::Window;

pub fn create_instance(window: &Window) -> (Instance, Entry) {
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
        .api_version(vk::make_api_version(0, 1, 0, 0));

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&extension_names_raw);

    unsafe {
        let entry = Entry::load().unwrap();
        let instance: Instance = entry
            .create_instance(&create_info, None)
            .expect("Instance creation error");

        (instance, entry)
    }
}

pub(crate) fn create_image(
    Tuple(w, h): Tuple<u32, u32>,
    mip_levels: u32,
    samples: vk::SampleCountFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
    devices: &Devices,
    instance: &Instance,
) -> (vk::Image, vk::DeviceMemory) {
    let image_info = vk::ImageCreateInfo {
        s_type: vk::StructureType::IMAGE_CREATE_INFO,
        image_type: vk::ImageType::TYPE_2D,
        extent: vk::Extent3D {
            width: w,
            height: h,
            depth: 1,
        },
        mip_levels,
        array_layers: 1,
        format,
        tiling,
        initial_layout: vk::ImageLayout::UNDEFINED,
        usage,
        samples,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    unsafe {
        let image = devices
            .logical
            .create_image(&image_info, None)
            .expect("Faild to create image!");

        let memory_requirements = devices.logical.get_image_memory_requirements(image);

        let alloc_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.size,
            memory_type_index: memory::find_memory_type(
                instance,
                devices,
                memory_requirements.memory_type_bits,
                properties,
            ),
            ..Default::default()
        };

        let image_memory = devices
            .logical
            .allocate_memory(&alloc_info, None)
            .expect("Failed to allocate image memory!");

        devices
            .logical
            .bind_image_memory(image, image_memory, 0)
            .expect("Failed to bind image memory");

        (image, image_memory)
    }
}

pub(crate) fn create_image_view(
    image: vk::Image,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
    level_count: u32,
    devices: &Devices,
) -> vk::ImageView {
    let image_view_info = vk::ImageViewCreateInfo {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
        image,
        view_type: vk::ImageViewType::TYPE_2D,
        format,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count,
            base_array_layer: 0,
            layer_count: 1,
        },
        ..Default::default()
    };

    unsafe {
        devices
            .logical
            .create_image_view(&image_view_info, None)
            .expect("Failed to create textured image view!")
    }
}

pub fn create_frame_buffers(
    swapchain: &SwapChain,
    depth_image_view: vk::ImageView,
    render_pass: vk::RenderPass,
    device: &Device,
    color_resource: vk::ImageView,
) -> Vec<vk::Framebuffer> {
    let mut frame_buffers = Vec::new();

    for i in 0..swapchain.images.len() {
        let attachments = &[color_resource, depth_image_view, swapchain.image_views[i]];

        let frame_buffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(attachments)
            .width(swapchain.extent.width)
            .height(swapchain.extent.height)
            .layers(1);

        unsafe {
            frame_buffers.push(
                device
                    .create_framebuffer(&frame_buffer_info, None)
                    .expect("Failed to create Frame Buffer!"),
            );
        }
    }

    frame_buffers
}
