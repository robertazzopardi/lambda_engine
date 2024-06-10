use nalgebra::Matrix4;
use wave_space::space;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, PhysicalSize, Size},
    event::{DeviceEvent, DeviceId, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Cursor, CursorIcon, Window, WindowId},
};

pub trait Drawable {
    fn draw(&mut self, window: &Window, input: &mut Input, renderer: &mut Box<dyn RenderBackend>);

    fn create_renderer(&self, window: &Window) -> Box<dyn RenderBackend>;
}

pub trait RenderBackend {
    fn create(window: &Window) -> Self
    where
        Self: Sized;

    fn render(&mut self, window: &Window, current_frame: &mut usize, resized: &mut bool, dt: f32);

    fn update(&mut self, view: Matrix4<f32>);

    fn destroy(&self);
}

#[derive(Clone, Copy, Debug, Default)]
pub enum Resolution {
    ResSD,
    #[default]
    ResHD,
    ResFullHD,
    ResQHD,
    Res2K,
    Res4K,
    Res8K,
}

impl Resolution {
    pub fn sized(w: u32, h: u32) -> LogicalSize<u32> {
        LogicalSize::new(w, h)
    }
}

impl From<Resolution> for LogicalSize<f64> {
    fn from(res: Resolution) -> Self {
        match res {
            Resolution::ResSD => LogicalSize::new(640., 480.),
            Resolution::ResHD => LogicalSize::new(1_280., 720.),
            Resolution::ResFullHD => LogicalSize::new(1_920., 1_080.),
            Resolution::ResQHD => LogicalSize::new(2_560., 1_440.),
            Resolution::Res2K => LogicalSize::new(2_048., 1_080.),
            Resolution::Res4K => LogicalSize::new(3_840., 2_160.),
            Resolution::Res8K => LogicalSize::new(7_680., 4_320.),
        }
    }
}

pub struct Display {
    pub window: Option<Window>,
    drawable: Box<dyn Drawable>,
    pub renderer: Option<Box<dyn RenderBackend>>,
    pub input: Input,
}

impl ApplicationHandler for Display {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let logical_size: LogicalSize<f64> = Resolution::ResHD.into();

        let window_attributes = Window::default_attributes()
            .with_inner_size(Size::Logical(logical_size))
            .with_resizable(false)
            .with_cursor(Cursor::Icon(CursorIcon::Crosshair));

        let window = event_loop
            .create_window(window_attributes)
            .expect("Could not create window");

        let PhysicalSize { width, height } = window.inner_size();
        window
            .set_cursor_position(PhysicalPosition::new(width / 2, height / 2))
            .expect("Could not center the cursor");

        self.renderer = Some(self.drawable.create_renderer(&window));

        self.window = Some(window);
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if let Some(window) = self.window.as_ref() {
            window.request_redraw();
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if let Some(window) = &self.window {
                    // Redraw the application.
                    //
                    // It's preferable for applications that do not render continuously to render in
                    // this event rather than in AboutToWait, since rendering in here allows
                    // the program to gracefully handle redraws requested by the OS.

                    if let Some(renderer) = &mut self.renderer {
                        self.drawable.draw(window, &mut self.input, renderer);
                    }

                    // Queue a RedrawRequested event.
                    //
                    // You only need to call this if you've determined that you need to redraw in
                    // applications which do not always need to. Applications that redraw continuously
                    // can render here instead.
                    window.request_redraw();
                }
            }
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                // Dispatch actions only on press.
                if event.state.is_pressed() {
                    process_keyboard(&mut self.input, &event.logical_key);
                }

                dbg!("here");
            }
            WindowEvent::MouseInput {
                device_id,
                state,
                button,
            } => {}
            _ => (),
        }
    }

    fn device_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        device_id: DeviceId,
        event: DeviceEvent,
    ) {
        match event {
            DeviceEvent::MouseWheel { delta, .. } => {
                self.input.mouse_scroll = -match delta {
                    MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.,
                    MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                        scroll as f32
                    }
                };
            }
            DeviceEvent::MouseMotion { delta } => {
                if !self.input.first_mouse_event.0 {
                    self.input.mouse_delta = delta
                }
                self.input.first_mouse_event.0 = false;
            }
            _ => {}
        }
    }

    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(renderer) = &self.renderer {
            renderer.destroy();
        }
    }
}

