use crate::Camera;
use ash::vk;
use nalgebra::{Matrix4, Perspective3, Point3, Vector3};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UniformBufferObject {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

impl UniformBufferObject {
    pub fn new(extent: &vk::Extent2D, camera: &Camera) -> Self {
        let mut mvp = Self {
            model: Matrix4::identity(),
            view: Matrix4::identity(),
            proj: Matrix4::identity(),
        };
        mvp.update(extent, camera);
        mvp
    }

    pub fn update(&mut self, extent: &vk::Extent2D, camera: &Camera) {
        self.model = Matrix4::from_axis_angle(&Vector3::x_axis(), 90.0f32.to_radians());

        self.view = camera.calc_matrix(Point3::new(0., 0., 0.));

        let aspect = extent.width as f32 / extent.height as f32;

        self.proj = *Perspective3::new(aspect, 45.0_f32.to_radians(), 0.00001, 100.).as_matrix();
        self.proj[(1, 1)] *= -1.;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_uniform_buffer_object() {
        let extent = vk::Extent2D::builder().height(1920).width(1080);
        let camera = Camera::new(5., 6., 7.);
        let ubo = UniformBufferObject::new(&extent, &camera);

        let expected_ubo = UniformBufferObject {
            model: Matrix4::new(
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                -4.371139e-8,
                -1.0,
                0.0,
                0.0,
                1.0,
                -4.371139e-8,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ),
            view: Matrix4::new(
                0.0, 0.0, 1.0, -7.0, 0.0, 1.0, 0.0, -6.0, -1.0, -0.0, -0.0, 5.0, 0.0, 0.0, 0.0, 1.0,
            ),
            proj: Matrix4::new(
                4.291935, 0.0, 0.0, 0.0, 0.0, -2.4142134, 0.0, 0.0, 0.0, 0.0, -1.0000001, -2e-5,
                0.0, 0.0, -1.0, 0.0,
            ),
        };

        assert_eq!(ubo, expected_ubo);
    }

    #[test]
    fn test_ubo_update() {
        let extent = vk::Extent2D::builder().height(1920).width(1080);
        let mut camera = Camera::new(5., 6., 7.);
        let mut ubo = UniformBufferObject::new(&extent, &camera);

        camera.pos = Vector3::new(-5., -1., 3.);
        ubo.update(&extent, &camera);

        let expected_ubo = UniformBufferObject {
            model: Matrix4::new(
                1.0,
                0.0,
                0.0,
                0.0,
                0.0,
                -4.371139e-8,
                -1.0,
                0.0,
                0.0,
                1.0,
                -4.371139e-8,
                0.0,
                0.0,
                0.0,
                0.0,
                1.0,
            ),
            view: Matrix4::new(
                0.0, 0.0, 1.0, -3.0, 0.0, 1.0, 0.0, 1.0, -1.0, -0.0, -0.0, -5.0, 0.0, 0.0, 0.0, 1.0,
            ),
            proj: Matrix4::new(
                4.291935, 0.0, 0.0, 0.0, 0.0, -2.4142134, 0.0, 0.0, 0.0, 0.0, -1.0000001, -2e-5,
                0.0, 0.0, -1.0, 0.0,
            ),
        };

        assert_eq!(ubo, expected_ubo);
    }
}
