extern crate ash;
extern crate winit;

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
use ash::{Device, InstanceError};
use ash::{Entry, Instance};
use cgmath::{
    Deg, Matrix4, Point3, Quaternion, Rad, Rotation, Rotation3, SquareMatrix, Vector2, Vector3,
    Zero,
};

use memoffset::offset_of;
use winit::event::ElementState;

use std::borrow::Cow;
use std::cmp::max;
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

struct LambdaDevices {
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

struct LambdaSwapchain {
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

enum ShapeType {
    SPHERE,
    CUBE,
}

struct Texture {
    mip_levels: u32,
    image: Image,
    image_view: ImageView,
    image_memory: DeviceMemory,
    sampler: Sampler,
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

impl Texture {
    fn new(
        instance: &Instance,
        devices: &LambdaDevices,
        image_buffer: &[u8],
        command_pool: CommandPool,
        command_buffer_count: u32,
    ) -> Self {
        let (image, image_memory, mip_levels) = Self::create_texture_image(
            instance,
            devices,
            image_buffer,
            command_pool,
            command_buffer_count,
        );
        let image_view = Self::create_texture_image_view(devices, image, mip_levels);
        let sampler = Self::create_texture_sampler(instance, devices, mip_levels);

        Self {
            mip_levels,
            image,
            image_view,
            image_memory,
            sampler,
        }
    }

    fn create_buffer(
        instance: &Instance,
        devices: &LambdaDevices,
        size: u64,
        usage: BufferUsageFlags,
        properties: vk::MemoryPropertyFlags,
    ) -> (Buffer, DeviceMemory) {
        let image_buffer_info = vk::BufferCreateInfo::builder()
            .size(size)
            .usage(usage)
            .sharing_mode(vk::SharingMode::EXCLUSIVE);

        unsafe {
            let buffer = devices
                .logical
                .create_buffer(&image_buffer_info, None)
                .expect("Faild to create buffer");

            let memory_requirements = devices.logical.get_buffer_memory_requirements(buffer);

            let memory_type_index = Vulkan::find_memory_type(
                instance,
                devices,
                memory_requirements.memory_type_bits,
                properties,
            );

            let image_buffer_allocate_info = vk::MemoryAllocateInfo::builder()
                .allocation_size(memory_requirements.size)
                .memory_type_index(memory_type_index);

            let buffer_memory = devices
                .logical
                .allocate_memory(&image_buffer_allocate_info, None)
                .expect("Failed to allocate buffer memory!");

            devices
                .logical
                .bind_buffer_memory(buffer, buffer_memory, 0)
                .expect("Could not bind command buffer memory");

            (buffer, buffer_memory)
        }
    }

    fn begin_single_time_command(
        device: &ash::Device,
        command_pool: vk::CommandPool,
    ) -> vk::CommandBuffer {
        let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
            p_next: ptr::null(),
            command_buffer_count: 1,
            command_pool,
            level: vk::CommandBufferLevel::PRIMARY,
        };

        let command_buffer = unsafe {
            device
                .allocate_command_buffers(&command_buffer_allocate_info)
                .expect("Failed to allocate Command Buffers!")
        }[0];

        let command_buffer_begin_info = vk::CommandBufferBeginInfo {
            s_type: vk::StructureType::COMMAND_BUFFER_BEGIN_INFO,
            p_next: ptr::null(),
            p_inheritance_info: ptr::null(),
            flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
        };

        unsafe {
            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Failed to begin recording Command Buffer at beginning!");
        }

        command_buffer
    }

    fn end_single_time_command(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        command_buffer: vk::CommandBuffer,
    ) {
        unsafe {
            device
                .end_command_buffer(command_buffer)
                .expect("Failed to record Command Buffer at Ending!");
        }

        let buffers_to_submit = [command_buffer];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: 0,
            p_wait_semaphores: ptr::null(),
            p_wait_dst_stage_mask: ptr::null(),
            command_buffer_count: 1,
            p_command_buffers: buffers_to_submit.as_ptr(),
            signal_semaphore_count: 0,
            p_signal_semaphores: ptr::null(),
        }];

        unsafe {
            device
                .queue_submit(submit_queue, &submit_infos, vk::Fence::null())
                .expect("Failed to Queue Submit!");
            device
                .queue_wait_idle(submit_queue)
                .expect("Failed to wait Queue idle!");
            device.free_command_buffers(command_pool, &buffers_to_submit);
        }
    }

