use lambda_space::space;
use nalgebra::Matrix4;
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalPosition, Size},
    event::{DeviceEvent, ElementState, Event, MouseScrollDelta, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::KeyCode,
    window::{CursorIcon, Window, WindowAttributes, WindowId},
};

pub trait Drawable {
    fn draw(&mut self, window: &Window, renderer: &mut Box<dyn RenderBackend>);

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
    pub drawable: Box<dyn Drawable>,
    pub renderer: Option<Box<dyn RenderBackend>>,
}

impl ApplicationHandler for Display {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let logical_size: LogicalSize<f64> = Resolution::ResHD.into();

        //         // NOTE/TODO add these properties to the public api
        //         window.set_resizable(false);
        //         window.set_cursor_icon(CursorIcon::Crosshair);
        //         // let PhysicalSize { width, height } = window.inner_size();
        //         // window
        //         //     .set_cursor_position(PhysicalPosition::new(width / 2, height / 2))
        //         //     .expect("Could not center the mouse");
        //         // window
        //         //     .set_cursor_grab(true)
        //         //     .expect("Could not container mouse");
        //         // window.set_cursor_visible(false);

        let mut window_attributes =
            Window::default_attributes().with_inner_size(Size::Logical(logical_size));

        let mut window = event_loop
            .create_window(window_attributes)
            .expect("Could not create window");

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
                        self.drawable.draw(&window, renderer);
                    }

                    // Queue a RedrawRequested event.
                    //
                    // You only need to call this if you've determined that you need to redraw in
                    // applications which do not always need to. Applications that redraw continuously
                    // can render here instead.
                    window.request_redraw();
                }
            }
            _ => (),
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
        let display = Self {
            drawable,
            window: None,
            renderer: None,
        };
        display
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

fn process_keyboard(input: &mut Input, key: KeyCode, state: ElementState) {
    let amount = (state == ElementState::Pressed) as i8;
    match key {
        KeyCode::KeyW | KeyCode::ArrowUp => input.look.set_forward(amount),
        KeyCode::KeyS | KeyCode::ArrowDown => {
            input.look.set_back(amount);
        }
        KeyCode::KeyA | KeyCode::ArrowLeft => {
            input.look.set_left(amount);
        }
        KeyCode::KeyD | KeyCode::ArrowRight => {
            input.look.set_right(amount);
        }
        KeyCode::Space => {
            input.look.set_up(amount);
        }
        KeyCode::ShiftLeft => {
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
    pub mouse_pressed: bool,
    pub mouse_scroll: f32,
    pub mouse_delta: (f64, f64),
    pub look: space::LookDirection,
    first_mouse_event: FirstCheck,
}

// pub fn handle_inputs(
//     control_flow: &mut ControlFlow,
//     event: Event<()>,
//     window: &Window,
//     input: &mut Input,
// ) {
//     *control_flow = ControlFlow::Poll;
//
//     match event {
//         Event::WindowEvent {
//             window_id,
//             event: WindowEvent::CloseRequested,
//         } => {
//             if window_id == window.id() {
//                 *control_flow = ControlFlow::Exit;
//             }
//         }
//         Event::WindowEvent {
//             event:
//                 WindowEvent::KeyboardInput {
//                     input:
//                         KeyboardInput {
//                             virtual_keycode: Some(key),
//                             state,
//                             ..
//                         },
//                     ..
//                 },
//             ..
//         } => process_keyboard(input, key, state),
//         Event::WindowEvent {
//             event: WindowEvent::MouseInput { state, .. },
//             ..
//         } => input.mouse_pressed = state == ElementState::Pressed,
//         Event::DeviceEvent { event, .. } => match event {
//             DeviceEvent::MouseWheel { delta, .. } => {
//                 input.mouse_scroll = -match delta {
//                     MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.,
//                     MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
//                         scroll as f32
//                     }
//                 };
//             }
//             DeviceEvent::MouseMotion { delta } => {
//                 if !input.first_mouse_event.0 {
//                     input.mouse_delta = delta
//                 }
//                 input.first_mouse_event.0 = false;
//             }
//             _ => {}
//         },
//         _ => {}
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;

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
