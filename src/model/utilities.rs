use super::{Vertex, WHITE};
use crate::{command, device::Devices, memory, texture};
use ash::{vk, Instance};
use cgmath::{Vector2, Vector3};
use std::ops::{Mul, Sub};

fn copy_buffer(
    devices: &Devices,
    command_pool: vk::CommandPool,
    _command_buffer_count: u32,
    size: u64,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
) {
    let command_buffer = command::begin_single_time_command(&devices.logical, command_pool);

    let copy_region = vk::BufferCopy::builder().size(size);

    unsafe {
        devices.logical.cmd_copy_buffer(
            command_buffer,
            src_buffer,
            dst_buffer,
            std::slice::from_ref(&copy_region),
        );
    }

    command::end_single_time_command(
        &devices.logical,
        command_pool,
        devices.graphics_queue,
        command_buffer,
    );
}

pub(crate) fn create_vertex_index_buffer<T>(
    instance: &Instance,
    devices: &Devices,
    buffer_size: u64,
    data: &[T],
    usage_flags: vk::BufferUsageFlags,
    command_pool: vk::CommandPool,
    command_buffer_count: u32,
) -> (vk::Buffer, vk::DeviceMemory)
where
    T: std::marker::Copy,
{
    let (staging_buffer, staging_buffer_memory) = texture::create_buffer(
        instance,
        devices,
        buffer_size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    unsafe {
        memory::map_memory(&devices.logical, staging_buffer_memory, buffer_size, data);
    }

    let (buffer, buffer_memory) = texture::create_buffer(
        instance,
        devices,
        buffer_size,
        usage_flags,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    );

    copy_buffer(
        devices,
        command_pool,
        command_buffer_count,
        buffer_size,
        staging_buffer,
        buffer,
    );

    unsafe {
        devices.logical.destroy_buffer(staging_buffer, None);
        devices.logical.free_memory(staging_buffer_memory, None);
    }

    (buffer, buffer_memory)
}

pub(crate) fn calculate_normals(model: &mut [Vertex; 4]) {
    let normal = normal(model[0].pos, model[1].pos, model[2].pos);

    for point in model {
        point.normal = normal;
    }
}

fn normal(p1: Vector3<f32>, p2: Vector3<f32>, p3: Vector3<f32>) -> Vector3<f32> {
    let a = p3.sub(p2);
    let b = p1.sub(p2);
    a.cross(b)
}

pub(crate) fn make_point(
    angle: &mut f32,
    radius: f32,
    step: f32,
    length: f32,
    tex_coord: Vector2<f32>,
) -> Vertex {
    let pos_0 = angle.to_radians().cos() * radius;
    let pos_1 = angle.to_radians().sin() * radius;
    *angle += step;

    let pos = Vector3::new(pos_0, pos_1, 0.);

    Vertex::new(pos, WHITE, pos.mul(length), tex_coord)
}

pub(crate) fn calculate_sphere_indices(sector_count: u32, stack_count: u32) -> Vec<u16> {
    let mut k1: u16;
    let mut k2: u16;

    let mut indices: Vec<u16> = Vec::new();
    for i in 0..stack_count {
        k1 = i as u16 * (sector_count + 1) as u16;
        k2 = k1 + (stack_count + 1) as u16;

        for _j in 0..sector_count {
            if i != 0 {
                indices.push(k1);
                indices.push(k2);
                indices.push(k1 + 1);
            }

            if i != (stack_count - 1) {
                indices.push(k1 + 1);
                indices.push(k2);
                indices.push(k2 + 1);
            }

            k1 += 1;
            k2 += 1;
        }
    }

    indices
}
