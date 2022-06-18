use crate::time::Time;
use derive_builder::Builder;
use lambda_camera::camera::Camera;
use lambda_geometry::{Behavior, GeomBuilder, Geometries};
use lambda_vulkan::{debug::Debugger, renderer, Vulkan};
use lambda_window::{
    prelude::Resolution,
    window::{self, Display},
};
use winit::platform::run_return::EventLoopExtRunReturn;

#[derive(Default, Builder)]
#[builder(build_fn(skip))]
#[builder(name = "Engine")]
pub struct EngineRunner {
    current_frame: usize,
    is_frame_buffer_resized: bool,
    geometries: Geometries,
    time: Time,
    resolution: Resolution,
    camera: Camera,
    debugging: Option<Debugger>,
}

impl Engine {
    pub fn build(&mut self) -> EngineRunner {
        EngineRunner {
            current_frame: self.current_frame.unwrap_or_default(),
            is_frame_buffer_resized: self.is_frame_buffer_resized.unwrap_or_default(),
            geometries: self.geometries.take().unwrap_or_default(),
            time: self.time.unwrap_or_default(),
            resolution: self.resolution.unwrap_or_default(),
            camera: self.camera.unwrap_or_default(),
            debugging: self.debugging.unwrap_or_default(),
        }
    }
}

impl EngineRunner {
    fn main_loop(&mut self, display: &mut Display, backend: &mut Vulkan) {
        let mut mouse_pressed = false;

        display.event_loop.run_return(|event, _, control_flow| {
            self.time.tick();

            window::handle_inputs(
                control_flow,
                event,
                &display.window,
                &mut self.camera,
                &mut mouse_pressed,
            );

            self.geometries.iter_mut().for_each(|geom| geom.actions());

            self.time.step(&mut self.camera, backend);

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

    pub fn run(&mut self) {
        let mut display = Display::new(self.resolution);

        let geom_properties = self
            .geometries
            .iter()
            .map(|model| model.features())
            .collect();

        let mut backend = Vulkan::new(&display, &self.camera, geom_properties, self.debugging);

        self.main_loop(&mut display, &mut backend);

        backend.wait_device_idle()
    }
}
