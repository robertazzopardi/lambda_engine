extern crate ash;
extern crate winit;

pub mod camera;
mod command;
mod debug;
mod device;
pub mod display;
mod memory;
pub mod model;
mod pipeline;
mod render;
mod resource;
mod swap_chain;
mod sync_objects;
mod texture;
pub mod time;
mod uniform;
mod utility;

use ash::{
    extensions::khr::Surface,
    vk::{self, Extent2D},
    Instance,
};
use camera::Camera;
use debug::Debug;
use device::Devices;
use display::Display;
use model::{Model, ModelProperties};
use pipeline::GraphicsPipeline;
use resource::Resources;
use std::ptr;
use swap_chain::SwapChain;
use sync_objects::{SyncObjects, MAX_FRAMES_IN_FLIGHT};
use time::Time;
use uniform::UniformBufferObject;
use utility::{EntryInstance, InstanceDevices};
use winit::window::Window;

// pub struct Models<const S: usize> {
//     models: [Model; S],
// }

pub struct SceneProperties {
    pub models: Vec<ModelProperties>,
}

pub struct Vulkan {
    command_buffers: Vec<vk::CommandBuffer>,
    command_pool: vk::CommandPool,
    current_frame: usize,
    debugging: Option<Debug>,
    devices: Devices,
    frame_buffers: Vec<vk::Framebuffer>,
    instance: Instance,
    is_frame_buffer_resized: bool,
    models: Vec<Model>,
    render_pass: vk::RenderPass,
    resources: Resources,
    surface: vk::SurfaceKHR,
    surface_loader: Surface,
    swap_chain: SwapChain,
    sync_objects: SyncObjects,
    ubo: UniformBufferObject,
}

impl Vulkan {
    pub fn new(window: &Window, camera: &mut Camera, scene_properties: SceneProperties) -> Self {
        let entry_instance = EntryInstance::new(window);

        let debugging = entry_instance.debugger();

        let surface = entry_instance.create_surface(window);

        let surface_loader = Surface::new(&entry_instance.entry, &entry_instance.instance);

        let devices = Devices::new(&entry_instance.instance, &surface, &surface_loader);

        let instance_devices = InstanceDevices::new(&entry_instance.instance, &devices);

        let swap_chain = SwapChain::new(
            &entry_instance.instance,
            &devices,
            surface,
            &surface_loader,
            window,
        );

        let render_pass =
            render::create_render_pass(&entry_instance.instance, &devices, &swap_chain);

        let resources = Resources::new(&swap_chain, &instance_devices);

        let frame_buffers =
            utility::create_frame_buffers(&swap_chain, render_pass, &devices.logical, &resources);

        let command_pool = command::create_command_pool(
            &entry_instance.instance,
            &devices,
            &surface_loader,
            &surface,
        );

        let sync_objects = SyncObjects::new(&devices.logical);

        let swap_chain_len = swap_chain.images.len() as u32;

        let models = scene_properties
            .models
            .into_iter()
            .map(|property| {
                Model::new(
                    command_pool,
                    swap_chain_len,
                    &swap_chain,
                    render_pass,
                    property,
                    &instance_devices,
                )
            })
            .collect::<Vec<Model>>();

        let command_buffers = command::create_command_buffers(
            command_pool,
            &swap_chain,
            &devices,
            render_pass,
            &frame_buffers,
            &models,
        );

        Self {
            instance: entry_instance.instance,
            debugging,
            surface,
            devices,
            swap_chain,
            command_buffers,
            sync_objects,
            surface_loader,
            current_frame: 0,
            models,
            ubo: UniformBufferObject::new(camera),
            render_pass,
            resources,
            frame_buffers,
            command_pool,
            is_frame_buffer_resized: false,
        }
    }

