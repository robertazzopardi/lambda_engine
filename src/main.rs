extern crate ash;
extern crate winit;

mod Texture;
mod model;
mod pipeline;
mod texture;

use model::{Model, ModelType};
use ash::extensions::ext::DebugUtils;
use ash::extensions::khr::{Surface, Swapchain};
use ash::util::Align;
use ash::vk::{
    self, Buffer, BufferUsageFlags, CommandBuffer, CommandPool, CullModeFlags,
    DebugUtilsMessengerEXT, DescriptorPool, DescriptorPoolSize, DescriptorSet, DescriptorSetLayout,
    DeviceSize, Extent2D, Extent3D, Fence, Format, Framebuffer, Image, ImageAspectFlags,
    ImageSubresourceRange, ImageTiling, ImageUsageFlags, ImageView, MemoryPropertyFlags, Offset2D,
    Pipeline, PipelineLayout, PresentModeKHR, PrimitiveTopology, Queue, RenderPass,
    SampleCountFlags, Semaphore, ShaderModule, SurfaceCapabilitiesKHR, SurfaceFormatKHR,
};
use ash::vk::{DeviceMemory, PhysicalDevice, Sampler};
use ash::vk::{SurfaceKHR, SwapchainKHR};
use ash::Device;
use ash::{Entry, Instance};
use cgmath::{
    Deg, Matrix4, Point3, Quaternion, Rotation, Rotation3, SquareMatrix, Vector2, Vector3, Zero,
};

use memoffset::offset_of;
use pipeline::GraphicsPipeline;
use winit::event::ElementState;

use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::mem::size_of;
use std::ops::{Mul, Sub};
use std::ptr;

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

pub struct LambdaDevices {
    physical: PhysicalDevice,
    logical: Device,
    present_queue: Queue,
    graphics_queue: Queue,
}

impl LambdaDevices {
    fn new(
        physical: PhysicalDevice,
        logical: Device,
        present_queue: Queue,
        graphics_queue: Queue,
    ) -> Self {
        Self {
            physical,
            logical,
            present_queue,
            graphics_queue,
        }
    }
}

struct SyncObjects {
    image_available_semaphores: [Semaphore; MAX_FRAMES_IN_FLIGHT],
    render_finished_semaphores: [Semaphore; MAX_FRAMES_IN_FLIGHT],
    in_flight_fences: [Fence; MAX_FRAMES_IN_FLIGHT],
    images_in_flight: Vec<Fence>,
}

struct SwapChainSupportDetails {
    capabilities: vk::SurfaceCapabilitiesKHR,
    formats: Vec<SurfaceFormatKHR>,
    present_modes: Vec<PresentModeKHR>,
}

pub struct LambdaSwapchain {
    loader: Swapchain,
    swapchain: SwapchainKHR,
    swapchain_images: Vec<Image>,
    swapchain_image_format: Format,
    swapchain_extent: Extent2D,
    swapchain_image_views: Vec<ImageView>,
}

pub struct QueueFamilyIndices {
    pub graphics_family: Option<u32>,
    pub present_family: Option<u32>,
}

impl QueueFamilyIndices {
    pub fn new() -> QueueFamilyIndices {
        QueueFamilyIndices {
            graphics_family: None,
            present_family: None,
        }
    }

    pub fn is_complete(&self) -> bool {
        self.graphics_family.is_some() && self.present_family.is_some()
    }
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

struct DepthResource {
    image: Image,
    memory: DeviceMemory,
    view: ImageView,
}

struct ColourResource {
    image: Image,
    memory: DeviceMemory,
    view: ImageView,
}

pub struct LambdaDescriptorSet {
    descriptor_sets: Vec<DescriptorSet>,
    descriptor_set_layout: DescriptorSetLayout,
    descriptor_pool: DescriptorPool,
    uniform_buffers: Vec<Buffer>,
    uniform_buffers_memory: Vec<DeviceMemory>,
}

struct Camera {
    eye: Point3<f32>,
}

struct Vulkan {
    instance: Instance,
    entry: Entry,
    // debug_messenger: Option<DebugUtilsMessengerEXT>,
    surface: SurfaceKHR,
    devices: LambdaDevices,
    swapchain: LambdaSwapchain,
    surface_loader: Surface,
    command_buffers: Vec<CommandBuffer>,
    sync_objects: SyncObjects,
    current_frame: usize,
    models: Vec<Model>,

