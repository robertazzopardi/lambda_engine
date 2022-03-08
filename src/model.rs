use crate::{
    command::{begin_single_time_command, end_single_time_command},
    device::Devices,
    memory::map_memory,
    pipeline::GraphicsPipeline,
    swapchain::SwapChain,
    texture::{self, Texture},
};
use ash::{vk, Instance};
use cgmath::{Vector2, Vector3, Zero};
use std::{
    mem::size_of,
    ops::{Mul, Sub},
};

const WHITE: Vector3<f32> = Vector3::new(1., 1., 1.);
const VEC3_ZERO: Vector3<f32> = Vector3::new(0., 0., 0.);

const CUBE_VERTICES: [[Vertex; 4]; 6] = [
    [
        Vertex {
            pos: Vector3::new(-0.5, -0.5, 0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(1., 0.),
        },
        Vertex {
            pos: Vector3::new(0.5, -0.5, 0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(0., 0.),
        },
        Vertex {
            pos: Vector3::new(0.5, 0.5, 0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(0., 1.),
        },
        Vertex {
            pos: Vector3::new(-0.5, 0.5, 0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(1., 1.),
        },
    ],
    [
        Vertex {
            pos: Vector3::new(-0.5, 0.5, -0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(1., 0.),
        },
        Vertex {
            pos: Vector3::new(0.5, 0.5, -0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(0., 0.),
        },
        Vertex {
            pos: Vector3::new(0.5, -0.5, -0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(0., 1.),
        },
        Vertex {
            pos: Vector3::new(-0.5, -0.5, -0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(1., 1.),
        },
    ],
    [
        Vertex {
            pos: Vector3::new(-0.5, 0.5, 0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(1., 0.),
        },
        Vertex {
            pos: Vector3::new(-0.5, 0.5, -0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(0., 0.),
        },
        Vertex {
            pos: Vector3::new(-0.5, -0.5, -0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(0., 1.),
        },
        Vertex {
            pos: Vector3::new(-0.5, -0.5, 0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(1., 1.),
        },
    ],
    [
        Vertex {
            pos: Vector3::new(0.5, -0.5, 0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(1., 0.),
        },
        Vertex {
            pos: Vector3::new(0.5, -0.5, -0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(0., 0.),
        },
        Vertex {
            pos: Vector3::new(0.5, 0.5, -0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(0., 1.),
        },
        Vertex {
            pos: Vector3::new(0.5, 0.5, 0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(1., 1.),
        },
    ],
    [
        Vertex {
            pos: Vector3::new(0.5, 0.5, 0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(1., 0.),
        },
        Vertex {
            pos: Vector3::new(0.5, 0.5, -0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(0., 0.),
        },
        Vertex {
            pos: Vector3::new(-0.5, 0.5, -0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(0., 1.),
        },
        Vertex {
            pos: Vector3::new(-0.5, 0.5, 0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(1., 1.),
        },
    ],
    [
        Vertex {
            pos: Vector3::new(0.5, -0.5, -0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(1., 0.),
        },
        Vertex {
            pos: Vector3::new(0.5, -0.5, 0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(0., 0.),
        },
        Vertex {
            pos: Vector3::new(-0.5, -0.5, 0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(0., 1.),
        },
        Vertex {
            pos: Vector3::new(-0.5, -0.5, -0.5),
            colour: WHITE,
            normal: VEC3_ZERO,
            tex_coord: Vector2::new(1., 1.),
        },
    ],
];

const CUBE_INDICES: [u16; 36] = [
    0, 1, 2, 2, 3, 0, // top
    4, 5, 6, 6, 7, 4, // bottom
    8, 9, 10, 8, 10, 11, // right
    12, 13, 14, 12, 14, 15, // left
    16, 17, 18, 16, 18, 19, // front
    20, 21, 22, 20, 22, 23, // back
];

pub enum ModelTopology {
    TriangleList,
    TriangleStrip,
}

impl From<ModelTopology> for vk::PrimitiveTopology {
    fn from(topology: ModelTopology) -> Self {
        match topology {
            ModelTopology::TriangleList => vk::PrimitiveTopology::TRIANGLE_LIST,
            ModelTopology::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
        }
    }
}

pub enum ModelCullMode {
    Front,
    Back,
    None,
}

impl From<ModelCullMode> for vk::CullModeFlags {
    fn from(cull_mode: ModelCullMode) -> Self {
        match cull_mode {
            ModelCullMode::Front => vk::CullModeFlags::FRONT,
            ModelCullMode::Back => vk::CullModeFlags::BACK,
            ModelCullMode::None => vk::CullModeFlags::NONE,
        }
    }
}

pub enum ModelType {
    Sphere,
    Cube,
    Ring,
}

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub colour: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

pub struct Model {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u16>,
    indexed: bool,
    pub texture: Texture,
    pub graphics_pipeline: GraphicsPipeline,
    pub vertex_buffer: vk::Buffer,
    pub vertex_buffer_memory: vk::DeviceMemory,
    pub index_buffer: vk::Buffer,
    pub index_buffer_memory: vk::DeviceMemory,
}

impl Model {
    pub fn new(
        instance: &Instance,
        devices: &Devices,
        image_buffer: &[u8],
        command_pool: vk::CommandPool,
        command_buffer_count: u32,
        shape_type: ModelType,
        indexed: bool,
        topology: Option<vk::PrimitiveTopology>,
        cull_mode: Option<vk::CullModeFlags>,
        swapchain: &SwapChain,
        render_pass: vk::RenderPass,
    ) -> Self {
        let (vertices, indices) = match shape_type {
            ModelType::Sphere => sphere(0.4, 40, 40),
            ModelType::Cube => cube(),
            ModelType::Ring => ring(0.6, 40),
        };

        let texture = texture::Texture::new(
            instance,
            devices,
            image_buffer,
            command_pool,
            command_buffer_count,
        );

        let (vertex_buffer, vertex_buffer_memory) = Self::create_vertex_index_buffer(
            instance,
            devices,
            (size_of::<Vertex>() * vertices.len()).try_into().unwrap(),
            &vertices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            command_pool,
            command_buffer_count,
        );

        let (index_buffer, index_buffer_memory) = Self::create_vertex_index_buffer(
            instance,
            devices,
            (size_of::<u16>() * indices.len()).try_into().unwrap(),
            &indices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            command_pool,
            command_buffer_count,
        );

        let graphics_pipeline = GraphicsPipeline::new(
            instance,
            topology,
            cull_mode,
            devices,
            swapchain,
            render_pass,
            texture.image_view,
            texture.sampler,
        );

        Self {
            vertices,
            indices,
            indexed,
            texture,
            graphics_pipeline,
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
        }
    }

    /// # Safety
    ///
    /// Expand on safety of this function
    pub unsafe fn bind_index_and_vertex_buffers(
        &self,
        devices: &Devices,
        command_buffer: vk::CommandBuffer,
        offsets: &[vk::DeviceSize],
        i: usize,
    ) {
        devices.logical.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.graphics_pipeline.pipeline,
        );

        devices.logical.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.graphics_pipeline.layout,
            0,
            std::slice::from_ref(&self.graphics_pipeline.descriptor_set.descriptor_sets[i]),
            &[],
        );

        let vertex_buffers = [self.vertex_buffer];

        devices
            .logical
            .cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, offsets);

        devices
            .logical
            .cmd_draw(command_buffer, self.vertices.len() as u32, 1, 0, 0);

        if self.indexed {
            devices.logical.cmd_bind_index_buffer(
                command_buffer,
                self.index_buffer,
                0,
                vk::IndexType::UINT16,
            );

            devices
                .logical
                .cmd_draw_indexed(command_buffer, self.indices.len() as u32, 1, 0, 0, 0);
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
        let command_buffer = begin_single_time_command(&devices.logical, command_pool);

        let copy_region = vk::BufferCopy::builder().size(size);

        unsafe {
            devices.logical.cmd_copy_buffer(
                command_buffer,
                src_buffer,
                dst_buffer,
                std::slice::from_ref(&copy_region),
            );
        }

        end_single_time_command(
            &devices.logical,
            command_pool,
            devices.graphics_queue,
            command_buffer,
        );
    }

    fn create_vertex_index_buffer<T>(
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
            map_memory(&devices.logical, staging_buffer_memory, buffer_size, data);
        }

        let (buffer, buffer_memory) = texture::create_buffer(
            instance,
            devices,
            buffer_size,
            usage_flags,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        Self::copy_buffer(
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
}

fn calculate_normals(model: &mut [Vertex; 4]) {
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

fn make_point(
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
    Vertex {
        pos,
        colour: WHITE,
        normal: pos.mul(length),
        tex_coord,
    }
}

fn ring(_radius: f32, sector_count: u32) -> (Vec<Vertex>, Vec<u16>) {
    let stack_count = 2;

    let mut angle = 0.;
    let angle_step = 180. / sector_count as f32;
    let length = 1.;

    let outside_radius = 1.;
    let inside_radius = 0.5;

    let mut vertices = Vec::new();

    for _ in 0..=sector_count {
        vertices.push(make_point(
            &mut angle,
            outside_radius,
            angle_step,
            length,
            Vector2::new(0., 0.),
        ));
        vertices.push(make_point(
            &mut angle,
            inside_radius,
            angle_step,
            length,
            Vector2::new(1., 1.),
        ));
    }

    (
        vertices,
        calculate_sphere_indices(sector_count, stack_count),
    )
}

fn sphere(radius: f32, sector_count: u32, stack_count: u32) -> (Vec<Vertex>, Vec<u16>) {
    let length = 1. / radius;

    let sector_step = 2. * std::f32::consts::PI / sector_count as f32;
    let stack_step = std::f32::consts::PI / stack_count as f32;

    let mut pos = Vector3::<f32>::zero();

    let mut vertices = Vec::<Vertex>::new();

    for i in 0..=stack_count {
        let stack_angle = std::f32::consts::FRAC_PI_2 - i as f32 * stack_step;
        let xy = radius * stack_angle.cos();
        pos[2] = radius * stack_angle.sin();

        for j in 0..=sector_count {
            let sector_angle = j as f32 * sector_step;

            pos[0] = xy * sector_angle.cos();
            pos[1] = xy * sector_angle.sin();

            let normal = pos.mul(length);

            let tex_coord = Vector2 {
                x: j as f32 / sector_count as f32,
                y: i as f32 / stack_count as f32,
            };

            vertices.push(Vertex {
                pos,
                colour: WHITE,
                normal,
                tex_coord,
            });
        }
    }

    (
        vertices,
        calculate_sphere_indices(sector_count, stack_count),
    )
}

fn cube() -> (Vec<Vertex>, Vec<u16>) {
    let cube = CUBE_VERTICES;
    // for model in cube.iter_mut() {
    //     Model::calculate_normals(model);
    // }

    cube.map(|_| calculate_normals);

    (cube.into_iter().flatten().collect(), CUBE_INDICES.to_vec())
}

fn calculate_sphere_indices(sector_count: u32, stack_count: u32) -> Vec<u16> {
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
