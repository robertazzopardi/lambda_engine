use crate::types::Tuple;
use crate::{command, memory, utility, Devices};
use ash::{vk, Instance};
use std::cmp::max;

pub struct Texture {
    pub image: vk::Image,
    pub memory: vk::DeviceMemory,
    pub image_view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl Texture {
    pub fn new(
        instance: &Instance,
        devices: &Devices,
        image_buffer: &[u8],
        command_pool: vk::CommandPool,
        command_buffer_count: u32,
    ) -> Self {
        let (image, memory, mip_levels) = create_texture_image(
            instance,
            devices,
            image_buffer,
            command_pool,
            command_buffer_count,
        );
        let image_view = create_texture_image_view(devices, image, mip_levels);
        let sampler = create_texture_sampler(instance, devices, mip_levels);

        Self {
            image,
            memory,
            image_view,
            sampler,
        }
    }
}

fn create_texture_image_view(
    devices: &Devices,
    image: vk::Image,
    mip_levels: u32,
) -> vk::ImageView {
    utility::create_image_view(
        image,
        vk::Format::R8G8B8A8_SRGB,
        vk::ImageAspectFlags::COLOR,
        mip_levels,
        devices,
    )
}

fn create_texture_sampler(instance: &Instance, devices: &Devices, mip_levels: u32) -> vk::Sampler {
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

fn create_texture_image(
    instance: &Instance,
    devices: &Devices,
    image_buffer: &[u8],
    command_pool: vk::CommandPool,
    command_buffer_count: u32,
) -> (vk::Image, vk::DeviceMemory, u32) {
    let image_texture = image::load_from_memory(image_buffer)
        .unwrap()
        // .adjust_contrast(-25.)
        .to_rgba8();

    let image_dimensions = image_texture.dimensions();
    let image_data = image_texture.into_raw();

    let mip_levels = ((image_dimensions.0.max(image_dimensions.1) as f32)
        .log2()
        .floor()
        + 1.) as u32;

    let size = (std::mem::size_of::<u8>() as u32 * image_dimensions.0 * image_dimensions.1 * 4)
        as vk::DeviceSize;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        devices,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    );

    unsafe {
        memory::map_memory(
            &devices.logical,
            staging_buffer_memory,
            size,
            image_data.as_slice(),
        );

        let (image, memory) = utility::create_image(
            Tuple(image_dimensions.0, image_dimensions.1),
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

        transition_image_layout(
            &devices.logical,
            command_pool,
            devices.graphics_queue,
            image,
            vk::Format::R8G8B8A8_SRGB,
            Tuple(
                vk::ImageLayout::UNDEFINED,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            ),
            mip_levels,
        );

        copy_buffer_to_image(
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

        generate_mipmaps(
            instance,
            devices,
            vk::Format::R8G8B8A8_SRGB,
            image,
            command_pool,
            Tuple(
                image_dimensions.0.try_into().unwrap(),
                image_dimensions.1.try_into().unwrap(),
            ),
            mip_levels,
        );

        (image, memory, mip_levels)
    }
}

pub fn create_buffer(
    instance: &Instance,
    devices: &Devices,
    size: u64,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
) -> (vk::Buffer, vk::DeviceMemory) {
    let image_buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    unsafe {
        let buffer = devices
            .logical
            .create_buffer(&image_buffer_info, None)
            .expect("Failed to create buffer");

        let memory_requirements = devices.logical.get_buffer_memory_requirements(buffer);

        let memory_type_index = memory::find_memory_type(
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

fn transition_image_layout(
    device: &ash::Device,
    command_pool: vk::CommandPool,
    submit_queue: vk::Queue,
    image: vk::Image,
    _format: vk::Format,
    Tuple(old_layout, new_layout): Tuple<vk::ImageLayout, vk::ImageLayout>,
    mip_levels: u32,
) {
    let command_buffer = command::begin_single_time_command(device, command_pool);

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
        p_next: std::ptr::null(),
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

    command::end_single_time_command(device, command_pool, submit_queue, command_buffer);
}

fn copy_buffer_to_image(
    devices: &Devices,
    command_pool: vk::CommandPool,
    _command_buffer_count: u32,
    width: u32,
    height: u32,
    src_buffer: vk::Buffer,
    dst_image: vk::Image,
) {
    let command_buffer = command::begin_single_time_command(&devices.logical, command_pool);

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

    command::end_single_time_command(
        &devices.logical,
        command_pool,
        devices.graphics_queue,
        command_buffer,
    );
}

fn generate_mipmaps(
    instance: &Instance,
    devices: &Devices,
    format: vk::Format,
    image: vk::Image,
    command_pool: vk::CommandPool,
    mip_dimension: Tuple<i32, i32>,
    mip_levels: u32,
) {
    let format_properties =
        unsafe { instance.get_physical_device_format_properties(devices.physical, format) };
    if format_properties.optimal_tiling_features
        & vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR
        != vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR
    {
        panic!("Texture image format does not support linear bilitting!");
    }

    let command_buffer = command::begin_single_time_command(&devices.logical, command_pool);

    let mut image_barrier = vk::ImageMemoryBarrier::builder()
        .image(image)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            base_mip_level: mip_levels,
            level_count: 1,
            base_array_layer: 0,
            layer_count: 1,
        });

    let Tuple(mut w, mut h) = mip_dimension;

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
                &[*image_barrier],
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
                vk::Offset3D { x: w, y: h, z: 1 },
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
                    x: max(w / 2, 1),
                    y: max(h / 2, 1),
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
                &[*image_barrier],
            );
        }

        w = max(w / 2, 1);
        h = max(h / 2, 1);
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
            &[*image_barrier],
        );
    }

    command::end_single_time_command(
        &devices.logical,
        command_pool,
        devices.graphics_queue,
        command_buffer,
    );
}
