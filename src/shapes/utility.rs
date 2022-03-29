use super::{Buffer, Indices, Vertex, WHITE};
use crate::{
    command_buffer,
    device::Devices,
    memory,
    space::{Coordinate3d, DirectionVector},
    texture,
    utility::InstanceDevices,
};
use ash::vk;
use cgmath::Vector2;
use std::ops::{Mul, Sub};

#[derive(Clone, Copy, Debug, Default)]
pub struct ModelTopology(pub(crate) vk::PrimitiveTopology);

type VkTop = vk::PrimitiveTopology;

impl ModelTopology {
    pub const LINE_LIST: Self = Self(VkTop::LINE_LIST);
    pub const LINE_LIST_WITH_ADJACENCY: Self = Self(VkTop::LINE_LIST_WITH_ADJACENCY);
    pub const LINE_STRIP: Self = Self(VkTop::LINE_STRIP);
    pub const LINE_STRIP_WITH_ADJACENCY: Self = Self(VkTop::LINE_STRIP_WITH_ADJACENCY);
    pub const PATCH_LIST: Self = Self(VkTop::PATCH_LIST);
    pub const POINT_LIST: Self = Self(VkTop::POINT_LIST);
    pub const TRIANGLE_FAN: Self = Self(VkTop::TRIANGLE_FAN);
    pub const TRIANGLE_LIST: Self = Self(VkTop::TRIANGLE_LIST);
    pub const TRIANGLE_LIST_WITH_ADJACENCY: Self = Self(VkTop::TRIANGLE_LIST_WITH_ADJACENCY);
    pub const TRIANGLE_STRIP: Self = Self(VkTop::TRIANGLE_STRIP);
    pub const TRIANGLE_STRIP_WITH_ADJACENCY: Self = Self(VkTop::TRIANGLE_STRIP_WITH_ADJACENCY);
}

#[derive(Clone, Copy, Debug, Default)]
pub struct ModelCullMode(pub(crate) vk::CullModeFlags);

type VkCull = vk::CullModeFlags;

impl ModelCullMode {
    pub const BACK: Self = Self(VkCull::BACK);
    pub const FRONT: Self = Self(VkCull::FRONT);
    pub const FRONT_AND_BACK: Self = Self(VkCull::FRONT_AND_BACK);
    pub const NONE: Self = Self(VkCull::NONE);
}

fn copy_buffer(
    devices: &Devices,
    command_pool: vk::CommandPool,
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

pub(crate) fn create_vertex_index_buffer<T>(
    buffer_size: u64,
    data: &[T],
    usage_flags: vk::BufferUsageFlags,
    command_pool: vk::CommandPool,
    command_buffer_count: u32,
    instance_devices: &InstanceDevices,
) -> Buffer
where
    T: std::marker::Copy,
{
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

pub(crate) fn scale(model: &mut [Vertex; 4], radius: f32) {
    model.iter_mut().for_each(|face| {
        face.pos.0 = face.pos.mul(radius);
    });
}

pub(crate) fn calculate_normals(model: &mut [Vertex; 4]) {
    let normal = normal(
        model[0].pos.into(),
        model[1].pos.into(),
        model[2].pos.into(),
    );

    model.iter_mut().for_each(|point| {
        point.normal = normal;
    });
}

fn normal(p1: DirectionVector, p2: DirectionVector, p3: DirectionVector) -> DirectionVector {
    let a = p3.sub(p2);
    let b = p1.sub(p2);
    DirectionVector(a.cross(*b))
}

pub(crate) fn make_point(
    angle: &mut f32,
    radius: f32,
    step: f32,
    length: f32,
    tex_coord: Vector2<f32>,
    pos: &Coordinate3d,
) -> Vertex {
    let x = (angle.to_radians().cos() * radius) + pos.x;
    let y = (angle.to_radians().sin() * radius) + pos.y;

    *angle += step;

    let pos = Coordinate3d::new(x, y, pos.z);

    Vertex::new(pos, WHITE, pos.mul(length).into(), tex_coord)
}

pub(crate) fn spherical_indices(sector_count: u32, stack_count: u32) -> Indices {
    let mut k1: u32;
    let mut k2: u32;

    let mut indices = Vec::new();

    for i in 0..stack_count {
        k1 = i * (sector_count + 1);
        k2 = k1 + sector_count + 1;

        for _j in 0..sector_count {
            if i != 0 {
                indices.push(k1 as u16);
                indices.push(k2 as u16);
                indices.push(k1 as u16 + 1);
            }

            if i != (stack_count - 1) {
                indices.push(k1 as u16 + 1);
                indices.push(k2 as u16);
                indices.push(k2 as u16 + 1);
            }

            k1 += 1;
            k2 += 1;
        }
    }

    indices.into()
}