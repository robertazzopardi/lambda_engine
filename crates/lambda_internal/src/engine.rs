use crate::time::Time;
use derive_builder::Builder;
use lambda_camera::prelude::{Camera, CameraInternal};
use lambda_geometry::{Behavior, GeomBuilder};
use lambda_vulkan::{debug::Debugger, renderer, GeomProperties, Vulkan};
use lambda_window::{
    prelude::Resolution,
    window::{self, Display, Drawable, Input, RenderBackend},
};
use winit::{
    event_loop::{ControlFlow, EventLoop},
    window::Window,
};

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
    // pub fn build(&mut self) -> EngineRunner {
    //     EngineRunner {
    //         current_frame: self.current_frame,
    //         is_frame_buffer_resized: self.is_frame_buffer_resized,
    //         geometries: self.geometries,
    //         time: self.time.unwrap_or_default(),
    //         resolution: self.resolution.unwrap_or_default(),
    //         camera: self.camera.unwrap_or_else(|| Camera::default().build()),
    //         debugging: self.debugging.unwrap_or_default(),
    //         input: Input::default(),
    //     }
    // }

    // pub fn register<T: GeomBuilder + Behavior>(&mut self, geom: T) -> &mut Self {
    //     if self.geometries.is_none() {
    //         self.geometries = Some(Vec::new());
    //     }
    //     // let mut geometries = self.geometries.take().unwrap();
    //     // geometries.push(geom);
    //     // self.geometries = Some(geometries);
    //     self
    // }
}

impl Engine {
    // fn main_loop(&mut self, display: &mut Display, backend: &mut Vulkan) {
    //     display.event_loop.run_return(|event, _, control_flow| {
    //         self.time.tick();
    //
    //         // window::handle_inputs(control_flow, event, &display.window, &mut self.input);
    //
    //         self.geometries.iter_mut().for_each(Behavior::actions);
    //
    //         backend.update_objects(&self.get_geom_properties());
    //
    //         self.time.step(&mut self.camera, backend, &mut self.input);
    //
    //         renderer::render(
    //             backend,
    //             &display.window,
    //             &mut self.camera,
    //             &mut self.current_frame,
    //             &mut self.is_frame_buffer_resized,
    //             self.time.delta.as_secs_f32(),
    //         );
    //     });
    // }

    #[inline]
    fn get_geom_properties(&self) -> Vec<GeomProperties> {
        // self.geometries
        //     .iter()
        //     .map(|model| model.features())
        //     .collect::<Vec<GeomProperties>>()
        vec![]
    }

    pub fn run(self) {
        // let geom_properties = self.get_geom_properties();

        let mut display = Display::new(Box::new(self));

        // let mut backend = Vulkan::new(&display.window.unwrap(), &geom_properties, self.debugging);

        // self.main_loop(&mut display, &mut backend);

        display.start();

        // backend.wait_device_idle()
    }
}

impl Drawable for Engine {
    fn draw(&mut self, window: &Window, renderer: &mut Box<dyn RenderBackend>) {
        self.time.tick();

        // self.geometries.iter_mut().for_each(Behavior::actions);

        // backend.update_objects(&self.get_geom_properties());

        self.time
            .step(&mut self.camera.unwrap(), &mut self.input, renderer);

        // renderer::render(
        //     backend,
        //     &window,
        //     &mut self.current_frame,
        //     &mut self.is_frame_buffer_resized,
        //     self.time.delta.as_secs_f32(),
        // );

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
        Box::new(Vulkan::new(window, &vec![], None))
    }
}
