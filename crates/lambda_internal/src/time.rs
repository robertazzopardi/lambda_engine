use lambda_camera::camera::Camera;
use lambda_vulkan::{uniform_buffer::UniformBufferObject, WindowSize};
use std::time::{Duration, Instant};

fn calculate_fps(fps: f64) -> f64 {
    (1000. / fps) / 1000.
}

#[derive(Debug, Clone, Copy)]
pub struct Time {
    delta: Duration,
    elapsed: Duration,
    now: Instant,
    accumulator: Duration,
}

impl Time {
    pub fn new(fps: f64) -> Self {
        Self {
            delta: Duration::from_secs_f64(calculate_fps(fps)),
            elapsed: Duration::ZERO,
            now: std::time::Instant::now(),
            accumulator: Duration::ZERO,
        }
    }

    pub fn tick(&mut self) {
        let new_time = std::time::Instant::now();
        let frame_time = new_time - self.now; // from ns to s
        self.now = new_time;
        self.accumulator += frame_time;
    }

    pub fn step(
        &mut self,
        camera: &mut Camera,
        ubo: &mut UniformBufferObject,
        extent: &WindowSize,
    ) {
        while self.accumulator >= self.delta {
            camera.rotate(self.delta.as_secs_f32());
            ubo.update(&extent.0, camera);

            self.accumulator -= self.delta;
            self.elapsed += self.delta;

            // println!("{:?}", frame_time);
            // println!("{:?}", 1. / self.delta.as_secs_f32())
        }
    }
}