impl Display {
    pub fn new(drawable: Box<dyn Drawable>) -> Self {
        
        Self {
            drawable,
            input: Input::default(),
            window: None,
            renderer: None,
        }
    }

    pub fn start(&mut self) {
        let event_loop = EventLoop::new().unwrap();

        // ControlFlow::Poll continuously runs the event loop, even if the OS hasn't
        // dispatched any events. This is ideal for games and similar applications.
        event_loop.set_control_flow(ControlFlow::Poll);

        // ControlFlow::Wait pauses the event loop if no events are available to process.
        // This is ideal for non-game applications that only update in response to user
        // input, and uses significantly less power/CPU time than ControlFlow::Poll.
        // event_loop.set_control_flow(ControlFlow::Wait);

        event_loop.run_app(self).expect("Could not start app");
    }
}

fn process_keyboard(input: &mut Input, key: &Key) {
    let amount = 1;
    match key.as_ref() {
        Key::Character("w") | Key::Named(NamedKey::ArrowUp) => input.look.set_forward(amount),
        Key::Character("s") | Key::Named(NamedKey::ArrowDown) => {
            input.look.set_back(amount);
        }
        Key::Character("a") | Key::Named(NamedKey::ArrowLeft) => {
            input.look.set_left(amount);
        }
        Key::Character("d") | Key::Named(NamedKey::ArrowRight) => {
            input.look.set_right(amount);
        }
        Key::Named(NamedKey::Space) => {
            input.look.set_up(amount);
        }
        Key::Named(NamedKey::Shift) => {
            input.look.set_down(amount);
        }
        _ => (),
    }
}

#[derive(Debug, Clone, Copy)]
struct FirstCheck(bool);

impl Default for FirstCheck {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Input {
    pub mouse_scroll: f32,
    pub mouse_delta: (f64, f64),
    pub look: space::LookDirection,
    first_mouse_event: FirstCheck,
}

#[cfg(test)]
mod tests {
    

    #[test]
    fn test_resolution() {
        // let res_2k = Resolution::Res2K;
        // let res_4k = Resolution::Res4K;
        // let res_8k = Resolution::Res8K;
        // let res_fhd = Resolution::ResFullHD;
        // let res_hd = Resolution::ResHD;
        // let res_qhd = Resolution::ResQHD;
        // let res_sd = Resolution::ResSD;
        //
        // let res_custom = Resolution::sized(3_442, 2_349);
        //
        // let logical_res: LogicalSize<u32> = res_2k.into();
        // assert_eq!(logical_res, LogicalSize::new(2_048, 1_080));
        //
        // let logical_res: LogicalSize<u32> = res_4k.into();
        // assert_eq!(logical_res, LogicalSize::new(3_840, 2_160));
        //
        // let logical_res: LogicalSize<u32> = res_8k.into();
        // assert_eq!(logical_res, LogicalSize::new(7_680, 4_320));
        //
        // let logical_res: LogicalSize<u32> = res_fhd.into();
        // assert_eq!(logical_res, LogicalSize::new(1_920, 1_080));
        //
        // let logical_res: LogicalSize<u32> = res_hd.into();
        // assert_eq!(logical_res, LogicalSize::new(1_280, 720));
        //
        // let logical_res: LogicalSize<u32> = res_qhd.into();
        // assert_eq!(logical_res, LogicalSize::new(2_560, 1_440));
        //
        // let logical_res: LogicalSize<u32> = res_sd.into();
        // assert_eq!(logical_res, LogicalSize::new(640, 480));
        //
        // assert_eq!(res_custom, LogicalSize::new(3_442, 2_349));
    }
}
