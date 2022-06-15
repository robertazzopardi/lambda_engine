use crate::time::Time;
use derive_builder::Builder;
use lambda_camera::camera::Camera;
use lambda_geometry::{GeomBehavior, Geometries};
use lambda_vulkan::{debug::DebugMessageProperties, renderer, Vulkan, WindowSize};
use lambda_window::{
    prelude::Resolution,
    window::{self, Display},
};
use winit::platform::run_return::EventLoopExtRunReturn;

pub struct Engine {
    backend: Vulkan,
    current_frame: usize,
    is_frame_buffer_resized: bool,
    models: Geometries,
    time: Time,
    display: Display,
    camera: Camera,
}

impl Engine {
    pub fn new(
        res: Resolution,
        models: Geometries,
        debugging: Option<DebugMessageProperties>,
    ) -> Self {
        let display = Display::new(res);

        let camera = Camera::new(-2., 1., 0.);

        Self {
            backend: Vulkan::new(
                &display,
                &camera,
                models.iter().map(|model| model.features()).collect(),
                debugging,
            ),
            current_frame: 0,
            is_frame_buffer_resized: false,
            models,
            time: Time::new(60.),
            display,
            camera,
        }
    }

    pub fn run(&mut self) {
        let mut mouse_pressed = false;

        self.display
            .event_loop
            .run_return(|event, _, control_flow| {
                self.time.tick();

                window::handle_inputs(
                    control_flow,
                    event,
                    &self.display.window,
                    &mut self.camera,
                    &mut mouse_pressed,
                );

                self.time.step(
                    &mut self.camera,
                    &mut self.backend.ubo,
                    &WindowSize(self.backend.swap_chain.extent),
                );

                unsafe {
                    renderer::render(
                        &mut self.backend,
                        &self.display.window,
                        &mut self.camera,
                        &mut self.current_frame,
                        &mut self.is_frame_buffer_resized,
                        self.time.delta.as_secs_f32(),
                    )
                };
            });

        unsafe {
            self.backend
                .instance_devices
                .devices
                .logical
                .device
                .device_wait_idle()
                .expect("Failed to wait for device idle state");
        }
    }
}
