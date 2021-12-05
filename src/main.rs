extern crate ash;
extern crate winit;

mod device;
mod model;
mod pipeline;
mod swapchain;
mod texture;

use ash::extensions::{ext::DebugUtils, khr::Surface};
use ash::{util::Align, vk, Device, Entry, Instance};
use cgmath::{Deg, Matrix4, Point3, SquareMatrix, Transform, Vector3};
use device::Devices;
use model::{Model, ModelType};

use pipeline::GraphicsPipeline;
use swapchain::SwapChain;
use winit::event::{ElementState, KeyboardInput, VirtualKeyCode};

use std::{
    borrow::Cow,
    ffi::{CStr, CString},
    ptr,
};

use winit::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::Window,
    window::WindowBuilder,
};

const WINDOW_WIDTH: u32 = 1280;
const WINDOW_HEIGHT: u32 = 720;
const MAX_FRAMES_IN_FLIGHT: usize = 2;

#[inline]
fn enable_validation_layers() -> bool {
    cfg!(debug_assertions)
}

#[inline]
fn check_validation_layer_support(_window_handle: &Window) -> bool {
    // let mut _layer_count: u32;

    true
}

unsafe extern "system" fn vulkan_debug_callback(
    message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
    message_type: vk::DebugUtilsMessageTypeFlagsEXT,
    p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
    _user_data: *mut std::os::raw::c_void,
) -> vk::Bool32 {
    let callback_data = *p_callback_data;
    let message_id_number: i32 = callback_data.message_id_number as i32;

    let message_id_name = if callback_data.p_message_id_name.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message_id_name).to_string_lossy()
    };

    let message = if callback_data.p_message.is_null() {
        Cow::from("")
    } else {
        CStr::from_ptr(callback_data.p_message).to_string_lossy()
    };

    println!(
        "{:?}:\n{:?} [{} ({})] : {}",
        message_severity,
        message_type,
        message_id_name,
        &message_id_number.to_string(),
        message,
    );

    vk::FALSE
}

struct SyncObjects {
    image_available_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    render_finished_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT],
    in_flight_fences: [vk::Fence; MAX_FRAMES_IN_FLIGHT],
    images_in_flight: Vec<vk::Fence>,
}

#[derive(Clone, Copy, Debug)]
struct UniformBufferObject {
    model: Matrix4<f32>,
    view: Matrix4<f32>,
    proj: Matrix4<f32>,
}

impl Default for UniformBufferObject {
    fn default() -> Self {
        Self {
            model: Matrix4::identity(),
            view: Matrix4::identity(),
            proj: Matrix4::identity(),
        }
    }
}

struct Resource {
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
}

impl Resource {
    fn create_colour_resources(
        devices: &Devices,
        swapchain: &SwapChain,
        instance: &Instance,
    ) -> Self {
        let color_format = swapchain.image_format;

        let (image, memory) = Vulkan::create_image(
            swapchain.extent.width,
            swapchain.extent.height,
            1,
            devices.msaa_samples,
            color_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::MemoryPropertyFlags::LAZILY_ALLOCATED,
            devices,
            instance,
        );

        let view =
            Vulkan::create_image_view(image, color_format, vk::ImageAspectFlags::COLOR, 1, devices);

        Self {
            image,
            memory,
            view,
        }
    }

    fn create_depth_resource(
        devices: &Devices,
        swapchain: &SwapChain,
        instance: &Instance,
    ) -> Self {
        let depth_format = unsafe { Self::find_depth_format(instance, &devices.physical) };

        let (image, memory) = Vulkan::create_image(
            swapchain.extent.width,
            swapchain.extent.height,
            1,
            devices.msaa_samples,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT
                | vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::LAZILY_ALLOCATED,
            devices,
            instance,
        );

        let view =
            Vulkan::create_image_view(image, depth_format, vk::ImageAspectFlags::DEPTH, 1, devices);

        Self {
            image,
            memory,
            view,
        }
    }

    unsafe fn find_depth_format(
        instance: &Instance,
        physical_device: &vk::PhysicalDevice,
    ) -> vk::Format {
        let candidates = [
            vk::Format::D32_SFLOAT,
            vk::Format::D32_SFLOAT_S8_UINT,
            vk::Format::D24_UNORM_S8_UINT,
        ];
        Self::find_supported_format(
            instance,
            *physical_device,
            &candidates,
            vk::ImageTiling::OPTIMAL,
            vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT,
        )
    }

    fn find_supported_format(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        candidate_formats: &[vk::Format],
        tiling: vk::ImageTiling,
        features: vk::FormatFeatureFlags,
    ) -> vk::Format {
        for format in candidate_formats.iter() {
            let format_properties =
                unsafe { instance.get_physical_device_format_properties(physical_device, *format) };

            if (tiling == vk::ImageTiling::LINEAR
                && (format_properties.linear_tiling_features & features) == features)
                || (tiling == vk::ImageTiling::OPTIMAL
                    && (format_properties.optimal_tiling_features & features) == features)
            {
                return *format;
            }
        }

        panic!("Failed to find supported format!")
    }
}

