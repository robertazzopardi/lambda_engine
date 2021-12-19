use cgmath::{Matrix4, Point3, Transform, Vector3};
use winit::{event::VirtualKeyCode, event_loop::ControlFlow};

pub struct Camera {
    pub pos: Point3<f32>,
}

impl Camera {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self {
            pos: Point3::new(x, y, z),
        }
    }

    pub fn update_camera(&mut self, control_flow: &mut ControlFlow, virtual_code: VirtualKeyCode) {
        match virtual_code {
            VirtualKeyCode::Up => {
                let v = Vector3::new(-0.01 * self.pos.x, -0.01 * self.pos.y, -0.01 * self.pos.z);

                let m = Matrix4::<f32>::from_translation(v);
                let z = m.transform_point(self.pos);
                self.pos = z;
            }
            VirtualKeyCode::Down => {
                let v = Vector3::new(0.01 * self.pos.x, 0.01 * self.pos.y, 0.01 * self.pos.z);

                let m = Matrix4::<f32>::from_translation(v);
                let z = m.transform_point(self.pos);
                self.pos = z;
            }
            VirtualKeyCode::Left => {
                println!("here")
            }
            VirtualKeyCode::Right => {
                println!("here")
            }
            VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
            _ => (),
        }
    }

    pub fn rotate(&mut self, pitch: f32, roll: f32, yaw: f32) {
        let cosa = yaw.cos();
        let sina = yaw.sin();

        let cosb = pitch.cos();
        let sinb = pitch.sin();

        let cosc = roll.cos();
        let sinc = roll.sin();

        let axx = cosa * cosb;
        let axy = cosa * sinb * sinc - sina * cosc;
        let axz = cosa * sinb * cosc + sina * sinc;

        let ayx = sina * cosb;
        let ayy = sina * sinb * sinc + cosa * cosc;
        let ayz = sina * sinb * cosc - cosa * sinc;

        let azx = -sinb;
        let azy = cosb * sinc;
        let azz = cosb * cosc;

        let px = self.pos.x;
        let py = self.pos.y;
        let pz = self.pos.z;

        self.pos.x = axx * px + axy * py + axz * pz;
        self.pos.y = ayx * px + ayy * py + ayz * pz;
        self.pos.z = azx * px + azy * py + azz * pz;
    }
}
