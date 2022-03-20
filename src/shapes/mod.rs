pub mod l2d;
pub mod l3d;
pub mod utility;

use self::{
    l2d::ring::RingProperties,
    l3d::{cube::CubeProperties, sphere::SphereProperties},
    utility::{ModelCullMode, ModelTopology},
};
use crate::{
    device::{Devices, LogicalDeviceFeatures},
    memory,
    pipeline::GraphicsPipeline,
    swap_chain::SwapChain,
    texture::Texture,
    uniform::UniformBufferObject,
    utility::InstanceDevices,
};
use ash::vk;
use cgmath::{Vector2, Vector3};
use enum_as_inner::EnumAsInner;
use std::mem::size_of;

pub(crate) const WHITE: Vector3<f32> = Vector3::new(1., 1., 1.);

#[derive(Debug, EnumAsInner)]
pub enum ShapeProperties {
    Cube(CubeProperties),
    Sphere(SphereProperties),
    Ring(RingProperties),
}

pub trait Object {
    // TODO move
    fn object_topology(&self) -> &ModelTopology;
    fn object_cull_mode(&self) -> &ModelCullMode;

    fn object_graphics_pipeline(&self) -> &GraphicsPipeline;
    fn object_buffers(&self) -> &ModelBuffers;
    fn object_texture(&self) -> &Texture;
    fn object_vertices_and_indices(&self) -> &VerticesAndIndices;

    fn translate(&mut self) {}
    fn rotate(&mut self) {}
    fn scale(&mut self) {}

    fn vertices_and_indices(&mut self);

    fn buffers(&mut self, model_buffers: ModelBuffers);
    fn texture(&mut self, command_pool: vk::CommandPool, instance_devices: &InstanceDevices);
    fn graphics_pipeline(
        &mut self,
        swap_chain: &SwapChain,
        render_pass: ash::vk::RenderPass,
        instance_devices: &InstanceDevices,
    );

    fn builder(properties: ShapeProperties) -> Self
    where
        Self: Sized;

    fn build(
        &mut self,
        command_pool: vk::CommandPool,
        command_buffer_count: u32,
        swap_chain: &SwapChain,
        render_pass: vk::RenderPass,
        instance_devices: &InstanceDevices,
    ) {
        self.texture(command_pool, instance_devices);

        self.vertices_and_indices();

        let model_buffers = self.object_vertices_and_indices().create_buffers(
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        self.buffers(model_buffers);

        self.graphics_pipeline(swap_chain, render_pass, instance_devices);
    }

    /// # Safety
    ///
    /// Expand on safety of this function
    unsafe fn bind_index_and_vertex_buffers(
        &self,
        devices: &Devices,
        command_buffer: vk::CommandBuffer,
        offsets: &[vk::DeviceSize],
        index: usize,
    ) {
        devices.logical.device.cmd_bind_pipeline(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.object_graphics_pipeline().features.pipeline,
        );

        devices.logical.device.cmd_bind_descriptor_sets(
            command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.object_graphics_pipeline().features.layout,
            0,
            std::slice::from_ref(
                &self
                    .object_graphics_pipeline()
                    .descriptor_set
                    .descriptor_sets[index],
            ),
            &[],
        );

        let vertex_buffers = [self.object_buffers().vertex.buffer];

        devices
            .logical
            .device
            .cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, offsets);

        devices.logical.device.cmd_draw(
            command_buffer,
            self.object_vertices_and_indices().vertices.len() as u32,
            1,
            0,
            0,
        );

        if self.is_indexed() {
            devices.logical.device.cmd_bind_index_buffer(
                command_buffer,
                self.object_buffers().index.buffer,
                0,
                vk::IndexType::UINT16,
            );

            devices.logical.device.cmd_draw_indexed(
                command_buffer,
                self.object_vertices_and_indices().indices.len() as u32,
                1,
                0,
                0,
                0,
            );
        }
    }

    fn is_indexed(&self) -> bool;

    fn map_memory(
        &self,
        logical: &LogicalDeviceFeatures,
        current_image: usize,
        buffer_size: u64,
        ubos: &[UniformBufferObject; 1],
    ) {
        memory::map_memory(
            &logical.device,
            self.object_graphics_pipeline()
                .descriptor_set
                .uniform_buffers[current_image]
                .memory,
            buffer_size,
            ubos,
        );
    }

    /// # Safety
    ///
    ///
    unsafe fn recreate_drop(&self, logical: &LogicalDeviceFeatures, swap_chain: &SwapChain) {
        logical
            .device
            .destroy_pipeline(self.object_graphics_pipeline().features.pipeline, None);
        logical
            .device
            .destroy_pipeline_layout(self.object_graphics_pipeline().features.layout, None);

        logical.device.destroy_descriptor_pool(
            self.object_graphics_pipeline()
                .descriptor_set
                .descriptor_pool,
            None,
        );

        for i in 0..swap_chain.images.len() {
            logical.device.destroy_buffer(
                self.object_graphics_pipeline()
                    .descriptor_set
                    .uniform_buffers[i]
                    .buffer,
                None,
            );
            logical.device.free_memory(
                self.object_graphics_pipeline()
                    .descriptor_set
                    .uniform_buffers[i]
                    .memory,
                None,
            );
        }
    }

    /// # Safety
    ///
    ///
    unsafe fn destroy(&self, logical: &LogicalDeviceFeatures) {
        logical
            .device
            .destroy_sampler(self.object_texture().sampler, None);

        logical
            .device
            .destroy_image_view(self.object_texture().image_view, None);

        logical
            .device
            .destroy_image(self.object_texture().image.image, None);

        logical
            .device
            .free_memory(self.object_texture().image.memory, None);

        logical.device.destroy_descriptor_set_layout(
            self.object_graphics_pipeline()
                .descriptor_set
                .descriptor_set_layout,
            None,
        );

        logical
            .device
            .destroy_buffer(self.object_buffers().vertex.buffer, None);

        logical
            .device
            .free_memory(self.object_buffers().vertex.memory, None);

        logical
            .device
            .destroy_buffer(self.object_buffers().index.buffer, None);

        logical
            .device
            .free_memory(self.object_buffers().index.memory, None);
    }
}

pub trait ObjectBuilder: Object + Sized {
    fn texture_buffer(self, texture_buffer: Vec<u8>) -> Box<Self>;
    fn indexed(self, indexed: bool) -> Box<Self>;
    fn topology(self, topology: ModelTopology) -> Box<Self>;
    fn cull_mode(self, cull_mode: ModelCullMode) -> Box<Self>;
}

#[derive(Clone, new)]
pub struct VerticesAndIndices {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
}

impl VerticesAndIndices {
    pub fn create_buffers(
        &self,
        command_pool: ash::vk::CommandPool,
        command_buffer_count: u32,
        instance_devices: &crate::utility::InstanceDevices,
    ) -> ModelBuffers {
        let vertex = utility::create_vertex_index_buffer(
            (size_of::<Vertex>() * self.vertices.len())
                .try_into()
                .unwrap(),
            &self.vertices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        let index = utility::create_vertex_index_buffer(
            (size_of::<u16>() * self.indices.len()).try_into().unwrap(),
            &self.indices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        ModelBuffers::new(vertex, index)
    }
}

#[derive(Clone, Copy, Debug, new)]
pub struct Vertex {
    pub pos: Vector3<f32>,
    pub colour: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

#[derive(new, Clone, Copy)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

#[derive(new, Clone, Copy)]
pub struct ModelBuffers {
    pub vertex: Buffer,
    pub index: Buffer,
}
