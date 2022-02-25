extern crate lambda_engine;

use std::time::Duration;

use lambda_engine::{camera::Camera, Vulkan, window, update};
use winit::{event_loop::EventLoop, window::WindowBuilder};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .unwrap();

    let mut camera = Camera::new(5., 1., 2.);

    let mut vulkan: Vulkan = Vulkan::new(&window, &mut camera);

    let mut mouse_pressed = false;

    let dt = Duration::from_secs_f32(0.01666);
    let mut t = Duration::ZERO;
    let mut current_time = std::time::Instant::now();
    let mut accumulator = Duration::ZERO;

    event_loop.run(move |event, _, control_flow| {
        let new_time = std::time::Instant::now();
        let frame_time = new_time - current_time; // from ns to s
        current_time = new_time;
        accumulator += frame_time;

        window::handle_inputs(
            control_flow,
            event,
            &window,
            &mut camera,
            &mut mouse_pressed,
        );

        while accumulator >= dt {
            // update(t, dt);
            update(&mut vulkan, &mut camera, dt.as_secs_f32());
            accumulator -= dt;
            t += dt;

            // println!("{:?}", frame_time);
            // println!("{:?}", 1. / dt.as_secs_f32())
        }

        unsafe { vulkan.render(&window, &mut camera) };
    });
}