    fn transition_image_layout(
        device: &ash::Device,
        command_pool: vk::CommandPool,
        submit_queue: vk::Queue,
        image: vk::Image,
        _format: vk::Format,
        old_layout: vk::ImageLayout,
        new_layout: vk::ImageLayout,
        mip_levels: u32,
    ) {
        let command_buffer = Self::begin_single_time_command(device, command_pool);

        let src_access_mask;
        let dst_access_mask;
        let source_stage;
        let destination_stage;

        if old_layout == vk::ImageLayout::UNDEFINED
            && new_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::empty();
            dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
            destination_stage = vk::PipelineStageFlags::TRANSFER;
        } else if old_layout == vk::ImageLayout::TRANSFER_DST_OPTIMAL
            && new_layout == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
        {
            src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            dst_access_mask = vk::AccessFlags::SHADER_READ;
            source_stage = vk::PipelineStageFlags::TRANSFER;
            destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        } else {
            panic!("Unsupported layout transition!")
        }

        let image_barriers = [vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask,
            dst_access_mask,
            old_layout,
            new_layout,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: mip_levels,
                base_array_layer: 0,
                layer_count: 1,
            },
        }];

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                source_stage,
                destination_stage,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &image_barriers,
            );
        }

        Self::end_single_time_command(device, command_pool, submit_queue, command_buffer);
    }

    fn copy_buffer_to_image(
        devices: &LambdaDevices,
        command_pool: CommandPool,
        _command_buffer_count: u32,
        width: u32,
        height: u32,
        src_buffer: Buffer,
        dst_image: Image,
    ) {
        let command_buffer = Self::begin_single_time_command(&devices.logical, command_pool);

        let image_subresource = vk::ImageSubresourceLayers::builder()
            .aspect_mask(vk::ImageAspectFlags::COLOR)
            .mip_level(0)
            .base_array_layer(0)
            .layer_count(1);
        let region = vk::BufferImageCopy::builder()
            .buffer_offset(0)
            .buffer_row_length(0)
            .buffer_image_height(0)
            .image_subresource(*image_subresource)
            .image_offset(vk::Offset3D { x: 0, y: 0, z: 0 })
            .image_extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            });

        unsafe {
            devices.logical.cmd_copy_buffer_to_image(
                command_buffer,
                src_buffer,
                dst_image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(&region),
            )
        }

        Self::end_single_time_command(
            &devices.logical,
            command_pool,
            devices.graphics_queue,
            command_buffer,
        );
    }

    fn generate_mipmaps(
        _instance: &Instance,
        devices: &LambdaDevices,
        _format: Format,
        image: Image,
        command_pool: CommandPool,
        _command_buffer_count: u32,
        tex_width: i32,
        tex_height: i32,
        mip_levels: u32,
    ) {
        let command_buffer = Texture::begin_single_time_command(&devices.logical, command_pool);

        let mut image_barrier = vk::ImageMemoryBarrier {
            s_type: vk::StructureType::IMAGE_MEMORY_BARRIER,
            p_next: ptr::null(),
            src_access_mask: vk::AccessFlags::empty(),
            dst_access_mask: vk::AccessFlags::empty(),
            old_layout: vk::ImageLayout::UNDEFINED,
            new_layout: vk::ImageLayout::UNDEFINED,
            src_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            dst_queue_family_index: vk::QUEUE_FAMILY_IGNORED,
            image,
            subresource_range: vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                base_mip_level: 0,
                level_count: 1,
                base_array_layer: 0,
                layer_count: 1,
            },
        };

        let mut mip_width = tex_width as i32;
        let mut mip_height = tex_height as i32;

        for i in 1..mip_levels {
            image_barrier.subresource_range.base_mip_level = i - 1;
            image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            image_barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            image_barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

            unsafe {
                devices.logical.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier],
                );
            }

            let blits = [vk::ImageBlit {
                src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i - 1,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                src_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: mip_width,
                        y: mip_height,
                        z: 1,
                    },
                ],
                dst_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: i,
                    base_array_layer: 0,
                    layer_count: 1,
                },
                dst_offsets: [
                    vk::Offset3D { x: 0, y: 0, z: 0 },
                    vk::Offset3D {
                        x: max(mip_width / 2, 1),
                        y: max(mip_height / 2, 1),
                        z: 1,
                    },
                ],
            }];

            unsafe {
                devices.logical.cmd_blit_image(
                    command_buffer,
                    image,
                    vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                    image,
                    vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    &blits,
                    vk::Filter::LINEAR,
                );
            }

            image_barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
            image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
            image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

            unsafe {
                devices.logical.cmd_pipeline_barrier(
                    command_buffer,
                    vk::PipelineStageFlags::TRANSFER,
                    vk::PipelineStageFlags::FRAGMENT_SHADER,
                    vk::DependencyFlags::empty(),
                    &[],
                    &[],
                    &[image_barrier],
                );
            }

            mip_width = max(mip_width / 2, 1);
            mip_height = max(mip_height / 2, 1);
        }

        image_barrier.subresource_range.base_mip_level = mip_levels - 1;
        image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            devices.logical.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &[image_barrier],
            );
        }

        Texture::end_single_time_command(
            &devices.logical,
            command_pool,
            devices.graphics_queue,
            command_buffer,
        );
    }

    fn create_texture_image(
        instance: &Instance,
        devices: &LambdaDevices,
        image_buffer: &[u8],
        command_pool: CommandPool,
        command_buffer_count: u32,
    ) -> (Image, DeviceMemory, u32) {
        let image_texture = image::load_from_memory(image_buffer).unwrap().to_rgba8();

        let image_dimensions = image_texture.dimensions();
        let image_data = image_texture.into_raw();

        let mip_levels = ((image_dimensions.0.max(image_dimensions.1) as f32)
            .log2()
            .floor()
            + 1.) as u32;

        // let size = (std::mem::size_of::<u8>() * image_data.len()) as u64;
        let size = (std::mem::size_of::<u8>() as u32 * image_dimensions.0 * image_dimensions.1 * 4)
            as vk::DeviceSize;

        let (staging_buffer, staging_buffer_memory) = Self::create_buffer(
            instance,
            devices,
            size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        unsafe {
            Vulkan::map_memory(
                &devices.logical,
                staging_buffer_memory,
                size,
                image_data.as_slice(),
            );

            let (image, image_memory) = Vulkan::create_image(
                image_dimensions.0,
                image_dimensions.1,
                mip_levels,
                vk::SampleCountFlags::TYPE_1,
                vk::Format::R8G8B8A8_SRGB,
                vk::ImageTiling::OPTIMAL,
                vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::SAMPLED,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
                devices,
                instance,
            );

            Self::transition_image_layout(
                &devices.logical,
                command_pool,
                devices.graphics_queue,
                image,
                vk::Format::R8G8B8A8_SRGB,
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                mip_levels,
            );

            Self::copy_buffer_to_image(
                devices,
                command_pool,
                command_buffer_count,
                image_dimensions.0,
                image_dimensions.1,
                staging_buffer,
                image,
            );

            devices.logical.destroy_buffer(staging_buffer, None);
            devices.logical.free_memory(staging_buffer_memory, None);

            Self::generate_mipmaps(
                instance,
                devices,
                vk::Format::R8G8B8A8_SRGB,
                image,
                command_pool,
                command_buffer_count,
                image_dimensions.0.try_into().unwrap(),
                image_dimensions.1.try_into().unwrap(),
                mip_levels,
            );

            (image, image_memory, mip_levels)
        }
    }

    fn create_texture_image_view(
        devices: &LambdaDevices,
        image: Image,
        mip_levels: u32,
    ) -> ImageView {
        Vulkan::create_image_view(
            image,
            vk::Format::R8G8B8A8_SRGB,
            vk::ImageAspectFlags::COLOR,
            mip_levels,
            devices,
        )
    }

    fn create_texture_sampler(
        instance: &Instance,
        devices: &LambdaDevices,
        mip_levels: u32,
    ) -> vk::Sampler {
        unsafe {
            let properties = instance.get_physical_device_properties(devices.physical);

            let sampler_create_info = vk::SamplerCreateInfo::builder()
                .mag_filter(vk::Filter::LINEAR)
                .min_filter(vk::Filter::LINEAR)
                .address_mode_u(vk::SamplerAddressMode::REPEAT)
                .address_mode_v(vk::SamplerAddressMode::REPEAT)
                .address_mode_w(vk::SamplerAddressMode::REPEAT)
                .anisotropy_enable(true)
                .max_anisotropy(properties.limits.max_sampler_anisotropy)
                .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
                .unnormalized_coordinates(false)
                .compare_enable(false)
                .compare_op(vk::CompareOp::ALWAYS)
                .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
                .min_lod(0.)
                .max_lod(mip_levels as f32)
                .mip_lod_bias(0.);

            devices
                .logical
                .create_sampler(&sampler_create_info, None)
                .expect("Failed to create Sampler!")
        }
    }
}

