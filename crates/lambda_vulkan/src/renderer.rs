use crate::{
    resource,
    swap_chain::{recreate_swap_chain, SwapChain},
    sync_objects::MAX_FRAMES_IN_FLIGHT,
    uniform_buffer::update_uniform_buffers,
    utility::InstanceDevices,
    RenderPass, Vulkan,
};
use ash::vk;
use lambda_camera::prelude::Camera;
use std::ptr;
use winit::window::Window;

pub(crate) fn create_render_pass(
    instance_devices: &InstanceDevices,
    swap_chain: &SwapChain,
) -> RenderPass {
    let InstanceDevices { devices, .. } = instance_devices;

    let render_pass_attachments = [
        vk::AttachmentDescription {
            format: swap_chain.image_format,
            samples: devices.physical.samples,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            ..Default::default()
        },
        vk::AttachmentDescription {
            format: resource::find_depth_format(instance_devices),
            samples: devices.physical.samples,
            load_op: vk::AttachmentLoadOp::CLEAR,
            store_op: vk::AttachmentStoreOp::DONT_CARE,
            stencil_load_op: vk::AttachmentLoadOp::DONT_CARE,
            stencil_store_op: vk::AttachmentStoreOp::DONT_CARE,
            initial_layout: vk::ImageLayout::UNDEFINED,
            final_layout: vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            ..Default::default()
        },
        vk::AttachmentDescription {
            format: swap_chain.image_format,
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

    let sub_passes = vk::SubpassDescription::builder()
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

    let render_pass_create_info = vk::RenderPassCreateInfo::builder()
        .attachments(&render_pass_attachments)
        .subpasses(std::slice::from_ref(&sub_passes))
        .dependencies(std::slice::from_ref(&dependencies));

    RenderPass(unsafe {
        devices
            .logical
            .device
            .create_render_pass(&render_pass_create_info, None)
            .expect("Failed to create render pass!")
    })
}

pub fn render(
    vulkan: &mut Vulkan,
    window: &Window,
    camera: &mut Camera,
    current_frame: &mut usize,
    resized: &mut bool,
    dt: f32,
) {
    unsafe {
        vulkan
            .instance_devices
            .devices
            .logical
            .device
            .wait_for_fences(&vulkan.sync_objects.in_flight_fences, true, std::u64::MAX)
            .expect("Failed to wait for Fence!");

        let (image_index, _is_sub_optimal) = {
            let result = vulkan.swap_chain.loader.acquire_next_image(
                vulkan.swap_chain.swap_chain,
                std::u64::MAX,
                vulkan.sync_objects.image_available_semaphores[*current_frame],
                vk::Fence::null(),
            );
            match result {
                Ok(image_index) => image_index,
                Err(vk_result) => match vk_result {
                    vk::Result::ERROR_OUT_OF_DATE_KHR => {
                        recreate_swap_chain(vulkan, window);
                        return;
                    }
                    _ => panic!("Failed to acquire Swap Chain vk::Image!"),
                },
            }
        };

        update_uniform_buffers(vulkan, camera, image_index.try_into().unwrap(), dt);

        if vulkan.sync_objects.images_in_flight[image_index as usize] != vk::Fence::null() {
            vulkan
                .instance_devices
                .devices
                .logical
                .device
                .wait_for_fences(
                    &[vulkan.sync_objects.images_in_flight[image_index as usize]],
                    true,
                    std::u64::MAX,
                )
                .expect("Could not wait for images in flight");
        }
        vulkan.sync_objects.images_in_flight[image_index as usize] =
            vulkan.sync_objects.in_flight_fences[*current_frame];

        let wait_semaphores = &[vulkan.sync_objects.image_available_semaphores[*current_frame]];
        let signal_semaphores = [vulkan.sync_objects.render_finished_semaphores[*current_frame]];
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];

        let submit_infos = [vk::SubmitInfo {
            s_type: vk::StructureType::SUBMIT_INFO,
            p_next: ptr::null(),
            wait_semaphore_count: wait_semaphores.len() as u32,
            p_wait_semaphores: wait_semaphores.as_ptr(),
            p_wait_dst_stage_mask: wait_stages.as_ptr(),
            command_buffer_count: 1,
            p_command_buffers: &vulkan.commander.buffers[image_index as usize],
            signal_semaphore_count: signal_semaphores.len() as u32,
            p_signal_semaphores: signal_semaphores.as_ptr(),
        }];

        vulkan
            .instance_devices
            .devices
            .logical
            .device
            .reset_fences(&[vulkan.sync_objects.in_flight_fences[*current_frame]])
            .expect("Failed to reset Fence!");

        vulkan
            .instance_devices
            .devices
            .logical
            .device
            .queue_submit(
                vulkan.instance_devices.devices.logical.queues.present,
                &submit_infos,
                vulkan.sync_objects.in_flight_fences[*current_frame],
            )
            .expect("Failed to execute queue submit.");

        let present_info = vk::PresentInfoKHR {
            s_type: vk::StructureType::PRESENT_INFO_KHR,
            p_next: ptr::null(),
            wait_semaphore_count: 1,
            p_wait_semaphores: signal_semaphores.as_ptr(),
            swapchain_count: 1,
            p_swapchains: &vulkan.swap_chain.swap_chain,
            p_image_indices: &image_index,
            p_results: ptr::null_mut(),
        };

        let result = vulkan.swap_chain.loader.queue_present(
            vulkan.instance_devices.devices.logical.queues.present,
            &present_info,
        );

        let is_resized = match result {
            Ok(_) => *resized,
            Err(vk_result) => match vk_result {
                vk::Result::ERROR_OUT_OF_DATE_KHR | vk::Result::SUBOPTIMAL_KHR => true,
                _ => panic!("Failed to execute queue present."),
            },
        };

        if is_resized {
            *resized = false;
            recreate_swap_chain(vulkan, window);
        }

        *current_frame = (*current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
    }
}