    fn recreate_swap_chain(&mut self, window: &Window) {
        let size = window.inner_size();
        let _w = size.width;
        let _h = size.height;

        unsafe {
            self.devices
                .logical
                .device_wait_idle()
                .expect("Failed to wait for device idle!")
        };

        self.cleanup_swap_chain();

        self.swap_chain = SwapChain::new(
            &self.instance,
            &self.devices,
            self.surface,
            &self.surface_loader,
            window,
        );

        self.render_pass =
            render::create_render_pass(&self.instance, &self.devices, &self.swap_chain);

        let instance_devices = InstanceDevices::new(&self.instance, &self.devices);

        self.resources = Resources::new(&self.swap_chain, &instance_devices);

        self.frame_buffers = utility::create_frame_buffers(
            &self.swap_chain,
            self.render_pass,
            &self.devices.logical,
            &self.resources,
        );

        self.sync_objects.images_in_flight = vec![vk::Fence::null(); 1];

        let _ = self.models.iter_mut().map(|mut model| {
            model.graphics_pipeline = GraphicsPipeline::new(
                &self.swap_chain,
                self.render_pass,
                model.texture.image_view,
                model.texture.sampler,
                model.properties.clone(),
                &instance_devices,
            )
        });

        self.command_buffers = command::create_command_buffers(
            self.command_pool,
            &self.swap_chain,
            &self.devices,
            self.render_pass,
            &self.frame_buffers,
            &self.models,
        );
    }

    fn cleanup_swap_chain(&self) {
        unsafe {
            self.devices
                .logical
                .destroy_image_view(self.resources.colour.view, None);
            self.devices
                .logical
                .destroy_image(self.resources.colour.image, None);
            self.devices
                .logical
                .free_memory(self.resources.colour.memory, None);

            self.devices
                .logical
                .destroy_image_view(self.resources.depth.view, None);
            self.devices
                .logical
                .destroy_image(self.resources.depth.image, None);
            self.devices
                .logical
                .free_memory(self.resources.depth.memory, None);

            for model in &self.models {
                self.devices
                    .logical
                    .destroy_pipeline(model.graphics_pipeline.pipeline, None);
                self.devices
                    .logical
                    .destroy_pipeline_layout(model.graphics_pipeline.layout, None);

                self.devices.logical.destroy_descriptor_pool(
                    model.graphics_pipeline.descriptor_set.descriptor_pool,
                    None,
                );
            }

            self.devices
                .logical
                .free_command_buffers(self.command_pool, &self.command_buffers);

            for model in &self.models {
                for i in 0..self.swap_chain.images.len() {
                    self.devices.logical.destroy_buffer(
                        model.graphics_pipeline.descriptor_set.uniform_buffers[i],
                        None,
                    );
                    self.devices.logical.free_memory(
                        model
                            .graphics_pipeline
                            .descriptor_set
                            .uniform_buffers_memory[i],
                        None,
                    );
                }
            }

            self.devices
                .logical
                .destroy_render_pass(self.render_pass, None);

            self.swap_chain
                .loader
                .destroy_swapchain(self.swap_chain.swapchain, None);

            for i in 0..self.swap_chain.images.len() {
                self.devices
                    .logical
                    .destroy_framebuffer(self.frame_buffers[i], None);

                self.devices
                    .logical
                    .destroy_image_view(self.swap_chain.image_views[i], None);
            }
        }
    }

    // TODO marked for refactor
    fn update_uniform_buffer(&self, _camera: &mut Camera, current_image: usize) {
        // let rot = Quaternion::from_axis_angle(Vector3::unit_z(), Deg(1.0))
        //     .rotate_point(self.camera.pos);
        // self.camera.pos = rot;

        let ubos = [self.ubo];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        for model in self.models.iter() {
            memory::map_memory(
                &self.devices.logical,
                model
                    .graphics_pipeline
                    .descriptor_set
                    .uniform_buffers_memory[current_image],
                buffer_size,
                &ubos,
            );
        }
    }