struct LambdaDescriptorSet {
    descriptor_sets: Vec<DescriptorSet>,
    descriptor_set_layout: DescriptorSetLayout,
    descriptor_pool: DescriptorPool,
    uniform_buffers: Vec<Buffer>,
    uniform_buffers_memory: Vec<DeviceMemory>,
}

struct GraphicsPipeline {
    topology: vk::PrimitiveTopology,
    cull_mode: vk::CullModeFlags,
    pipeline: vk::Pipeline,
    layout: vk::PipelineLayout,
    descriptor_set: LambdaDescriptorSet,
}

impl GraphicsPipeline {
    fn new(
        instance: &Instance,
        topology: Option<vk::PrimitiveTopology>,
        cull_mode: Option<vk::CullModeFlags>,
        devices: &LambdaDevices,
        swapchain: &LambdaSwapchain,
        msaa_samples: SampleCountFlags,
        render_pass: RenderPass,
        texture_image_view: ImageView,
        sampler: Sampler,
    ) -> Self {
        let topology = match topology {
            Some(topology) => topology,
            None => vk::PrimitiveTopology::TRIANGLE_LIST,
        };

        let cull_mode = match cull_mode {
            Some(cull_mode) => cull_mode,
            None => vk::CullModeFlags::BACK,
        };

        let descriptor_set_layout = Self::create_descriptor_set_layout(devices);

        let (pipeline, layout) = Self::create_pipeline_and_layout(
            devices,
            topology,
            cull_mode,
            swapchain,
            msaa_samples,
            &descriptor_set_layout,
            render_pass,
        );

        let descriptor_pool =
            Self::create_descriptor_pool(devices, swapchain.swapchain_images.len() as u32);

        let (uniform_buffers, uniform_buffers_memory) = GraphicsPipeline::create_uniform_buffers(
            instance,
            devices,
            swapchain.swapchain_images.len() as u32,
        );

        let descriptor_sets = Self::create_descriptor_sets(
            devices,
            descriptor_set_layout,
            descriptor_pool,
            swapchain.swapchain_images.len() as u32,
            texture_image_view,
            sampler,
            &uniform_buffers,
        );

        Self {
            topology,
            cull_mode,
            pipeline,
            layout,
            descriptor_set: LambdaDescriptorSet {
                descriptor_sets,
                descriptor_set_layout,
                descriptor_pool,
                uniform_buffers,
                uniform_buffers_memory,
            },
        }
    }

