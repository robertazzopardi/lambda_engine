use crate::time::Time;
use lambda_camera::camera::Camera;
use lambda_geometry::{GeomBehavior, Geometries};
use lambda_vulkan::{debug::DebugMessageProperties, renderer, Vulkan, WindowSize};
use lambda_window::window::{self, Display};
use winit::platform::run_return::EventLoopExtRunReturn;

pub struct Engine {
    vulkan: Vulkan,
    current_frame: usize,
    is_frame_buffer_resized: bool,
    time: Time,
}

impl Engine {
    pub fn new(
        display: &Display,
        camera: &mut Camera,
        models: Geometries,
        debugging: Option<DebugMessageProperties>,
    ) -> Self {
        let time = Time::new(60.);

        let geom_properties = models.iter().map(|model| model.features()).collect();

        let vulkan = Vulkan::new(display, camera, geom_properties, debugging);

        Self {
            vulkan,
            current_frame: 0,
            is_frame_buffer_resized: false,
            time,
        }
    }

    pub fn run(&mut self, display: &mut Display, mut camera: Camera) {
        let mut mouse_pressed = false;

        display.event_loop.run_return(|event, _, control_flow| {
            self.time.tick();

            window::handle_inputs(
                control_flow,
                event,
                &display.window,
                &mut camera,
                &mut mouse_pressed,
            );

            self.time.step(
                &mut camera,
                &mut self.vulkan.ubo,
                &WindowSize(self.vulkan.swap_chain.extent),
            );

            unsafe {
                renderer::render(
                    &mut self.vulkan,
                    &display.window,
                    &mut camera,
                    &mut self.current_frame,
                    &mut self.is_frame_buffer_resized,
                    self.time.delta.as_secs_f32(),
                )
            };
        });

        unsafe {
            self.vulkan
                .instance_devices
                .devices
                .logical
                .device
                .device_wait_idle()
                .expect("Failed to wait for device idle state");
        }
    }
}
