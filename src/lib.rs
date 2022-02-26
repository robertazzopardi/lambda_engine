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
mod swapchain;
mod sync_objects;
mod texture;
pub mod time;
mod uniform;

use ash::{
    extensions::{ext::DebugUtils, khr::Surface},
    vk, Device, Entry, Instance,
};
use camera::Camera;
use command::{create_command_buffers, create_command_pool};
use debug::Debug;
use device::Devices;
use display::{create_surface, Display};
use memory::{find_memory_type, map_memory};
use model::{Model, ModelType};
use pipeline::GraphicsPipeline;
use render::create_render_pass;
use resource::{Resource, ResourceType};
use std::{ffi::CString, ptr};
use swapchain::SwapChain;
use sync_objects::{SyncObjects, MAX_FRAMES_IN_FLIGHT};
use time::Time;
use uniform::UniformBufferObject;
use winit::window::Window;

pub struct Vulkan {
    instance: Instance,
    debugging: Option<Debug>,
    surface: vk::SurfaceKHR,
    devices: Devices,
    swapchain: SwapChain,
    surface_loader: Surface,
    command_buffers: Vec<vk::CommandBuffer>,
    sync_objects: SyncObjects,
    current_frame: usize,
    models: Vec<Model>,

    render_pass: vk::RenderPass,
    color_resource: Resource,
    depth_resource: Resource,
    frame_buffers: Vec<vk::Framebuffer>,
    command_pool: vk::CommandPool,

    ubo: UniformBufferObject,

    is_framebuffer_resized: bool,
}

impl Vulkan {
    pub fn new(window: &Window, camera: &mut Camera) -> Self {
        let (instance, entry) = create_instance(window);

        let debugging = debug::setup_debug_messenger(&instance, &entry);

        let surface = create_surface(&instance, &entry, window);

        let surface_loader = Surface::new(&entry, &instance);

        let devices = Devices::new(&instance, &surface, &surface_loader);

        let swapchain = SwapChain::new(&instance, &devices, surface, &surface_loader, window);

        let render_pass = create_render_pass(&instance, &devices, &swapchain);

        let color_resource = Resource::new(&devices, &swapchain, &instance, ResourceType::Colour);
        let depth_resource = Resource::new(&devices, &swapchain, &instance, ResourceType::Depth);

        let frame_buffers = create_frame_buffers(
            &swapchain,
            depth_resource.view,
            render_pass,
            &devices.logical,
            color_resource.view,
        );

        let command_pool = create_command_pool(&instance, &devices, &surface_loader, &surface);

        let shapes = vec![
            Model::new(
                &instance,
                &devices,
                include_bytes!("../assets/2k_saturn.jpg"),
                command_pool,
                swapchain.images.len() as u32,
                ModelType::Sphere,
                true,
                Some(vk::PrimitiveTopology::TRIANGLE_LIST),
                Some(vk::CullModeFlags::BACK),
                &swapchain,
                render_pass,
            ),
            Model::new(
                &instance,
                &devices,
                include_bytes!("../assets/2k_saturn_ring_alpha.png"),
                command_pool,
                swapchain.images.len() as u32,
                ModelType::Ring,
                false,
                Some(vk::PrimitiveTopology::TRIANGLE_STRIP),
                Some(vk::CullModeFlags::NONE),
                &swapchain,
                render_pass,
            ),
        ];

        let command_buffers = create_command_buffers(
            command_pool,
            &swapchain,
            &devices,
            render_pass,
            &frame_buffers,
            &shapes,
        );

        let sync_objects = SyncObjects::create_sync_objects(&devices.logical, &swapchain);

        Self {
            instance,
            debugging,
            surface,
            devices,
            swapchain,
            command_buffers,
            sync_objects,
            surface_loader,
            current_frame: 0,
            models: shapes,
            ubo: UniformBufferObject::new(camera),
            render_pass,
            color_resource,
            depth_resource,
            frame_buffers,
            command_pool,
            is_framebuffer_resized: false,
        }
    }

