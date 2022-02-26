extern crate lambda_engine;

use lambda_engine::{camera::Camera, display::Display, time::Time, Vulkan};

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(3., 1., 2.);

    let vulkan: Vulkan = Vulkan::new(&display.window, &mut camera);

    let mouse_pressed = false;

    let time = Time::new(60.);

    lambda_engine::run(vulkan, display, time, camera, mouse_pressed)
}
