#[macro_use]
extern crate derive_new;
extern crate derive_builder;

pub mod l2d;
pub mod l3d;
pub mod macros;
pub mod utility;

use derive_builder::Builder;
use enum_dispatch::enum_dispatch;
use lambda_space::space::{Vertex, VerticesAndIndices};
use lambda_vulkan::{
    command_buffer::CommandPool, swap_chain::SwapChain, utility::InstanceDevices, CullMode,
    ModelTopology, RenderPass, Shader, VulkanObject,
};
use nalgebra::Vector3;
use prelude::{Cube, Model, Ring, Sphere, Square};
use std::{fs::File, io::Read};

pub mod prelude {
    pub use crate::{
        l2d::{
            ring::{Ring, RingBuilder},
            square::{Square, SquareBuilder},
        },
        l3d::{
            cube::{Cube, CubeBuilder},
            model::{Model, ModelBuilder},
            sphere::{Sphere, SphereBuilder},
        },
        Geometries, GeometryBuilder,
    };
}

pub type Geometries = Vec<Geom>;

pub const WHITE: Vector3<f32> = Vector3::new(1., 1., 1.);
pub const VEC3_ZERO: Vector3<f32> = Vector3::new(0., 0., 0.);

#[enum_dispatch]
#[derive(Debug, Clone)]
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

#[derive(Default, Builder, Debug, Clone)]
#[builder(build_fn(skip))]
pub struct Geometry<T> {
    pub properties: T,

    #[builder(setter(custom))]
    pub texture: Vec<u8>,
    #[builder(setter(custom))]
    pub indexed: bool,
    pub topology: ModelTopology,
    pub cull_mode: CullMode,
    pub shader: Shader,

    #[builder(setter(custom))]
    pub behavior: Option<fn()>,

    pub vulkan_object: VulkanObject,
}

impl<T: Clone> GeometryBuilder<T> {
    pub fn texture(&mut self, path: &str) -> &mut Self {
        let file = File::open(path);

        if let Ok(mut texture_file) = file {
            let mut data = Vec::new();
            texture_file
                .read_to_end(&mut data)
                .expect("Failed to read contents of texture file");
            self.texture = Some(data);
        }

        self
    }

    pub fn behavior(&mut self, actions: fn()) -> &mut Self {
        self.behavior = Some(Some(actions));
        self
    }

    pub fn indexed(&mut self) -> &mut Self {
        self.indexed = Some(true);
        self
    }

    pub fn build(&mut self) -> Geometry<T> {
        let properties = self
            .properties
            .take()
            .expect("Expected the field `properties` to be defined for this geometry");

        Geometry {
            properties,
            texture: self.texture.take().unwrap_or_default(),
            indexed: self.indexed.unwrap_or_default(),
            topology: self.topology.unwrap_or_default(),
            cull_mode: self.cull_mode.unwrap_or_default(),
            shader: self.shader.unwrap_or_default(),
            behavior: self.behavior.unwrap_or_default(),
            vulkan_object: VulkanObject::new(self.indexed.unwrap_or_default()),
        }
    }
}