struct Camera {
    pos: Point3<f32>,
}

impl Camera {}

struct Debugging {
    debug_messenger: vk::DebugUtilsMessengerEXT,
    debug_utils: DebugUtils,
}

struct Vulkan {
    instance: Instance,
    debugging: Option<Debugging>,
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
    camera: Camera,

    is_framebuffer_resized: bool,
}

impl Vulkan {
    fn new(window: &Window) -> Self {
        let (instance, entry) = Self::create_instance(window);

        let debugging = unsafe { Self::setup_debug_messenger(&instance, &entry) };

        let surface = Self::create_surface(&instance, &entry, window);

        let surface_loader = Surface::new(&entry, &instance);

        let devices = Devices::new(&instance, &surface, &surface_loader);

        let swapchain =
            SwapChain::create_swapchain(&instance, &devices, surface, &surface_loader, window);

        let render_pass = Self::create_render_pass(&instance, &devices, &swapchain);

        let color_resource = Resource::create_colour_resources(&devices, &swapchain, &instance);
        let depth_resource = Resource::create_depth_resource(&devices, &swapchain, &instance);

        let frame_buffers = Self::create_frame_buffers(
            &swapchain,
            depth_resource.view,
            render_pass,
            &devices.logical,
            color_resource.view,
        );

        let command_pool =
            Self::create_command_pool(&instance, &devices, &surface_loader, &surface);

        let shapes = vec![
            Model::new(
                &instance,
                &devices,
                include_bytes!("/Users/rob/Downloads/2k_saturn.jpg"),
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
                include_bytes!("/Users/rob/Downloads/2k_saturn_ring_alpha.png"),
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

        let command_buffers = Self::create_command_buffers(
            command_pool,
            &swapchain,
            &devices,
            render_pass,
            &frame_buffers,
            &shapes,
        );

        let sync_objects = Self::create_sync_objects(&devices.logical, &swapchain);

        // camera
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
            ubo: UniformBufferObject::default(),
            render_pass,
            color_resource,
            depth_resource,
            frame_buffers,
            command_pool,
            camera: Camera {
                pos: Point3::new(5., 1., 2.),
            },
            is_framebuffer_resized: false,
        }
    }

    fn create_command_pool(
        instance: &Instance,
        devices: &Devices,
        surface_loader: &Surface,
        surface: &vk::SurfaceKHR,
    ) -> vk::CommandPool {
        let queue_family_indices =
            Devices::find_queue_family(instance, devices.physical, surface_loader, surface);

        let pool_info = vk::CommandPoolCreateInfo::builder()
            .queue_family_index(queue_family_indices.graphics_family.unwrap());

        unsafe {
            devices
                .logical
                .create_command_pool(&pool_info, None)
                .expect("Failed to create command pool!")
        }
    }

    fn create_instance(window: &Window) -> (Instance, Entry) {
        if enable_validation_layers() && !check_validation_layer_support(window) {
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
            let entry = Entry::new().unwrap();
            let instance: Instance = entry
                .create_instance(&create_info, None)
                .expect("Instance creation error");

            (instance, entry)
        }
    }

    unsafe fn setup_debug_messenger(instance: &Instance, entry: &Entry) -> Option<Debugging> {
        if enable_validation_layers() {
            let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
                .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
                .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
                .pfn_user_callback(Some(vulkan_debug_callback));

            let debug_utils_loader = DebugUtils::new(entry, instance);
            Some(Debugging {
                debug_messenger: debug_utils_loader
                    .create_debug_utils_messenger(&create_info, None)
                    .unwrap(),
                debug_utils: debug_utils_loader,
            });
        }
        None
    }

    fn create_surface(instance: &Instance, entry: &Entry, window: &Window) -> vk::SurfaceKHR {
        unsafe {
            ash_window::create_surface(entry, instance, window, None)
                .expect("Failed to create window surface!")
        }
    }

    fn create_render_pass(
        instance: &Instance,
        devices: &Devices,
        swapchain: &SwapChain,
    ) -> vk::RenderPass {
        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: swapchain.image_format,
                samples: devices.msaa_samples,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
            vk::AttachmentDescription {
                format: unsafe { Resource::find_depth_format(instance, &devices.physical) },
                samples: devices.msaa_samples,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
            vk::AttachmentDescription {
                format: swapchain.image_format,
                samples: vk::SampleCountFlags::TYPE_1,
                load_op: vk::AttachmentLoadOp::DONT_CARE,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::PRESENT_SRC_KHR,
                ..Default::default()
            },
        ];

        let color_attachment_refs = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };
        let depth_attachment_ref = vk::AttachmentReference {
            attachment: 1,
            layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
        };
        let color_attachment_resolver_ref = vk::AttachmentReference {
            attachment: 2,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };

        let subpasses = vk::SubpassDescription::builder()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_refs))
            .depth_stencil_attachment(&depth_attachment_ref)
            .resolve_attachments(std::slice::from_ref(&color_attachment_resolver_ref));

        let dependencies = vk::SubpassDependency::builder()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_access_mask(vk::AccessFlags::NONE_KHR)
            .src_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_stage_mask(
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
                    | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS,
            )
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            );

        let renderpass_create_info = vk::RenderPassCreateInfo::builder()
            .attachments(&renderpass_attachments)
            .subpasses(std::slice::from_ref(&subpasses))
            .dependencies(std::slice::from_ref(&dependencies));

        unsafe {
            devices
                .logical
                .create_render_pass(&renderpass_create_info, None)
                .expect("Failed to create render pass!")
        }
    }

    pub fn find_memory_type(
        instance: &Instance,
        devices: &Devices,
        type_filter: u32,
        properties: vk::MemoryPropertyFlags,
    ) -> u32 {
        unsafe {
            let mem_properties = instance.get_physical_device_memory_properties(devices.physical);

            for i in 0..mem_properties.memory_type_count {
                if ((1 << i) & type_filter) != 0
                    && mem_properties.memory_types[i as usize].property_flags & properties
                        == properties
                {
                    return i;
                }
            }

            panic!("Failed to find suitable memory type!")
        }
    }

    fn create_image(
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
                memory_type_index: Self::find_memory_type(
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

    fn create_image_view(
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

    fn create_command_buffers(
        command_pool: vk::CommandPool,
        swapchain: &SwapChain,
        devices: &Devices,
        render_pass: vk::RenderPass,
        frame_buffers: &[vk::Framebuffer],
        shapes: &[Model],
    ) -> Vec<vk::CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(swapchain.images.len() as u32)
            .level(vk::CommandBufferLevel::PRIMARY);

        let command_buffers = unsafe {
            devices
                .logical
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Faild to allocate command renderbuffers")
        };
        let view_port = vk::Viewport::builder()
            .x(0.)
            .y(0.)
            .width(swapchain.extent.width as f32)
            .height(swapchain.extent.height as f32)
            .min_depth(0.)
            .max_depth(1.);

        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(vk::Extent2D {
                width: swapchain.extent.width,
                height: swapchain.extent.height,
            });

        let begin_info = vk::CommandBufferBeginInfo::builder();

        let clear_values = [
            vk::ClearValue {
                color: vk::ClearColorValue {
                    float32: [0., 0., 0., 1.],
                },
            },
            vk::ClearValue {
                depth_stencil: vk::ClearDepthStencilValue {
                    depth: 1.,
                    stencil: 0,
                },
            },
        ];

        let offsets = [0_u64];

        unsafe {
            for i in 0..swapchain.images.len() {
                devices
                    .logical
                    .begin_command_buffer(command_buffers[i as usize], &begin_info)
                    .expect("Faild to begin recording command buffer!");

                let render_pass_begin_info = vk::RenderPassBeginInfo {
                    s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                    p_next: ptr::null(),
                    render_pass,
                    framebuffer: frame_buffers[i],
                    render_area: vk::Rect2D {
                        offset: vk::Offset2D { x: 0, y: 0 },
                        extent: swapchain.extent,
                    },
                    clear_value_count: clear_values.len() as u32,
                    p_clear_values: clear_values.as_ptr(),
                };

                devices.logical.cmd_begin_render_pass(
                    command_buffers[i as usize],
                    &render_pass_begin_info,
                    vk::SubpassContents::INLINE,
                );

                devices.logical.cmd_set_viewport(
                    command_buffers[i as usize],
                    0,
                    std::slice::from_ref(&view_port),
                );

                devices.logical.cmd_set_scissor(
                    command_buffers[i as usize],
                    0,
                    std::slice::from_ref(&scissor),
                );

                for (_j, model) in shapes.iter().enumerate() {
                    model.bind(devices, command_buffers[i], &offsets, i);
                }

                devices
                    .logical
                    .cmd_end_render_pass(command_buffers[i as usize]);

                devices
                    .logical
                    .end_command_buffer(command_buffers[i as usize])
                    .expect("Failed to record command buffer!");
            }
        }

        command_buffers
    }

    fn create_sync_objects(device: &Device, _swapchain: &SwapChain) -> SyncObjects {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let mut image_available_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT] =
            Default::default();
        let mut render_finished_semaphores: [vk::Semaphore; MAX_FRAMES_IN_FLIGHT] =
            Default::default();
        let mut in_flight_fences: [vk::Fence; MAX_FRAMES_IN_FLIGHT] = Default::default();
        let images_in_flight: Vec<vk::Fence> = [vk::Fence::null(); 3].to_vec();

        unsafe {
            for i in 0..MAX_FRAMES_IN_FLIGHT {
                image_available_semaphores[i] = device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap();
                render_finished_semaphores[i] = device
                    .create_semaphore(&semaphore_create_info, None)
                    .unwrap();
                in_flight_fences[i] = device.create_fence(&fence_info, None).unwrap();
            }

            SyncObjects {
                image_available_semaphores,
                render_finished_semaphores,
                in_flight_fences,
                images_in_flight,
            }
        }
    }

    #[inline]
    unsafe fn map_memory<T>(
        device: &Device,
        device_memory: vk::DeviceMemory,
        device_size: vk::DeviceSize,
        to_map: &[T],
    ) where
        T: std::marker::Copy,
    {
        let data = device
            .map_memory(device_memory, 0, device_size, vk::MemoryMapFlags::empty())
            .unwrap();
        let mut vert_align = Align::new(data, std::mem::align_of::<T>() as u64, device_size);
        vert_align.copy_from_slice(to_map);
        device.unmap_memory(device_memory);
    }

    fn update_uniform_buffer(&mut self, current_image: usize) {
        // let rot = Quaternion::from_axis_angle(Vector3::unit_z(), Deg(1.0))
        //     .rotate_point(self.camera.pos);
        // self.camera.pos = rot;

        let aspect = self.swapchain.extent.width as f32 / self.swapchain.extent.height as f32;

        self.ubo = UniformBufferObject {
            model: Matrix4::identity(),
            view: Matrix4::look_at_rh(self.camera.pos, Point3::new(0., 0., 0.), Vector3::unit_z()),
            proj: {
                let mut p = cgmath::perspective(Deg(45.), aspect, 0.1, 10.);
                p[1][1] *= -1.;
                p
            },
        };

        let ubos = [self.ubo];

        let buffer_size = (std::mem::size_of::<UniformBufferObject>() * ubos.len()) as u64;

        unsafe {
            for model in self.models.iter() {
                Self::map_memory(
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

        self.swapchain = SwapChain::create_swapchain(
            &self.instance,
            &self.devices,
            self.surface,
            &self.surface_loader,
            window,
        );

        self.render_pass = Self::create_render_pass(&self.instance, &self.devices, &self.swapchain);
        self.color_resource =
            Resource::create_colour_resources(&self.devices, &self.swapchain, &self.instance);
        self.depth_resource =
            Resource::create_depth_resource(&self.devices, &self.swapchain, &self.instance);
        self.frame_buffers = Self::create_frame_buffers(
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

        self.command_buffers = Self::create_command_buffers(
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

    unsafe fn render(&mut self, window: &Window) {
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

        self.update_uniform_buffer(image_index.try_into().unwrap());

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

            if enable_validation_layers() {
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

fn main() {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_inner_size(winit::dpi::LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build(&event_loop)
        .unwrap();

    let mut vulkan: Vulkan = Vulkan::new(&window);

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == window.id() => *control_flow = ControlFlow::Exit,
            Event::WindowEvent {
                event:
                    WindowEvent::MouseInput {
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {}
            Event::WindowEvent {
                event:
                    WindowEvent::KeyboardInput {
                        input:
                            KeyboardInput {
                                virtual_keycode: Some(virtual_code),
                                state: ElementState::Pressed,
                                ..
                            },
                        ..
                    },
                ..
            } => match virtual_code {
                VirtualKeyCode::Up => {
                    let v = Vector3::new(
                        -0.01 * vulkan.camera.pos.x,
                        -0.01 * vulkan.camera.pos.y,
                        -0.01 * vulkan.camera.pos.z,
                    );

                    let m = Matrix4::<f32>::from_translation(v);
                    let z = m.transform_point(vulkan.camera.pos);
                    vulkan.camera.pos = z;
                }
                VirtualKeyCode::Down => {
                    let v = Vector3::new(
                        0.01 * vulkan.camera.pos.x,
                        0.01 * vulkan.camera.pos.y,
                        0.01 * vulkan.camera.pos.z,
                    );

                    let m = Matrix4::<f32>::from_translation(v);
                    let z = m.transform_point(vulkan.camera.pos);
                    vulkan.camera.pos = z;
                }
                VirtualKeyCode::Left => {
                    println!("here")
                }
                VirtualKeyCode::Right => {
                    println!("here")
                }
                VirtualKeyCode::Escape => *control_flow = ControlFlow::Exit,
                _ => (),
            },
            _ => (),
        }

        unsafe { vulkan.render(&window) };
    });
}