    fn recreate_swapchain(&mut self, window: &Window) {
        let size = window.inner_size();
        let _w = size.width;
        let _h = size.height;

        unsafe {
            self.devices
                .logical
                .device_wait_idle()
                .expect("Failed to wait for device idle!")
        };

        self.cleanup_swapchain();

        self.swapchain = SwapChain::new(
            &self.instance,
            &self.devices,
            self.surface,
            &self.surface_loader,
            window,
        );

        self.render_pass = create_render_pass(&self.instance, &self.devices, &self.swapchain);
        self.color_resource = Resource::new(
            &self.devices,
            &self.swapchain,
            &self.instance,
            ResourceType::Colour,
        );
        self.depth_resource = Resource::new(
            &self.devices,
            &self.swapchain,
            &self.instance,
            ResourceType::Depth,
        );

        self.frame_buffers = create_frame_buffers(
            &self.swapchain,
            self.depth_resource.view,
            self.render_pass,
            &self.devices.logical,
            self.color_resource.view,
        );

        for model in &mut self.models {
            model.graphics_pipeline = GraphicsPipeline::new(
                &self.instance,
                Some(model.graphics_pipeline.topology),
                Some(model.graphics_pipeline.cull_mode),
                &self.devices,
                &self.swapchain,
                self.render_pass,
                model.texture.image_view,
                model.texture.sampler,
            );
        }

        self.command_buffers = create_command_buffers(
            self.command_pool,
            &self.swapchain,
            &self.devices,
            self.render_pass,
            &self.frame_buffers,
            &self.models,
        );

        self.sync_objects.images_in_flight = vec![vk::Fence::null(); 1];
    }

