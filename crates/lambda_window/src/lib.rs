extern crate winit;

pub mod window;

use ash::{vk, Entry, Instance};
use winit::window::Window;

pub mod prelude {
    pub use crate::window::{Display, Resolution};
}

pub fn create_surface(window: &Window, instance: &Instance, entry: &Entry) -> vk::SurfaceKHR {
    unsafe {
        ash_window::create_surface(entry, instance, window, None)
            .expect("Failed to create window surface!")
    }
}
