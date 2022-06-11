use crate::{utility::spherical_indices, GeomBehavior, Geometry, VerticesAndIndices, WHITE};
use derive_builder::Builder;
use derive_more::{Deref, DerefMut};
use lambda_space::{
    space::{Coordinate3, Orientation, Vertices},
    vertex,
};
use lambda_vulkan::{
    buffer::ModelBuffers, command_buffer::CommandPool, graphics_pipeline::GraphicsPipeline,
    swap_chain::SwapChain, texture::Texture, utility::InstanceDevices, RenderPass, VulkanObject,
};
use nalgebra::{Point3, Vector2};
use std::ops::Mul;

#[derive(Builder, Default, Debug, Clone, Copy)]
#[builder(default, build_fn(skip))]
#[builder(name = "SphereBuilder")]
pub struct SphereInfo {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub radius: f32,
    pub sector_count: u32,
    pub stack_count: u32,
}

impl SphereBuilder {
    pub fn build(&mut self) -> SphereInfo {
        SphereInfo {
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            radius: self.radius.expect("Field `Radius` expected"),
            sector_count: self.sector_count.unwrap_or_default(),
            stack_count: self.stack_count.unwrap_or_default(),
        }
    }
}

#[derive(new, Deref, DerefMut, Debug)]
pub struct Sphere(Geometry<SphereInfo>);

impl GeomBehavior for Sphere {
    fn vertices_and_indices(&mut self) -> VerticesAndIndices {
        let length = 1. / self.properties.radius;

        let sector_step = 2. * std::f32::consts::PI / self.properties.sector_count as f32;
        let stack_step = std::f32::consts::PI / self.properties.stack_count as f32;

        let pos = self.properties.position;

        let mut vertices = Vertices::default();

        for i in 0..=self.properties.stack_count {
            let stack_angle = std::f32::consts::FRAC_PI_2 - i as f32 * stack_step;
            let xy = self.properties.radius * stack_angle.cos();

            let mut vec = Point3::new(0., 0., (self.properties.radius * stack_angle.sin()) + pos.z);

            for j in 0..=self.properties.sector_count {
                let sector_angle = j as f32 * sector_step;

                vec.x = (xy * sector_angle.cos()) + pos.x;
                vec.y = (xy * sector_angle.sin()) + pos.y;

                let normal = vec.mul(length);

                let tex_coord = Vector2::new(
                    j as f32 / self.properties.sector_count as f32,
                    i as f32 / self.properties.stack_count as f32,
                );

                vertices.push(vertex!(vec, WHITE, normal.coords, tex_coord));
            }
        }

        VerticesAndIndices::new(
            vertices,
            spherical_indices(self.properties.sector_count, self.properties.stack_count),
        )
    }

    fn vulkan_object(&self) -> &VulkanObject {
        &self.vulkan_object
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
