extern crate derive_builder;
extern crate winit;

pub mod display;
pub mod engine;
mod time;

pub use lambda_camera;
pub use lambda_geometry;
pub use lambda_vulkan;

pub mod prelude {
    pub use crate::{
        display::{Display, Resolution},
        engine::Engine,
        lambda_camera::prelude::*,
        lambda_geometry::prelude::*,
        lambda_vulkan::prelude::*,
    };
}
