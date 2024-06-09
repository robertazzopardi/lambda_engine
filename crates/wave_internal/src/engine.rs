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
    input: Input,
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
            input: Input::default(),
        }
    }
}

impl Engine {
    pub fn with_geometry<T: GeomBuilder + Behavior>(&mut self, geometries: &[T]) -> &mut Self {
        geometries
            .iter()
            .for_each(|geom| self.geometries.push(geom.features()));

        self
    }

    #[inline]
    pub fn add_geometry<T: GeomBuilder + Behavior>(&mut self, geometries: &[T]) -> &mut Self {
        self.with_geometry(geometries)
    }

    pub fn run(self) {
        let mut display = Display::new(Box::new(self));

        display.start();
    }
}

impl Drawable for Engine {
    fn draw(&mut self, window: &Window, renderer: &mut Box<dyn RenderBackend>) {
        self.time.tick();

        // self.geometries.iter_mut().for_each(Behavior::actions);

        // backend.update_objects(&self.get_geom_properties());

        self.time
            .step(&mut self.camera.unwrap(), &mut self.input, renderer);

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
