#[macro_use]
extern crate derive_new;
extern crate derive_builder;

pub mod l2d;
pub mod l3d;
pub mod macros;
pub mod utility;

use derive_builder::{Builder, UninitializedFieldError};
use enum_dispatch::enum_dispatch;
use l3d::cube::Cube;
use lambda_space::space::{Vertex, VerticesAndIndices};
use lambda_vulkan::{
    command_buffer::CommandPool, swap_chain::SwapChain, utility::InstanceDevices, ModelCullMode,
    ModelTopology, RenderPass, ShaderType, VulkanObject,
};
use nalgebra::Vector3;
use prelude::{Model, Ring, Sphere, Square};
use std::{fs::File, io::Read};

pub mod prelude {
    pub use crate::{
        l2d::{
            ring::{Ring, RingInfoBuilder},
            square::{Square, SquareInfoBuilder},
        },
        l3d::{
            cube::{Cube, CubeInfoBuilder},
            model::{Model, ModelInfoBuilder},
            sphere::{Sphere, SphereInfoBuilder},
        },
        Geometries, GeometryBuilder,
    };
}

pub type Geometries = Vec<Geom>;

pub const WHITE: Vector3<f32> = Vector3::new(1., 1., 1.);
pub const VEC3_ZERO: Vector3<f32> = Vector3::new(0., 0., 0.);

#[enum_dispatch]
pub enum Geom {
    Cube,
    Square,
    Sphere,
    Ring,
    Model,
}

#[enum_dispatch(Geom)]
pub trait GeomBehavior {
    fn vertices_and_indices(&mut self) -> VerticesAndIndices;

    fn vulkan_object(&self) -> VulkanObject;

    fn defer_build(
        &mut self,
        _: &CommandPool,
        _: u32,
        _: &SwapChain,
        _: &RenderPass,
        _: &InstanceDevices,
    ) {
        unimplemented!()
    }
}

#[derive(Default, Builder, Debug, Clone, new)]
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

impl<T: Clone> GeometryBuilder<T> {
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

    pub fn build(&self) -> Result<Geometry<T>, GeometryBuilderError> {
        let properties = self
            .properties
            .as_ref()
            .ok_or_else(|| GeometryBuilderError::from(UninitializedFieldError::new("properties")))?
            .clone();

        let mut res = Geometry::new(
            properties,
            self.texture.clone().unwrap_or(None),
            self.indexed.unwrap_or_default(),
            self.topology.unwrap_or_default(),
            self.cull_mode.unwrap_or_default(),
            self.shader.unwrap_or_default(),
            VulkanObject::default(),
        );

        res.vulkan_object.indexed = res.indexed;

        Ok(res)
    }
}
