use crate::{
    resource,
    swap_chain::{recreate_swap_chain, SwapChain},
    sync_objects::MAX_FRAMES_IN_FLIGHT,
    uniform_buffer::update_uniform_buffers,
    utility::InstanceDevices,
    Vulkan,
};
use ash::vk;
use winit::window::Window;

#[derive(Default, Debug, Clone, new)]
pub(crate) struct RenderPass(pub vk::RenderPass);

pub trait Renderer {
    fn render(&mut self, window: &Window);
}

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
            store_op: vk::AttachmentStoreOp::STORE,
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
            store_op: vk::AttachmentStoreOp::STORE,
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

    let sub_passes = vk::SubpassDescription::default()
        .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
        .color_attachments(std::slice::from_ref(&color_attachment_refs))
        .depth_stencil_attachment(&depth_attachment_ref)
        .resolve_attachments(std::slice::from_ref(&color_attachment_resolver_ref));

    let dependencies = vk::SubpassDependency::default()
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

    let render_pass_create_info = vk::RenderPassCreateInfo::default()
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