    fn cleanup_swapchain(&self) {
        unsafe {
            self.devices
                .logical
                .destroy_image_view(self.color_resource.view, None);
            self.devices
                .logical
                .destroy_image(self.color_resource.image, None);
            self.devices
                .logical
                .free_memory(self.color_resource.memory, None);

            self.devices
                .logical
                .destroy_image_view(self.depth_resource.view, None);
            self.devices
                .logical
                .destroy_image(self.depth_resource.image, None);
            self.devices
                .logical
                .free_memory(self.depth_resource.memory, None);

            self.devices
                .logical
                .free_command_buffers(self.command_pool, &self.command_buffers);

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
                .destroy_render_pass(self.render_pass, None);

            self.swapchain
                .loader
                .destroy_swapchain(self.swapchain.swapchain, None);

            for model in &self.models {
                for i in 0..self.swapchain.images.len() {
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

            for i in 0..self.swapchain.images.len() {
                self.devices
                    .logical
                    .destroy_framebuffer(self.frame_buffers[i], None);

                self.devices
                    .logical
                    .destroy_image_view(self.swapchain.image_views[i], None);
            }
        }
    }

    // TODO marked for refactor
    pub fn update_uniform_buffer(&self, camera: &mut Camera, current_image: usize) {
        // let rot = Quaternion::from_axis_angle(Vector3::unit_z(), Deg(1.0))
        //     .rotate_point(self.camera.pos);
        // self.camera.pos = rot;

        let ubos = [self.ubo];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            for model in self.models.iter() {
                map_memory(
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
    }

    /// # Safety
    ///
    /// This function can probably be optimized
    pub unsafe fn render(&mut self, window: &Window, camera: &mut Camera) {
        self.devices
            .logical
            .wait_for_fences(&self.sync_objects.in_flight_fences, true, std::u64::MAX)
            .expect("Failed to wait for Fence!");

        let (image_index, _is_sub_optimal) = {
            let result = self.swapchain.loader.acquire_next_image(
                self.swapchain.swapchain,
                std::u64::MAX,
                self.sync_objects.image_available_semaphores[self.current_frame],
                vk::Fence::null(),
            );
            match result {
                Ok(image_index) => image_index,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        self.recreate_swapchain(window);
                        return;
                    }
                    _ => panic!("Failed to acquire Swap Chain vk::Image!"),
                },
            }
        };

        self.update_uniform_buffer(camera, image_index.try_into().unwrap());
        // self.ubo.map_to_models(
        //     &self.devices.logical,
        //     camera,
        //     &self.models,
        //     image_index.try_into().unwrap(),
        // );

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

        let swapchains = [self.swapchain.swapchain];

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
            .swapchain
            .loader
            .queue_present(self.devices.present_queue, &present_info);

        let is_resized = match result {
            Ok(_) => self.is_framebuffer_resized,
            Err(vk_result) => match vk_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to execute queue present."),
            },
        };
        if is_resized {
            self.is_framebuffer_resized = false;
            self.recreate_swapchain(window);
        }

        self.current_frame = (self.current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }

    pub fn update_state(&mut self, camera: &mut Camera, dt: f32) {
        camera.rotate(dt);
        self.ubo.update(self.swapchain.extent, camera);
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        unsafe {
            self.devices.logical.device_wait_idle().unwrap();

            self.cleanup_swapchain();

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
                    .destroy_buffer(model.vertex_buffer, None);
                self.devices
                    .logical
                    .free_memory(model.vertex_buffer_memory, None);

                self.devices
                    .logical
                    .destroy_buffer(model.index_buffer, None);
                self.devices
                    .logical
                    .free_memory(model.index_buffer_memory, None);
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

fn create_instance(window: &Window) -> (Instance, Entry) {
    if debug::enable_validation_layers() && !debug::check_validation_layer_support(window) {
        panic!("Validation layers requested, but not available!")
    }

    let layer_names = [CString::new("VK_LAYER_KHRONOS_validation").unwrap()];
    let layers_names_raw: Vec<*const i8> = layer_names
        .iter()
        .map(|raw_name| raw_name.as_ptr())
        .collect();

    let surface_extensions = ash_window::enumerate_required_extensions(window).unwrap();
    let mut extension_names_raw = surface_extensions
        .iter()
        .map(|ext| ext.as_ptr())
        .collect::<Vec<_>>();
    extension_names_raw.push(DebugUtils::name().as_ptr());

    let app_name = CString::new("Vulkan").unwrap();
    let engine_name = CString::new("No Engine").unwrap();

    let app_info = vk::ApplicationInfo::builder()
        .application_name(&app_name)
        .application_version(0)
        .engine_name(&engine_name)
        .engine_version(0)
        .api_version(vk::make_api_version(0, 1, 0, 0));

    let create_info = vk::InstanceCreateInfo::builder()
        .application_info(&app_info)
        .enabled_layer_names(&layers_names_raw)
        .enabled_extension_names(&extension_names_raw);

    unsafe {
        let entry = Entry::load().unwrap();
        let instance: Instance = entry
            .create_instance(&create_info, None)
            .expect("Instance creation error");

        (instance, entry)
    }
}

pub(crate) fn create_image(
    width: u32,
    height: u32,
    mip_levels: u32,
    samples: vk::SampleCountFlags,
    format: vk::Format,
    tiling: vk::ImageTiling,
    usage: vk::ImageUsageFlags,
    properties: vk::MemoryPropertyFlags,
    devices: &Devices,
    instance: &Instance,
) -> (vk::Image, vk::DeviceMemory) {
    let image_info = vk::ImageCreateInfo {
        s_type: vk::StructureType::IMAGE_CREATE_INFO,
        image_type: vk::ImageType::TYPE_2D,
        extent: vk::Extent3D {
            width,
            height,
            depth: 1,
        },
        mip_levels,
        array_layers: 1,
        format,
        tiling,
        initial_layout: vk::ImageLayout::UNDEFINED,
        usage,
        samples,
        sharing_mode: vk::SharingMode::EXCLUSIVE,
        ..Default::default()
    };

    unsafe {
        let image = devices
            .logical
            .create_image(&image_info, None)
            .expect("Faild to create image!");

        let memory_requirements = devices.logical.get_image_memory_requirements(image);

        let alloc_info = vk::MemoryAllocateInfo {
            s_type: vk::StructureType::MEMORY_ALLOCATE_INFO,
            allocation_size: memory_requirements.size,
            memory_type_index: find_memory_type(
                instance,
                devices,
                memory_requirements.memory_type_bits,
                properties,
            ),
            ..Default::default()
        };

        let image_memory = devices
            .logical
            .allocate_memory(&alloc_info, None)
            .expect("Failed to allocate image memory!");

        devices
            .logical
            .bind_image_memory(image, image_memory, 0)
            .expect("Failed to bind image memory");

        (image, image_memory)
    }
}

pub(crate) fn create_image_view(
    image: vk::Image,
    format: vk::Format,
    aspect_mask: vk::ImageAspectFlags,
    level_count: u32,
    devices: &Devices,
) -> vk::ImageView {
    let image_view_info = vk::ImageViewCreateInfo {
        s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
        image,
        view_type: vk::ImageViewType::TYPE_2D,
        format,
        subresource_range: vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: 0,
            level_count,
            base_array_layer: 0,
            layer_count: 1,
        },
        ..Default::default()
    };

    unsafe {
        devices
            .logical
            .create_image_view(&image_view_info, None)
            .expect("Failed to create textured image view!")
    }
}

fn create_frame_buffers(
    swapchain: &SwapChain,
    depth_image_view: vk::ImageView,
    render_pass: vk::RenderPass,
    device: &Device,
    color_resource: vk::ImageView,
) -> Vec<vk::Framebuffer> {
    let mut frame_buffers = Vec::new();

    for i in 0..swapchain.images.len() {
        let attachments = &[color_resource, depth_image_view, swapchain.image_views[i]];

        let frame_buffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass)
            .attachments(attachments)
            .width(swapchain.extent.width)
            .height(swapchain.extent.height)
            .layers(1);

        unsafe {
            frame_buffers.push(
                device
                    .create_framebuffer(&frame_buffer_info, None)
                    .expect("Failed to create Frame Buffer!"),
            );
        }
    }

    frame_buffers
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

        time.step(&mut vulkan, &mut camera);

        unsafe { vulkan.render(&display.window, &mut camera) };
    });
}