    fn create_shader_module(devices: &LambdaDevices, code: &[u32]) -> ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(code);

        unsafe {
            devices
                .logical
                .create_shader_module(&create_info, None)
                .expect("Failed to create shader module!")
        }
    }

    fn create_pipeline_and_layout(
        devices: &LambdaDevices,
        topology: PrimitiveTopology,
        cull_mode: CullModeFlags,
        swapchain: &LambdaSwapchain,
        msaa_samples: SampleCountFlags,
        descriptor_set_layout: &DescriptorSetLayout,
        render_pass: RenderPass,
    ) -> (Pipeline, PipelineLayout) {
        let entry_point = CString::new("main").unwrap();

        let mut vertex_file =
            std::fs::File::open("/Users/rob/_CODE/C/vulkan-tmp/src/shaders/light_texture/vert.spv")
                .unwrap();
        let vertex_spv = ash::util::read_spv(&mut vertex_file).unwrap();
        let vert_shader_module = Self::create_shader_module(devices, &vertex_spv);

        let mut frag_file =
            std::fs::File::open("/Users/rob/_CODE/C/vulkan-tmp/src/shaders/light_texture/frag.spv")
                .unwrap();
        let frag_spv = ash::util::read_spv(&mut frag_file).unwrap();
        let frag_shader_module = Self::create_shader_module(devices, &frag_spv);

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::VERTEX,
                module: vert_shader_module,
                p_name: entry_point.as_ptr(),
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::FRAGMENT,
                module: frag_shader_module,
                p_name: entry_point.as_ptr(),
                ..Default::default()
            },
        ];

        let binding_description = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>().try_into().unwrap())
            .input_rate(vk::VertexInputRate::VERTEX);

        let attribute_descriptions = [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, colour) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, normal) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 3,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, tex_coord) as u32,
            },
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(std::slice::from_ref(&binding_description))
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(topology)
            .primitive_restart_enable(false);

        let view_port = vk::Viewport::builder()
            .x(0.)
            .y(0.)
            .width(swapchain.swapchain_extent.width as f32)
            .height(swapchain.swapchain_extent.height as f32)
            .min_depth(0.)
            .max_depth(1.);

        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain.swapchain_extent);

        let view_port_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(std::slice::from_ref(&view_port))
            .scissors(std::slice::from_ref(&scissor));

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            // .polygon_mode(vk::PolygonMode::LINE)
            // .polygon_mode(vk::PolygonMode::POINT)
            .line_width(1.)
            .cull_mode(cull_mode)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(msaa_samples)
            .sample_shading_enable(true)
            .min_sample_shading(0.2)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        };

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .front(stencil_state)
            .back(stencil_state)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(std::slice::from_ref(&color_blend_attachment))
            .blend_constants([0., 0., 0., 0.]);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(std::slice::from_ref(descriptor_set_layout));

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state_create_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        unsafe {
            let layout = devices
                .logical
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("Failed to create pipeline layout!");

            let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_info)
                .input_assembly_state(&input_assembly)
                .viewport_state(&view_port_state)
                .rasterization_state(&rasterizer)
                .multisample_state(&multisampling)
                .dynamic_state(&dynamic_state_create_info)
                .color_blend_state(&color_blending)
                .layout(layout)
                .render_pass(render_pass)
                .subpass(0)
                .base_pipeline_handle(vk::Pipeline::null())
                .depth_stencil_state(&depth_stencil);

            let pipeline = devices
                .logical
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_info),
                    None,
                )
                .expect("Failed to create graphics pipeline!");

            devices
                .logical
                .destroy_shader_module(vert_shader_module, None);
            devices
                .logical
                .destroy_shader_module(frag_shader_module, None);

            (pipeline[0], layout)
        }
    }

    fn create_descriptor_set_layout(devices: &LambdaDevices) -> DescriptorSetLayout {
        let bindings = [
            vk::DescriptorSetLayoutBinding {
                binding: 0,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                stage_flags: vk::ShaderStageFlags::VERTEX,
                ..Default::default()
            },
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

        unsafe {
            devices
                .logical
                .create_descriptor_set_layout(&layout_info, None)
                .expect("Failed to create descriptor set layout")
        }
    }

    fn create_descriptor_pool(
        devices: &LambdaDevices,
        swapchain_image_count: u32,
    ) -> vk::DescriptorPool {
        let pool_sizes = &[
            DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: swapchain_image_count,
            },
            DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: swapchain_image_count,
            },
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_sizes)
            .max_sets(swapchain_image_count);

        unsafe {
            devices
                .logical
                .create_descriptor_pool(&pool_info, None)
                .expect("Failed to create descriptor pool!")
        }
    }

    fn create_uniform_buffers(
        instance: &Instance,
        devices: &LambdaDevices,
        swapchain_image_count: u32,
    ) -> (Vec<vk::Buffer>, Vec<vk::DeviceMemory>) {
        let mut uniform_buffers = Vec::new();
        let mut uniform_buffer_memory = Vec::new();

        for _i in 0..swapchain_image_count {
            let (buffer, memory) = Texture::create_buffer(
                instance,
                devices,
                size_of::<UniformBufferObject>() as u64,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            );
            uniform_buffers.push(buffer);
            uniform_buffer_memory.push(memory);
        }

        (uniform_buffers, uniform_buffer_memory)
    }

    fn create_descriptor_sets(
        devices: &LambdaDevices,
        descriptor_layout: DescriptorSetLayout,
        descriptor_pool: DescriptorPool,
        swapchain_image_count: u32,
        texture_image_view: ImageView,
        sampler: Sampler,
        uniform_buffers: &[Buffer],
    ) -> Vec<DescriptorSet> {
        let layouts = vec![descriptor_layout; swapchain_image_count as usize];

        let alloc_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool,
            descriptor_set_count: swapchain_image_count,
            p_set_layouts: layouts.as_slice().as_ptr(),
            ..Default::default()
        };

        let image_info = vk::DescriptorImageInfo {
            sampler,
            image_view: texture_image_view,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };

        unsafe {
            let descriptor_sets = devices
                .logical
                .allocate_descriptor_sets(&alloc_info)
                .expect("Failed to allocate descriptor sets!");

            for i in 0..swapchain_image_count as usize {
                let buffer_info = vk::DescriptorBufferInfo {
                    buffer: uniform_buffers[i],
                    offset: 0,
                    range: size_of::<UniformBufferObject>() as u64,
                };

                let descriptor_writes = [
                    vk::WriteDescriptorSet {
                        dst_set: descriptor_sets[i],
                        dst_binding: 0,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        p_buffer_info: std::slice::from_ref(&buffer_info).as_ptr(),
                        descriptor_count: 1,
                        ..Default::default()
                    },
                    vk::WriteDescriptorSet {
                        dst_set: descriptor_sets[i],
                        dst_binding: 1,
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        p_image_info: std::slice::from_ref(&image_info).as_ptr(),
                        descriptor_count: 1,
                        ..Default::default()
                    },
                ];

                devices
                    .logical
                    .update_descriptor_sets(&descriptor_writes, &[]);
            }

            descriptor_sets
        }
    }
}

