use cgmath::{InnerSpace, Matrix4, Point3, Rad, Vector3};
use std::{cmp::PartialEq, f32::consts::FRAC_PI_2};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseScrollDelta, VirtualKeyCode},
};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(PartialEq, Debug)]
pub struct Camera {
    pub pos: Point3<f32>,
    rotate_horizontal: f32,
    rotate_vertical: f32,
    yaw: Rad<f32>,
    pitch: Rad<f32>,
    sensitivity: f32,
    speed: f32,
    scroll: f32,
    amount_left: f32,
    amount_right: f32,
    amount_forward: f32,
    amount_backward: f32,
    amount_up: f32,
    amount_down: f32,
}

impl Camera {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Point3::new(x, y, z),
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            sensitivity: 0.5,
            speed: 0.5,
            scroll: 0.0,
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
        }
    }

    pub fn calc_matrix(&self, center: Point3<f32>) -> Matrix4<f32> {
        // Matrix4::look_at_rh(self.pos, center, Vector3::unit_z())
        Matrix4::look_to_rh(
            self.pos,
            Vector3::new(self.yaw.0.cos(), self.pitch.0.sin(), self.yaw.0.sin()).normalize(),
            Vector3::unit_y(),
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
                self.amount_forward = amount;
            }
            VirtualKeyCode::S | VirtualKeyCode::Down => {
                self.amount_backward = amount;
            }
            VirtualKeyCode::A | VirtualKeyCode::Left => {
                self.amount_left = amount;
            }
            VirtualKeyCode::D | VirtualKeyCode::Right => {
                self.amount_right = amount;
            }
            VirtualKeyCode::Space => {
                self.amount_up = amount;
            }
            VirtualKeyCode::LShift => {
                self.amount_down = amount;
            }
            _ => (),
        }
    }

    pub fn process_mouse(&mut self, mouse_dx: f64, mouse_dy: f64) {
        self.rotate_horizontal = mouse_dx as f32;
        self.rotate_vertical = mouse_dy as f32;
    }

    pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn rotate(&mut self, dt: f32) {
        // Movement
        let (yaw_sin, yaw_cos) = self.yaw.0.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.pos += forward * (self.amount_forward - self.amount_backward) * self.speed * dt;
        self.pos += right * (self.amount_right - self.amount_left) * self.speed * dt;

        // Zoom
        let (pitch_sin, pitch_cos) = self.pitch.0.sin_cos();
        let scrollward =
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        self.pos += scrollward * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        self.pos.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotation
        self.yaw += Rad(self.rotate_horizontal) * self.sensitivity * dt;
        self.pitch += Rad(-self.rotate_vertical) * self.sensitivity * dt;

        self.rotate_horizontal = 0.0;
        self.rotate_vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        if self.pitch < -Rad(SAFE_FRAC_PI_2) {
            self.pitch = -Rad(SAFE_FRAC_PI_2);
        } else if self.pitch > Rad(SAFE_FRAC_PI_2) {
            self.pitch = Rad(SAFE_FRAC_PI_2);
        }
    }
}

#[cfg(test)]
mod tests {
    use cgmath::Vector4;

    use super::*;

    #[test]
    fn test_camera_new() {
        let camera = Camera::new(0.91, 0.3, 0.7);

        assert_eq!(camera.pos, Point3::new(0.91, 0.3, 0.7));

        let expected_camera = Camera {
            pos: Point3::new(0.91, 0.3, 0.7),
            rotate_horizontal: 0.0,
            rotate_vertical: 0.0,
            yaw: Rad(0.0),
            pitch: Rad(0.0),
            sensitivity: 0.5,
            speed: 0.5,
            scroll: 0.0,
            amount_left: 0.0,
            amount_right: 0.0,
            amount_forward: 0.0,
            amount_backward: 0.0,
            amount_up: 0.0,
            amount_down: 0.0,
        };

        assert_eq!(expected_camera, camera);
    }

