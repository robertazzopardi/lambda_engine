use crate::{
    device, frame_buffer::FrameBuffers, object::InternalObject, swap_chain::SwapChain,
    utility::InstanceDevices, Devices,
};
use ash::{extensions::khr::Surface, vk};
use derive_more::{Deref, From};
use std::ptr;

#[derive(new, Debug, From, Deref)]
pub struct CommandBuffers(Vec<vk::CommandBuffer>);

#[derive(new, Debug)]
pub(crate) struct VkCommander {
    pub buffers: CommandBuffers,
    pub pool: vk::CommandPool,
}

pub(crate) fn create_command_pool(
    instance_devices: &InstanceDevices,
    surface_loader: &Surface,
    surface: &vk::SurfaceKHR,
) -> vk::CommandPool {
    let InstanceDevices { devices, instance } = instance_devices;

    let queue_family_indices =
        device::find_queue_family(instance, devices.physical.device, surface_loader, surface);

    let pool_info = vk::CommandPoolCreateInfo::builder()
        .queue_family_index(queue_family_indices.graphics_family.unwrap());

    unsafe {
        devices
            .logical
            .device
            .create_command_pool(&pool_info, None)
            .expect("Failed to create command pool!")
    }
}

pub(crate) fn create_command_buffers(
    command_pool: vk::CommandPool,
    swap_chain: &SwapChain,
    devices: &Devices,
    render_pass: vk::RenderPass,
    frame_buffers: &FrameBuffers,
    models: &[Box<dyn InternalObject>],
) -> CommandBuffers {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(command_pool)
        .command_buffer_count(swap_chain.images.len() as u32)
        .level(vk::CommandBufferLevel::PRIMARY);

    let command_buffers = unsafe {
        devices
            .logical
            .device
            .allocate_command_buffers(&command_buffer_allocate_info)
            .expect("Failed to allocate command render buffers")
    };
    let view_port = vk::Viewport::builder()
        .x(0.)
        .y(0.)
        .width(swap_chain.extent.width as f32)
        .height(swap_chain.extent.height as f32)
        .min_depth(0.)
        .max_depth(1.);

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(vk::Extent2D {
            width: swap_chain.extent.width,
            height: swap_chain.extent.height,
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
        for i in 0..swap_chain.images.len() {
            devices
                .logical
                .device
                .begin_command_buffer(command_buffers[i as usize], &begin_info)
                .expect("Failed to begin recording command buffer!");

            let render_pass_begin_info = vk::RenderPassBeginInfo {
                s_type: vk::StructureType::RENDER_PASS_BEGIN_INFO,
                p_next: ptr::null(),
                render_pass,
                framebuffer: frame_buffers[i],
                render_area: vk::Rect2D {
                    offset: vk::Offset2D { x: 0, y: 0 },
                    extent: swap_chain.extent,
                },
                clear_value_count: clear_values.len() as u32,
                p_clear_values: clear_values.as_ptr(),
            };

            devices.logical.device.cmd_begin_render_pass(
                command_buffers[i as usize],
                &render_pass_begin_info,
                vk::SubpassContents::INLINE,
            );

            devices.logical.device.cmd_set_viewport(
                command_buffers[i as usize],
                0,
                std::slice::from_ref(&view_port),
            );

            devices.logical.device.cmd_set_scissor(
                command_buffers[i as usize],
                0,
                std::slice::from_ref(&scissor),
            );

            models.iter().for_each(|model| {
                model.bind_index_and_vertex_buffers(devices, command_buffers[i], &offsets, i);
            });

            devices
                .logical
                .device
                .cmd_end_render_pass(command_buffers[i as usize]);

            devices
                .logical
                .device
                .end_command_buffer(command_buffers[i as usize])
                .expect("Failed to record command buffer!");
        }
    }

    command_buffers.into()
}

pub fn begin_single_time_command(
    device: &ash::Device,
    command_pool: vk::CommandPool,
) -> vk::CommandBuffer {
    let command_buffer_allocate_info = vk::CommandBufferAllocateInfo {
        s_type: vk::StructureType::COMMAND_BUFFER_ALLOCATE_INFO,
        p_next: std::ptr::null(),
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
        p_next: std::ptr::null(),
        p_inheritance_info: std::ptr::null(),
        flags: vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT,
    };

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
    pool: vk::CommandPool,
    buffer: vk::CommandBuffer,
) {
    unsafe {
        device
            .end_command_buffer(buffer)
            .expect("Failed to record Command Buffer at Ending!");
    }

    let buffers_to_submit = [buffer];

    let submit_infos = [vk::SubmitInfo {
        command_buffer_count: 1,
        p_command_buffers: buffers_to_submit.as_ptr(),
        p_next: std::ptr::null(),
        p_signal_semaphores: std::ptr::null(),
        p_wait_dst_stage_mask: std::ptr::null(),
        p_wait_semaphores: std::ptr::null(),
        s_type: vk::StructureType::SUBMIT_INFO,
        signal_semaphore_count: 0,
        wait_semaphore_count: 0,
    }];

    unsafe {
        device
            .queue_submit(submit_queue, &submit_infos, vk::Fence::null())
            .expect("Failed to Queue Submit!");
        device
            .queue_wait_idle(submit_queue)
            .expect("Failed to wait Queue idle!");
        device.free_command_buffers(pool, &buffers_to_submit);
    }
}
