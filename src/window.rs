use crate::camera::Camera;
use winit::event::{DeviceEvent, ElementState, KeyboardInput, WindowEvent};
use winit::{event::Event, event_loop::ControlFlow, window::Window};

pub fn handle_inputs(
    control_flow: &mut ControlFlow,
    event: Event<()>,
    window: &Window,
    camera: &mut Camera,
    mouse_pressed: &mut bool,
) {
    *control_flow = ControlFlow::Poll;

    match event {
        Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            window_id,
        } if window_id == window.id() => *control_flow = ControlFlow::Exit,
        Event::WindowEvent {
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
        } => camera.process_keyboard(key, state),
        Event::WindowEvent {
            event: WindowEvent::MouseInput { state, .. },
            ..
        } => {
            *mouse_pressed = state == ElementState::Pressed;
            // println!("mouse")
        }
        Event::DeviceEvent { event, .. } => match event {
            // DeviceEvent::MouseWheel { delta } => match delta {
            //     winit::event::MouseScrollDelta::LineDelta(x, y) => {
            //         println!("mouse wheel Line Delta: ({},{})", x, y);
            //         let pixels_per_line = 120.0;
            //         let mut pos = window.outer_position().unwrap();
            //         pos.x -= (x * pixels_per_line) as i32;
            //         pos.y -= (y * pixels_per_line) as i32;
            //         window.set_outer_position(pos)
            //     }
            //     winit::event::MouseScrollDelta::PixelDelta(p) => {
            //         println!("mouse wheel Pixel Delta: ({},{})", p.x, p.y);
            //         let mut pos = window.outer_position().unwrap();
            //         pos.x -= p.x as i32;
            //         pos.y -= p.y as i32;
            //         window.set_outer_position(pos)
            //     }
            // },
            DeviceEvent::MouseWheel { delta, .. } => {
                camera.process_scroll(&delta);
            }
            DeviceEvent::MouseMotion { delta } => {
                if *mouse_pressed {
                    // println!("{:?}", delta);
                    camera.process_mouse(delta.0, delta.1);
                }
            }
            _ => {}
        },
        _ => (),
    }
}
