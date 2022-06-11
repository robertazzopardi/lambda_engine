use crate::{
    swap_chain::SwapChain,
    utility::{self, Image, ImageInfo, InstanceDevices},
};
use ash::vk;

#[derive(Clone, Copy, Debug)]
pub enum ResourceType {
    Colour,
    Depth,
}

pub struct Resources {
    pub colour: Resource,
    pub depth: Resource,
}

impl Resources {
    pub fn new(swap_chain: &SwapChain, instance_devices: &InstanceDevices) -> Self {
        Self {
            depth: Resource::new(swap_chain, ResourceType::Depth, instance_devices),
            colour: Resource::new(swap_chain, ResourceType::Colour, instance_devices),
        }
    }
}

pub struct Resource {
    pub image: Image,
    pub view: vk::ImageView,
}

impl Resource {
    fn new(
        swap_chain: &SwapChain,
        image_type: ResourceType,
        instance_devices: &InstanceDevices,
    ) -> Self {
        let InstanceDevices { devices, .. } = instance_devices;

        let (format, usage_flags, aspect_flags) = match image_type {
            ResourceType::Colour => (
                swap_chain.image_format,
                vk::ImageUsageFlags::COLOR_ATTACHMENT,
                vk::ImageAspectFlags::COLOR,
            ),
            ResourceType::Depth => (
                find_depth_format(instance_devices),
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

        let image = utility::create_image(image_info, instance_devices);

        let view = utility::create_image_view(&image, format, aspect_flags, devices);

        Self { image, view }
    }
}

pub(crate) fn find_depth_format(instance_devices: &InstanceDevices) -> vk::Format {
    let candidates = [
        vk::Format::D32_SFLOAT,
        vk::Format::D32_SFLOAT_S8_UINT,
        vk::Format::D24_UNORM_S8_UINT,
    ];
    find_supported_format(
        instance_devices,
        &candidates,
        vk::ImageTiling::OPTIMAL,
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
    )
}

fn find_supported_format(
    InstanceDevices { instance, devices }: &InstanceDevices,
    candidate_formats: &[vk::Format],
    tiling: vk::ImageTiling,
    features: vk::FormatFeatureFlags,
) -> vk::Format {
    for format in candidate_formats.iter() {
        let format_properties = unsafe {
            instance.get_physical_device_format_properties(devices.physical.device, *format)
        };

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
