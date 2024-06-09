pub mod l2d;
pub mod l3d;
pub mod macros;
pub mod utility;

use derive_more::Deref;
pub use enum_dispatch::enum_dispatch;
use wave_space::space::{Vertex, VerticesAndIndices};
use wave_vulkan::GeomProperties;
use nalgebra::{vector, Vector3};

pub mod prelude {
    pub use crate::{
        enum_dispatch,
        l2d::prelude::*,
        l3d::prelude::*,
        utility::{scaled_axis_matrix_4, Transformation},
        Behavior, GeomBuilder, Indexed,
    };
}

pub const WHITE: Vector3<f32> = vector![1., 1., 1.];
pub const VEC3_ZERO: Vector3<f32> = vector![0., 0., 0.];

#[derive(Clone, Copy, Debug, Deref)]
pub struct Indexed(pub bool);

impl Default for Indexed {
    fn default() -> Self {
        Self(true)
    }
}

#[enum_dispatch]
pub trait GeomBuilder {
    fn vertices_and_indices(&self) -> VerticesAndIndices;

    fn features(&self) -> GeomProperties;
}

#[enum_dispatch]
pub trait Behavior {
    fn actions(&mut self);
}
