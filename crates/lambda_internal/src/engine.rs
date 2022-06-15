use crate::time::{Fps, Time};
use derive_builder::Builder;
use lambda_camera::camera::Camera;
use lambda_geometry::{GeomBehavior, Geometries};
use lambda_vulkan::{debug::DebugMessageProperties, renderer, Vulkan, WindowSize};
use lambda_window::{
    prelude::Resolution,
    window::{self, Display},
};
use winit::platform::run_return::EventLoopExtRunReturn;

#[derive(Default)]
pub struct Engine {
    backend: Option<Vulkan>,
    current_frame: usize,
    is_frame_buffer_resized: bool,
    models: Geometries,
    time: Time,
    display: Display,
    camera: Camera,
    debugging: Option<DebugMessageProperties>,
}

impl Engine {
    pub fn display(&mut self, res: Resolution) -> &mut Self {
        self.display = Display::new(res);
        self
    }

    pub fn geometries(&mut self, models: Geometries) -> &mut Self {
        self.models = models;
        self
    }

    pub fn time(&mut self, fps: impl Fps) -> &mut Self {
        self.time = Time::new(fps);
        self
    }

    pub fn debugging(&mut self, debugging: DebugMessageProperties) -> &mut Self {
        self.debugging = Some(debugging);
        self
    }

    pub fn camera(&mut self, camera: Camera) -> &mut Self {
        self.camera = camera;
        self
    }

    pub fn build(&mut self) -> &mut Self {
        dbg!(self.debugging);
        self.backend = Some(Vulkan::new(
            &self.display,
            &self.camera,
            self.models.iter().map(|model| model.features()).collect(),
            self.debugging,
        ));

        self
    }

    // pub fn new(
    //     res: Resolution,
    //     models: Geometries,
    //     debugging: Option<DebugMessageProperties>,
    // ) -> Self {
    //     let display = Display::new(res);

    //     let camera = Camera::default();

    //     Self {
    //         backend: Vulkan::new(
    //             &display,
    //             &camera,
    //             models.iter().map(|model| model.features()).collect(),
    //             debugging,
    //         ),
    //         current_frame: 0,
    //         is_frame_buffer_resized: false,
    //         models,
    //         time: Time::new(60.),
    //         display,
    //         camera,
    //     }
    // }

    pub fn run(&mut self) {
        let mut mouse_pressed = false;

        if self.backend.is_none() {
            panic!("Engine must call build to instantiate the renderer")
        }

        let mut backend = self.backend.take().unwrap();

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

                self.time.step(&mut self.camera, &mut backend);

                unsafe {
                    renderer::render(
                        &mut backend,
                        &self.display.window,
                        &mut self.camera,
                        &mut self.current_frame,
                        &mut self.is_frame_buffer_resized,
                        self.time.delta.as_secs_f32(),
                    )
                };
            });

        unsafe {
            backend
                .instance_devices
                .devices
                .logical
                .device
                .device_wait_idle()
                .expect("Failed to wait for device idle state");
        }
    }
}
