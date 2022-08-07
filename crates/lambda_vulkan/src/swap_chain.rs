use crate::{
    command_buffer,
    device::{self, Devices},
    frame_buffer, renderer,
    resource::Resources,
    utility::InstanceDevices,
    Vulkan,
};
use ash::{
    extensions::khr::{Surface, Swapchain},
    vk::{self, PresentModeKHR, SurfaceCapabilitiesKHR, SurfaceFormatKHR},
};
use winit::window::Window;

#[derive(new)]
pub(crate) struct SwapChainSupport {
    capabilities: SurfaceCapabilitiesKHR,
    surface_formats: Vec<SurfaceFormatKHR>,
    present_modes: Vec<PresentModeKHR>,
}

#[derive(Clone)]
pub struct SwapChain {
    pub swap_chain: Swapchain,
    pub swap_chain_khr: vk::SwapchainKHR,
    pub image_format: vk::Format,
    pub extent: vk::Extent2D,
    pub images: Vec<vk::Image>,
    pub image_views: Vec<vk::ImageView>,
}

impl SwapChain {
    pub fn new(
        InstanceDevices { instance, devices }: &InstanceDevices,
        surface: vk::SurfaceKHR,
        surface_loader: &Surface,
        window: &Window,
    ) -> SwapChain {
        let SwapChainSupport {
            capabilities,
            surface_formats,
            present_modes,
        } = query_swap_chain_support(devices, surface, surface_loader);

        let surface_format = choose_swap_surface_format(&surface_formats);

        let present_mode = choose_present_mode(present_modes);

        let extent = choose_swap_extent(capabilities, window);

        let mut swap_chain_image_count = capabilities.min_image_count + 1;

        if capabilities.max_image_count > 0 && swap_chain_image_count > capabilities.max_image_count
        {
            swap_chain_image_count = capabilities.max_image_count;
        }

        let mut create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(swap_chain_image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let queue_family_indices =
            device::find_queue_family(instance, devices.physical.device, surface_loader, &surface);

        if queue_family_indices.graphics_family != queue_family_indices.present_family {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.queue_family_index_count = 2;

            let queue_family_indices_arr = [
                queue_family_indices.graphics_family.unwrap(),
                queue_family_indices.present_family.unwrap(),
            ];

            create_info.p_queue_family_indices = queue_family_indices_arr.as_ptr();
        }

        let swap_chain = Swapchain::new(instance, &devices.logical.device);

        unsafe {
            let swap_chain_khr = swap_chain
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain");

            let swap_chain_images = swap_chain
                .get_swapchain_images(swap_chain_khr)
                .expect("Could not get swapchain images");

            let image_views = create_image_views(devices, &swap_chain_images, &surface_format, 1);

            SwapChain {
                swap_chain,
                swap_chain_khr,
                images: swap_chain_images,
                image_format: surface_format.format,
                extent,
                image_views,
            }
        }
    }
}

fn create_image_views(
    devices: &Devices,
    swap_chain_images: &[vk::Image],
    surface_format: &vk::SurfaceFormatKHR,
    mip_levels: u32,
) -> Vec<vk::ImageView> {
    let mut swap_chain_image_views = vec![];

    let components = vk::ComponentMapping::builder()
        .r(vk::ComponentSwizzle::IDENTITY)
        .g(vk::ComponentSwizzle::IDENTITY)
        .b(vk::ComponentSwizzle::IDENTITY)
        .a(vk::ComponentSwizzle::IDENTITY);

    let sub_resource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);

    for &image in swap_chain_images.iter() {
        let image_view_create_info = vk::ImageViewCreateInfo::builder()
            .image(image)
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(surface_format.format)
            .components(*components)
            .subresource_range(*sub_resource_range);

        let image_view = unsafe {
            devices
                .logical
                .device
                .create_image_view(&image_view_create_info, None)
                .expect("Failed to create vk::Image View!")
        };

        swap_chain_image_views.push(image_view);
    }

    swap_chain_image_views
}

