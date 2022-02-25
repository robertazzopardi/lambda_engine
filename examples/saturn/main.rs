extern crate lambda_engine;

use lambda_engine::{camera::Camera, update, window, Vulkan};
use std::time::{Duration, Instant};
use winit::{event_loop::EventLoop, window::WindowBuilder};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;

// pub fn create_window() -> Window {
//     let event_loop = EventLoop::new();
//     let window = WindowBuilder::new()
//         .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
//         .build(&event_loop)
//         .unwrap();
// }

#[derive(Debug)]
pub struct Time {
    delta: Duration,
    elapsed: Duration,
    now: Instant,
    accumulator: Duration,
}

impl Time {
    fn new() -> Self {
        Self {
            delta: Duration::from_secs_f32(0.01666),
            elapsed: Duration::ZERO,
            now: std::time::Instant::now(),
            accumulator: Duration::ZERO,
        }
    }

    fn step(&mut self) {
        let new_time = std::time::Instant::now();
        let frame_time = new_time - self.now; // from ns to s
        self.now = new_time;
        self.accumulator += frame_time;
    }

    fn update(&mut self) {}
}

pub fn run() {}

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .unwrap();

    let mut camera = Camera::new(5., 1., 2.);

    let mut vulkan: Vulkan = Vulkan::new(&window, &mut camera);

    let mut mouse_pressed = false;

    let mut time = Time::new();

    event_loop.run(move |event, _, control_flow| {
        time.step();

        window::handle_inputs(
            control_flow,
            event,
            &window,
            &mut camera,
            &mut mouse_pressed,
        );

        while time.accumulator >= time.delta {
            update(&mut vulkan, &mut camera, time.delta.as_secs_f32());
            time.accumulator -= time.delta;
            time.elapsed += time.delta;

            // println!("{:?}", frame_time);
            // println!("{:?}", 1. / dt.as_secs_f32())
        }

        unsafe { vulkan.render(&window, &mut camera) };
    });
}
