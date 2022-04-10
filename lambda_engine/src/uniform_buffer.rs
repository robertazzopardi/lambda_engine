use crate::{camera, Camera};
use ash::vk;
use cgmath::{Deg, Matrix4, Point3, SquareMatrix};

#[derive(Clone, Copy, Debug)]
pub struct UniformBufferObject {
    model: nalgebra::Matrix4<f32>,
    view: Matrix4<f32>,
    proj: nalgebra::Matrix4<f32>,
}

impl UniformBufferObject {
    pub fn new(camera: &mut Camera) -> Self {
        Self {
            model: nalgebra::Matrix4::identity(),
            view: Matrix4::identity(),
            proj: nalgebra::Matrix4::identity(),
        }
    }

    pub fn update(&mut self, extent: vk::Extent2D, camera: &mut Camera) {
        self.model =
            nalgebra::Matrix4::from_axis_angle(&nalgebra::Vector3::x_axis(), camera::to_rad(90.));

        self.view = camera.calc_matrix(Point3::new(0., 0., 0.));

        let aspect = extent.width as f32 / extent.height as f32;

        self.proj =
            *nalgebra::Perspective3::new(aspect, camera::to_rad(45.), 0.00001, 100.).as_matrix();
        self.proj[(1, 1)] *= -1.;
    }
}
