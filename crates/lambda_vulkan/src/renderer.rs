use crate::{
    resource,
    swap_chain::{recreate_swap_chain, SwapChain},
    sync_objects::MAX_FRAMES_IN_FLIGHT,
    uniform_buffer::update_uniform_buffers,
    utility::InstanceDevices,
    Vulkan,
};
use ash::vk;
use lambda_camera::prelude::CameraInternal;
use winit::window::Window;

#[derive(Default, Debug, Clone, new)]
pub(crate) struct RenderPass(pub vk::RenderPass);

pub(crate) fn create_render_pass(
    instance_devices: &InstanceDevices,
    swap_chain: &SwapChain,
) -> RenderPass {
    let InstanceDevices { devices, .. } = instance_devices;

    let render_pass_attachments = [
        vk::AttachmentDescription::builder()
            .format(swap_chain.image_format)
            .samples(devices.physical.samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)
            .build(),
        vk::AttachmentDescription::builder()
            .format(resource::find_depth_format(instance_devices))
            .samples(devices.physical.samples)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL)
            .build(),
        vk::AttachmentDescription::builder()
            .format(swap_chain.image_format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::DONT_CARE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR)
            .build(),
    ];

    let color_attachment_refs = vk::AttachmentReference::builder()
        .attachment(0)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
    let depth_attachment_ref = vk::AttachmentReference::builder()
        .attachment(1)
        .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
    let color_attachment_resolver_ref = vk::AttachmentReference::builder()
        .attachment(2)
        .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);

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
    camera: &mut CameraInternal,
    current_frame: &mut usize,
    resized: &mut bool,
    dt: f32,
) {
    let Vulkan {
        command_buffers,
        swap_chain,
        sync_objects,
        ubo,
        instance_devices,
        objects,
        ..
    } = vulkan;

    let device = &mut instance_devices.devices.logical.device;

    let image_available_semaphore = sync_objects.image_available_semaphores[*current_frame];
    let in_flight_fence = sync_objects.in_flight_fences[*current_frame];
    let render_finished_semaphore = sync_objects.render_finished_semaphores[*current_frame];

    unsafe {
        device
            .wait_for_fences(&sync_objects.in_flight_fences, true, vk::DeviceSize::MAX)
            .expect("Failed to wait for Fence!");

        let (image_index, _is_sub_optimal) = {
            let result = swap_chain.swap_chain.acquire_next_image(
                swap_chain.swap_chain_khr,
                vk::DeviceSize::MAX,
                image_available_semaphore,
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

        update_uniform_buffers(
            device,
            objects,
            ubo,
            camera,
            image_index.try_into().unwrap(),
            dt,
        );

        if sync_objects.images_in_flight[image_index as usize] != vk::Fence::null() {
            device
                .wait_for_fences(
                    std::slice::from_ref(&sync_objects.images_in_flight[image_index as usize]),
                    true,
                    vk::DeviceSize::MAX,
                )
                .expect("Could not wait for images in flight");
        }

        sync_objects.images_in_flight[image_index as usize] = in_flight_fence;

        let submit_infos = vk::SubmitInfo::builder()
            .wait_semaphores(std::slice::from_ref(&image_available_semaphore))
            .wait_dst_stage_mask(std::slice::from_ref(
                &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            ))
            .command_buffers(std::slice::from_ref(&command_buffers[image_index as usize]))
            .signal_semaphores(std::slice::from_ref(&render_finished_semaphore));

        device
            .reset_fences(std::slice::from_ref(&in_flight_fence))
            .expect("Failed to reset Fence!");

        device
            .queue_submit(
                instance_devices.devices.logical.queues.present,
                std::slice::from_ref(&submit_infos),
                in_flight_fence,
            )
            .expect("Failed to execute queue submit.");

        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(std::slice::from_ref(&render_finished_semaphore))
            .swapchains(std::slice::from_ref(&swap_chain.swap_chain_khr))
            .image_indices(std::slice::from_ref(&image_index));

        let result = swap_chain.swap_chain.queue_present(
            instance_devices.devices.logical.queues.present,
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
