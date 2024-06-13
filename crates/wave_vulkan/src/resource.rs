use crate::{
    device::{Devices, PhysicalDeviceProperties},
    swap_chain::SwapChain,
    utility::{self, Image, ImageInfo},
};
use ash::{vk, Device, Instance};

#[derive(Clone, Copy, Debug)]
pub enum ResourceType {
    Colour,
    Depth,
}

#[derive(Clone)]
pub struct Resource {
    pub image: Image,
    pub view: vk::ImageView,
}

impl Resource {
    fn new(
        swap_chain: &SwapChain,
        image_type: ResourceType,
        instance: &Instance,
        devices: &Devices,
    ) -> Self {
        let (format, usage_flags, aspect_flags) = match image_type {
            ResourceType::Colour => (
                swap_chain.image_format,
                vk::ImageUsageFlags::COLOR_ATTACHMENT,
                vk::ImageAspectFlags::COLOR,
            ),
            ResourceType::Depth => (
                find_depth_format(instance, &devices.physical.device),
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                vk::ImageAspectFlags::DEPTH,
            ),
        };

        let image_info = ImageInfo::new(
            (swap_chain.extent.width, swap_chain.extent.height),
            1,
            devices.physical.samples,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | usage_flags,
            vk::MemoryPropertyFlags::LAZILY_ALLOCATED,
        );

        let image = utility::create_image(image_info, instance, devices);

        let view =
            utility::create_image_view(&image, format, aspect_flags, &devices.logical.device);

        Self { image, view }
    }
}

#[derive(Clone)]
pub struct Resources {
    pub colour: Resource,
    pub depth: Resource,
}

impl Resources {
    pub fn new(swap_chain: &SwapChain, instance: &Instance, devices: &Devices) -> Self {
        Self {
            depth: Resource::new(swap_chain, ResourceType::Depth, instance, devices),
            colour: Resource::new(swap_chain, ResourceType::Colour, instance, devices),
        }
    }
}

pub(crate) fn find_depth_format(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
) -> vk::Format {
    let candidates = [
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];
    find_supported_format(
        instance,
        physical_device,
        &candidates,
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}

fn find_supported_format(
    instance: &Instance,
    physical_device: &vk::PhysicalDevice,
    candidate_formats: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    for format in candidate_formats.iter() {
        let format_properties =
            unsafe { instance.get_physical_device_format_properties(*physical_device, *format) };

        if (tiling == vk::ImageTiling::LINEAR
            && (format_properties.linear_tiling_features & features) == features)
            || (tiling == vk::ImageTiling::OPTIMAL
                && (format_properties.optimal_tiling_features & features) == features)
        {
            return *format;
        }
    }

    panic!("Failed to find supported format!")
}
