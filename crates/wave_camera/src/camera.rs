use nalgebra::{Matrix4, Perspective3, Point3, UnitQuaternion, Vector3};
use std::f32::consts::FRAC_PI_2;
use wave_space::space::Pos3;
use wave_window::window::Input;

#[rustfmt::skip]
pub const OPENGL_TO_WGPU_MATRIX: Matrix4<f32> = Matrix4::new(
    1.0, 0.0, 0.0, 0.0,
    0.0, 1.0, 0.0, 0.0,
    0.0, 0.0, 0.5, 0.5,
    0.0, 0.0, 0.0, 1.0,
);

const SAFE_FRAC_PI_2: f32 = FRAC_PI_2 - 0.0001;

#[derive(Debug)]
pub struct Camera {
    position: Pos3,
    rotation: UnitQuaternion<f32>,
    pub projection: Projection,

    amount_up: f32,
    amount_down: f32,

    speed: f32,
    sensitivity: f32,
}

impl Default for Camera {
    fn default() -> Self {
        let projection = Projection::new(1280, 720, 90., 0.1, 100.0);
        Self {
            position: Pos3::new(-2., 1., 0.),
            rotation: UnitQuaternion::default(),
            projection,

            amount_up: 0.0,
            amount_down: 0.0,

            speed: 4.0,
            sensitivity: 0.4,
        }
    }
}

impl Camera {
    pub fn matrix(&self) -> Matrix4<f32> {
        let (roll, pitch, yaw) = self.rotation.euler_angles();
        let (sin_pitch, cos_pitch) = pitch.sin_cos();
        let (sin_yaw, cos_yaw) = yaw.sin_cos();

        Matrix4::look_at_rh(
            &self.position.0.into(),
            &Point3::new(cos_pitch * cos_yaw, sin_pitch, cos_pitch * sin_yaw),
            &Vector3::y(),
        )
    }

    // pub fn process_scroll(&mut self, delta: &MouseScrollDelta) {
    //     self.scroll = -match delta {
    //         // I'm assuming a line is about 100 pixels
    //         MouseScrollDelta::LineDelta(_, scroll) => scroll * 100.0,
    //         MouseScrollDelta::PixelDelta(PhysicalPosition { y: scroll, .. }) => *scroll as f32,
    //     };
    // }

    pub fn update(&mut self, input: &mut Input, dt: f32) {
        let (roll, pitch, yaw) = self.rotation.euler_angles();

        // Move forward/backward and left/right
        let (yaw_sin, yaw_cos) = yaw.sin_cos();
        let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        self.position += forward * input.look.z() as f32 * self.speed * dt;
        self.position += right * input.look.x() as f32 * self.speed * dt;
        self.position.0.y += input.look.y() as f32 * self.speed * dt;

        // Move in/out (aka. "zoom")
        // Note: this isn't an actual zoom. The camera's position
        // changes when zooming. I've added this to make it easier
        // to get closer to an object you want to focus on.
        let (pitch_sin, pitch_cos) = pitch.sin_cos();
        let scrollward =
            Vector3::new(pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin).normalize();
        self.position +=
            scrollward * input.mouse_scroll as f32 * self.speed * self.sensitivity * dt;
        input.mouse_scroll = 0.0;

        // Move up/down. Since we don't use roll, we can just
        // modify the y coordinate directly.
        self.position.y += (self.amount_up - self.amount_down) * self.speed * dt;

        // Rotate
        let rot_speed = self.sensitivity * dt;
        let mut rot =
            UnitQuaternion::from_euler_angles(0., 0., input.mouse_delta.0 as f32 * rot_speed)
                * self.rotation;
        rot = UnitQuaternion::from_euler_angles(-input.mouse_delta.1 as f32 * rot_speed, 0., 0.)
            * rot;
        self.rotation = self.rotation.slerp(&rot, 0.2);

        // If process_mouse isn't called every frame, these values
        // will not get set to zero, and the camera will rotate
        // when moving in a non-cardinal direction.
        input.mouse_delta.0 = 0.0;
        input.mouse_delta.1 = 0.0;

        // Keep the camera's angle from going too high/low.
        // if camera.pitch < -Rad(SAFE_FRAC_PI_2) {
        //     camera.pitch = -Rad(SAFE_FRAC_PI_2);
        // } else if camera.pitch > Rad(SAFE_FRAC_PI_2) {
        //     camera.pitch = Rad(SAFE_FRAC_PI_2);
        // }
    }
}

#[derive(Debug)]
pub struct Projection {
    aspect: f32,
    fovy: f32, // rad
    znear: f32,
    zfar: f32,
}

impl Projection {
    pub fn new(width: u32, height: u32, fovy: f32, znear: f32, zfar: f32) -> Self {
        Self {
            aspect: width as f32 / height as f32,
            fovy,
            znear,
            zfar,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.aspect = width as f32 / height as f32;
    }

    pub fn calc_matrix(&self) -> Matrix4<f32> {
        // OPENGL_TO_WGPU_MATRIX
        // *
        *Perspective3::new(self.aspect, self.fovy, self.znear, self.zfar).as_matrix()
    }
}

//     pub fn update(&mut self, input: &mut Input, dt: f32) {
//         let (_, pitch, yaw) = self.rotation.euler_angles();
//
//         let look = input.look;
//
//         // Movement
//         let speed = self.speed.0 * dt;
//         // let (yaw_sin, yaw_cos) = yaw.sin_cos();
//         // let speed = self.speed.0 * dt;
//         // let forward = vector![yaw_cos, 0., yaw_sin].normalize() * look.z() as f32 * speed;
//         // let right = vector![-yaw_sin, 0., yaw_cos].normalize() * look.x() as f32 * speed;
//         // self.pos.0 += forward + right;
//         let (yaw_sin, yaw_cos) = yaw.sin_cos();
//         let forward = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
//         let right = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
//         self.pos += forward * look.z() as f32 * speed;
//         self.pos += right * look.x() as f32 * speed;
//         self.pos.0.y += look.y() as f32 * speed;
//
//         // Zoom
//         let (pitch_sin, pitch_cos) = pitch.sin_cos();
//         let scroll_dir = vector![pitch_cos * yaw_cos, pitch_sin, pitch_cos * yaw_sin];
//         self.pos.0 += scroll_dir * input.mouse_scroll as f32 * self.sensitivity.0 * speed;
//
//         // Rotation
//         let rot_speed = self.sensitivity.0 * dt;
//         let mut rot =
//             UnitQuaternion::from_euler_angles(0., 0., input.mouse_delta.0 as f32 * rot_speed)
//                 * self.rotation;
//         rot = UnitQuaternion::from_euler_angles(0., -input.mouse_delta.1 as f32 * rot_speed, 0.)
//             * rot;
//
//         self.rotation = self.rotation.slerp(&rot, 0.2);
//
//         input.mouse_delta = Default::default();
//     }
