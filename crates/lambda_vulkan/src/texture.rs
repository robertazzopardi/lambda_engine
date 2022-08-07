use crate::{
    buffer::Buffer,
    command_buffer,
    device::Devices,
    memory,
    utility::{self, Image, ImageInfo, InstanceDevices},
};
use ash::vk;
use nalgebra::Point2;
use std::cmp;

#[derive(Default, Debug, Clone)]
pub struct Texture {
    pub image: Image,
    pub view: vk::ImageView,
    pub sampler: vk::Sampler,
}

impl Texture {
    pub fn new(
        image_properties: ImageProperties,
        command_pool: &vk::CommandPool,
        instance_devices: &InstanceDevices,
        format: vk::Format,
        image_info: ImageInfo,
    ) -> Self {
        let image = create_texture_image(
            image_properties,
            command_pool,
            instance_devices,
            format,
            image_info,
        );
        let view = utility::create_image_view(
            &image,
            format,
            vk::ImageAspectFlags::COLOR,
            &instance_devices.devices.logical.device,
        );
        let sampler = create_texture_sampler(image.mip_levels, instance_devices);
        Self {
            image,
            view,
            sampler,
        }
    }
}

fn create_texture_sampler(
    mip_levels: u32,
    InstanceDevices { instance, devices }: &InstanceDevices,
) -> vk::Sampler {
    unsafe {
        let properties = instance.get_physical_device_properties(devices.physical.device);

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
            .device
            .create_sampler(&sampler_create_info, None)
            .expect("Failed to create Sampler!")
    }
}

fn create_texture_image(
    image_properties: ImageProperties,
    command_pool: &vk::CommandPool,
    instance_devices: &InstanceDevices,
    format: vk::Format,
    image_info: ImageInfo,
) -> Image {
    let ImageProperties {
        image_dimensions,
        image_data,
        mip_levels,
        size,
    } = image_properties;

    let Buffer { buffer, memory } = create_buffer(
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        instance_devices,
    );

    let InstanceDevices { devices, .. } = instance_devices;

    let device = &instance_devices.devices.logical.device;

    memory::map_memory(device, memory, size, image_data.as_slice());

    let image = utility::create_image(image_info, instance_devices);

    transition_image_layout(
        device,
        command_pool,
        devices.logical.queues.graphics,
        image.image,
        Point2::new(
            vk::ImageLayout::UNDEFINED,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        ),
        mip_levels,
    );

    copy_buffer_to_image(devices, command_pool, image_dimensions, buffer, image.image);

    unsafe {
        device.destroy_buffer(buffer, None);
        device.free_memory(memory, None);
    }

    generate_mip_maps(
        format,
        image.image,
        command_pool,
        Point2::new(
            image_dimensions.0.try_into().unwrap(),
            image_dimensions.1.try_into().unwrap(),
        ),
        mip_levels,
        instance_devices,
    );

    image.mip_levels(mip_levels)
}

#[derive(Clone, Debug, new)]
pub struct ImageProperties {
    pub image_dimensions: (u32, u32),
    pub image_data: Vec<u8>,
    pub mip_levels: u32,
    pub size: u64,
}

impl ImageProperties {
    pub fn get_image_properties_from_buffer(image_buffer: &[u8]) -> Self {
        let image_texture = image::load_from_memory(image_buffer).unwrap().to_rgba8();
        let image_dimensions = image_texture.dimensions();
        let image_data = image_texture.into_raw();
        let mip_levels = ((image_dimensions.0.max(image_dimensions.1) as f32)
            .log2()
            .floor()
            + 1.) as u32;
        let size = (std::mem::size_of::<u8>() as u32 * image_dimensions.0 * image_dimensions.1 * 4)
            as vk::DeviceSize;
        Self::new(image_dimensions, image_data, mip_levels, size)
    }
}

pub(crate) fn create_buffer(
    size: vk::DeviceSize,
    usage: vk::BufferUsageFlags,
    properties: vk::MemoryPropertyFlags,
    instance_devices: &InstanceDevices,
) -> Buffer {
    let device = &instance_devices.devices.logical.device;

    let image_buffer_info = vk::BufferCreateInfo::builder()
        .size(size)
        .usage(usage)
        .sharing_mode(vk::SharingMode::EXCLUSIVE);

    unsafe {
        let buffer = device
            .create_buffer(&image_buffer_info, None)
            .expect("Failed to create buffer");

        let memory_requirements = device.get_buffer_memory_requirements(buffer);

        let memory_type_index = memory::find_memory_type(
            memory_requirements.memory_type_bits,
            properties,
            instance_devices,
        );

        let image_buffer_allocate_info = vk::MemoryAllocateInfo::builder()
            .allocation_size(memory_requirements.size)
            .memory_type_index(memory_type_index);

        let buffer_memory = device
            .allocate_memory(&image_buffer_allocate_info, None)
            .expect("Failed to allocate buffer memory!");

        device
            .bind_buffer_memory(buffer, buffer_memory, 0)
            .expect("Could not bind command buffer memory");

        Buffer::new(buffer, buffer_memory)
    }
}

