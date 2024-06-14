use crate::{command_buffer, device::Devices, memory, texture};
use ash::{vk, Instance};
use gpu_allocator::vulkan::{Allocation, Allocator};
use std::mem::size_of;
use wave_space::space::{Vertex, VerticesAndIndices};

#[derive(Default, Debug)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    // pub memory: vk::DeviceMemory,
    pub allocation: Allocation,
}

impl Buffer {
    pub fn new(buffer: vk::Buffer, allocation: Allocation) -> Self {
        Self { buffer, allocation }
    }
}

#[derive(Default, Debug)]
pub struct ModelBuffers {
    pub vertex: Buffer,
    pub index: Buffer,
}

impl ModelBuffers {
    pub fn new(
        allocator: &mut Allocator,
        vertices_and_indices: &VerticesAndIndices,
        command_pool: &vk::CommandPool,
        command_buffer_count: u32,
        devices: &Devices,
    ) -> Self {
        let vertex = create_vertex_index_buffer(
            allocator,
            (size_of::<Vertex>() * vertices_and_indices.vertices.len())
                .try_into()
                .unwrap(),
            &vertices_and_indices.vertices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            command_pool,
            command_buffer_count,
            devices,
        );

        let index = create_vertex_index_buffer(
            allocator,
            (size_of::<u16>() * vertices_and_indices.indices.len())
                .try_into()
                .unwrap(),
            &vertices_and_indices.indices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            command_pool,
            command_buffer_count,
            devices,
        );

        ModelBuffers { vertex, index }
    }
}

pub(crate) fn create_vertex_index_buffer<T: Copy>(
    allocator: &mut Allocator,
    buffer_size: vk::DeviceSize,
    data: &[T],
    usage_flags: vk::BufferUsageFlags,
    command_pool: &vk::CommandPool,
    command_buffer_count: u32,
    devices: &Devices,
) -> Buffer {
    let device = &devices.logical.device;

    let staging = texture::create_buffer(
        allocator,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        devices,
        "Vertex Index Staging Buffer",
    );

    unsafe {
        let mapped_ptr = staging.allocation.mapped_ptr().unwrap().as_ptr() as *mut f32;
        mapped_ptr.copy_from_nonoverlapping(data.as_ptr() as *const f32, buffer_size as usize);
    }

    let buffer = texture::create_buffer(
        allocator,
        buffer_size,
        usage_flags,
        devices,
        "Vertex Index Buffer",
    );

    copy_buffer(
        devices,
        command_pool,
        command_buffer_count,
        buffer_size,
        staging.buffer,
        buffer.buffer,
    );

    // unsafe {
    //     device.destroy_buffer(staging.buffer, None);
    //     device.free_memory(staging.memory, None);
    // }
    allocator.free(staging.allocation).unwrap();
    unsafe { device.destroy_buffer(staging.buffer, None) };

    buffer
}

fn copy_buffer(
    devices: &Devices,
    command_pool: &vk::CommandPool,
    _command_buffer_count: u32,
    size: vk::DeviceSize,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
) {
    let device = &devices.logical.device;

    let command_buffer = command_buffer::begin_single_time_command(device, command_pool);

    let copy_region = vk::BufferCopy::default().size(size);

    unsafe {
        device.cmd_copy_buffer(
            command_buffer,
            src_buffer,
            dst_buffer,
            std::slice::from_ref(&copy_region),
        );
    }

    command_buffer::end_single_time_command(
        device,
        devices.logical.queues.graphics,
        command_pool,
        command_buffer,
    );
}
