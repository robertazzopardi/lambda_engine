use crate::space::{self, Coordinate3d};
use cgmath::{Angle, InnerSpace, Matrix4, Point3, Rad, Vector3};
use std::{cmp::PartialEq, f32::consts::FRAC_PI_2};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, MouseScrollDelta, VirtualKeyCode},
};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(PartialEq, Debug, Clone, Copy)]
pub struct Camera {
    pub pos: Coordinate3d,
    rotation: space::Rotation,
    orientation: space::Orientation,
    sensitivity: f32,
    speed: f32,
    scroll: f32,
    direction: space::LookDirection,
}

impl Camera {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Point3::new(x, y, z).into(),
            rotation: space::Rotation::default(),
            orientation: space::Orientation::default(),
            sensitivity: 0.9,
            speed: 0.5,
            scroll: 0.0,
            direction: space::LookDirection::default(),
        }
    }

    pub fn calc_matrix(&self, _center: Point3<f32>) -> Matrix4<f32> {
        let space::Orientation { yaw, pitch, _roll } = self.orientation;
        // Matrix4::look_at_rh(self.pos.0, center, Vector3::unit_z())
        Matrix4::look_to_rh(
            *self.pos,
            Vector3::new(yaw.cos(), pitch.sin(), yaw.sin()).normalize(),
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
        self.scroll = -match delta {
            // I'm assuming a line is about 100 pixels
            MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
            MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
        };
    }

    pub fn rotate(&mut self, dt: f32) {
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
        let forward_dir = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right_dir = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.pos += forward_dir * (forward - backward) * self.speed * dt;
        self.pos += right_dir * (right - left) * self.speed * dt;

        // Zoom
        let (pitch_sin, pitch_cos) = self.orientation.pitch.sin_cos();
        let scroll_dir =
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        self.pos += scroll_dir * self.scroll * self.speed * self.sensitivity * dt;
        self.scroll = 0.0;

        self.pos.y += (up - down) * self.speed * dt;

        // Rotation
        self.orientation.yaw += Rad(self.rotation.horizontal) * self.sensitivity * dt;
        self.orientation.pitch += Rad(-self.rotation.vertical) * self.sensitivity * dt;

        self.rotation.horizontal = 0.0;
        self.rotation.vertical = 0.0;

        // Keep the camera's angle from going too high/low.
        let angle = space::Angle(Rad(SAFE_FRAC_PI_2));
        if self.orientation.pitch < -angle {
            self.orientation.pitch.0 = -angle.0;
        } else if self.orientation.pitch > angle {
            self.orientation.pitch.0 = angle.0;
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

        assert_eq!(camera.pos, Point3::new(0.91, 0.3, 0.7).into());

        let expected_camera = Camera {
            pos: Point3::new(0.91, 0.3, 0.7).into(),
            rotation: space::Rotation::default(),
            orientation: space::Orientation::default(),
            sensitivity: 0.5,
            speed: 0.5,
            scroll: 0.0,
            direction: space::LookDirection::default(),
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
        let mut camera = Camera::new(0., 0., 0.);

        camera.process_mouse(-65., 32.);

        assert_eq!(camera.rotation.horizontal, -65.);
        assert_eq!(camera.rotation.vertical, 32.);
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
