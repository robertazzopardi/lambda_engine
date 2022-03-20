use super::{Buffer, Vertex, WHITE};
use crate::{command, device::Devices, memory, texture, utility::InstanceDevices};
use ash::vk;
use cgmath::{Vector2, Vector3};
use std::ops::{Mul, Sub};

#[derive(Clone, Copy)]
pub enum ModelTopology {
    LineList,
    LineListWithAdjacency,
    LineStrip,
    LineStripWithADjacency,
    PatchList,
    PointList,
    TriangleFan,
    TriangleList,
    TriangleListWithAdjacency,
    TriangleStrip,
    TriangleStripWithAdjacency,
    Default,
}

impl From<ModelTopology> for vk::PrimitiveTopology {
    fn from(topology: ModelTopology) -> Self {
        match topology {
            ModelTopology::TriangleList => vk::PrimitiveTopology::TRIANGLE_LIST,
            ModelTopology::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
            ModelTopology::TriangleFan => vk::PrimitiveTopology::TRIANGLE_FAN,
            ModelTopology::TriangleListWithAdjacency => {
                vk::PrimitiveTopology::TRIANGLE_LIST_WITH_ADJACENCY
            }
            ModelTopology::TriangleStripWithAdjacency => {
                vk::PrimitiveTopology::TRIANGLE_STRIP_WITH_ADJACENCY
            }
            ModelTopology::LineList => vk::PrimitiveTopology::LINE_LIST,
            ModelTopology::LineListWithAdjacency => vk::PrimitiveTopology::LINE_LIST_WITH_ADJACENCY,
            ModelTopology::LineStrip => vk::PrimitiveTopology::LINE_STRIP,
            ModelTopology::LineStripWithADjacency => {
                vk::PrimitiveTopology::LINE_STRIP_WITH_ADJACENCY
            }
            ModelTopology::PatchList => vk::PrimitiveTopology::PATCH_LIST,
            ModelTopology::PointList => vk::PrimitiveTopology::POINT_LIST,
            ModelTopology::Default => vk::PrimitiveTopology::default(),
        }
    }
}

impl From<&ModelTopology> for vk::PrimitiveTopology {
    fn from(topology: &ModelTopology) -> Self {
        match topology {
            ModelTopology::TriangleList => vk::PrimitiveTopology::TRIANGLE_LIST,
            ModelTopology::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
            ModelTopology::TriangleFan => vk::PrimitiveTopology::TRIANGLE_FAN,
            ModelTopology::TriangleListWithAdjacency => {
                vk::PrimitiveTopology::TRIANGLE_LIST_WITH_ADJACENCY
            }
            ModelTopology::TriangleStripWithAdjacency => {
                vk::PrimitiveTopology::TRIANGLE_STRIP_WITH_ADJACENCY
            }
            ModelTopology::LineList => vk::PrimitiveTopology::LINE_LIST,
            ModelTopology::LineListWithAdjacency => vk::PrimitiveTopology::LINE_LIST_WITH_ADJACENCY,
            ModelTopology::LineStrip => vk::PrimitiveTopology::LINE_STRIP,
            ModelTopology::LineStripWithADjacency => {
                vk::PrimitiveTopology::LINE_STRIP_WITH_ADJACENCY
            }
            ModelTopology::PatchList => vk::PrimitiveTopology::PATCH_LIST,
            ModelTopology::PointList => vk::PrimitiveTopology::POINT_LIST,
            ModelTopology::Default => vk::PrimitiveTopology::default(),
        }
    }
}

#[derive(Clone, Copy)]
pub enum ModelCullMode {
    Front,
    Back,
    FrontAndBack,
    None,
}

impl From<ModelCullMode> for vk::CullModeFlags {
    fn from(cull_mode: ModelCullMode) -> Self {
        match cull_mode {
            ModelCullMode::Front => vk::CullModeFlags::FRONT,
            ModelCullMode::Back => vk::CullModeFlags::BACK,
            ModelCullMode::FrontAndBack => vk::CullModeFlags::FRONT_AND_BACK,
            ModelCullMode::None => vk::CullModeFlags::NONE,
        }
    }
}

impl From<&ModelCullMode> for vk::CullModeFlags {
    fn from(cull_mode: &ModelCullMode) -> Self {
        match cull_mode {
            ModelCullMode::Front => vk::CullModeFlags::FRONT,
            ModelCullMode::Back => vk::CullModeFlags::BACK,
            ModelCullMode::FrontAndBack => vk::CullModeFlags::FRONT_AND_BACK,
            ModelCullMode::None => vk::CullModeFlags::NONE,
        }
    }
}

fn copy_buffer(
    devices: &Devices,
    command_pool: vk::CommandPool,
    _command_buffer_count: u32,
    size: u64,
    src_buffer: vk::Buffer,
    dst_buffer: vk::Buffer,
) {
    let command_buffer = command::begin_single_time_command(&devices.logical.device, command_pool);

    let copy_region = vk::BufferCopy::builder().size(size);

    unsafe {
        devices.logical.device.cmd_copy_buffer(
            command_buffer,
            src_buffer,
            dst_buffer,
            std::slice::from_ref(&copy_region),
        );
    }

    command::end_single_time_command(
        &devices.logical.device,
        devices.logical.graphics,
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

pub(crate) fn spherical_indices(sector_count: u32, stack_count: u32) -> Vec<u16> {
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
