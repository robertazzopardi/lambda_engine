use crate::{
    pipeline::GraphicsPipeline,
    shapes::{
        utility::{self, ModelCullMode, ModelTopology},
        ModelBuffers, Object, ObjectBuilder, ShapeProperties, Vertex, VerticesAndIndices, WHITE,
    },
    space,
    texture::{self, Texture},
    utility::InstanceDevices,
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

#[derive(Debug, Clone, new)]
pub struct CubeProperties {
    pub position: Point3<f32>,
    pub orientation: space::Orientation,
    pub radius: f32,
}

impl From<CubeProperties> for ShapeProperties {
    fn from(obj: CubeProperties) -> Self {
        ShapeProperties::Cube(obj)
    }
}

#[derive(Clone)]
pub struct Cube {
    properties: CubeProperties,

    pub texture_buffer: Option<Vec<u8>>,
    pub indexed: bool,
    pub topology: ModelTopology,
    pub cull_mode: ModelCullMode,

    pub vertices_and_indices: Option<VerticesAndIndices>,
    pub(crate) texture: Option<Texture>,
    pub(crate) graphics_pipeline: Option<GraphicsPipeline>,
    pub(crate) buffers: Option<ModelBuffers>,
}

impl Object for Cube {
    fn object_topology(&self) -> &ModelTopology {
        &self.topology
    }

    fn object_cull_mode(&self) -> &ModelCullMode {
        &self.cull_mode
    }

    fn object_graphics_pipeline(&self) -> &GraphicsPipeline {
        self.graphics_pipeline.as_ref().unwrap()
    }

    fn object_buffers(&self) -> &ModelBuffers {
        self.buffers.as_ref().unwrap()
    }

    fn object_texture(&self) -> &Texture {
        self.texture.as_ref().unwrap()
    }

    fn object_vertices_and_indices(&self) -> &VerticesAndIndices {
        self.vertices_and_indices.as_ref().unwrap()
    }

    fn builder(properties: ShapeProperties) -> Self {
        Self {
            properties: properties.into_cube().unwrap(),
            texture_buffer: None,
            indexed: true,
            topology: ModelTopology::Default,
            cull_mode: ModelCullMode::None,
            vertices_and_indices: None,
            texture: None,
            graphics_pipeline: None,
            buffers: None,
        }
    }

    fn is_indexed(&self) -> bool {
        self.indexed
    }

    fn vertices_and_indices(&mut self) {
        let cube = CUBE_VERTICES;

        cube.map(|_| utility::calculate_normals);

        let vertices = cube.into_iter().flatten().collect::<Vec<Vertex>>();

        self.vertices_and_indices = Some(VerticesAndIndices::new(vertices, CUBE_INDICES.to_vec()));
    }

    fn buffers(&mut self, model_buffers: ModelBuffers) {
        self.buffers = Some(model_buffers);
    }

    fn texture(&mut self, command_pool: vk::CommandPool, instance_devices: &InstanceDevices) {
        if let Some(buffer) = &self.texture_buffer {
            self.texture = Some(texture::Texture::new(
                buffer,
                command_pool,
                instance_devices,
            ));
        }
    }

    fn graphics_pipeline(
        &mut self,
        swap_chain: &crate::swap_chain::SwapChain,
        render_pass: ash::vk::RenderPass,
        instance_devices: &crate::utility::InstanceDevices,
    ) {
        self.graphics_pipeline = Some(GraphicsPipeline::new(
            swap_chain,
            render_pass,
            self.object_texture(),
            self,
            instance_devices,
        ));
    }
}

impl ObjectBuilder for Cube {
    fn texture_buffer(mut self, texture_buffer: Vec<u8>) -> Box<Self> {
        self.texture_buffer = Some(texture_buffer);
        self.into()
    }

    fn indexed(mut self, indexed: bool) -> Box<Self> {
        self.indexed = indexed;
        self.into()
    }

    fn topology(mut self, topology: ModelTopology) -> Box<Self> {
        self.topology = topology;
        self.into()
    }

    fn cull_mode(mut self, cull_mode: ModelCullMode) -> Box<Self> {
        self.cull_mode = cull_mode;
        self.into()
    }
}