const WHITE: Vector3<f32> = Vector3::new(1., 1., 1.);

#[derive(Clone, Copy, Debug)]
struct Vertex {
    pos: Vector3<f32>,
    colour: Vector3<f32>,
    normal: Vector3<f32>,
    tex_coord: Vector2<f32>,
}

struct Shape {
    vertices: Vec<Vertex>,
    indices: Vec<u16>,
    indexed: bool,
    texture: Texture,
    graphics_pipeline: GraphicsPipeline,
    vertex_buffer: Buffer,
    vertex_buffer_memory: DeviceMemory,
    index_buffer: Buffer,
    index_buffer_memory: DeviceMemory,
}

impl Shape {
    fn new(
        instance: &Instance,
        devices: &LambdaDevices,
        image_buffer: &[u8],
        command_pool: CommandPool,
        command_buffer_count: u32,
        _shape_type: ShapeType,
        indexed: bool,
        topology: Option<PrimitiveTopology>,
        cull_mode: Option<CullModeFlags>,
        swapchain: &LambdaSwapchain,
        msaa_samples: SampleCountFlags,
        render_pass: RenderPass,
    ) -> Self {
        // let (vertices, indices) = Self::cube();
        let (vertices, indices) = Self::make_sphere(0.4, 40, 40);

        let texture = Texture::new(
            instance,
            devices,
            image_buffer,
            command_pool,
            command_buffer_count,
        );

        let (vertex_buffer, vertex_buffer_memory) = Self::create_vertex_index_buffer(
            instance,
            devices,
            (size_of::<Vertex>() * vertices.len()).try_into().unwrap(),
            &vertices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
            command_pool,
            command_buffer_count,
        );

        let (index_buffer, index_buffer_memory) = Self::create_vertex_index_buffer(
            instance,
            devices,
            (size_of::<u16>() * indices.len()).try_into().unwrap(),
            &indices,
            vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::INDEX_BUFFER,
            command_pool,
            command_buffer_count,
        );

        let graphics_pipeline = GraphicsPipeline::new(
            instance,
            topology,
            cull_mode,
            devices,
            swapchain,
            msaa_samples,
            render_pass,
            texture.image_view,
            texture.sampler,
        );

        Self {
            vertices,
            indices,
            indexed,
            texture,
            graphics_pipeline,
            vertex_buffer,
            vertex_buffer_memory,
            index_buffer,
            index_buffer_memory,
        }
    }

