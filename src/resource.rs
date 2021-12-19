use crate::{Devices, SwapChain, Vulkan};
use ash::{vk, Instance};

pub enum ResourceType {
    Colour,
    Depth,
}

pub struct Resource {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub view: vk::ImageView,
}

impl Resource {
    pub fn create_resource(
        devices: &Devices,
        swapchain: &SwapChain,
        instance: &Instance,
        image_type: ResourceType,
    ) -> Self {
        let (format, usage_flags, aspect_flags) = match image_type {
            ResourceType::Colour => (
                swapchain.image_format,
                vk::ImageUsageFlags::COLOR_ATTACHMENT,
                vk::ImageAspectFlags::COLOR,
            ),
            ResourceType::Depth => (
                unsafe { Self::find_depth_format(instance, &devices.physical) },
                vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
                vk::ImageAspectFlags::DEPTH,
            ),
        };

        let (image, memory) = Vulkan::create_image(
            swapchain.extent.width,
            swapchain.extent.height,
            1,
            devices.msaa_samples,
            format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | usage_flags,
            vk::MemoryPropertyFlags::LAZILY_ALLOCATED,
            devices,
            instance,
        );

        let view = Vulkan::create_image_view(image, format, aspect_flags, 1, devices);

        Self {
            image,
            memory,
            view,
        }
    }

    pub unsafe fn find_depth_format(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
    ) -> vk::Format {
        let candidates = [
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];
        Self::find_supported_format(
            instance,
            *physical_device,
            &candidates,
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    fn find_supported_format(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        candidate_formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> vk::Format {
        for format in candidate_formats.iter() {
            let format_properties =
                unsafe { instance.get_physical_device_format_properties(physical_device, *format) };

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
}
