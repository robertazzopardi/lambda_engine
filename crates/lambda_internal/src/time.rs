use lambda_camera::camera::Camera;
use lambda_vulkan::{uniform_buffer::UniformBufferObject, Vulkan, WindowSize};
use std::time::{Duration, Instant};

pub trait Fps {
    fn duration(self) -> Duration;
}

impl Fps for f32 {
    fn duration(self) -> Duration {
        Duration::from_secs_f32((1000. / self) / 1000.)
    }
}

impl Fps for f64 {
    fn duration(self) -> Duration {
        Duration::from_secs_f64((1000. / self) / 1000.)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Time {
    pub delta: Duration,
    elapsed: Duration,
    now: Instant,
    accumulator: Duration,
}

impl Default for Time {
    fn default() -> Self {
        Self::new(60.)
    }
}

impl Time {
    pub fn new(fps: impl Fps) -> Self {
        Self {
            delta: fps.duration(),
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

    pub fn step(&mut self, camera: &mut Camera, backend: &mut Vulkan) {
        while self.accumulator >= self.delta {
            camera.rotate(self.delta.as_secs_f32());
            backend.ubo.update(&backend.swap_chain.extent, camera);

            self.accumulator -= self.delta;
            self.elapsed += self.delta;

            // println!("{:?}", frame_time);
            // println!("{:?}", 1. / self.delta.as_secs_f32())
        }
    }
}
