pub mod l2d;
pub mod l3d;
pub mod macros;
pub mod utility;

use self::utility::{ModelCullMode, ModelTopology};
use crate::{
    device::{Devices, LogicalDeviceFeatures},
    pipeline::GraphicsPipeline,
    swap_chain::SwapChain,
    texture::{self, Texture},
    utility::InstanceDevices,
};
use ash::vk;
use derive_builder::{Builder, UninitializedFieldError};
use derive_more::{Deref, DerefMut, From};
use nalgebra::{Point3, Vector2, Vector3};
use std::{fs::File, io::Read, mem::size_of};

pub type Shape = Box<dyn InternalObject>;
pub type Shapes = Vec<Shape>;

pub const WHITE: Vector3<f32> = Vector3::new(1., 1., 1.);
pub const VEC3_ZERO: Vector3<f32> = Vector3::new(0., 0., 0.);

#[derive(Default, Builder, Debug, Clone)]
#[builder(build_fn(skip))]
pub struct Object<T: Default + Clone> {
    pub properties: T,

    #[builder(setter(custom))]
    pub texture: Vec<u8>,
    pub indexed: bool,
    pub topology: ModelTopology,
    pub cull_mode: ModelCullMode,

    pub(crate) vertices_and_indices: Option<VerticesAndIndices>,
    pub(crate) texture_buffer: Option<Texture>,
    pub(crate) graphics_pipeline: Option<GraphicsPipeline>,
    pub(crate) buffers: Option<ModelBuffers>,
}

impl<'a, T: Default + Clone> ObjectBuilder<T> {
    pub fn texture(&mut self, path: &'a str) -> &mut Self {
        let file = File::open(path);

        if let Ok(mut texture_file) = file {
            let mut data = Vec::new();
            texture_file.read_to_end(&mut data).unwrap();
            self.texture = Some(data);
        }

        self
    }

    pub fn build(&self) -> Result<Box<Object<T>>, ObjectBuilderError> {
        let properties = self
            .properties
            .as_ref()
            .ok_or_else(|| ObjectBuilderError::from(UninitializedFieldError::new("properties")))?
            .clone();

        let texture = self
            .texture
            .as_ref()
            .ok_or_else(|| ObjectBuilderError::from(UninitializedFieldError::new("texture")))?
            .clone();

        Ok(Box::new(Object {
            properties,
            texture,
            indexed: self.indexed.unwrap_or_default(),
            topology: self.topology.unwrap_or_default(),
            cull_mode: self.cull_mode.unwrap_or_default(),
            vertices_and_indices: None,
            texture_buffer: None,
            graphics_pipeline: None,
            buffers: None,
        }))
    }
}

impl<T: Default + Clone> private::InternalObject for Object<T>
where
    Object<T>: InternalObject,
{
    fn buffers(&mut self, model_buffers: ModelBuffers) {
        self.buffers = Some(model_buffers);
    }

    fn texture(&mut self, command_pool: vk::CommandPool, instance_devices: &InstanceDevices) {
        if !self.texture.is_empty() {
            self.texture_buffer = Some(texture::Texture::new(
                &self.texture,
                command_pool,
                instance_devices,
            ));
        }
    }

    fn is_indexed(&self) -> bool {
        self.indexed
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
            &self.texture_buffer.unwrap(),
            self.topology,
            self.cull_mode,
            instance_devices,
        ));
    }

    fn object_graphics_pipeline(&self) -> &GraphicsPipeline {
        self.graphics_pipeline.as_ref().unwrap()
    }

    fn object_buffers(&self) -> &ModelBuffers {
        self.buffers.as_ref().unwrap()
    }

    fn object_texture(&self) -> &Texture {
        self.texture_buffer.as_ref().unwrap()
    }

    fn object_vertices_and_indices(&self) -> &VerticesAndIndices {
        self.vertices_and_indices.as_ref().unwrap()
    }
}

pub trait InternalObject: private::InternalObject {
    fn vertices_and_indices(&mut self);

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

        let model_buffers = ModelBuffers::new(
            self.object_vertices_and_indices(),
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        self.buffers(model_buffers);

        self.graphics_pipeline(swap_chain, render_pass, instance_devices);
    }
}

pub(crate) mod private {
    use super::{ModelBuffers, VerticesAndIndices};
    use crate::{
        pipeline::GraphicsPipeline, swap_chain::SwapChain, texture::Texture,
        utility::InstanceDevices,
    };
    use ash::vk;

    pub trait InternalObject {
        fn object_graphics_pipeline(&self) -> &GraphicsPipeline {
            unimplemented!()
        }
        fn object_buffers(&self) -> &ModelBuffers {
            unimplemented!()
        }
        fn object_texture(&self) -> &Texture {
            unimplemented!()
        }
        fn object_vertices_and_indices(&self) -> &VerticesAndIndices {
            unimplemented!()
        }

        fn buffers(&mut self, model_buffers: ModelBuffers);
        fn texture(&mut self, command_pool: vk::CommandPool, instance_devices: &InstanceDevices);
        fn is_indexed(&self) -> bool {
            unimplemented!()
        }
        fn graphics_pipeline(
            &mut self,
            _: &SwapChain,
            _: ash::vk::RenderPass,
            _: &InstanceDevices,
        ) {
            unimplemented!()
        }
    }
}

