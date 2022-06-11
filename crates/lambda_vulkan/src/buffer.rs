use crate::{command_buffer, device::Devices, memory, texture, utility::InstanceDevices};
use ash::vk;
use lambda_space::space::{Vertex, VerticesAndIndices};
use std::mem::size_of;

#[derive(new, Default, Debug)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

#[derive(Default, Debug)]
pub struct ModelBuffers {
    pub vertex: Buffer,
    pub index: Buffer,
}

impl ModelBuffers {
    pub fn new(
        vertices_and_indices: &VerticesAndIndices,
        command_pool: &vk::CommandPool,
        command_buffer_count: u32,
        instance_devices: &InstanceDevices,
    ) -> Self {
        let vertex = create_vertex_index_buffer(
            (size_of::<Vertex>() * vertices_and_indices.vertices.len())
                .try_into()
                .unwrap(),
            &vertices_and_indices.vertices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        let index = create_vertex_index_buffer(
            (size_of::<u16>() * vertices_and_indices.indices.len())
                .try_into()
                .unwrap(),
            &vertices_and_indices.indices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        ModelBuffers { vertex, index }
    }
}

pub(crate) fn create_vertex_index_buffer<T: Copy>(
    buffer_size: u64,
    data: &[T],
    usage_flags: vk::BufferUsageFlags,
    command_pool: &vk::CommandPool,
    command_buffer_count: u32,
    instance_devices: &InstanceDevices,
) -> Buffer {
    let InstanceDevices { devices, .. } = instance_devices;

    let staging = texture::create_buffer(
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        instance_devices,
    );

    memory::map_memory(&devices.logical.device, staging.memory, buffer_size, data);

    let buffer = texture::create_buffer(
        buffer_size,
        usage_flags,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
        instance_devices,
    );

    copy_buffer(
        devices,
        command_pool,
        command_buffer_count,
        buffer_size,
        staging.buffer,
        buffer.buffer,
    );

    unsafe {
        devices.logical.device.destroy_buffer(staging.buffer, None);
        devices.logical.device.free_memory(staging.memory, None);
    }

    buffer
}

fn copy_buffer(
    devices: &Devices,
    command_pool: &vk::CommandPool,
    _command_buffer_count: u32,
    size: u64,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
) {
    let command_buffer =
        command_buffer::begin_single_time_command(&devices.logical.device, command_pool);

    let copy_region = vk::BufferCopy::builder().size(size);

    unsafe {
        devices.logical.device.cmd_copy_buffer(
            command_buffer,
            src_buffer,
            dst_buffer,
            std::slice::from_ref(&copy_region),
        );
    }

    command_buffer::end_single_time_command(
        &devices.logical.device,
        devices.logical.queues.graphics,
        command_pool,
        command_buffer,
    );
}