pub fn cleanup_swap_chain(vulkan: &Vulkan) {
    let device = &vulkan.instance_devices.devices.logical.device;

    unsafe {
        device.destroy_image_view(vulkan.resources.depth.view, None);
        device.destroy_image(vulkan.resources.depth.image.image, None);
        device.free_memory(vulkan.resources.depth.image.memory, None);

        device.destroy_image_view(vulkan.resources.colour.view, None);
        device.destroy_image(vulkan.resources.colour.image.image, None);
        device.free_memory(vulkan.resources.colour.image.memory, None);

        vulkan.frame_buffers.iter().for_each(|frame_buffer| {
            device.destroy_framebuffer(*frame_buffer, None);
        });

        vulkan.objects.0.iter().for_each(|object| {
            device.destroy_pipeline(object.graphics_pipeline.features.pipeline, None);
            device.destroy_pipeline_layout(object.graphics_pipeline.features.layout, None);
        });

        device.destroy_render_pass(vulkan.render_pass.0, None);

        vulkan.swap_chain.image_views.iter().for_each(|image_view| {
            device.destroy_image_view(*image_view, None);
        });

        vulkan
            .swap_chain
            .swap_chain
            .destroy_swapchain(vulkan.swap_chain.swap_chain_khr, None);
    }
}

pub fn recreate_swap_chain(vulkan: &mut Vulkan, window: &Window) {
    // let size = window.inner_size();
    // let _w = size.width;
    // let _h = size.height;

    let device = &vulkan.instance_devices.devices.logical.device;

    unsafe {
        device
            .device_wait_idle()
            .expect("Failed to wait for device idle!")
    };

    cleanup_swap_chain(vulkan);

    vulkan.swap_chain = SwapChain::new(
        &vulkan.instance_devices,
        vulkan.surface,
        &vulkan.surface_loader,
        window,
    );

    vulkan.render_pass = renderer::create_render_pass(&vulkan.instance_devices, &vulkan.swap_chain);

    vulkan.resources = Resources::new(&vulkan.swap_chain, &vulkan.instance_devices);

    vulkan.frame_buffers = frame_buffer::create_frame_buffers(
        &vulkan.swap_chain,
        &vulkan.render_pass,
        device,
        &vulkan.resources,
    );

    vulkan.sync_objects.images_in_flight = vec![vk::Fence::null(); 1];

    let models = Vec::new();

    vulkan.objects.0.iter().for_each(|object| {
        object.graphics_pipeline.recreate(
            &vulkan.instance_devices,
            &vulkan.swap_chain,
            vulkan.render_pass.0,
            &object.texture,
        );
    });

    vulkan.command_buffers = command_buffer::create_command_buffers(
        &vulkan.command_pool,
        &vulkan.swap_chain,
        &vulkan.instance_devices,
        &vulkan.render_pass,
        &vulkan.frame_buffers,
        &models,
        &mut vulkan.gui,
    );
}

pub(crate) fn query_swap_chain_support(
    devices: &Devices,
    surface: vk::SurfaceKHR,
    surface_loader: &Surface,
) -> SwapChainSupport {
    let capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(devices.physical.device, surface)
            .unwrap()
    };

    let formats = unsafe {
        surface_loader
            .get_physical_device_surface_formats(devices.physical.device, surface)
            .expect("Could not get Physical Device Surface Formats")
    };

    let present_modes = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(devices.physical.device, surface)
            .expect("Could not get Physical Device Present Modes")
    };

    SwapChainSupport::new(capabilities, formats, present_modes)
}

fn choose_swap_surface_format(formats: &[vk::SurfaceFormatKHR]) -> vk::SurfaceFormatKHR {
    for format in formats {
        if format.format == vk::Format::R8G8B8A8_SRGB
            && format.color_space == vk::ColorSpaceKHR::EXTENDED_SRGB_NONLINEAR_EXT
        {
            return *format;
        }
    }

    formats[0]
}

fn choose_present_mode(present_modes: Vec<vk::PresentModeKHR>) -> vk::PresentModeKHR {
    for present_mode in present_modes {
        if present_mode == vk::PresentModeKHR::MAILBOX {
            return present_mode;
        }
    }
    vk::PresentModeKHR::FIFO
}

fn choose_swap_extent(capabilities: vk::SurfaceCapabilitiesKHR, window: &Window) -> vk::Extent2D {
    if capabilities.current_extent.width != u32::MAX {
        capabilities.current_extent
    } else {
        let size = window.inner_size();

        vk::Extent2D {
            width: size.width.clamp(
                capabilities.min_image_extent.width,
                capabilities.max_image_extent.width,
            ),
            height: size.height.clamp(
                capabilities.min_image_extent.height,
                capabilities.max_image_extent.height,
            ),
        }
    }
}
