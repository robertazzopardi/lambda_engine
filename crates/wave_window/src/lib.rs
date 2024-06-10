use ash::{vk, Entry, Instance};
use winit::{
    raw_window_handle::{HasDisplayHandle, HasWindowHandle},
    window::Window,
};

pub mod window;

pub mod prelude {
    pub use crate::window::{Display, Resolution};
}

pub fn create_surface(window: &Window, instance: &Instance, entry: &Entry) -> vk::SurfaceKHR {
    unsafe {
        ash_window::create_surface(
            entry,
            instance,
            window.display_handle().unwrap().as_raw(),
            window.window_handle().unwrap().as_raw(),
            None,
        )
        .expect("Failed to create window surface!")
    }
}
