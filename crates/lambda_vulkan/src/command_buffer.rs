use crate::{
    any_as_u8_slice, device, frame_buffer::FrameBuffers, renderer::RenderPass,
    swap_chain::SwapChain, utility::InstanceDevices, Shader, VulkanObject,
};
use ash::{extensions::khr::Surface, vk, Device};
use derive_more::{Deref, From};

#[derive(new, Debug, From, Deref, Clone)]
pub struct CommandBuffers(Vec<vk::CommandBuffer>);

#[derive(new, Debug, From, Deref, Clone)]
pub struct CommandPool(vk::CommandPool);

pub fn create_command_pool(
    instance_devices: &InstanceDevices,
    surface_loader: &Surface,
    surface: &vk::SurfaceKHR,
) -> CommandPool {
    let InstanceDevices { devices, instance } = instance_devices;

    let queue_family_indices =
        device::find_queue_family(instance, devices.physical.device, surface_loader, surface);

    let pool_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(queue_family_indices.graphics_family.unwrap())
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);

    CommandPool(unsafe {
        devices
            .logical
            .device
            .create_command_pool(&pool_info, None)
            .expect("Failed to create command pool!")
    })
}

pub(crate) fn create_command_buffers(
    command_pool: &CommandPool,
    swap_chain: &SwapChain,
    instance_devices: &InstanceDevices,
    render_pass: &RenderPass,
    frame_buffers: &FrameBuffers,
    objects: &[VulkanObject],
) -> CommandBuffers {
    let device = &instance_devices.devices.logical.device;

    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(**command_pool)
        .command_buffer_count(swap_chain.images.len() as u32)
        .level(vk::CommandBufferLevel::PRIMARY);

    let command_buffers = unsafe {
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate command render buffers")
    };

    let vk::Extent2D { width, height } = swap_chain.extent;

    let view_port = vk::Viewport::builder()
        .x(0.)
        .y(0.)
        .width(width as f32)
        .height(height as f32)
        .min_depth(0.)
        .max_depth(1.);

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D::default())
        .extent(swap_chain.extent);

    let begin_info = vk::CommandBufferBeginInfo::builder();

    let clear_values = [
        vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0., 0., 0., 1.],
            },
        },
        vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue::builder()
                .depth(1.)
                .stencil(0)
                .build(),
        },
    ];

    unsafe {
        for i in 0..swap_chain.images.len() {
            device
                .begin_command_buffer(command_buffers[i], &begin_info)
                .expect("Failed to begin recording command buffer!");

            let render_pass_begin_info = vk::RenderPassBeginInfo::builder()
                .render_pass(render_pass.0)
                .framebuffer(frame_buffers[i])
                .render_area(*scissor)
                .clear_values(&clear_values);

            device.cmd_begin_render_pass(
                command_buffers[i],
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            device.cmd_set_viewport(command_buffers[i], 0, std::slice::from_ref(&view_port));

            device.cmd_set_scissor(command_buffers[i], 0, std::slice::from_ref(&scissor));

            objects.iter().for_each(|object| {
                if object.shader == Shader::PushConstant || object.shader == Shader::Ui {
                    let push = any_as_u8_slice(&object.model);
                    device.cmd_push_constants(
                        command_buffers[i],
                        object.graphics_pipeline.features.layout,
                        vk::ShaderStageFlags::VERTEX,
                        0,
                        push,
                    )
                }
                bind_index_and_vertex_buffers(object, device, command_buffers[i], &[0_u64], i)
            });

            device.cmd_end_render_pass(command_buffers[i]);

            device
                .end_command_buffer(command_buffers[i])
                .expect("Failed to record command buffer!");
        }
    }

    command_buffers.into()
}

pub fn begin_single_time_command(
    device: &ash::Device,
    command_pool: &vk::CommandPool,
) -> vk::CommandBuffer {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_buffer_count(1)
        .command_pool(*command_pool)
        .level(vk::CommandBufferLevel::PRIMARY);

    let command_buffer_begin_info =
        vk::CommandBufferBeginInfo::builder().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

    let command_buffer = unsafe {
        device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate Command Buffers!")
    }[0];

    unsafe {
        device
            .begin_command_buffer(command_buffer, &command_buffer_begin_info)
            .expect("Failed to begin recording Command Buffer at beginning!");
    }

    command_buffer
}

pub fn end_single_time_command(
    device: &ash::Device,
    submit_queue: vk::Queue,
    pool: &vk::CommandPool,
    buffer: vk::CommandBuffer,
) {
    let buffers_to_submit = [buffer];

    let submit_info = vk::SubmitInfo::builder().command_buffers(&buffers_to_submit);

    unsafe {
        device
            .end_command_buffer(buffer)
            .expect("Failed to record Command Buffer at Ending!");
        device
            .queue_submit(
                submit_queue,
                std::slice::from_ref(&submit_info),
                vk::Fence::null(),
            )
            .expect("Failed to Queue Submit!");
        device
            .queue_wait_idle(submit_queue)
            .expect("Failed to wait Queue idle!");
        device.free_command_buffers(*pool, &buffers_to_submit);
    }
}

/// # Safety
///
/// Expand on safety of this function
pub(crate) unsafe fn bind_index_and_vertex_buffers(
    object: &VulkanObject,
    device: &Device,
    command_buffer: vk::CommandBuffer,
    offsets: &[vk::DeviceSize],
    index: usize,
) {
    device.cmd_bind_pipeline(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        object.graphics_pipeline.features.pipeline,
    );

    let descriptor_sets = if object.shader == Shader::Ui || object.shader == Shader::PushConstant {
        std::slice::from_ref(&object.graphics_pipeline.descriptors.sets[0])
    } else {
        std::slice::from_ref(&object.graphics_pipeline.descriptors.sets[index])
    };
    device.cmd_bind_descriptor_sets(
        command_buffer,
        vk::PipelineBindPoint::GRAPHICS,
        object.graphics_pipeline.features.layout,
        0,
        descriptor_sets,
        &[],
    );

    device.cmd_bind_vertex_buffers(
        command_buffer,
        0,
        std::slice::from_ref(&object.buffers.vertex.buffer),
        offsets,
    );

    if object.indexed {
        device.cmd_bind_index_buffer(
            command_buffer,
            object.buffers.index.buffer,
            0,
            vk::IndexType::UINT16,
        );

        device.cmd_draw_indexed(
            command_buffer,
            object.vertices_and_indices.indices.len() as u32,
            1,
            0,
            0,
            0,
        );
    } else {
        device.cmd_draw(
            command_buffer,
            object.vertices_and_indices.vertices.len() as u32,
            1,
            0,
            0,
        );
    }
}