    fn get_normal(p1: Vector3<f32>, p2: Vector3<f32>, p3: Vector3<f32>) -> Vector3<f32> {
        let a = p3.sub(p2);
        let b = p1.sub(p2);
        a.cross(b)
    }

    fn calculate_normals(shape: &mut [Vertex]) {
        let normal = Self::get_normal(shape[0].pos, shape[1].pos, shape[2].pos);

        for point in shape {
            point.normal = normal;
        }
    }

    fn make_sphere(radius: f32, sector_count: u32, stack_count: u32) -> (Vec<Vertex>, Vec<u16>) {
        let length = 1. / radius;

        let sector_step = 2. * std::f32::consts::PI / sector_count as f32;
        let stack_step = std::f32::consts::PI / stack_count as f32;

        let mut pos = Vector3::<f32>::zero();

        let mut vertices = Vec::<Vertex>::new();

        for i in 0..=stack_count {
            let stack_angle = std::f32::consts::FRAC_PI_2 - i as f32 * stack_step;
            let xy = radius * stack_angle.cos();
            pos[2] = radius * stack_angle.sin();

            for j in 0..=sector_count {
                let sector_angle = j as f32 * sector_step;

                pos[0] = xy * sector_angle.cos();
                pos[1] = xy * sector_angle.sin();

                let normal = pos.mul(length);

                let tex_coord = Vector2 {
                    x: j as f32 / sector_count as f32,
                    y: i as f32 / stack_count as f32,
                };

                vertices.push(Vertex {
                    pos,
                    colour: WHITE,
                    normal,
                    tex_coord,
                });
            }
        }

        (vertices, Self::calculate_indices(sector_count, stack_count))
    }

