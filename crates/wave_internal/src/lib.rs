pub mod engine;
mod time;

pub use wave_camera;
pub use wave_geometry;
pub use wave_proc_macro;
pub use wave_space;
pub use wave_vulkan;
pub use wave_window;

pub mod prelude {
    pub use crate::{
        engine::Engine,
        wave_camera::prelude::*,
        wave_geometry::prelude::*,
        wave_proc_macro::{geometry, geometry_system},
        wave_space::prelude::*,
        wave_vulkan::prelude::*,
        wave_window::prelude::*,
    };
}
