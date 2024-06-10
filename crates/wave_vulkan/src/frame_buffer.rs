use crate::{renderer::RenderPass, resource::Resources, swap_chain::SwapChain};
use ash::{vk, Device};
use derive_more::{Deref, From};

#[derive(Debug, From, Deref, Clone)]
pub struct FrameBuffers(pub(crate) Vec<vk::Framebuffer>);

pub(crate) fn create_frame_buffers(
    swap_chain: &SwapChain,
    render_pass: &RenderPass,
    device: &Device,
    resources: &Resources,
) -> FrameBuffers {
    let mut frame_buffers = Vec::new();

    for i in 0..swap_chain.images.len() {
        let attachments = &[
            resources.colour.view,
            resources.depth.view,
            swap_chain.image_views[i],
        ];

        let vk::Extent2D { width, height } = swap_chain.extent;

        let frame_buffer = create_frame_buffer(render_pass, attachments, device, width, height);

        frame_buffers.push(frame_buffer);
    }

    frame_buffers.into()
}

pub(crate) fn create_frame_buffer<const N: usize>(
    render_pass: &RenderPass,
    attachments: &[vk::ImageView; N],
    device: &Device,
    width: u32,
    height: u32,
) -> vk::Framebuffer {
    let frame_buffer_info = vk::FramebufferCreateInfo::default()
        .render_pass(render_pass.0)
        .attachments(attachments)
        .width(width)
        .height(height)
        .layers(1);
    unsafe {
        device
            .create_framebuffer(&frame_buffer_info, None)
            .expect("Failed to create Frame Buffer!")
    }
}