    render_pass: RenderPass,
    color_resource: ColourResource,
    depth_resource: DepthResource,
    frame_buffers: Vec<Framebuffer>,
    command_pool: CommandPool,

    msaa_samples: SampleCountFlags,

    ubo: UniformBufferObject,
    camera: Camera,

    is_framebuffer_resized: bool,
}

impl Vulkan {
    fn new(window: &Window) -> Self {
        let (instance, entry) = Self::create_instance(window);

        let _debug_messenger = unsafe { Self::setup_debug_messenger(&instance, &entry) };

        let surface = Self::create_surface(&instance, &entry, window);

        let (physical_device, queue_family_index, surface_loader, msaa_samples) =
            Self::pick_physical_device(&instance, &entry, &surface);

        let (logical_device, present_queue, graphics_queue) = Self::create_logical_device(
            &instance,
            physical_device,
            queue_family_index,
            &surface,
            &surface_loader,
        );

        let devices = LambdaDevices::new(
            physical_device,
            logical_device,
            present_queue,
            graphics_queue,
        );

        let swapchain =
            Self::create_swapchain(&instance, &devices, surface, &surface_loader, window);

        let render_pass = Self::create_render_pass(&instance, &devices, &swapchain, msaa_samples);

        let color_resource =
            Self::create_colour_resources(&devices, &swapchain, msaa_samples, &instance);
        let depth_resource =
            Self::create_depth_resource(&devices, &swapchain, msaa_samples, &instance);

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
                swapchain.swapchain_images.len() as u32,
                ModelType::SPHERE,
                true,
                Some(vk::PrimitiveTopology::TRIANGLE_LIST),
                Some(vk::CullModeFlags::BACK),
                &swapchain,
                msaa_samples,
                render_pass,
            ),
            Model::new(
                &instance,
                &devices,
                include_bytes!("/Users/rob/Downloads/2k_saturn_ring_alpha.png"),
                command_pool,
                swapchain.swapchain_images.len() as u32,
                ModelType::RING,
                false,
                Some(vk::PrimitiveTopology::TRIANGLE_STRIP),
                Some(vk::CullModeFlags::NONE),
                &swapchain,
                msaa_samples,
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
            surface,
            devices,
            swapchain,
            command_buffers,
            msaa_samples,
            sync_objects,
            surface_loader,
            current_frame: 0,
            models: shapes,
            ubo: UniformBufferObject::default(),
            entry,
            render_pass,
            color_resource,
            depth_resource,
            frame_buffers,
            command_pool,

            camera: Camera {
                eye: Point3::new(5., 1., 2.),
            },

            is_framebuffer_resized: false,
        }
    }

