extern crate derive_builder;

pub mod engine;
mod time;

pub use lambda_camera;
pub use lambda_geometry;
pub use lambda_vulkan;
pub use lambda_window;

pub mod prelude {
    pub use crate::{
        engine::Engine, lambda_camera::prelude::*, lambda_geometry::prelude::*,
        lambda_vulkan::prelude::*, lambda_window::prelude::*,
    };
}
