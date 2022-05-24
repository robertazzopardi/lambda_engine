#[macro_use]
extern crate derive_new;
extern crate derive_builder;

pub mod l2d;
pub mod l3d;
pub mod macros;
pub mod utility;

use derive_builder::{Builder, UninitializedFieldError};
use enum_dispatch::enum_dispatch;
use l2d::square::square_from_vertices;
use l3d::cube::{CubeInfo, CUBE_VERTICES};
use lambda_space::space::{Vertex, Vertices, VerticesAndIndices};
use lambda_vulkan::{
    buffer::ModelBuffers, command_buffer::CommandPool, graphics_pipeline::GraphicsPipeline,
    swap_chain::SwapChain, texture::Texture, utility::InstanceDevices, ModelCullMode,
    ModelTopology, RenderPass, ShaderType, VulkanObject,
};
use nalgebra::Vector3;
use std::{fs::File, io::Read};
use utility::calculate_indices;

pub mod prelude {
    pub use crate::{
        l2d::{ring::RingInfoBuilder, square::SquareInfoBuilder},
        l3d::{cube::CubeInfoBuilder, model::ModelInfoBuilder, sphere::SphereInfoBuilder},
        GeometryBuilder, Shape, Shapes,
    };
}

pub type Shape = Box<dyn InternalGeometry>;
pub type Shapes = Vec<Shape>;

pub const WHITE: Vector3<f32> = Vector3::new(1., 1., 1.);
pub const VEC3_ZERO: Vector3<f32> = Vector3::new(0., 0., 0.);

#[enum_dispatch]
pub enum Geom {
    Cube,
}

#[enum_dispatch(Geom)]
pub trait GeomBehavior {
    fn vertices_and_indices(&self) -> u32 {
        unimplemented!()
    }
}

#[derive(new)]
pub struct Cube(Geometry<CubeInfo>);

impl GeomBehavior for Cube {
    fn vertices_and_indices(&self) -> u32 {
        32
    }
}

//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//
//

#[derive(Default, Builder, Debug, Clone)]
#[builder(build_fn(skip))]
pub struct Geometry<T> {
    pub properties: T,

    #[builder(setter(custom))]
    pub texture: Option<Vec<u8>>,
    #[builder(setter(custom))]
    pub indexed: bool,
    pub topology: ModelTopology,
    pub cull_mode: ModelCullMode,
    pub shader: ShaderType,

    pub vulkan_object: VulkanObject,
}

impl<T: Default + Clone> GeometryBuilder<T> {
    pub fn texture<'a>(&mut self, path: &'a str) -> &mut Self {
        let file = File::open(path);

        if let Ok(mut texture_file) = file {
            let mut data = Vec::new();
            texture_file.read_to_end(&mut data).unwrap();
            self.texture = Some(Some(data));
        }

        self
    }

    pub fn indexed(&mut self) -> &mut Self {
        self.indexed = Some(true);
        self
    }

    pub fn build(&self) -> Result<Box<Geometry<T>>, GeometryBuilderError> {
        let properties = self
            .properties
            .as_ref()
            .ok_or_else(|| GeometryBuilderError::from(UninitializedFieldError::new("properties")))?
            .clone();

        let mut res = Box::new(Geometry {
            properties,
            texture: self.texture.clone().unwrap_or(None),
            indexed: self.indexed.unwrap_or_default(),
            topology: self.topology.unwrap_or_default(),
            cull_mode: self.cull_mode.unwrap_or_default(),
            shader: self.shader.unwrap_or_default(),
            vulkan_object: VulkanObject::default(),
        });

        res.vulkan_object.indexed = res.indexed;

        Ok(res)
    }
}

impl<T: Default + Clone> private::InternalGeometry for Geometry<T>
where
    Geometry<T>: InternalGeometry,
{
    fn buffers(&mut self, model_buffers: ModelBuffers) {
        self.vulkan_object.buffers = Some(model_buffers);
    }

    fn texture(&mut self, command_pool: &CommandPool, instance_devices: &InstanceDevices) {
        if let Some(texture) = self.texture.clone() {
            self.vulkan_object.texture_buffer =
                Some(Texture::new(&texture, command_pool, instance_devices));
        }
    }

    fn indexed(&self) -> bool {
        self.indexed
    }

    fn graphics_pipeline(
        &mut self,
        swap_chain: &SwapChain,
        render_pass: &RenderPass,
        instance_devices: &InstanceDevices,
    ) {
        self.vulkan_object.graphics_pipeline = Some(GraphicsPipeline::new(
            swap_chain,
            render_pass.0,
            &self.vulkan_object.texture_buffer,
            self.topology,
            self.cull_mode,
            instance_devices,
            self.shader,
        ));
    }

    fn vulkan_object(&self) -> &VulkanObject {
        &self.vulkan_object
    }
}

pub trait InternalGeometry: private::InternalGeometry {
    fn vertices_and_indices(&mut self) -> &VerticesAndIndices {
        unimplemented!()
    }

    fn defer_build(
        &mut self,
        command_pool: &CommandPool,
        command_buffer_count: u32,
        swap_chain: &SwapChain,
        render_pass: &RenderPass,
        instance_devices: &InstanceDevices,
    ) {
        self.texture(command_pool, instance_devices);

        let vertices_and_indices = self.vertices_and_indices();

        let model_buffers = ModelBuffers::new(
            vertices_and_indices,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        self.buffers(model_buffers);

        self.graphics_pipeline(swap_chain, render_pass, instance_devices);
    }
}

pub(crate) mod private {
    use lambda_vulkan::{
        buffer::ModelBuffers, command_buffer::CommandPool, swap_chain::SwapChain,
        utility::InstanceDevices, RenderPass, VulkanObject,
    };

    pub trait InternalGeometry {
        // NOTE TODO we want this pass by value
        fn vulkan_object(&self) -> &VulkanObject;

        fn buffers(&mut self, _: ModelBuffers);

        fn texture(&mut self, _: &CommandPool, _: &InstanceDevices);

        fn indexed(&self) -> bool;

        fn graphics_pipeline(&mut self, _: &SwapChain, _: &RenderPass, _: &InstanceDevices) {
            unimplemented!()
        }
    }
}
