use crate::{memory, VulkanObject};
use ash::{vk, Device};
use gpu_allocator::vulkan::Allocator;
use nalgebra::{Matrix, Matrix4, Perspective3};

#[derive(Debug, PartialEq, Default)]
pub struct UniformBufferObject {
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct UniformBuffer {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

impl UniformBuffer {
    pub fn new(model: Matrix4<f32>, view: Matrix4<f32>, proj: Matrix4<f32>) -> Self {
        Self { model, view, proj }
    }
}

impl UniformBufferObject {
    // pub fn new(extent: &vk::Extent2D, camera: &CameraInternal) -> Self {
    //     let mut mvp = Self {
    //         view: Matrix4::identity(),
    //         proj: Matrix4::identity(),
    //     };
    //     mvp.update(extent, camera);
    //     mvp
    // }

    pub fn update(&mut self, extent: &vk::Extent2D, view: Matrix4<f32>) {
        self.view = view;

        let aspect = extent.width as f32 / extent.height as f32;

        self.proj = *Perspective3::new(aspect, 45.0_f32.to_radians(), 0.00001, 100.).as_matrix();
        self.proj[(1, 1)] *= -1.;
    }

    pub const fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

pub(crate) fn update_uniform_buffers(
    allocator: &mut Allocator,
    device: &Device,
    objects: &mut Vec<VulkanObject>,
    ubo: &UniformBufferObject,
    current_image: usize,
    _dt: f32,
) {
    // let axis_angle = nalgebra::Vector3::y() * 0.05;
    // let rot = nalgebra::Rotation3::new(axis_angle);
    // *camera.pos = rot * *camera.pos;

    let buffer_size = UniformBufferObject::size().try_into().unwrap();

    let mut uniform_buffer = UniformBuffer::new(Matrix::default(), ubo.view, ubo.proj);

    objects.iter_mut().for_each(|object| {
        uniform_buffer.model = object.model;

        // memory::map_memory(
        //     device,
        //     unsafe {
        //         object.graphics_pipeline.uniform_buffers[current_image]
        //             .allocation
        //             .memory()
        //     },
        //     buffer_size,
        //     std::slice::from_ref(&uniform_buffer),
        // );
        unsafe {
            let mapped_ptr = object.graphics_pipeline.uniform_buffers[current_image]
                .allocation
                .mapped_ptr()
                .unwrap()
                .as_ptr() as *mut f32;
            mapped_ptr.copy_from_nonoverlapping(
                std::slice::from_ref(&uniform_buffer).as_ptr() as *const f32,
                buffer_size,
            );
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use wave_camera::camera::Camera;
    use wave_space::space::Pos3;

    #[test]
    fn test_new_uniform_buffer_object() {
        let extent = vk::Extent2D::default().height(1920).width(1080);
        let camera = Camera::default().pos(Pos3::new(5., 6., 7.)).build();
        // let ubo = UniformBufferObject::new(&extent, &camera);
        //
        // let expected_ubo = UniformBufferObject {
        //     view: Matrix4::new(
        //         0., 0., 1., -7., 0., 1., 0., -6., -1., -0., -0., 5., 0., 0., 0., 1.,
        //     ),
        //     proj: Matrix4::new(
        //         4.291935, 0., 0., 0., 0., -2.4142134, 0., 0., 0., 0., -1.0000001, -2e-5, 0., 0.,
        //         -1., 0.,
        //     ),
        // };
        //
        // assert_eq!(ubo, expected_ubo);
    }

    #[test]
    fn test_ubo_update() {
        let extent = vk::Extent2D::default().height(1920).width(1080);
        let camera = Camera::default().pos(Pos3::new(5., 6., 7.)).build();
        // let mut ubo = UniformBufferObject::new(&extent, &camera);
        //
        // camera.pos = Pos3::new(-5., -1., 3.);
        // ubo.update(&extent, &camera);
        //
        // let expected_ubo = UniformBufferObject {
        //     view: Matrix4::new(
        //         0., 0., 1., -3., 0., 1., 0., 1., -1., -0., -0., -5., 0., 0., 0., 1.,
        //     ),
        //     proj: Matrix4::new(
        //         4.291935, 0., 0., 0., 0., -2.4142134, 0., 0., 0., 0., -1.0000001, -2e-5, 0., 0.,
        //         -1., 0.,
        //     ),
        // };
        //
        // assert_eq!(ubo, expected_ubo);
    }
}