/// # Safety
///
/// Expand on safety of this function
pub(crate) unsafe fn bind_index_and_vertex_buffers(
    object: &Shape,
    devices: &Devices,
    command_buffer: vk::CommandBuffer,
    offsets: &[vk::DeviceSize],
    index: usize,
) {
    let object_graphics_pipeline = object.object_graphics_pipeline();

    devices.logical.device.cmd_bind_pipeline(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        object_graphics_pipeline.features.pipeline,
    );

    devices.logical.device.cmd_bind_descriptor_sets(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        object_graphics_pipeline.features.layout,
        0,
        std::slice::from_ref(&object_graphics_pipeline.descriptor_set.descriptor_sets[index]),
        &[],
    );

    let object_buffers = object.object_buffers();

    let vertex_buffers = [object_buffers.vertex.buffer];

    devices
        .logical
        .device
        .cmd_bind_vertex_buffers(command_buffer, 0, &vertex_buffers, offsets);

    let object_and_vertices_and_indices = object.object_vertices_and_indices();

    devices.logical.device.cmd_draw(
        command_buffer,
        object_and_vertices_and_indices.vertices.len() as u32,
        1,
        0,
        0,
    );

    if object.is_indexed() {
        devices.logical.device.cmd_bind_index_buffer(
            command_buffer,
            object_buffers.index.buffer,
            0,
            vk::IndexType::UINT16,
        );

        devices.logical.device.cmd_draw_indexed(
            command_buffer,
            object_and_vertices_and_indices.indices.len() as u32,
            1,
            0,
            0,
            0,
        );
    }
}

/// # Safety
///
///
pub(crate) unsafe fn recreate_drop(
    graphics_pipeline: &GraphicsPipeline,
    logical: &LogicalDeviceFeatures,
    swap_chain: &SwapChain,
) {
    logical
        .device
        .destroy_pipeline(graphics_pipeline.features.pipeline, None);
    logical
        .device
        .destroy_pipeline_layout(graphics_pipeline.features.layout, None);

    logical
        .device
        .destroy_descriptor_pool(graphics_pipeline.descriptor_set.descriptor_pool, None);

    for i in 0..swap_chain.images.len() {
        logical.device.destroy_buffer(
            graphics_pipeline.descriptor_set.uniform_buffers[i].buffer,
            None,
        );
        logical.device.free_memory(
            graphics_pipeline.descriptor_set.uniform_buffers[i].memory,
            None,
        );
    }
}

/// # Safety
///
///
pub(crate) unsafe fn destroy(object: &Shape, logical: &LogicalDeviceFeatures) {
    let object_texture = object.object_texture();

    logical.device.destroy_sampler(object_texture.sampler, None);

    logical
        .device
        .destroy_image_view(object_texture.image_view, None);

    logical
        .device
        .destroy_image(object_texture.image.image, None);

    logical
        .device
        .free_memory(object_texture.image.memory, None);

    logical.device.destroy_descriptor_set_layout(
        object
            .object_graphics_pipeline()
            .descriptor_set
            .descriptor_set_layout,
        None,
    );

    let object_buffers = object.object_buffers();

    logical
        .device
        .destroy_buffer(object_buffers.vertex.buffer, None);

    logical
        .device
        .free_memory(object_buffers.vertex.memory, None);

    logical
        .device
        .destroy_buffer(object_buffers.index.buffer, None);

    logical
        .device
        .free_memory(object_buffers.index.memory, None);
}

#[derive(new, Clone, Default, Debug, From, Deref, DerefMut)]
pub struct Vertices(Vec<Vertex>);

#[derive(new, Clone, Default, Debug, From, Deref, DerefMut)]
pub struct Indices(Vec<u16>);

#[derive(new, Clone, Default, Debug)]
pub struct VerticesAndIndices {
    vertices: Vertices,
    indices: Indices,
}

#[derive(Clone, Copy, Debug, new)]
pub struct Vertex {
    pub pos: Point3<f32>,
    pub colour: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

#[derive(new, Clone, Copy, Default, Debug)]
pub struct Buffer {
    pub buffer: vk::Buffer,
    pub memory: vk::DeviceMemory,
}

#[derive(Clone, Copy, Default, Debug)]
pub struct ModelBuffers {
    pub vertex: Buffer,
    pub index: Buffer,
}

impl ModelBuffers {
    fn new(
        vertices_and_indices: &VerticesAndIndices,
        command_pool: ash::vk::CommandPool,
        command_buffer_count: u32,
        instance_devices: &crate::utility::InstanceDevices,
    ) -> Self {
        let vertex = utility::create_vertex_index_buffer(
            (size_of::<Vertex>() * vertices_and_indices.vertices.len())
                .try_into()
                .unwrap(),
            &vertices_and_indices.vertices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        let index = utility::create_vertex_index_buffer(
            (size_of::<u16>() * vertices_and_indices.indices.len())
                .try_into()
                .unwrap(),
            &vertices_and_indices.indices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        ModelBuffers { vertex, index }
    }
}
