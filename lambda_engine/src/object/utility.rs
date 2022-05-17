use super::{Buffer, Indices, Vertex, Vertices, WHITE};
use crate::{
    command_buffer, device::Devices, memory, space::Coordinate3, texture, utility::InstanceDevices,
};
use ash::vk;
use nalgebra::{Point3, Vector2, Vector3};
use std::{
    collections::HashMap,
    ops::{Mul, Sub},
};

const LIGHT: &str = "light";
const LIGHT_TEXTURE: &str = "light_texture";
const TEXTURE: &str = "texture";
const VERTEX: &str = "vertex";

#[derive(Clone, Copy, Debug)]
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

impl Default for ModelTopology {
    fn default() -> Self {
        Self::TRIANGLE_LIST
    }
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

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
    Light,
    LightTexture,
    Texture,
    Vertex,
}

impl Default for ShaderType {
    fn default() -> Self {
        Self::Light
    }
}

impl From<ShaderType> for &str {
    fn from(texture_type: ShaderType) -> Self {
        match texture_type {
            ShaderType::Light => LIGHT,
            ShaderType::LightTexture => LIGHT_TEXTURE,
            ShaderType::Texture => TEXTURE,
            ShaderType::Vertex => VERTEX,
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

pub(crate) fn scale(model: &mut [Vertex], radius: f32) {
    model.iter_mut().for_each(|face| {
        face.pos = face.pos.mul(radius);
    });
}

pub(crate) fn calculate_normals(model: &mut [Vertex]) {
    let normal = normal(model[0].pos, model[1].pos, model[2].pos);

    model.iter_mut().for_each(|point| {
        point.normal = normal.coords;
    });
}

fn normal(p1: Point3<f32>, p2: Point3<f32>, p3: Point3<f32>) -> Point3<f32> {
    let a = p3.sub(p2);
    let b = p1.sub(p2);
    Point3::from(a.cross(&b))
}

pub(crate) fn make_point(
    angle: &mut f32,
    radius: f32,
    step: f32,
    length: f32,
    tex_coord: Vector2<f32>,
    pos: &Coordinate3,
) -> Vertex {
    let x = (angle.to_radians().cos() * radius) + pos.x;
    let y = (angle.to_radians().sin() * radius) + pos.y;

    *angle += step;

    let pos = Vector3::new(x, y, pos.z);

    Vertex::new(pos.into(), WHITE, pos.mul(length), tex_coord)
}

pub(crate) fn calculate_indices(vertices: &Vertices) -> Indices {
    let mut unique_vertices: HashMap<String, u16> = HashMap::new();
    let mut indices = Vec::new();
    let mut v = Vec::new();

    vertices.iter().for_each(|vertex| {
        let vertex_hash = &format!("{:p}", vertex);

        if !unique_vertices.contains_key(vertex_hash) {
            unique_vertices.insert(vertex_hash.to_string(), v.len() as u16);
            v.push(vertex);
        }

        indices.push(unique_vertices[vertex_hash]);
    });

    indices.into()
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
