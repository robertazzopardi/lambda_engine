use ash::vk::Extent2D;
use cgmath::{Deg, Matrix4, Point3, SquareMatrix, Vector3};

use crate::camera::Camera;

#[derive(Clone, Copy, Debug)]
pub struct UniformBufferObject {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

impl UniformBufferObject {
    pub fn update(&mut self, extent: Extent2D, camera: &mut Camera) {
        let aspect = extent.width as f32 / extent.height as f32;

        // self.model = Matrix4::from_angle_y(Deg(0.));

        self.view = camera.calc_matrix(Point3::new(0., 0., 0.));
        self.proj = {
            let mut p = cgmath::perspective(Deg(45.), aspect, 0.1, 100.);
            p[1][1] *= -1.;
            p
        };
    }

    pub fn new(camera: &mut Camera) -> Self {
        Self {
            model: Matrix4::identity(),
            // view: Matrix4::identity(),
            view: Matrix4::look_at_rh(camera.pos, Point3::new(0., 0., 0.), Vector3::unit_z()),
            proj: Matrix4::identity(),
        }
    }
}
