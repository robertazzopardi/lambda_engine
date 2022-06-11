use crate::{resource::Resources, swap_chain::SwapChain, utility::InstanceDevices, RenderPass};
use ash::vk;
use derive_more::{Deref, From};

#[derive(new, Debug, From, Deref)]
pub struct FrameBuffers(pub(crate) Vec<vk::Framebuffer>);

pub fn create_frame_buffers(
    swap_chain: &SwapChain,
    render_pass: &RenderPass,
    instance_devices: &InstanceDevices,
    resources: &Resources,
) -> FrameBuffers {
    let mut frame_buffers = Vec::new();

    for i in 0..swap_chain.images.len() {
        let attachments = &[
            resources.colour.view,
            resources.depth.view,
            swap_chain.image_views[i],
        ];

        let frame_buffer_info = vk::FramebufferCreateInfo::builder()
            .render_pass(render_pass.0)
            .attachments(attachments)
            .width(swap_chain.extent.width)
            .height(swap_chain.extent.height)
            .layers(1);

        frame_buffers.push(unsafe {
            instance_devices
                .devices
                .logical
                .device
                .create_framebuffer(&frame_buffer_info, None)
                .expect("Failed to create Frame Buffer!")
        });
    }

    frame_buffers.into()
}
