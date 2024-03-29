use crate::utility::InstanceDevices;
use ash::{util::Align, vk, Device};

pub(crate) fn find_memory_type(
    type_filter: u32,
    properties: vk::MemoryPropertyFlags,
    InstanceDevices { instance, devices }: &InstanceDevices,
) -> u32 {
    let mem_properties =
        unsafe { instance.get_physical_device_memory_properties(devices.physical.device) };

    for i in 0..mem_properties.memory_type_count {
        if ((1 << i) & type_filter) != 0
            && mem_properties.memory_types[i as usize].property_flags & properties == properties
        {
            return i;
        }
    }

    panic!("Failed to find suitable memory type!")
}

pub fn map_memory<T: Copy>(
    device: &Device,
    device_memory: vk::DeviceMemory,
    device_size: vk::DeviceSize,
    to_map: &[T],
) {
    unsafe {
        let data = device
            .map_memory(device_memory, 0, device_size, vk::MemoryMapFlags::empty())
            .expect("Failed to map memory");
        let mut vert_align = Align::new(
            data,
            std::mem::align_of::<T>().try_into().unwrap(),
            device_size,
        );
        vert_align.copy_from_slice(to_map);
        device.unmap_memory(device_memory);
    }
}
