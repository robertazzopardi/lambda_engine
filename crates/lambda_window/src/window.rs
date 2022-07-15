use lambda_space::space;
use winit::{
    dpi::{LogicalSize, PhysicalPosition},
    event::{
        DeviceEvent, ElementState, Event, KeyboardInput, MouseScrollDelta, VirtualKeyCode,
        WindowEvent,
    },
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

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

impl From<Resolution> for LogicalSize<u32> {
    fn from(res: Resolution) -> Self {
        match res {
            Resolution::ResSD => LogicalSize::new(640, 480),
            Resolution::ResHD => LogicalSize::new(1_280, 720),
            Resolution::ResFullHD => LogicalSize::new(1_920, 1_080),
            Resolution::ResQHD => LogicalSize::new(2_560, 1_440),
            Resolution::Res2K => LogicalSize::new(2_048, 1_080),
            Resolution::Res4K => LogicalSize::new(3_840, 2_160),
            Resolution::Res8K => LogicalSize::new(7_680, 4_320),
        }
    }
}

#[derive(Debug)]
pub struct Display {
    pub window: Window,
    pub event_loop: EventLoop<()>,
}

impl Default for Display {
    fn default() -> Self {
        let event_loop = EventLoop::new();

        let logical_size: LogicalSize<u32> = Resolution::ResHD.into();
        let window = WindowBuilder::new()
            .with_inner_size(logical_size)
            .build(&event_loop)
            .unwrap();

        Self { window, event_loop }
    }
}

impl Display {
    pub fn new(res: Resolution) -> Self {
        let display = Self::default();
        let logical_size: LogicalSize<u32> = res.into();
        display.window.set_inner_size(logical_size);
        display
    }
}

fn process_keyboard(input: &mut Input, key: VirtualKeyCode, state: ElementState) {
    let amount = (state == ElementState::Pressed) as u32 as f32;
    if let VirtualKeyCode::W | VirtualKeyCode::Up = key {
        input.direction.f = amount;
    }
    if let VirtualKeyCode::S | VirtualKeyCode::Down = key {
        input.direction.b = amount;
    }
    if let VirtualKeyCode::A | VirtualKeyCode::Left = key {
        input.direction.l = amount;
    }
    if let VirtualKeyCode::D | VirtualKeyCode::Right = key {
        input.direction.r = amount;
    }
    if let VirtualKeyCode::Space = key {
        input.direction.u = amount;
    }
    if let VirtualKeyCode::LShift = key {
        input.direction.d = amount;
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct Input {
    pub mouse_pressed: bool,
    pub mouse_scroll: f32,
    pub mouse_delta: (f64, f64),
    pub direction: space::LookDirection,
}

pub fn handle_inputs(
    control_flow: &mut ControlFlow,
    event: Event<()>,
    window: &Window,
    input: &mut Input,
) {
    *control_flow = ControlFlow::Poll;

    if let Event::WindowEvent {
        window_id,
        event: WindowEvent::CloseRequested,
    } = event
    {
        if window_id == window.id() {
            *control_flow = ControlFlow::Exit;
        }
    }

    if let Event::WindowEvent {
        event:
            WindowEvent::KeyboardInput {
                input:
                    KeyboardInput {
                        virtual_keycode: Some(key),
                        state,
                        ..
                    },
                ..
            },
        ..
    } = event
    {
        process_keyboard(input, key, state);
    }

    if let Event::WindowEvent {
        event: WindowEvent::MouseInput { state, .. },
        ..
    } = event
    {
        input.mouse_pressed = state == ElementState::Pressed;
    }

    if let Event::DeviceEvent { event, .. } = event {
        if let DeviceEvent::MouseWheel { delta, .. } = event {
            input.mouse_scroll = -match delta {
                MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.,
                MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => scroll as f32,
            };
        }
        if let DeviceEvent::MouseMotion { delta } = event {
            input.mouse_delta = delta
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolution() {
        let res_2k = Resolution::Res2K;
        let res_4k = Resolution::Res4K;
        let res_8k = Resolution::Res8K;
        let res_fhd = Resolution::ResFullHD;
        let res_hd = Resolution::ResHD;
        let res_qhd = Resolution::ResQHD;
        let res_sd = Resolution::ResSD;

        let res_custom = Resolution::sized(3_442, 2_349);

        let logical_res: LogicalSize<u32> = res_2k.into();
        assert_eq!(logical_res, LogicalSize::new(2_048, 1_080));

        let logical_res: LogicalSize<u32> = res_4k.into();
        assert_eq!(logical_res, LogicalSize::new(3_840, 2_160));

        let logical_res: LogicalSize<u32> = res_8k.into();
        assert_eq!(logical_res, LogicalSize::new(7_680, 4_320));

        let logical_res: LogicalSize<u32> = res_fhd.into();
        assert_eq!(logical_res, LogicalSize::new(1_920, 1_080));

        let logical_res: LogicalSize<u32> = res_hd.into();
        assert_eq!(logical_res, LogicalSize::new(1_280, 720));

        let logical_res: LogicalSize<u32> = res_qhd.into();
        assert_eq!(logical_res, LogicalSize::new(2_560, 1_440));

        let logical_res: LogicalSize<u32> = res_sd.into();
        assert_eq!(logical_res, LogicalSize::new(640, 480));

        assert_eq!(res_custom, LogicalSize::new(3_442, 2_349));
    }
}
