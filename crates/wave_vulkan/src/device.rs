use crate::{
    graphics_pipeline::GraphicsPipeline, sync_objects::MAX_FRAMES_IN_FLIGHT, VulkanObject,
};
use ash::{khr::surface, vk, Device, Instance};
use gpu_allocator::vulkan::Allocator;

#[derive(Debug, Clone)]
pub struct PhysicalDeviceProperties {
    pub device: vk::PhysicalDevice,
    pub queue_family_index: u32,
    pub samples: vk::SampleCountFlags,
}

impl PhysicalDeviceProperties {
    pub fn new(
        device: vk::PhysicalDevice,
        queue_family_index: u32,
        samples: vk::SampleCountFlags,
    ) -> Self {
        Self {
            device,
            queue_family_index,
            samples,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Queues {
    pub present: vk::Queue,
    pub graphics: vk::Queue,
}

impl Queues {
    pub fn new(present: vk::Queue, graphics: vk::Queue) -> Self {
        Self { present, graphics }
    }
}

#[derive(Clone)]
pub struct LogicalDeviceFeatures {
    pub device: Device,
    pub queues: Queues,
}

impl LogicalDeviceFeatures {
    pub fn new(device: Device, queues: Queues) -> Self {
        Self { device, queues }
    }
}

#[derive(Default, Debug)]
pub(crate) struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub const fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
}

#[derive(Clone)]
pub struct Devices {
    pub physical: PhysicalDeviceProperties,
    pub logical: LogicalDeviceFeatures,
}

impl Devices {
    pub fn new(
        instance: &Instance,
        surface: &vk::SurfaceKHR,
        surface_loader: &surface::Instance,
    ) -> Self {
        let physical = pick_physical_device(instance, surface, surface_loader);

        let logical = create_logical_device(instance, &physical, surface, surface_loader);

        Self { physical, logical }
    }
}

pub(crate) fn find_queue_family(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
    surface_loader: &surface::Instance,
    surface: &vk::SurfaceKHR,
) -> QueueFamilyIndices {
    let queue_families =
        unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

    let mut queue_family_indices = QueueFamilyIndices::default();

    for (index, queue_family) in queue_families.iter().enumerate() {
        if queue_family.queue_count > 0
            && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
        {
            queue_family_indices.graphics_family = Some(index.try_into().unwrap());
        }

        let is_present_support = unsafe {
            surface_loader.get_physical_device_surface_support(
                physical_device,
                index as u32,
                *surface,
            )
        }
        .unwrap();

        if queue_family.queue_count > 0 && is_present_support {
            queue_family_indices.present_family = Some(index.try_into().unwrap());
        }

        if queue_family_indices.is_complete() {
            break;
        }
    }

    queue_family_indices
}

fn create_logical_device(
    instance: &Instance,
    physical_device_properties: &PhysicalDeviceProperties,
    surface: &vk::SurfaceKHR,
    surface_loader: &surface::Instance,
) -> LogicalDeviceFeatures {
    let PhysicalDeviceProperties {
        device,
        queue_family_index,
        ..
    } = physical_device_properties;

    // let portability_subset_extension = CString::new("VK_KHR_portability_subset").unwrap();
    // let device_extension_names_raw = [
    //     Swapchain::name().as_ptr(),
    //     portability_subset_extension.as_ptr(),
    // ];

    let features = vk::PhysicalDeviceFeatures::default()
        .sampler_anisotropy(true)
        .sample_rate_shading(true)
        .fill_mode_non_solid(true)
        .shader_clip_distance(true);

    let priorities = 1.0;

    let queue_info = vk::DeviceQueueCreateInfo::default()
        .queue_family_index(*queue_family_index)
        .queue_priorities(std::slice::from_ref(&priorities));

    let device_extension_names_raw = [
        vk::KHR_SWAPCHAIN_NAME.as_ptr(),
        #[cfg(any(target_os = "macos", target_os = "ios"))]
        vk::KHR_PORTABILITY_SUBSET_NAME.as_ptr(),
    ];

    let device_create_info = vk::DeviceCreateInfo::default()
        .queue_create_infos(std::slice::from_ref(&queue_info))
        .enabled_extension_names(&device_extension_names_raw)
        .enabled_features(&features);

    let queue_family = find_queue_family(instance, *device, surface_loader, surface);

    unsafe {
        let device = instance
            .create_device(*device, &device_create_info, None)
            .expect("Could not create device");

        let graphics_queue = device.get_device_queue(queue_family.graphics_family.unwrap(), 0);
        let present_queue = device.get_device_queue(queue_family.present_family.unwrap(), 0);

        LogicalDeviceFeatures::new(device, Queues::new(present_queue, graphics_queue))
    }
}

fn pick_physical_device(
    instance: &Instance,
    surface: &vk::SurfaceKHR,
    surface_loader: &surface::Instance,
) -> PhysicalDeviceProperties {
    unsafe {
        let devices = instance
            .enumerate_physical_devices()
            .expect("Failed to find GPUs with Vulkan support!");

        let (physical_device, queue_family_index) = devices
            .iter()
            .find_map(|p_device| {
                instance
                    .get_physical_device_queue_family_properties(*p_device)
                    .iter()
                    .enumerate()
                    .find_map(|(index, info)| {
                        let supports_graphic_and_surface =
                            info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                && surface_loader
                                    .get_physical_device_surface_support(
                                        *p_device,
                                        index as u32,
                                        *surface,
                                    )
                                    .unwrap();
                        if supports_graphic_and_surface {
                            Some((*p_device, index))
                        } else {
                            None
                        }
                    })
            })
            .expect("Couldn't find suitable device.");

        let samples = get_max_usable_sample_count(instance, physical_device);

        PhysicalDeviceProperties::new(physical_device, queue_family_index as u32, samples)
    }
}

fn get_max_usable_sample_count(
    instance: &Instance,
    physical_device: vk::PhysicalDevice,
) -> vk::SampleCountFlags {
    unsafe {
        let physical_device_properties = instance.get_physical_device_properties(physical_device);

        let counts = physical_device_properties
            .limits
            .framebuffer_color_sample_counts
            & physical_device_properties
                .limits
                .framebuffer_depth_sample_counts;

        if (counts & vk::SampleCountFlags::TYPE_64) == vk::SampleCountFlags::TYPE_64 {
            return vk::SampleCountFlags::TYPE_64;
        }
        if (counts & vk::SampleCountFlags::TYPE_32) == vk::SampleCountFlags::TYPE_32 {
            return vk::SampleCountFlags::TYPE_32;
        }
        if (counts & vk::SampleCountFlags::TYPE_16) == vk::SampleCountFlags::TYPE_16 {
            return vk::SampleCountFlags::TYPE_16;
        }
        if (counts & vk::SampleCountFlags::TYPE_8) == vk::SampleCountFlags::TYPE_8 {
            return vk::SampleCountFlags::TYPE_8;
        }
        if (counts & vk::SampleCountFlags::TYPE_4) == vk::SampleCountFlags::TYPE_4 {
            return vk::SampleCountFlags::TYPE_4;
        }
        if (counts & vk::SampleCountFlags::TYPE_2) == vk::SampleCountFlags::TYPE_2 {
            return vk::SampleCountFlags::TYPE_2;
        }

        vk::SampleCountFlags::TYPE_1
    }
}

/// # Safety
///
///
pub unsafe fn recreate_drop(allocator: &mut Allocator, object: VulkanObject, device: &Device) {
    for i in 0..MAX_FRAMES_IN_FLIGHT {
        // device.destroy_buffer(graphics_pipeline.uniform_buffers[i].buffer, None);
        // device.free_memory(graphics_pipeline.uniform_buffers[i].memory, None);
    }

    for buffer in object.graphics_pipeline.uniform_buffers {
        allocator.free(buffer.allocation).unwrap();
        unsafe { device.destroy_buffer(buffer.buffer, None) };
    }

    device.destroy_descriptor_pool(object.graphics_pipeline.descriptors.pool, None);
}

/// # Safety
///
///
pub(crate) unsafe fn destroy(allocator: &mut Allocator, object: VulkanObject, device: &Device) {
    for i in 0..MAX_FRAMES_IN_FLIGHT {
        // device.destroy_buffer(graphics_pipeline.uniform_buffers[i].buffer, None);
        // device.free_memory(graphics_pipeline.uniform_buffers[i].memory, None);
    }

    for buffer in object.graphics_pipeline.uniform_buffers {
        allocator.free(buffer.allocation).unwrap();
        unsafe { device.destroy_buffer(buffer.buffer, None) };
    }

    device.destroy_descriptor_pool(object.graphics_pipeline.descriptors.pool, None);

    if let Some(object_texture) = &object.texture {
        device.destroy_sampler(object_texture.sampler, None);
        device.destroy_image_view(object_texture.view, None);

        device.destroy_image(object_texture.image.image, None);
        device.free_memory(object_texture.image.memory, None);
    }

    device.destroy_descriptor_set_layout(object.graphics_pipeline.descriptors.set_layout, None);

    // device.destroy_buffer(object.buffers.index.buffer, None);
    // device.free_memory(object.buffers.index.memory, None);

    allocator.free(object.buffers.index.allocation).unwrap();
    unsafe { device.destroy_buffer(object.buffers.index.buffer, None) };

    // device.destroy_buffer(object.buffers.vertex.buffer, None);
    // device.free_memory(object.buffers.vertex.memory, None);

    allocator.free(object.buffers.vertex.allocation).unwrap();
    unsafe { device.destroy_buffer(object.buffers.vertex.buffer, None) };
}
