pub mod cube;
pub mod model;
pub mod sphere;

pub mod prelude {
    pub use super::{
        cube::{Cube, CubeBuilder},
        model::{Model, ModelBuilder},
        sphere::{Sphere, SphereBuilder},
    };
}
