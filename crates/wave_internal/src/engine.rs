use crate::time::Time;
use wave_camera::prelude::{Camera, CameraInternal};
use wave_geometry::{Behavior, GeomBuilder};
use wave_vulkan::{debug::Debugger, GeomProperties, Vulkan};
use wave_window::{
    prelude::Resolution,
    window::{Display, Drawable, Input, RenderBackend},
};
use winit::window::Window;

pub struct Engine {
    current_frame: usize,
    is_frame_buffer_resized: bool,
    geometries: Vec<GeomProperties>,
    time: Time,
    resolution: Resolution,
    camera: Option<CameraInternal>,
    debugging: Option<Debugger>,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            current_frame: 0,
            is_frame_buffer_resized: false,
            geometries: Vec::new(),
            time: Time::default(),
            resolution: Resolution::ResFullHD,
            camera: Some(Camera::default().build()),
            debugging: None,
        }
    }
}

impl Engine {
    pub fn with_geometry<T: GeomBuilder + Behavior>(mut self, geometries: &[T]) -> Self {
        geometries
            .iter()
            .for_each(|geom| self.geometries.push(geom.features()));

        self
    }

    pub fn run(self) {
        Display::new(Box::new(self)).start();
    }
}

impl Drawable for Engine {
    fn draw(&mut self, window: &Window, input: &mut Input, renderer: &mut Box<dyn RenderBackend>) {
        self.time.tick();

        // self.geometries.iter_mut().for_each(Behavior::actions);

        // backend.update_objects(&self.geometries);

        self.time.step(&mut self.camera.unwrap(), input, renderer);

        renderer.render(
            window,
            &mut self.current_frame,
            &mut self.is_frame_buffer_resized,
            self.time.delta.as_secs_f32(),
        );
    }

    fn create_renderer(&self, window: &Window) -> Box<dyn RenderBackend>
    where
        Self: Sized,
    {
        Box::new(Vulkan::new(window, &self.geometries, self.debugging))
    }
}
