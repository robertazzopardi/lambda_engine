use crate::{memory, Vulkan};
use ash::vk;
use lambda_camera::camera::Camera;
use nalgebra::{Matrix4, Perspective3, Vector3};

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct UniformBufferObject {
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, new)]
pub struct UniformBuffer {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

impl UniformBufferObject {
    pub fn new(extent: &vk::Extent2D, camera: &Camera) -> Self {
        let mut mvp = Self {
            view: Matrix4::identity(),
            proj: Matrix4::identity(),
        };
        mvp.update(extent, camera);
        mvp
    }

    pub fn update(&mut self, extent: &vk::Extent2D, camera: &Camera) {
        self.view = camera.calc_matrix();

        let aspect = extent.width as f32 / extent.height as f32;

        self.proj = *Perspective3::new(aspect, 45.0_f32.to_radians(), 0.00001, 100.).as_matrix();
        self.proj[(1, 1)] *= -1.;
    }
}

pub fn update_uniform_buffers(
    vulkan: &mut Vulkan,
    _camera: &mut Camera,
    current_image: usize,
    _dt: f32,
) {
    // let axis_angle = nalgebra::Vector3::y() * 0.05;
    // let rot = nalgebra::Rotation3::new(axis_angle);
    // *camera.pos = rot * *camera.pos;

    let buffer_size = std::mem::size_of::<UniformBufferObject>() as u64;

    vulkan.objects.0.iter_mut().for_each(|object| {
        // let rot = Matrix4::from_scaled_axis(&Vector3::y() * 0.01);
        // *object.model *= rot;

        let uniform_buffer = UniformBuffer::new(*object.model, vulkan.ubo.view, vulkan.ubo.proj);

        memory::map_memory(
            &vulkan.instance_devices.devices.logical.device,
            object.graphics_pipeline.descriptors.uniform_buffers[current_image].memory,
            buffer_size,
            &[uniform_buffer],
        );
    });
}

#[cfg(test)]
mod tests {
    use lambda_space::space::Coordinate3;

    use super::*;

    #[test]
    fn test_new_uniform_buffer_object() {
        let extent = vk::Extent2D::builder().height(1920).width(1080);
        let camera = Camera::new(5., 6., 7.);
        let ubo = UniformBufferObject::new(&extent, &camera);

        let expected_ubo = UniformBufferObject {
            view: Matrix4::new(
                0., 0., 1., -7., 0., 1., 0., -6., -1., -0., -0., 5., 0., 0., 0., 1.,
            ),
            proj: Matrix4::new(
                4.291935, 0., 0., 0., 0., -2.4142134, 0., 0., 0., 0., -1.0000001, -2e-5, 0., 0.,
                -1., 0.,
            ),
        };

        assert_eq!(ubo, expected_ubo);
    }

    #[test]
    fn test_ubo_update() {
        let extent = vk::Extent2D::builder().height(1920).width(1080);
        let mut camera = Camera::new(5., 6., 7.);
        let mut ubo = UniformBufferObject::new(&extent, &camera);

        camera.pos = Coordinate3::new(-5., -1., 3.);
        ubo.update(&extent, &camera);

        let expected_ubo = UniformBufferObject {
            view: Matrix4::new(
                0., 0., 1., -3., 0., 1., 0., 1., -1., -0., -0., -5., 0., 0., 0., 1.,
            ),
            proj: Matrix4::new(
                4.291935, 0., 0., 0., 0., -2.4142134, 0., 0., 0., 0., -1.0000001, -2e-5, 0., 0.,
                -1., 0.,
            ),
        };

        assert_eq!(ubo, expected_ubo);
    }
}