    #[test]
    fn test_camera_calc_matric() {
        let camera = Camera::new(5., 5., 5.);

        let matrix = camera.calc_matrix(Point3::new(0., 0., 0.));

        let expected_matrix = Matrix4 {
            x: Vector4::new(0.0, 0.0, -1.0, 0.0),
            y: Vector4::new(0.0, 1.0, -0.0, 0.0),
            z: Vector4::new(1.0, 0.0, -0.0, 0.0),
            w: Vector4::new(-5.0, -5.0, 5.0, 1.0),
        };

        assert_eq!(expected_matrix, matrix)
    }

    #[test]
    fn test_camera_process_keyboard() {
        let mut camera = Camera::new(0., 0., 0.);

        camera.process_keyboard(VirtualKeyCode::W, ElementState::Pressed);
        assert_eq!(camera.amount_forward, 1.);
        camera.process_keyboard(VirtualKeyCode::Up, ElementState::Pressed);
        assert_eq!(camera.amount_forward, 1.);
        camera.process_keyboard(VirtualKeyCode::W, ElementState::Released);
        assert_eq!(camera.amount_forward, 0.);
        camera.process_keyboard(VirtualKeyCode::Up, ElementState::Released);
        assert_eq!(camera.amount_forward, 0.);

        camera.process_keyboard(VirtualKeyCode::S, ElementState::Pressed);
        assert_eq!(camera.amount_backward, 1.);
        camera.process_keyboard(VirtualKeyCode::Down, ElementState::Pressed);
        assert_eq!(camera.amount_backward, 1.);
        camera.process_keyboard(VirtualKeyCode::S, ElementState::Released);
        assert_eq!(camera.amount_backward, 0.);
        camera.process_keyboard(VirtualKeyCode::Down, ElementState::Released);
        assert_eq!(camera.amount_backward, 0.);

        camera.process_keyboard(VirtualKeyCode::A, ElementState::Pressed);
        assert_eq!(camera.amount_left, 1.);
        camera.process_keyboard(VirtualKeyCode::Left, ElementState::Pressed);
        assert_eq!(camera.amount_left, 1.);
        camera.process_keyboard(VirtualKeyCode::A, ElementState::Released);
        assert_eq!(camera.amount_left, 0.);
        camera.process_keyboard(VirtualKeyCode::Left, ElementState::Released);
        assert_eq!(camera.amount_left, 0.);

        camera.process_keyboard(VirtualKeyCode::D, ElementState::Pressed);
        assert_eq!(camera.amount_right, 1.);
        camera.process_keyboard(VirtualKeyCode::Right, ElementState::Pressed);
        assert_eq!(camera.amount_right, 1.);
        camera.process_keyboard(VirtualKeyCode::D, ElementState::Released);
        assert_eq!(camera.amount_right, 0.);
        camera.process_keyboard(VirtualKeyCode::Right, ElementState::Released);
        assert_eq!(camera.amount_right, 0.);

        camera.process_keyboard(VirtualKeyCode::Space, ElementState::Pressed);
        assert_eq!(camera.amount_up, 1.);
        camera.process_keyboard(VirtualKeyCode::Space, ElementState::Released);
        assert_eq!(camera.amount_up, 0.);

        camera.process_keyboard(VirtualKeyCode::LShift, ElementState::Pressed);
        assert_eq!(camera.amount_down, 1.);
        camera.process_keyboard(VirtualKeyCode::LShift, ElementState::Released);
        assert_eq!(camera.amount_down, 0.);
    }

    #[test]
    fn test_camera_process_mouse() {
        let mut camera = Camera::new(0., 0., 0.);

        camera.process_mouse(-65., 32.);

        assert_eq!(camera.rotate_horizontal, -65.);
        assert_eq!(camera.rotate_vertical, 32.);
    }

    #[test]
    fn test_camera_process_scroll() {
        let mut camera = Camera::new(0., 0., 0.);

        camera.process_scroll(&MouseScrollDelta::LineDelta(1.2, 4.5));
        assert_eq!(camera.scroll, -4.5 * 100.);
        camera.process_scroll(&MouseScrollDelta::PixelDelta(PhysicalPosition {
            y: 30.,
            ..Default::default()
        }));
        assert_eq!(camera.scroll, -30.);
    }

    #[test]
    fn test_camera_rotate() {
        todo!();
    }
}