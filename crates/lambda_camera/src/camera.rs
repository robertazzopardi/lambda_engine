use derive_builder::Builder;
use lambda_space::space::{self, Pos3};
use nalgebra::{matrix, vector, Matrix4, Rotation, Rotation3, Vector3};
use std::{cmp::PartialEq, f32::consts::FRAC_PI_2};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseScrollDelta, VirtualKeyCode},
};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub fn look_to_rh(eye: Vector3<f32>, dir: Vector3<f32>, up: Vector3<f32>) -> Matrix4<f32> {
    let f = dir.normalize();
    let s = f.cross(&up).normalize();
    let u = s.cross(&f);

    matrix![
         s.x,  s.y,  s.z, -eye.dot(&s);
         u.x,  u.y,  u.z, -eye.dot(&u);
        -f.x, -f.y, -f.z,  eye.dot(&f);
          0.,   0.,   0.,            1.
    ]
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Sensitivity(f32);

impl Default for Sensitivity {
    fn default() -> Self {
        Self(1.)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CameraSpeed(f32);

impl Default for CameraSpeed {
    fn default() -> Self {
        Self(0.5)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct MouseScroll(f32);

#[derive(PartialEq, Debug, Clone, Copy, Builder)]
#[builder(build_fn(skip))]
#[builder(name = "Camera")]
pub struct CameraInternal {
    pub pos: Pos3,
    rot: Rotation3<f32>,
    rotation: space::Rotation,
    orientation: space::Orientation,
    sensitivity: Sensitivity,
    speed: CameraSpeed,
    scroll: MouseScroll,
    direction: space::LookDirection,
}

impl Camera {
    pub fn build(&mut self) -> CameraInternal {
        let space::Orientation { yaw, pitch, .. } = self.orientation.unwrap_or_default();
        let pos = self.pos.unwrap_or_else(|| Pos3::new(-2., 1., 0.));

        CameraInternal {
            pos,
            rot: Rotation3::default(),
            rotation: self.rotation.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            sensitivity: self.sensitivity.unwrap_or_default(),
            speed: self.speed.unwrap_or_default(),
            scroll: self.scroll.unwrap_or_default(),
            direction: self.direction.unwrap_or_default(),
        }
    }
}

impl CameraInternal {
    pub fn matrix(&self) -> Matrix4<f32> {
        let space::Orientation { yaw, pitch, .. } = self.orientation;

        look_to_rh(
            self.pos.0,
            vector![yaw.cos(), pitch.sin(), yaw.sin()],
            Vector3::y(),
        )
    }

    pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) {
        let amount = if state == ElementState::Pressed {
            1.0
        } else {
            0.0
        };

        match key {
            VirtualKeyCode::W | VirtualKeyCode::Up => {
                self.direction.forward = amount;
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.direction.backward = amount;
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.direction.left = amount;
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.direction.right = amount;
            }
            VirtualKeyCode::Space => {
                self.direction.up = amount;
            }
            VirtualKeyCode::LShift => {
                self.direction.down = amount;
            }
            _ => (),
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotation.horizontal = mouse_dx as f32;
        self.rotation.vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        // TODO improve
        self.scroll = MouseScroll(
            -match delta {
                // I'm assuming a line is about 100 pixels
                MouseScrollDelta::LineDelta(_, scroll) => MouseScroll(scroll * 100.),
                MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => {
                    MouseScroll(*scroll as f32)
                }
            }
            .0,
        );
    }

    pub fn update(&mut self, dt: f32) {
        let space::LookDirection {
            left,
            right,
            up,
            down,
            forward,
            backward,
        } = self.direction;

        // Movement
        let (yaw_sin, yaw_cos) = self.orientation.yaw.sin_cos();
        let forward_dir = vector![yaw_cos, 0.0, yaw_sin];
        let right_dir = vector![-yaw_sin, 0.0, yaw_cos];
        self.pos.0 += forward_dir * (forward - backward) * self.speed.0 * dt;
        self.pos.0 += right_dir * (right - left) * self.speed.0 * dt;

        // Zoom
        let (pitch_sin, pitch_cos) = self.orientation.pitch.sin_cos();
        let scroll_dir = vector![pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin];
        self.pos.0 += scroll_dir * self.scroll.0 * self.speed.0 * self.sensitivity.0 * dt;
        self.scroll = MouseScroll::default();
        self.pos.0.y += (up - down) * self.speed.0 * dt;

        // Rotation
        self.orientation.yaw += space::Angle(self.rotation.horizontal * self.sensitivity.0 * dt);
        self.orientation.pitch += space::Angle(-self.rotation.vertical * self.sensitivity.0 * dt);

        self.rotation.horizontal = 0.0;
        self.rotation.vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        let angle = space::Angle(SAFE_FRAC_PI_2);
        if self.orientation.pitch < -angle {
            self.orientation.pitch.0 = -angle.0;
        } else if self.orientation.pitch > angle {
            self.orientation.pitch.0 = angle.0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_camera_new() {
        let camera = Camera::default().pos(Pos3::new(0.91, 0.3, 0.7)).build();

        assert_eq!(*camera.pos, vector![0.91, 0.3, 0.7]);

        let expected_camera = CameraInternal {
            pos: Pos3::new(0.91, 0.3, 0.7),
            rot: Rotation3::default(),
            rotation: space::Rotation::default(),
            orientation: space::Orientation::default(),
            sensitivity: Sensitivity(0.9),
            speed: CameraSpeed(0.5),
            scroll: MouseScroll(0.),
            direction: space::LookDirection::default(),
        };

        assert_eq!(expected_camera, camera);
    }

    #[test]
    fn test_camera_calc_matrix() {
        let camera = Camera::default().pos(Pos3::new(5., 5., 5.)).build();

        let matrix = camera.matrix();

        let expected_matrix =
            matrix![0., 0., -1., 0., 0., 1., -0., 0., 1., 0., -0., 0., -5., -5., 5., 1.]
                .transpose();

        assert_eq!(expected_matrix, matrix)
    }

    #[test]
    fn test_camera_process_keyboard() {
        let mut camera = Camera::default().pos(Pos3::new(0., 0., 0.)).build();

        camera.process_keyboard(VirtualKeyCode::W, ElementState::Pressed);
        assert_eq!(camera.direction.forward, 1.);
        camera.process_keyboard(VirtualKeyCode::Up, ElementState::Pressed);
        assert_eq!(camera.direction.forward, 1.);
        camera.process_keyboard(VirtualKeyCode::W, ElementState::Released);
        assert_eq!(camera.direction.forward, 0.);
        camera.process_keyboard(VirtualKeyCode::Up, ElementState::Released);
        assert_eq!(camera.direction.forward, 0.);

        camera.process_keyboard(VirtualKeyCode::S, ElementState::Pressed);
        assert_eq!(camera.direction.backward, 1.);
        camera.process_keyboard(VirtualKeyCode::Down, ElementState::Pressed);
        assert_eq!(camera.direction.backward, 1.);
        camera.process_keyboard(VirtualKeyCode::S, ElementState::Released);
        assert_eq!(camera.direction.backward, 0.);
        camera.process_keyboard(VirtualKeyCode::Down, ElementState::Released);
        assert_eq!(camera.direction.backward, 0.);

        camera.process_keyboard(VirtualKeyCode::A, ElementState::Pressed);
        assert_eq!(camera.direction.left, 1.);
        camera.process_keyboard(VirtualKeyCode::Left, ElementState::Pressed);
        assert_eq!(camera.direction.left, 1.);
        camera.process_keyboard(VirtualKeyCode::A, ElementState::Released);
        assert_eq!(camera.direction.left, 0.);
        camera.process_keyboard(VirtualKeyCode::Left, ElementState::Released);
        assert_eq!(camera.direction.left, 0.);

        camera.process_keyboard(VirtualKeyCode::D, ElementState::Pressed);
        assert_eq!(camera.direction.right, 1.);
        camera.process_keyboard(VirtualKeyCode::Right, ElementState::Pressed);
        assert_eq!(camera.direction.right, 1.);
        camera.process_keyboard(VirtualKeyCode::D, ElementState::Released);
        assert_eq!(camera.direction.right, 0.);
        camera.process_keyboard(VirtualKeyCode::Right, ElementState::Released);
        assert_eq!(camera.direction.right, 0.);

        camera.process_keyboard(VirtualKeyCode::Space, ElementState::Pressed);
        assert_eq!(camera.direction.up, 1.);
        camera.process_keyboard(VirtualKeyCode::Space, ElementState::Released);
        assert_eq!(camera.direction.up, 0.);

        camera.process_keyboard(VirtualKeyCode::LShift, ElementState::Pressed);
        assert_eq!(camera.direction.down, 1.);
        camera.process_keyboard(VirtualKeyCode::LShift, ElementState::Released);
        assert_eq!(camera.direction.down, 0.);
    }

    #[test]
    fn test_camera_process_mouse() {
        let mut camera = Camera::default().pos(Pos3::new(0., 0., 0.)).build();

        camera.process_mouse(-65., 32.);

        assert_eq!(camera.rotation.horizontal, -65.);
        assert_eq!(camera.rotation.vertical, 32.);
    }

    #[test]
    fn test_camera_process_scroll() {
        let mut camera = Camera::default().pos(Pos3::new(0., 0., 0.)).build();

        camera.process_scroll(&MouseScrollDelta::LineDelta(1.2, 4.5));
        assert_eq!(camera.scroll.0, -4.5 * 100.);
        camera.process_scroll(&MouseScrollDelta::PixelDelta(PhysicalPosition {
            y: 30.,
            ..Default::default()
        }));
        assert_eq!(camera.scroll.0, -30.);
    }

    // #[test]
    // fn test_camera_rotate() {
    //     todo!();
    // }
}
