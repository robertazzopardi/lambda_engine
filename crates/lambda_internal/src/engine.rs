use crate::time::Time;
use derive_builder::Builder;
use lambda_camera::prelude::{Camera, CameraInternal};
use lambda_geometry::{Behavior, GeomBuilder};
use lambda_vulkan::{debug::Debugger, renderer, GeomProperties, Vulkan};
use lambda_window::{
    prelude::Resolution,
    window::{self, Display, Input},
};
use winit::platform::run_return::EventLoopExtRunReturn;

#[derive(Builder)]
#[builder(build_fn(skip))]
#[builder(name = "Engine")]
pub struct EngineRunner<T: GeomBuilder + Behavior> {
    current_frame: usize,
    is_frame_buffer_resized: bool,
    geometries: Vec<T>,
    time: Time,
    resolution: Resolution,
    camera: CameraInternal,
    debugging: Option<Debugger>,
    #[builder(setter(skip))]
    input: Input,
}

impl<T: GeomBuilder + Behavior> Engine<T> {
    pub fn build(&mut self) -> EngineRunner<T> {
        EngineRunner {
            current_frame: self.current_frame.unwrap_or_default(),
            is_frame_buffer_resized: self.is_frame_buffer_resized.unwrap_or_default(),
            geometries: self.geometries.take().unwrap_or_default(),
            time: self.time.unwrap_or_default(),
            resolution: self.resolution.unwrap_or_default(),
            camera: self.camera.unwrap_or_else(|| Camera::default().build()),
            debugging: self.debugging.unwrap_or_default(),
            input: Input::default(),
        }
    }

    pub fn register(&mut self, geom: T) -> &mut Self {
        if self.geometries.is_none() {
            self.geometries = Some(Vec::new());
        }
        let mut geometries = self.geometries.take().unwrap();
        geometries.push(geom);
        self.geometries = Some(geometries);
        self
    }
}

impl<T: GeomBuilder + Behavior> EngineRunner<T> {
    fn main_loop(&mut self, display: &mut Display, backend: &mut Vulkan) {
        display.event_loop.run_return(|event, _, control_flow| {
            self.time.tick();

            window::handle_inputs(control_flow, event, &display.window, &mut self.input);

            self.geometries.iter_mut().for_each(Behavior::actions);

            backend.update_objects(&self.get_geom_properties());

            self.time.step(&mut self.camera, backend, &mut self.input);

            renderer::render(
                backend,
                &display.window,
                &mut self.camera,
                &mut self.current_frame,
                &mut self.is_frame_buffer_resized,
                self.time.delta.as_secs_f32(),
            );
        });
    }

    #[inline]
    fn get_geom_properties(&self) -> Vec<GeomProperties> {
        self.geometries
            .iter()
            .map(|model| model.features())
            .collect::<Vec<GeomProperties>>()
    }

    pub fn run(&mut self) {
        let mut display = Display::new(self.resolution);

        let geom_properties = self.get_geom_properties();

        let mut backend = Vulkan::new(&display, &self.camera, &geom_properties, self.debugging);

        self.main_loop(&mut display, &mut backend);

        backend.wait_device_idle()
    }
}
