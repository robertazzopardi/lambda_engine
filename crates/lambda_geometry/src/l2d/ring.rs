use crate::{utility, GeomBehavior, Geometry, VerticesAndIndices};
use derive_builder::Builder;
use derive_more::{Deref, DerefMut};
use lambda_space::space::{Coordinate3, Orientation};
use lambda_vulkan::{
    buffer::ModelBuffers, command_buffer::CommandPool, graphics_pipeline::GraphicsPipeline,
    swap_chain::SwapChain, texture::Texture, utility::InstanceDevices, RenderPass, VulkanObject,
};
use nalgebra::Vector2;

#[derive(Builder, Default, Debug, Clone, new)]
#[builder(default, build_fn(skip))]
#[builder(name = "RingBuilder")]
pub struct RingInfo {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub sector_count: u32,
}

impl RingBuilder {
    pub fn build(&mut self) -> RingInfo {
        RingInfo {
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            inner_radius: self.inner_radius.unwrap_or_default(),
            outer_radius: self.outer_radius.unwrap_or_default(),
            sector_count: self.sector_count.unwrap_or_default(),
        }
    }
}

#[derive(new, Deref, DerefMut, Debug, Clone)]
pub struct Ring(Geometry<RingInfo>);

impl GeomBehavior for Ring {
    fn vertices_and_indices(&mut self) -> VerticesAndIndices {
        assert!(
            self.properties.inner_radius <= self.properties.outer_radius,
            "Ring inner radius mut be smaller or equal to its outer radius"
        );

        let mut angle = 0.;
        let angle_step = 180. / self.properties.sector_count as f32;
        let length = 1.;

        let pos = self.properties.position;

        let mut vertices = Vec::new();

        for _ in 0..=self.properties.sector_count {
            vertices.push(utility::make_point(
                &mut angle,
                self.properties.outer_radius,
                angle_step,
                length,
                Vector2::zeros(),
                &pos,
            ));
            vertices.push(utility::make_point(
                &mut angle,
                self.properties.inner_radius,
                angle_step,
                length,
                Vector2::new(1., 1.),
                &pos,
            ));
        }

        VerticesAndIndices::new(
            vertices.into(),
            utility::spherical_indices(self.properties.sector_count, 2),
        )
    }

    fn vulkan_object(&self) -> VulkanObject {
        self.vulkan_object.clone()
    }

    fn deferred_build(
        &mut self,
        command_pool: &CommandPool,
        command_buffer_count: u32,
        swap_chain: &SwapChain,
        render_pass: &RenderPass,
        instance_devices: &InstanceDevices,
    ) {
        if !self.texture.is_empty() {
            self.vulkan_object.texture =
                Some(Texture::new(&self.texture, command_pool, instance_devices));
        }

        let vertices_and_indices = self.vertices_and_indices();

        self.vulkan_object.buffers = ModelBuffers::new(
            &vertices_and_indices,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        self.vulkan_object.graphics_pipeline = GraphicsPipeline::new(
            swap_chain,
            render_pass.0,
            &self.vulkan_object.texture,
            self.topology,
            self.cull_mode,
            instance_devices,
            self.shader,
        );

        self.vulkan_object.vertices_and_indices = vertices_and_indices;
    }
}
