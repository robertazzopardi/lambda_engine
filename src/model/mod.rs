mod utilities;

use self::utilities::{
    calculate_normals, calculate_sphere_indices, create_vertex_index_buffer, make_point,
};
use crate::{
    device::Devices,
    pipeline::GraphicsPipeline,
    swapchain::SwapChain,
    texture::{self, Texture},
};
use ash::{vk, Instance};
use cgmath::{Vector2, Vector3, Zero};
use std::{mem::size_of, ops::Mul};

// pub type Vertex = Vector4<Vector3<f32>>;

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
    TriangleFan,
    TriangleList,
    TriangleListWithAdjacency,
    TriangleStrip,
    TriangleStripWithAdjacency,
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
        }
    }
}

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

pub enum ModelType {
    Sphere,
    Cube,
    Ring,
}

pub struct ModelProperties {
    pub texture: Vec<u8>,
    pub model_type: ModelType,
    pub indexed: bool,
    pub topology: ModelTopology,
    pub cull_mode: ModelCullMode,
}

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub colour: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

impl Vertex {
    pub fn new(
        pos: Vector3<f32>,
        colour: Vector3<f32>,
        normal: Vector3<f32>,
        tex_coord: Vector2<f32>,
    ) -> Self {
        Self {
            pos,
            colour,
            normal,
            tex_coord,
        }
    }
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
        command_pool: vk::CommandPool,
        command_buffer_count: u32,
        swapchain: &SwapChain,
        render_pass: vk::RenderPass,
        property: ModelProperties,
    ) -> Self {
        let (vertices, indices) = match property.model_type {
            ModelType::Sphere => sphere(0.4, 40, 40),
            ModelType::Cube => cube(),
            ModelType::Ring => ring(0.6, 40),
        };

        let texture = texture::Texture::new(
            instance,
            devices,
            &property.texture,
            command_pool,
            command_buffer_count,
        );

        let (vertex_buffer, vertex_buffer_memory) = create_vertex_index_buffer(
            instance,
            devices,
            (size_of::<Vertex>() * vertices.len()).try_into().unwrap(),
            &vertices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            command_pool,
            command_buffer_count,
        );

        let (index_buffer, index_buffer_memory) = create_vertex_index_buffer(
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
            Some(property.topology.into()),
            Some(property.cull_mode.into()),
            devices,
            swapchain,
            render_pass,
            texture.image_view,
            texture.sampler,
        );

        Self {
            vertices,
            indices,
            indexed: property.indexed,
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
        index: usize,
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
            std::slice::from_ref(&self.graphics_pipeline.descriptor_set.descriptor_sets[index]),
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
}

pub fn ring(_radius: f32, sector_count: u32) -> (Vec<Vertex>, Vec<u16>) {
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

pub fn sphere(radius: f32, sector_count: u32, stack_count: u32) -> (Vec<Vertex>, Vec<u16>) {
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

            let tex_coord = Vector2::new(
                j as f32 / sector_count as f32,
                i as f32 / stack_count as f32,
            );

            vertices.push(Vertex::new(pos, WHITE, normal, tex_coord));
        }
    }

    (
        vertices,
        calculate_sphere_indices(sector_count, stack_count),
    )
}

pub fn cube() -> (Vec<Vertex>, Vec<u16>) {
    let cube = CUBE_VERTICES;
    // for model in cube.iter_mut() {
    //     Model::calculate_normals(model);
    // }

    cube.map(|_| calculate_normals);

    (cube.into_iter().flatten().collect(), CUBE_INDICES.to_vec())
}