fn transition_image_layout(
    device: &ash::Device,
    command_pool: &vk::CommandPool,
    submit_queue: vk::Queue,
    image: vk::Image,
    layouts: Point2<vk::ImageLayout>,
    mip_levels: u32,
) {
    let command_buffer = command_buffer::begin_single_time_command(device, command_pool);

    let src_access_mask;
    let dst_access_mask;
    let source_stage;
    let destination_stage;

    let x = layouts.coords.data.0[0][0];
    let y = layouts.coords.data.0[0][1];

    if x == vk::ImageLayout::UNDEFINED && y == vk::ImageLayout::TRANSFER_DST_OPTIMAL {
        src_access_mask = vk::AccessFlags::empty();
        dst_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        source_stage = vk::PipelineStageFlags::TOP_OF_PIPE;
        destination_stage = vk::PipelineStageFlags::TRANSFER;
    } else if x == vk::ImageLayout::TRANSFER_DST_OPTIMAL
        && y == vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL
    {
        src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        dst_access_mask = vk::AccessFlags::SHADER_READ;
        source_stage = vk::PipelineStageFlags::TRANSFER;
        destination_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
    } else {
        panic!("Unsupported layout transition!")
    }

    let subresource_range = vk::ImageSubresourceRange::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .base_mip_level(0)
        .level_count(mip_levels)
        .base_array_layer(0)
        .layer_count(1);

    let image_barriers = vk::ImageMemoryBarrier::builder()
        .src_access_mask(src_access_mask)
        .dst_access_mask(dst_access_mask)
        .old_layout(x)
        .new_layout(y)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .image(image)
        .subresource_range(*subresource_range);

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            source_stage,
            destination_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&image_barriers),
        );
    }

    command_buffer::end_single_time_command(device, submit_queue, command_pool, command_buffer);
}

fn copy_buffer_to_image(
    devices: &Devices,
    command_pool: &vk::CommandPool,
    image_dimensions: (u32, u32),
    src_buffer: vk::Buffer,
    dst_image: vk::Image,
) {
    let command_buffer =
        command_buffer::begin_single_time_command(&devices.logical.device, command_pool);

    let image_sub_resource = vk::ImageSubresourceLayers::builder()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .mip_level(0)
        .base_array_layer(0)
        .layer_count(1);

    let (width, height) = image_dimensions;

    let region = vk::BufferImageCopy::builder()
        .buffer_offset(0)
        .buffer_row_length(0)
        .buffer_image_height(0)
        .image_subresource(*image_sub_resource)
        .image_offset(vk::Offset3D::default())
        .image_extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        });

    unsafe {
        devices.logical.device.cmd_copy_buffer_to_image(
            command_buffer,
            src_buffer,
            dst_image,
            vk::ImageLayout::TRANSFER_DST_OPTIMAL,
            std::slice::from_ref(&region),
        )
    }

    command_buffer::end_single_time_command(
        &devices.logical.device,
        devices.logical.queues.graphics,
        command_pool,
        command_buffer,
    );
}

fn generate_mip_maps(
    format: vk::Format,
    image: vk::Image,
    command_pool: &vk::CommandPool,
    mip_dimension: Point2<i32>,
    mip_levels: u32,
    InstanceDevices { instance, devices }: &InstanceDevices,
) {
    let device = &devices.logical.device;

    let format_properties =
        unsafe { instance.get_physical_device_format_properties(devices.physical.device, format) };

    if format_properties.optimal_tiling_features
        & vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR
        != vk::FormatFeatureFlags::SAMPLED_IMAGE_FILTER_LINEAR
    {
        panic!("Texture image format does not support linear bilitting!");
    }

    let command_buffer = command_buffer::begin_single_time_command(device, command_pool);

    let mut image_barrier = vk::ImageMemoryBarrier::builder()
        .image(image)
        .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        .subresource_range(
            vk::ImageSubresourceRange::builder()
                .aspect_mask(vk::ImageAspectFlags::COLOR)
                .base_mip_level(mip_levels)
                .level_count(1)
                .base_array_layer(0)
                .layer_count(1)
                .build(),
        );

    let mut x = mip_dimension.coords.data.0[0][0];
    let mut y = mip_dimension.coords.data.0[0][1];

    for i in 1..mip_levels {
        image_barrier.subresource_range.base_mip_level = i - 1;
        image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        image_barrier.new_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        image_barrier.dst_access_mask = vk::AccessFlags::TRANSFER_READ;

        let blits = vk::ImageBlit::builder()
            .src_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(i - 1)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .src_offsets([
                vk::Offset3D::default(),
                vk::Offset3D::builder().x(x).y(y).z(1).build(),
            ])
            .dst_subresource(
                vk::ImageSubresourceLayers::builder()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .mip_level(i)
                    .base_array_layer(0)
                    .layer_count(1)
                    .build(),
            )
            .dst_offsets([
                vk::Offset3D::default(),
                vk::Offset3D::builder()
                    .x(cmp::max(x / 2, 1))
                    .y(cmp::max(y / 2, 1))
                    .z(1)
                    .build(),
            ]);

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&image_barrier),
            );

            device.cmd_blit_image(
                command_buffer,
                image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                image,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(&blits),
                vk::Filter::LINEAR,
            );
        }

        image_barrier.old_layout = vk::ImageLayout::TRANSFER_SRC_OPTIMAL;
        image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_READ;
        image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

        unsafe {
            device.cmd_pipeline_barrier(
                command_buffer,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                std::slice::from_ref(&image_barrier),
            );
        }

        x = cmp::max(x / 2, 1);
        y = cmp::max(y / 2, 1);
    }

    image_barrier.subresource_range.base_mip_level = mip_levels - 1;
    image_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
    image_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
    image_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
    image_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;

    unsafe {
        device.cmd_pipeline_barrier(
            command_buffer,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            std::slice::from_ref(&image_barrier),
        );
    }

    command_buffer::end_single_time_command(
        device,
        devices.logical.queues.graphics,
        command_pool,
        command_buffer,
    );
}
