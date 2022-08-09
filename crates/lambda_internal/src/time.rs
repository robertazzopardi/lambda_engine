use lambda_camera::camera::CameraInternal;
use lambda_vulkan::Vulkan;
use lambda_window::window::Input;
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
    accumulator: Duration,
    elapsed: Duration,
    pub delta: Duration,
    now: Instant,
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
            now: std::time::Instant::now(),
            elapsed: Duration::ZERO,
            accumulator: Duration::ZERO,
        }
    }

    pub fn tick(&mut self) {
        let new_time = std::time::Instant::now();
        let frame_time = new_time - self.now;
        self.now = new_time;
        self.accumulator += frame_time;
    }

    pub fn step(&mut self, camera: &mut CameraInternal, backend: &mut Vulkan, input: &mut Input) {
        while self.accumulator >= self.delta {
            camera.update(input, self.delta.as_secs_f32());
            backend.ubo.update(&backend.swap_chain.extent, camera);

            self.accumulator -= self.delta;
            self.elapsed += self.delta;

            // println!("{:?}", frame_time);
            // println!("{:?}", 1. / self.delta.as_secs_f32())
        }
    }
}
