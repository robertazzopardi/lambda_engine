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
use cgmath::{Point3, Vector2, Vector3, Zero};
use std::ops::Mul;

#[derive(Debug, Clone, Copy, new)]
pub struct SphereProperties {
    pub position: Point3<f32>,
    pub orientation: space::Orientation,
    pub radius: f32,
    pub sector_count: u32,
    pub stack_count: u32,
}

impl From<SphereProperties> for ShapeProperties {
    fn from(obj: SphereProperties) -> Self {
        ShapeProperties::Sphere(obj)
    }
}

#[derive(Clone)]
pub struct Sphere {
    pub properties: SphereProperties,

    pub texture_buffer: Option<Vec<u8>>,
    pub indexed: bool,
    pub topology: ModelTopology,
    pub cull_mode: ModelCullMode,

    pub vertices_and_indices: Option<VerticesAndIndices>,
    pub(crate) texture: Option<Texture>,
    pub(crate) graphics_pipeline: Option<GraphicsPipeline>,
    pub(crate) buffers: Option<ModelBuffers>,
}

impl Object for Sphere {
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
            properties: properties.into_sphere().unwrap(),
            vertices_and_indices: None,
            texture_buffer: None,
            topology: ModelTopology::Default,
            indexed: true,
            cull_mode: ModelCullMode::None,
            texture: None,
            graphics_pipeline: None,
            buffers: None,
        }
    }

    fn is_indexed(&self) -> bool {
        self.indexed
    }

    fn vertices_and_indices(&mut self) {
        let length = 1. / self.properties.radius;

        let sector_step = 2. * std::f32::consts::PI / self.properties.sector_count as f32;
        let stack_step = std::f32::consts::PI / self.properties.stack_count as f32;

        let mut pos = Vector3::zero();

        let mut vertices = Vec::new();

        for i in 0..=self.properties.stack_count {
            let stack_angle = std::f32::consts::FRAC_PI_2 - i as f32 * stack_step;
            let xy = self.properties.radius * stack_angle.cos();
            pos[2] = self.properties.radius * stack_angle.sin();

            for j in 0..=self.properties.sector_count {
                let sector_angle = j as f32 * sector_step;

                pos[0] = xy * sector_angle.cos();
                pos[1] = xy * sector_angle.sin();

                let normal = pos.mul(length);

                let tex_coord = Vector2::new(
                    j as f32 / self.properties.sector_count as f32,
                    i as f32 / self.properties.stack_count as f32,
                );

                vertices.push(Vertex::new(pos, WHITE, normal, tex_coord));
            }
        }

        self.vertices_and_indices = Some(VerticesAndIndices::new(
            vertices,
            utility::spherical_indices(self.properties.sector_count, self.properties.stack_count),
        ));
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

impl ObjectBuilder for Sphere {
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