    fn calculate_indices(sector_count: u32, stack_count: u32) -> Vec<u16> {
        let mut k1: u16;
        let mut k2: u16;

        let mut indices: Vec<u16> = Vec::new();
        for i in 0..stack_count {
            k1 = i as u16 * (sector_count + 1) as u16;
            k2 = k1 + (stack_count + 1) as u16;

            for _j in 0..sector_count {
                if i != 0 {
                    indices.push(k1);
                    indices.push(k2);
                    indices.push(k1 + 1);
                }

                if i != (stack_count - 1) {
                    indices.push(k1 + 1);
                    indices.push(k2);
                    indices.push(k2 + 1);
                }

                k1 += 1;
                k2 += 1;
            }
        }

        indices
    }

    fn cube() -> (Vec<Vertex>, Vec<u16>) {
        let mut cube: [[Vertex; 4]; 6] = [
            [
                Vertex {
                    pos: Vector3::new(-0.5, -0.5, 0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(1., 0.),
                },
                Vertex {
                    pos: Vector3::new(0.5, -0.5, 0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(0., 0.),
                },
                Vertex {
                    pos: Vector3::new(0.5, 0.5, 0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(0., 1.),
                },
                Vertex {
                    pos: Vector3::new(-0.5, 0.5, 0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(1., 1.),
                },
            ],
            [
                Vertex {
                    pos: Vector3::new(-0.5, 0.5, -0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(1., 0.),
                },
                Vertex {
                    pos: Vector3::new(0.5, 0.5, -0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(0., 0.),
                },
                Vertex {
                    pos: Vector3::new(0.5, -0.5, -0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(0., 1.),
                },
                Vertex {
                    pos: Vector3::new(-0.5, -0.5, -0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(1., 1.),
                },
            ],
            [
                Vertex {
                    pos: Vector3::new(-0.5, 0.5, 0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(1., 0.),
                },
                Vertex {
                    pos: Vector3::new(-0.5, 0.5, -0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(0., 0.),
                },
                Vertex {
                    pos: Vector3::new(-0.5, -0.5, -0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(0., 1.),
                },
                Vertex {
                    pos: Vector3::new(-0.5, -0.5, 0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(1., 1.),
                },
            ],
            [
                Vertex {
                    pos: Vector3::new(0.5, -0.5, 0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(1., 0.),
                },
                Vertex {
                    pos: Vector3::new(0.5, -0.5, -0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(0., 0.),
                },
                Vertex {
                    pos: Vector3::new(0.5, 0.5, -0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(0., 1.),
                },
                Vertex {
                    pos: Vector3::new(0.5, 0.5, 0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(1., 1.),
                },
            ],
            [
                Vertex {
                    pos: Vector3::new(0.5, 0.5, 0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(1., 0.),
                },
                Vertex {
                    pos: Vector3::new(0.5, 0.5, -0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(0., 0.),
                },
                Vertex {
                    pos: Vector3::new(-0.5, 0.5, -0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(0., 1.),
                },
                Vertex {
                    pos: Vector3::new(-0.5, 0.5, 0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(1., 1.),
                },
            ],
            [
                Vertex {
                    pos: Vector3::new(0.5, -0.5, -0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(1., 0.),
                },
                Vertex {
                    pos: Vector3::new(0.5, -0.5, 0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(0., 0.),
                },
                Vertex {
                    pos: Vector3::new(-0.5, -0.5, 0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(0., 1.),
                },
                Vertex {
                    pos: Vector3::new(-0.5, -0.5, -0.5),
                    colour: WHITE,
                    normal: Vector3::zero(),
                    tex_coord: Vector2::new(1., 1.),
                },
            ],
        ];

        let indices: Vec<u16> = vec![
            0, 1, 2, 2, 3, 0, // top
            4, 5, 6, 6, 7, 4, // bottom
            8, 9, 10, 8, 10, 11, // right
            12, 13, 14, 12, 14, 15, // left
            16, 17, 18, 16, 18, 19, // front
            20, 21, 22, 20, 22, 23, // back
        ];

        for shape in cube.iter_mut() {
            Self::calculate_normals(shape);
        }

        (cube.into_iter().flatten().collect(), indices)
    }

    fn copy_buffer(
        devices: &LambdaDevices,
        command_pool: CommandPool,
        _command_buffer_count: u32,
        size: u64,
        src_buffer: Buffer,
        dst_buffer: Buffer,
    ) {
        let command_buffer = Texture::begin_single_time_command(&devices.logical, command_pool);

        let copy_region = vk::BufferCopy::builder().size(size);

        unsafe {
            devices.logical.cmd_copy_buffer(
                command_buffer,
                src_buffer,
                dst_buffer,
                std::slice::from_ref(&copy_region),
            );
        }

        Texture::end_single_time_command(
            &devices.logical,
            command_pool,
            devices.graphics_queue,
            command_buffer,
        );
    }

    fn create_vertex_index_buffer<T>(
        instance: &Instance,
        devices: &LambdaDevices,
        buffer_size: u64,
        data: &[T],
        usage_flags: BufferUsageFlags,
        command_pool: CommandPool,
        command_buffer_count: u32,
    ) -> (Buffer, DeviceMemory)
    where
        T: std::marker::Copy,
    {
        let (staging_buffer, staging_buffer_memory) = Texture::create_buffer(
            instance,
            devices,
            buffer_size,
            vk::BufferUsageFlags::TRANSFER_SRC,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        );

        unsafe {
            Vulkan::map_memory(&devices.logical, staging_buffer_memory, buffer_size, data);
        }

        let (buffer, buffer_memory) = Texture::create_buffer(
            instance,
            devices,
            buffer_size,
            usage_flags,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );

        Self::copy_buffer(
            devices,
            command_pool,
            command_buffer_count,
            buffer_size,
            staging_buffer,
            buffer,
        );

        unsafe {
            devices.logical.destroy_buffer(staging_buffer, None);
            devices.logical.free_memory(staging_buffer_memory, None);
        }

        (buffer, buffer_memory)
    }
}

// struct ShapeBuffers {
//     vertex_buffer: Vec<Buffer>,
//     vertex_buffer_memory: Vec<DeviceMemory>,
//     index_buffer: Vec<Buffer>,
//     index_buffer_memory: Vec<DeviceMemory>,
// }

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
    models: Vec<Shape>,

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

        let shapes = vec![Shape::new(
            &instance,
            &devices,
            include_bytes!("/Users/rob/Downloads/2k_saturn.jpg"),
            command_pool,
            swapchain.swapchain_images.len() as u32,
            ShapeType::CUBE,
            true,
            Some(vk::PrimitiveTopology::TRIANGLE_LIST),
            Some(vk::CullModeFlags::BACK),
            &swapchain,
            msaa_samples,
            render_pass,
        )];

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
        let camera = Camera {
            eye: Point3::new(5., 1., 1.),
        };

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

            camera,

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
                // components: vk::ComponentMapping {
                //     r: vk::ComponentSwizzle::IDENTITY,
                //     g: vk::ComponentSwizzle::IDENTITY,
                //     b: vk::ComponentSwizzle::IDENTITY,
                //     a: vk::ComponentSwizzle::IDENTITY,
                // },
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
        shapes: &[Shape],
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

                for (_j, shape) in shapes.iter().enumerate() {
                    devices.logical.cmd_bind_pipeline(
                        command_buffers[i as usize],
                        vk::PipelineBindPoint::GRAPHICS,
                        shape.graphics_pipeline.pipeline,
                    );

                    devices.logical.cmd_bind_descriptor_sets(
                        command_buffers[i as usize],
                        vk::PipelineBindPoint::GRAPHICS,
                        shape.graphics_pipeline.layout,
                        0,
                        std::slice::from_ref(
                            &shape.graphics_pipeline.descriptor_set.descriptor_sets[i as usize],
                        ),
                        &[],
                    );

                    let vertex_buffers = [shape.vertex_buffer];

                    devices.logical.cmd_bind_vertex_buffers(
                        command_buffers[i as usize],
                        0,
                        &vertex_buffers,
                        &offsets,
                    );

                    devices.logical.cmd_draw(
                        command_buffers[i as usize],
                        shape.vertices.len() as u32,
                        1,
                        0,
                        0,
                    );

                    if shape.indexed {
                        devices.logical.cmd_bind_index_buffer(
                            command_buffers[i as usize],
                            shape.index_buffer,
                            0,
                            vk::IndexType::UINT16,
                        );

                        devices.logical.cmd_draw_indexed(
                            command_buffers[i as usize],
                            shape.indices.len() as u32,
                            1,
                            0,
                            0,
                            0,
                        );
                    }
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
