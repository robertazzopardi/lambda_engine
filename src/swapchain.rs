use crate::device::{self, Devices};
use ash::{
    extensions::khr::{Surface, Swapchain},
    vk::{self, PresentModeKHR, SurfaceCapabilitiesKHR, SurfaceFormatKHR},
    Instance,
};
use winit::window::Window;

pub struct SwapChain {
    pub loader: Swapchain,
    pub swapchain: vk::SwapchainKHR,
    pub images: Vec<vk::Image>,
    pub image_format: vk::Format,
    pub extent: vk::Extent2D,
    pub image_views: Vec<vk::ImageView>,
}

impl SwapChain {
    pub fn new(
        instance: &Instance,
        devices: &Devices,
        surface: vk::SurfaceKHR,
        surface_loader: &Surface,
        window: &Window,
    ) -> SwapChain {
        let (capabilities, formats, present_modes) =
            query_swapchain_support(devices, surface, surface_loader);

        let surface_format = choose_swap_surface_format(&formats);

        let present_mode = choose_present_mode(present_modes);

        let extent = choose_swap_extent(capabilities, window);

        let mut swapchain_image_count = capabilities.min_image_count + 1;

        if capabilities.max_image_count > 0 && swapchain_image_count > capabilities.max_image_count
        {
            swapchain_image_count = capabilities.max_image_count;
        }

        let mut create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(swapchain_image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(vk::SwapchainKHR::null());

        let queue_family_indices =
            device::find_queue_family(instance, devices.physical, surface_loader, &surface);

        if queue_family_indices.graphics_family != queue_family_indices.present_family {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.queue_family_index_count = 2;

            let queue_family_indices_arr = [
                queue_family_indices.graphics_family.unwrap(),
                queue_family_indices.present_family.unwrap(),
            ];

            create_info.p_queue_family_indices = queue_family_indices_arr.as_ptr();
        } else {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        }

        let swapchain = Swapchain::new(instance, &devices.logical);

        unsafe {
            let swapchain_khr = swapchain
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain");

            let swapchain_images = swapchain
                .get_swapchain_images(swapchain_khr)
                .expect("Could not get swapchain images");

            let image_views = create_image_views(devices, &swapchain_images, &surface_format, 1);

            SwapChain {
                loader: swapchain,
                swapchain: swapchain_khr,
                images: swapchain_images,
                image_format: surface_format.format,
                extent,
                image_views,
            }
        }
    }
}

fn create_image_views(
    devices: &Devices,
    swapchain_images: &[vk::Image],
    surface_format: &vk::SurfaceFormatKHR,
    mip_levels: u32,
) -> Vec<vk::ImageView> {
    let mut swapchain_imageviews = vec![];

    for &image in swapchain_images.iter() {
        let imageview_create_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            image,
            view_type: vk::ImageViewType::TYPE_2D,
            format: surface_format.format,
            components: vk::ComponentMapping {
                r: vk::ComponentSwizzle::IDENTITY,
                g: vk::ComponentSwizzle::IDENTITY,
                b: vk::ComponentSwizzle::IDENTITY,
                a: vk::ComponentSwizzle::IDENTITY,
            },
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            },
            ..Default::default()
        };

        let imageview = unsafe {
            devices
                .logical
                .create_image_view(&imageview_create_info, None)
                .expect("Failed to create vk::Image View!")
        };
        swapchain_imageviews.push(imageview);
    }

    swapchain_imageviews
}

pub fn query_swapchain_support(
    devices: &Devices,
    surface: vk::SurfaceKHR,
    surface_loader: &Surface,
) -> (
    SurfaceCapabilitiesKHR,
    Vec<SurfaceFormatKHR>,
    Vec<PresentModeKHR>,
) {
    let capabilities = unsafe {
        surface_loader
            .get_physical_device_surface_capabilities(devices.physical, surface)
            .unwrap()
    };

    let formats = unsafe {
        surface_loader
            .get_physical_device_surface_formats(devices.physical, surface)
            .expect("Could not get Physical Device Surface Formats")
    };

    let present_modes = unsafe {
        surface_loader
            .get_physical_device_surface_present_modes(devices.physical, surface)
            .expect("Could not get Physical Device Present Modes")
    };

    (capabilities, formats, present_modes)
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
