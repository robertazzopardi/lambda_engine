use derive_builder::Builder;
use wave_space::space::{self, Pos3};
use wave_window::window::Input;
use nalgebra::{matrix, point, vector, Matrix4, Point3, UnitQuaternion, Vector3};
use std::{cmp::PartialEq, f32::consts::FRAC_PI_2};

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

pub fn look_to_rh(eye: Vector3<f32>, dir: Vector3<f32>, up: Vector3<f32>) -> Matrix4<f32> {
    let f = dir.normalize();
    let s = f.cross(&up).normalize();
    let u = s.cross(&f).normalize();

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

#[derive(PartialEq, Debug, Clone, Copy, Builder)]
#[builder(build_fn(skip))]
#[builder(name = "Camera")]
pub struct CameraInternal {
    pub pos: Pos3,
    rotation: UnitQuaternion<f32>,
    sensitivity: Sensitivity,
    speed: CameraSpeed,
}

impl Camera {
    pub fn build(&mut self) -> CameraInternal {
        CameraInternal {
            pos: self.pos.unwrap_or_else(|| Pos3::new(-2., 1., 0.)),
            rotation: UnitQuaternion::default(),
            sensitivity: self.sensitivity.unwrap_or_default(),
            speed: self.speed.unwrap_or_default(),
        }
    }
}

impl CameraInternal {
    pub fn matrix(&self) -> Matrix4<f32> {
        let (_, pitch, yaw) = self.rotation.euler_angles();
        let (yaw_sin, yaw_cos) = yaw.sin_cos();
        look_to_rh(
            self.pos.0,
            vector![yaw_cos, pitch.sin(), yaw_sin],
            Vector3::y(),
        )
    }

    pub fn update(&mut self, input: &mut Input, dt: f32) {
        let (_, pitch, yaw) = self.rotation.euler_angles();

        let look = input.look;

        // Movement
        let speed = self.speed.0 * dt;
        // let (yaw_sin, yaw_cos) = yaw.sin_cos();
        // let speed = self.speed.0 * dt;
        // let forward = vector![yaw_cos, 0., yaw_sin].normalize() * look.z() as f32 * speed;
        // let right = vector![-yaw_sin, 0., yaw_cos].normalize() * look.x() as f32 * speed;
        // self.pos.0 += forward + right;
        let (yaw_sin, yaw_cos) = yaw.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.pos += forward * look.z() as f32 * speed;
        self.pos += right * look.x() as f32 * speed;
        self.pos.0.y += look.y() as f32 * speed;

        // Zoom
        // let (pitch_sin, pitch_cos) = pitch.sin_cos();
        // let scroll_dir = vector![pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin];
        // self.pos.0 += scroll_dir * input.mouse_scroll * self.sensitivity.0 * speed;

        // Rotation
        let rot_speed = self.sensitivity.0 * dt;
        let mut rot =
            UnitQuaternion::from_euler_angles(0., 0., input.mouse_delta.0 as f32 * rot_speed)
                * self.rotation;
        rot = UnitQuaternion::from_euler_angles(0., -input.mouse_delta.1 as f32 * rot_speed, 0.)
            * rot;

        self.rotation = self.rotation.slerp(&rot, 0.2);

        input.mouse_delta = Default::default();
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
            rotation: UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.1),
            sensitivity: Sensitivity(0.9),
            speed: CameraSpeed(0.5),
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

    // #[test]
    // fn test_camera_process_keyboard() {
    //     let mut camera = Camera::default().pos(Pos3::new(0., 0., 0.)).build();

    //     camera.process_keyboard(VirtualKeyCode::W, 1.);
    //     assert_eq!(camera.direction.forward, 1.);
    //     camera.process_keyboard(VirtualKeyCode::Up, 1.);
    //     assert_eq!(camera.direction.forward, 1.);
    //     camera.process_keyboard(VirtualKeyCode::W, 0.);
    //     assert_eq!(camera.direction.forward, 0.);
    //     camera.process_keyboard(VirtualKeyCode::Up, 0.);
    //     assert_eq!(camera.direction.forward, 0.);

    //     camera.process_keyboard(VirtualKeyCode::S, 1.);
    //     assert_eq!(camera.direction.backward, 1.);
    //     camera.process_keyboard(VirtualKeyCode::Down, 1.);
    //     assert_eq!(camera.direction.backward, 1.);
    //     camera.process_keyboard(VirtualKeyCode::S, 0.);
    //     assert_eq!(camera.direction.backward, 0.);
    //     camera.process_keyboard(VirtualKeyCode::Down, 0.);
    //     assert_eq!(camera.direction.backward, 0.);

    //     camera.process_keyboard(VirtualKeyCode::A, 1.);
    //     assert_eq!(camera.direction.left, 1.);
    //     camera.process_keyboard(VirtualKeyCode::Left, 1.);
    //     assert_eq!(camera.direction.left, 1.);
    //     camera.process_keyboard(VirtualKeyCode::A, 0.);
    //     assert_eq!(camera.direction.left, 0.);
    //     camera.process_keyboard(VirtualKeyCode::Left, 0.);
    //     assert_eq!(camera.direction.left, 0.);

    //     camera.process_keyboard(VirtualKeyCode::D, 1.);
    //     assert_eq!(camera.direction.right, 1.);
    //     camera.process_keyboard(VirtualKeyCode::Right, 1.);
    //     assert_eq!(camera.direction.right, 1.);
    //     camera.process_keyboard(VirtualKeyCode::D, 0.);
    //     assert_eq!(camera.direction.right, 0.);
    //     camera.process_keyboard(VirtualKeyCode::Right, 0.);
    //     assert_eq!(camera.direction.right, 0.);

    //     camera.process_keyboard(VirtualKeyCode::Space, 1.);
    //     assert_eq!(camera.direction.up, 1.);
    //     camera.process_keyboard(VirtualKeyCode::Space, 0.);
    //     assert_eq!(camera.direction.up, 0.);

    //     camera.process_keyboard(VirtualKeyCode::LShift, 1.);
    //     assert_eq!(camera.direction.down, 1.);
    //     camera.process_keyboard(VirtualKeyCode::LShift, 0.);
    //     assert_eq!(camera.direction.down, 0.);
    // }

    // #[test]
    // fn test_camera_process_mouse() {
    //     let mut camera = Camera::default().pos(Pos3::new(0., 0., 0.)).build();

    //     camera.process_mouse(-65., 32.);

    //     assert_eq!(camera.rotation.y, -65.);
    //     assert_eq!(camera.rotation.x, 32.);
    // }

    // #[test]
    // fn test_camera_process_scroll() {
    //     let mut camera = Camera::default().pos(Pos3::new(0., 0., 0.)).build();

    //     camera.process_scroll(&MouseScrollDelta::LineDelta(1.2, 4.5));
    //     assert_eq!(camera.scroll.0, -4.5 * 100.);
    //     camera.process_scroll(&MouseScrollDelta::PixelDelta(PhysicalPosition {
    //         y: 30.,
    //         ..Default::default()
    //     }));
    //     assert_eq!(camera.scroll.0, -30.);
    // }

    // #[test]
    // fn test_camera_rotate() {
    //     todo!();
    // }
}