    fn create_command_pool(
        instance: &Instance,
        devices: &LambdaDevices,
        surface_loader: &Surface,
        surface: &SurfaceKHR,
    ) -> CommandPool {
        let queue_family_indices =
            Self::find_queue_family(instance, devices.physical, surface_loader, surface);

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

    unsafe fn setup_debug_messenger(
        instance: &Instance,
        entry: &Entry,
    ) -> Option<DebugUtilsMessengerEXT> {
        if !enable_validation_layers() {}

        let create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
            .message_severity(vk::DebugUtilsMessageSeverityFlagsEXT::all())
            .message_type(vk::DebugUtilsMessageTypeFlagsEXT::all())
            .pfn_user_callback(Some(vulkan_debug_callback));

        let debug_utils_loader = DebugUtils::new(entry, instance);
        Some(
            debug_utils_loader
                .create_debug_utils_messenger(&create_info, None)
                .unwrap(),
        )
    }

    fn create_surface(instance: &Instance, entry: &Entry, window: &Window) -> SurfaceKHR {
        unsafe {
            ash_window::create_surface(entry, instance, window, None)
                .expect("Failed to create window surface!")
        }
    }

    fn get_max_usable_sample_count(
        instance: &Instance,
        physical_device: PhysicalDevice,
    ) -> SampleCountFlags {
        unsafe {
            let physical_device_properties =
                instance.get_physical_device_properties(physical_device);

            let counts = physical_device_properties
                .limits
                .framebuffer_color_sample_counts
                & physical_device_properties
                    .limits
                    .framebuffer_depth_sample_counts;

            if (counts & vk::SampleCountFlags::TYPE_64) == vk::SampleCountFlags::TYPE_64 {
                return vk::SampleCountFlags::TYPE_64;
            }
            if (counts & vk::SampleCountFlags::TYPE_32) == vk::SampleCountFlags::TYPE_32 {
                return vk::SampleCountFlags::TYPE_32;
            }
            if (counts & vk::SampleCountFlags::TYPE_16) == vk::SampleCountFlags::TYPE_16 {
                return vk::SampleCountFlags::TYPE_16;
            }
            if (counts & vk::SampleCountFlags::TYPE_8) == vk::SampleCountFlags::TYPE_8 {
                return vk::SampleCountFlags::TYPE_8;
            }
            if (counts & vk::SampleCountFlags::TYPE_4) == vk::SampleCountFlags::TYPE_4 {
                return vk::SampleCountFlags::TYPE_4;
            }
            if (counts & vk::SampleCountFlags::TYPE_2) == vk::SampleCountFlags::TYPE_2 {
                return vk::SampleCountFlags::TYPE_2;
            }

            vk::SampleCountFlags::TYPE_1
        }
    }

    fn pick_physical_device(
        instance: &Instance,
        entry: &Entry,
        surface: &SurfaceKHR,
    ) -> (PhysicalDevice, u32, Surface, SampleCountFlags) {
        unsafe {
            let devices = instance
                .enumerate_physical_devices()
                .expect("Failed to find GPUs with Vulkan support!");

            let surface_loader = ash::extensions::khr::Surface::new(entry, instance);

            let (physical_device, queue_family_index) = devices
                .iter()
                .map(|pdevice| {
                    instance
                        .get_physical_device_queue_family_properties(*pdevice)
                        .iter()
                        .enumerate()
                        .find_map(|(index, info)| {
                            let supports_graphic_and_surface =
                                info.queue_flags.contains(vk::QueueFlags::GRAPHICS)
                                    && surface_loader
                                        .get_physical_device_surface_support(
                                            *pdevice,
                                            index as u32,
                                            *surface,
                                        )
                                        .unwrap();
                            if supports_graphic_and_surface {
                                Some((*pdevice, index))
                            } else {
                                None
                            }
                        })
                })
                .flatten()
                .next()
                .expect("Couldn't find suitable device.");

            let samples = Self::get_max_usable_sample_count(instance, physical_device);

            (
                physical_device,
                queue_family_index as u32,
                surface_loader,
                samples,
            )
        }
    }

    pub fn find_queue_family(
        instance: &ash::Instance,
        physical_device: vk::PhysicalDevice,
        surface_loader: &Surface,
        surface: &SurfaceKHR,
    ) -> QueueFamilyIndices {
        let queue_families =
            unsafe { instance.get_physical_device_queue_family_properties(physical_device) };

        let mut queue_family_indices = QueueFamilyIndices::new();

        let mut index = 0;
        for queue_family in queue_families.iter() {
            if queue_family.queue_count > 0
                && queue_family.queue_flags.contains(vk::QueueFlags::GRAPHICS)
            {
                queue_family_indices.graphics_family = Some(index);
            }

            let is_present_support = unsafe {
                surface_loader.get_physical_device_surface_support(
                    physical_device,
                    index as u32,
                    *surface,
                )
            }
            .unwrap();
            if queue_family.queue_count > 0 && is_present_support {
                queue_family_indices.present_family = Some(index);
            }

            if queue_family_indices.is_complete() {
                break;
            }

            index += 1;
        }

        queue_family_indices
    }

    fn create_logical_device(
        instance: &Instance,
        physical_device: PhysicalDevice,
        queue_family_index: u32,
        surface: &SurfaceKHR,
        surface_loader: &Surface,
    ) -> (Device, Queue, Queue) {
        let device_extension_names_raw = [Swapchain::name().as_ptr()];

        let features = vk::PhysicalDeviceFeatures::builder()
            .sampler_anisotropy(true)
            .sample_rate_shading(true)
            .fill_mode_non_solid(true)
            .shader_clip_distance(true);

        let priorities = 1.0;

        let queue_info = vk::DeviceQueueCreateInfo::builder()
            .queue_family_index(queue_family_index)
            .queue_priorities(std::slice::from_ref(&priorities));

        let device_create_info = vk::DeviceCreateInfo::builder()
            .queue_create_infos(std::slice::from_ref(&queue_info))
            .enabled_extension_names(&device_extension_names_raw)
            .enabled_features(&features);

        unsafe {
            let logical_device = instance
                .create_device(physical_device, &device_create_info, None)
                .unwrap();

            let queue_family =
                Self::find_queue_family(instance, physical_device, surface_loader, surface);

            let graphics_queue = unsafe {
                logical_device.get_device_queue(queue_family.graphics_family.unwrap(), 0)
            };
            let present_queue =
                unsafe { logical_device.get_device_queue(queue_family.present_family.unwrap(), 0) };

            (logical_device, present_queue, graphics_queue)
        }
    }

    fn query_swapchain_support(
        _instance: &Instance,
        devices: &LambdaDevices,
        surface: SurfaceKHR,
        surface_loader: &Surface,
    ) -> SwapChainSupportDetails {
        let mut details = SwapChainSupportDetails {
            formats: Vec::new(),
            present_modes: Vec::new(),
            capabilities: unsafe {
                surface_loader
                    .get_physical_device_surface_capabilities(devices.physical, surface)
                    .unwrap()
            },
        };

        details.formats = unsafe {
            surface_loader
                .get_physical_device_surface_formats(devices.physical, surface)
                .expect("Could not get Physical Device Surface Formats")
        };

        details.present_modes = unsafe {
            surface_loader
                .get_physical_device_surface_present_modes(devices.physical, surface)
                .expect("Could not get Physical Device Present Modes")
        };

        details
    }

    fn choose_swap_surface_format(formats: &[SurfaceFormatKHR]) -> SurfaceFormatKHR {
        for format in formats {
            if format.format == vk::Format::R8G8B8A8_SRGB
                && format.color_space == vk::ColorSpaceKHR::EXTENDED_SRGB_NONLINEAR_EXT
            {
                return *format;
            }
        }

        formats[0]
    }

    fn choose_present_mode(present_modes: Vec<PresentModeKHR>) -> PresentModeKHR {
        for present_mode in present_modes {
            if present_mode == vk::PresentModeKHR::MAILBOX {
                return present_mode;
            }
        }
        vk::PresentModeKHR::FIFO
    }

    fn choose_swap_extent(capabilities: SurfaceCapabilitiesKHR, window: &Window) -> Extent2D {
        if capabilities.current_extent.width != u32::MAX {
            capabilities.current_extent
        } else {
            let size = window.inner_size();

            Extent2D {
                width: size.width.clamp(
                    capabilities.min_image_extent.width,
                    capabilities.max_image_extent.width,
                ),
                height: size.height.clamp(
                    capabilities.min_image_extent.height,
                    capabilities.max_image_extent.height,
                ),
            }
        }
    }

    fn create_swapchain(
        instance: &Instance,
        devices: &LambdaDevices,
        surface: SurfaceKHR,
        surface_loader: &Surface,
        window: &Window,
    ) -> LambdaSwapchain {
        let swapchain_support =
            Self::query_swapchain_support(instance, devices, surface, surface_loader);

        let surface_format = Self::choose_swap_surface_format(&swapchain_support.formats);

        let present_mode = Self::choose_present_mode(swapchain_support.present_modes);

        let extent = Self::choose_swap_extent(swapchain_support.capabilities, window);

        let mut swapchain_image_count = swapchain_support.capabilities.min_image_count + 1;

        if swapchain_support.capabilities.max_image_count > 0
            && swapchain_image_count > swapchain_support.capabilities.max_image_count
        {
            swapchain_image_count = swapchain_support.capabilities.max_image_count;
        }

        let mut create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface)
            .min_image_count(swapchain_image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(swapchain_support.capabilities.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true)
            .old_swapchain(SwapchainKHR::null());

        let queue_family_indices =
            Self::find_queue_family(instance, devices.physical, surface_loader, &surface);

        if queue_family_indices.graphics_family != queue_family_indices.present_family {
            create_info.image_sharing_mode = vk::SharingMode::CONCURRENT;
            create_info.queue_family_index_count = 2;

            let queue_family_indices_arr = [
                queue_family_indices.graphics_family.unwrap(),
                queue_family_indices.present_family.unwrap(),
            ];

            create_info.p_queue_family_indices = queue_family_indices_arr.as_ptr();
        } else {
            create_info.image_sharing_mode = vk::SharingMode::EXCLUSIVE;
        }

        let swapchain = Swapchain::new(instance, &devices.logical);

        unsafe {
            let swapchain_khr = swapchain
                .create_swapchain(&create_info, None)
                .expect("Failed to create swapchain");

            let swapchain_images = swapchain
                .get_swapchain_images(swapchain_khr)
                .expect("Could not get swapchain images");

            let image_views =
                Self::create_image_views(devices, &swapchain_images, &surface_format, 1);

            LambdaSwapchain {
                loader: swapchain,
                swapchain: swapchain_khr,
                swapchain_images,
                swapchain_image_format: surface_format.format,
                swapchain_extent: extent,
                swapchain_image_views: image_views,
            }
        }
    }

    fn create_image_views(
        devices: &LambdaDevices,
        swapchain_images: &Vec<Image>,
        surface_format: &SurfaceFormatKHR,
        mip_levels: u32,
    ) -> Vec<vk::ImageView> {
        let mut swapchain_imageviews = vec![];

        for &image in swapchain_images.iter() {
            let imageview_create_info = vk::ImageViewCreateInfo {
                s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
                image,
                view_type: vk::ImageViewType::TYPE_2D,
                format: surface_format.format,
                components: vk::ComponentMapping {
                    r: vk::ComponentSwizzle::IDENTITY,
                    g: vk::ComponentSwizzle::IDENTITY,
                    b: vk::ComponentSwizzle::IDENTITY,
                    a: vk::ComponentSwizzle::IDENTITY,
                },
                subresource_range: vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: mip_levels,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                ..Default::default()
            };

            let imageview = unsafe {
                devices
                    .logical
                    .create_image_view(&imageview_create_info, None)
                    .expect("Failed to create Image View!")
            };
            swapchain_imageviews.push(imageview);
        }

        swapchain_imageviews
    }

    unsafe fn find_depth_format(instance: &Instance, physical_device: &PhysicalDevice) -> Format {
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

            if tiling == vk::ImageTiling::LINEAR
                && (format_properties.linear_tiling_features & features) == features
            {
                return *format;
            } else if tiling == vk::ImageTiling::OPTIMAL
                && (format_properties.optimal_tiling_features & features) == features
            {
                return *format;
            }
        }

        panic!("Failed to find supported format!")
    }

