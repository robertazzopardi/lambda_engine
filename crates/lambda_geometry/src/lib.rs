#[macro_use]
extern crate derive_new;
extern crate derive_builder;

pub mod l2d;
pub mod l3d;
pub mod macros;
pub mod utility;

use derive_builder::Builder;
use derive_more::Deref;
use enum_dispatch::enum_dispatch;
use lambda_space::space::{Vertex, VerticesAndIndices};
use lambda_vulkan::{CullMode, GeomProperties, ModelTopology, Shader};
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
            cube::{Cube, CubeBuilder, CubeInfo},
            model::{Model, ModelBuilder},
            sphere::{Sphere, SphereBuilder},
        },
        Behavior, Geometries, GeometryBuilder, Indexed, TextureBuffer,
    };
}

pub type Geometries = Vec<Geom>;

pub const WHITE: Vector3<f32> = Vector3::new(1., 1., 1.);
pub const VEC3_ZERO: Vector3<f32> = Vector3::new(0., 0., 0.);

#[derive(Clone, Copy, Debug, Deref)]
pub struct Indexed(pub bool);

impl Default for Indexed {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Clone, Debug, Deref, Default)]
pub struct TextureBuffer(pub Vec<u8>);

#[enum_dispatch]
pub trait GeomBuilder {
    fn vertices_and_indices(&self) -> VerticesAndIndices;

    fn features(&self) -> GeomProperties;
}

#[enum_dispatch]
pub trait Behavior {
    fn actions(&mut self);
}

#[enum_dispatch(GeomBuilder, Behavior)]
#[derive(Debug, Clone)]
pub enum Geom {
    Cube,
    Square,
    Sphere,
    Ring,
    Model,
}

#[derive(Default, Builder, Debug, Clone)]
#[builder(build_fn(skip))]
pub struct Geometry<T> {
    pub properties: T,
    #[builder(setter(custom))]
    pub texture: TextureBuffer,
    #[builder(setter(custom))]
    pub indexed: Indexed,
    pub topology: ModelTopology,
    pub cull_mode: CullMode,
    pub shader: Shader,
}

impl<T> GeometryBuilder<T> {
    pub fn texture(&mut self, path: &str) -> &mut Self {
        let file = File::open(path);

        if let Ok(mut texture_file) = file {
            let mut data = Vec::new();
            texture_file
                .read_to_end(&mut data)
                .expect("Failed to read contents of texture file");
            self.texture = Some(TextureBuffer(data));
        }

        self
    }

    pub fn no_index(&mut self) -> &mut Self {
        self.indexed = Some(Indexed(false));
        self
    }

    pub fn build(&mut self) -> Geometry<T> {
        Geometry {
            properties: self
                .properties
                .take()
                .expect("Expected the field `properties` to be defined for this geometry"),
            texture: self.texture.take().unwrap_or_default(),
            indexed: self.indexed.unwrap_or_default(),
            topology: self.topology.unwrap_or_default(),
            cull_mode: self.cull_mode.unwrap_or_default(),
            shader: self.shader.unwrap_or_default(),
        }
    }
}
