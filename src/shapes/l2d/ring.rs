use crate::{
    pipeline::GraphicsPipeline,
    shapes::{
        utility::{self, ModelCullMode, ModelTopology},
        ModelBuffers, Object, ObjectBuilder, ShapeProperties, VerticesAndIndices,
    },
    space,
    texture::{self, Texture},
    utility::InstanceDevices,
};
use ash::vk;
use cgmath::{Point3, Vector2, Zero};

#[derive(Debug, Clone, new)]
pub struct RingProperties {
    pub position: Point3<f32>,
    pub orientation: space::Orientation,

    pub inner_radius: f32,
    pub outer_radius: f32,
    pub sector_count: u32,
}

impl From<RingProperties> for ShapeProperties {
    fn from(obj: RingProperties) -> Self {
        ShapeProperties::Ring(obj)
    }
}

#[derive(Clone)]
pub struct Ring {
    properties: RingProperties,

    pub texture_buffer: Option<Vec<u8>>,
    pub indexed: bool,
    pub topology: ModelTopology,
    pub cull_mode: ModelCullMode,

    pub vertices_and_indices: Option<VerticesAndIndices>,
    pub(crate) texture: Option<Texture>,
    pub(crate) graphics_pipeline: Option<GraphicsPipeline>,
    pub(crate) buffers: Option<ModelBuffers>,
}

impl Object for Ring {
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
            properties: properties.into_ring().unwrap(),
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
        assert!(
            self.properties.inner_radius <= self.properties.outer_radius,
            "Ring inner radius mut be smaller or equal to its outer radius"
        );

        let mut angle = 0.;
        let angle_step = 180. / self.properties.sector_count as f32;
        let length = 1.;

        let mut vertices = Vec::new();

        for _ in 0..=self.properties.sector_count {
            vertices.push(utility::make_point(
                &mut angle,
                self.properties.outer_radius,
                angle_step,
                length,
                Vector2::zero(),
            ));
            vertices.push(utility::make_point(
                &mut angle,
                self.properties.inner_radius,
                angle_step,
                length,
                Vector2::new(1., 1.),
            ));
        }

        self.vertices_and_indices = Some(VerticesAndIndices::new(
            vertices,
            utility::spherical_indices(self.properties.sector_count, 2),
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

impl ObjectBuilder for Ring {
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
