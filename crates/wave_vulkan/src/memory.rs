use ash::{vk, Instance};

pub(crate) fn find_memory_type(
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
    instance: &Instance,
    device: &vk::PhysicalDevice,
) -> u32 {
    let mem_properties = unsafe { instance.get_physical_device_memory_properties(*device) };

    for i in 0..mem_properties.memory_type_count {
        if (type_filter & (1 << i)) != 0
            && (mem_properties.memory_types[i as usize].property_flags & properties) == properties
        {
            return i;
        }
    }

    panic!("Failed to find suitable memory type!")
}
