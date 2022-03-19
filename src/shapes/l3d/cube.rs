use std::mem::size_of;

use crate::{
    pipeline::GraphicsPipeline,
    shapes::{
        utility::{self, ModelCullMode, ModelTopology},
        ModelBuffers, Object, Vertex, VerticesAndIndices, WHITE,
    },
    space::{self, Orientation},
    texture::{self, Texture},
};
use ash::vk;
use cgmath::{Point3, Vector2, Vector3};

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

pub struct Cube {
    pub position: Point3<f32>,
    pub orientation: space::Orientation,

    pub texture_buffer: Option<Vec<u8>>,
    pub indexed: bool,
    pub topology: ModelTopology,
    pub cull_mode: ModelCullMode,
    pub vertices_and_indices: VerticesAndIndices,

    pub(crate) texture: Option<Texture>,
    pub(crate) graphics_pipeline: Option<GraphicsPipeline>,
    pub(crate) buffers: Option<ModelBuffers>,
}

impl Object for Cube {
    fn builder(position: Point3<f32>, orientation: Orientation) -> Self {
        let vertices_and_indices = Self::vertices_and_indices();

        Self {
            position,
            orientation,
            vertices_and_indices,
            texture_buffer: None,
            topology: ModelTopology::Default,
            indexed: true,
            cull_mode: ModelCullMode::None,
            texture: None,
            graphics_pipeline: None,
            buffers: None,
        }
    }

    fn vertices_and_indices() -> VerticesAndIndices {
        let cube = CUBE_VERTICES;

        cube.map(|_| utility::calculate_normals);

        VerticesAndIndices::new(cube.into_iter().flatten().collect(), CUBE_INDICES.to_vec())
    }

    fn texture_buffer(mut self, texture_buffer: Vec<u8>) -> Self {
        self.texture_buffer = Some(texture_buffer);
        self
    }

    fn indexed(mut self) -> Self {
        self.indexed = true;
        self
    }

    fn topology(mut self, topology: ModelTopology) -> Self {
        self.topology = topology;
        self
    }

    fn cull_mode(mut self, cull_mode: ModelCullMode) -> Self {
        self.cull_mode = cull_mode;
        self
    }

    // fn build(
    //     mut self,
    //     command_pool: ash::vk::CommandPool,
    //     command_buffer_count: u32,
    //     swap_chain: &crate::swap_chain::SwapChain,
    //     render_pass: ash::vk::RenderPass,
    //     instance_devices: &crate::utility::InstanceDevices,
    // ) -> Self {
    //     let VerticesAndIndices { vertices, indices } = self.vertices_and_indices.clone();

    //     if let Some(buffer) = &self.texture_buffer {
    //         self.texture = Some(texture::Texture::new(
    //             buffer,
    //             command_pool,
    //             instance_devices,
    //         ));
    //     }

    //     let vertex_buffer = utility::create_vertex_index_buffer(
    //         (size_of::<Vertex>() * vertices.len()).try_into().unwrap(),
    //         &vertices,
    //         vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
    //         command_pool,
    //         command_buffer_count,
    //         instance_devices,
    //     );

    //     let index_buffer = utility::create_vertex_index_buffer(
    //         (size_of::<u16>() * indices.len()).try_into().unwrap(),
    //         &indices,
    //         vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
    //         command_pool,
    //         command_buffer_count,
    //         instance_devices,
    //     );

    //     self.buffers = Some(ModelBuffers::new(vertex_buffer, index_buffer));

    //     // self.graphics_pipeline = Some(GraphicsPipeline::new(
    //     //     swap_chain,
    //     //     render_pass,
    //     //     self.texture.unwrap().image_view,
    //     //     self.texture.unwrap().sampler,
    //     //     &self,
    //     //     instance_devices,
    //     // ));

    //     self
    // }
}