    /// # Safety
    ///
    /// This function can probably be optimized
    unsafe fn render(&mut self, window: &Window, camera: &mut Camera) {
        self.devices
            .logical
            .wait_for_fences(&self.sync_objects.in_flight_fences, true, std::u64::MAX)
            .expect("Failed to wait for Fence!");

        let (image_index, _is_sub_optimal) = {
            let result = self.swap_chain.loader.acquire_next_image(
                self.swap_chain.swapchain,
                std::u64::MAX,
                self.sync_objects.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            );
            match result {
                Ok(image_index) => image_index,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swap_chain(window);
                        return;
                    }
                    _ => panic!("Failed to acquire Swap Chain vk::Image!"),
                },
            }
        };

        self.update_uniform_buffer(camera, image_index.try_into().unwrap());

        if self.sync_objects.images_in_flight[image_index as usize] != vk::Fence::null() {
            self.devices
                .logical
                .wait_for_fences(
                    &[self.sync_objects.images_in_flight[image_index as usize]],
                    true,
                    std::u64::MAX,
                )
                .expect("Could not wait for images in flight");
        }
        self.sync_objects.images_in_flight[image_index as usize] =
            self.sync_objects.in_flight_fences[self.current_frame];

        let wait_semaphores = [self.sync_objects.image_available_semaphores[self.current_frame]];
        let signal_semaphores = [self.sync_objects.render_finished_semaphores[self.current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &self.command_buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];

        self.devices
            .logical
            .reset_fences(&[self.sync_objects.in_flight_fences[self.current_frame]])
            .expect("Failed to reset Fence!");

        self.devices
            .logical
            .queue_submit(
                self.devices.present_queue,
                &submit_infos,
                self.sync_objects.in_flight_fences[self.current_frame],
            )
            .expect("Failed to execute queue submit.");

        let swapchains = [self.swap_chain.swapchain];

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: swapchains.as_ptr(),
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        let result = self
            .swap_chain
            .loader
            .queue_present(self.devices.present_queue, &present_info);

        let is_resized = match result {
            Ok(_) => self.is_frame_buffer_resized,
            Err(vk_result) => match vk_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to execute queue present."),
            },
        };
        if is_resized {
            self.is_frame_buffer_resized = false;
            self.recreate_swap_chain(window);
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}

pub(crate) fn update_state(
    ubo: &mut UniformBufferObject,
    extent: Extent2D,
    camera: &mut Camera,
    dt: f32,
) {
    camera.rotate(dt);
    ubo.update(extent, camera);
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
            self.devices.logical.device_wait_idle().unwrap();

            self.cleanup_swap_chain();

            for model in &self.models {
                self.devices
                    .logical
                    .destroy_pipeline(model.graphics_pipeline.pipeline, None);
                self.devices
                    .logical
                    .destroy_pipeline_layout(model.graphics_pipeline.layout, None);

                self.devices
                    .logical
                    .destroy_sampler(model.texture.sampler, None);
                self.devices
                    .logical
                    .destroy_image_view(model.texture.image_view, None);

                self.devices
                    .logical
                    .destroy_image(model.texture.image, None);
                self.devices.logical.free_memory(model.texture.memory, None);

                self.devices.logical.destroy_descriptor_set_layout(
                    model.graphics_pipeline.descriptor_set.descriptor_set_layout,
                    None,
                );

                self.devices
                    .logical
                    .destroy_buffer(model.vertex_buffer.buffer, None);
                self.devices
                    .logical
                    .free_memory(model.vertex_buffer.memory, None);

                self.devices
                    .logical
                    .destroy_buffer(model.index_buffer.buffer, None);
                self.devices
                    .logical
                    .free_memory(model.index_buffer.memory, None);
            }

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.devices
                    .logical
                    .destroy_semaphore(self.sync_objects.image_available_semaphores[i], None);
                self.devices
                    .logical
                    .destroy_semaphore(self.sync_objects.render_finished_semaphores[i], None);
                self.devices
                    .logical
                    .destroy_fence(self.sync_objects.in_flight_fences[i], None);
            }

            self.devices
                .logical
                .destroy_command_pool(self.command_pool, None);

            println!("here");

            self.devices.logical.destroy_device(None);

            if debug::enable_validation_layers() {
                if let Some(debugger) = &self.debugging {
                    debugger
                        .debug_utils
                        .destroy_debug_utils_messenger(debugger.debug_messenger, None);
                }
            }

            self.surface_loader.destroy_surface(self.surface, None);

            self.instance.destroy_instance(None);
        }
    }
}

pub fn run(
    mut vulkan: Vulkan,
    display: Display,
    mut time: Time,
    mut camera: Camera,
    mut mouse_pressed: bool,
) {
    display.event_loop.run(move |event, _, control_flow| {
        time.tick();

        display::handle_inputs(
            control_flow,
            event,
            &display.window,
            &mut camera,
            &mut mouse_pressed,
        );

        time.step(
            update_state,
            &mut camera,
            &mut vulkan.ubo,
            vulkan.swap_chain.extent,
        );

        unsafe { vulkan.render(&display.window, &mut camera) };
    });
}