    fn create_render_pass(
        instance: &Instance,
        devices: &LambdaDevices,
        swapchain: &LambdaSwapchain,
        msaa_samples: SampleCountFlags,
    ) -> RenderPass {
        let renderpass_attachments = [
            vk::AttachmentDescription {
                format: swapchain.swapchain_image_format,
                samples: msaa_samples,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
            vk::AttachmentDescription {
                format: unsafe { Self::find_depth_format(instance, &devices.physical) },
                samples: msaa_samples,
                load_op: vk::AttachmentLoadOp::CLEAR,
                store_op: vk::AttachmentStoreOp::DONT_CARE,
                stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
                stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
                initial_layout: vk::ImageLayout::UNDEFINED,
                final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
                ..Default::default()
            },
            vk::AttachmentDescription {
                format: swapchain.swapchain_image_format,
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
        devices: &LambdaDevices,
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
        samples: SampleCountFlags,
        format: Format,
        tiling: ImageTiling,
        usage: ImageUsageFlags,
        properties: MemoryPropertyFlags,
        devices: &LambdaDevices,
        instance: &Instance,
    ) -> (Image, DeviceMemory) {
        let image_info = vk::ImageCreateInfo {
            s_type: vk::StructureType::IMAGE_CREATE_INFO,
            image_type: vk::ImageType::TYPE_2D,
            extent: Extent3D {
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
        image: Image,
        format: Format,
        aspect_mask: ImageAspectFlags,
        level_count: u32,
        devices: &LambdaDevices,
    ) -> ImageView {
        let image_view_info = vk::ImageViewCreateInfo {
            s_type: vk::StructureType::IMAGE_VIEW_CREATE_INFO,
            image,
            view_type: vk::ImageViewType::TYPE_2D,
            format,
            subresource_range: ImageSubresourceRange {
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

    fn create_colour_resources(
        devices: &LambdaDevices,
        swapchain: &LambdaSwapchain,
        samples: SampleCountFlags,
        instance: &Instance,
    ) -> ColourResource {
        let color_format = swapchain.swapchain_image_format;

        let (image, memory) = Self::create_image(
            swapchain.swapchain_extent.width,
            swapchain.swapchain_extent.height,
            1,
            samples,
            color_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::MemoryPropertyFlags::LAZILY_ALLOCATED,
            devices,
            instance,
        );

        let view =
            Self::create_image_view(image, color_format, vk::ImageAspectFlags::COLOR, 1, devices);

        ColourResource {
            image,
            memory,
            view,
        }
    }

    fn create_resource(
        _devuces: &LambdaDevices,
        _swapchain: &SwapchainKHR,
        _samples: SampleCountFlags,
        _instance: &Instance,
    ) {
    }

    fn create_depth_resource(
        devices: &LambdaDevices,
        swapchain: &LambdaSwapchain,
        samples: SampleCountFlags,
        instance: &Instance,
    ) -> DepthResource {
        let depth_format = unsafe { Self::find_depth_format(instance, &devices.physical) };

        let (image, memory) = Self::create_image(
            swapchain.swapchain_extent.width,
            swapchain.swapchain_extent.height,
            1,
            samples,
            depth_format,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::TRANSIENT_ATTACHMENT
                | vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT,
            vk::MemoryPropertyFlags::LAZILY_ALLOCATED,
            devices,
            instance,
        );

        let view =
            Self::create_image_view(image, depth_format, vk::ImageAspectFlags::DEPTH, 1, devices);

        DepthResource {
            image,
            memory,
            view,
        }
    }

    fn create_frame_buffers(
        swapchain: &LambdaSwapchain,
        depth_image_view: ImageView,
        render_pass: RenderPass,
        device: &Device,
        color_resource: ImageView,
    ) -> Vec<vk::Framebuffer> {
        let mut frame_buffers = Vec::new();

        for i in 0..swapchain.swapchain_images.len() {
            let attachments = &[
                color_resource,
                depth_image_view,
                swapchain.swapchain_image_views[i],
            ];

            let frame_buffer_info = vk::FramebufferCreateInfo::builder()
                .render_pass(render_pass)
                .attachments(attachments)
                .width(swapchain.swapchain_extent.width)
                .height(swapchain.swapchain_extent.height)
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
        command_pool: CommandPool,
        swapchain: &LambdaSwapchain,
        devices: &LambdaDevices,
        render_pass: RenderPass,
        frame_buffers: &[vk::Framebuffer],
        shapes: &[Model],
    ) -> Vec<CommandBuffer> {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
            .command_pool(command_pool)
            .command_buffer_count(swapchain.swapchain_images.len() as u32)
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
            .width(swapchain.swapchain_extent.width as f32)
            .height(swapchain.swapchain_extent.height as f32)
            .min_depth(0.)
            .max_depth(1.);

        let scissor = vk::Rect2D::builder()
            .offset(Offset2D { x: 0, y: 0 })
            .extent(Extent2D {
                width: swapchain.swapchain_extent.width,
                height: swapchain.swapchain_extent.height,
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
            for i in 0..swapchain.swapchain_images.len() {
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
                        extent: swapchain.swapchain_extent,
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

    fn create_sync_objects(device: &Device, _swapchain: &LambdaSwapchain) -> SyncObjects {
        let semaphore_create_info = vk::SemaphoreCreateInfo {
            s_type: vk::StructureType::SEMAPHORE_CREATE_INFO,
            ..Default::default()
        };

        let fence_info = vk::FenceCreateInfo {
            s_type: vk::StructureType::FENCE_CREATE_INFO,
            flags: vk::FenceCreateFlags::SIGNALED,
            ..Default::default()
        };

        let mut image_available_semaphores: [Semaphore; MAX_FRAMES_IN_FLIGHT] = Default::default();
        let mut render_finished_semaphores: [Semaphore; MAX_FRAMES_IN_FLIGHT] = Default::default();
        let mut in_flight_fences: [Fence; MAX_FRAMES_IN_FLIGHT] = Default::default();
        let images_in_flight: Vec<Fence> = [Fence::null(); 3].to_vec();

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
        device_memory: DeviceMemory,
        device_size: DeviceSize,
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
        let rot = Quaternion::from_axis_angle(Vector3::new(0., 0., 1.), Deg(1.0))
            .rotate_point(self.camera.eye);
        self.camera.eye = rot;

        let aspect = self.swapchain.swapchain_extent.width as f32
            / self.swapchain.swapchain_extent.height as f32;

        self.ubo = UniformBufferObject {
            model: Matrix4::identity(),
            view: Matrix4::look_at_rh(
                self.camera.eye,
                Point3::new(0., 0., 0.),
                Vector3::new(0., 0., 1.),
            ),
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

        self.swapchain = Self::create_swapchain(
            &self.instance,
            &self.devices,
            self.surface,
            &self.surface_loader,
            window,
        );

        self.render_pass = Self::create_render_pass(
            &self.instance,
            &self.devices,
            &self.swapchain,
            self.msaa_samples,
        );
        self.color_resource = Self::create_colour_resources(
            &self.devices,
            &self.swapchain,
            self.msaa_samples,
            &self.instance,
        );
        self.depth_resource = Self::create_depth_resource(
            &self.devices,
            &self.swapchain,
            self.msaa_samples,
            &self.instance,
        );
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
                self.msaa_samples,
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

        self.sync_objects.images_in_flight = vec![Fence::null(); 1];
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
                for i in 0..self.swapchain.swapchain_images.len() {
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

            for i in 0..self.swapchain.swapchain_images.len() {
                self.devices
                    .logical
                    .destroy_framebuffer(self.frame_buffers[i], None);

                self.devices
                    .logical
                    .destroy_image_view(self.swapchain.swapchain_image_views[i], None);
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
                    _ => panic!("Failed to acquire Swap Chain Image!"),
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
        unsafe {}
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
        // *control_flow = ControlFlow::Wait;

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
            _ => (),
        }

        unsafe { vulkan.render(&window) };
    });
}
