#[macro_use]
extern crate derive_new;

mod buffer;
mod command_buffer;
pub mod debug;
mod device;
mod frame_buffer;
mod graphics_pipeline;
mod memory;
pub mod renderer;
mod resource;
mod swap_chain;
mod sync_objects;
mod texture;
mod uniform_buffer;
mod utility;

use crate::{debug::ENABLE_VALIDATION_LAYERS, sync_objects::MAX_FRAMES_IN_FLIGHT};
use ash::{
    extensions::khr::Surface,
    vk::{self, Extent3D, Queue},
    Device,
};
use buffer::{Buffer, ModelBuffers};
use command_buffer::{CommandBuffers, CommandPool};
use debug::{Debug, Debugger};
use derive_more::{Deref, DerefMut};
use device::Devices;
use frame_buffer::{create_frame_buffer, FrameBuffers};
use graphics_pipeline::{create_shader_stages, destroy_shader_modules, GraphicsPipeline};
use imgui::{Condition, Context, DrawCmd, DrawCmdParams, DrawData, DrawIdx, DrawVert, Ui, Window};
use lambda_camera::prelude::CameraInternal;
use lambda_space::space::VerticesAndIndices;
use lambda_window::prelude::Display;
use nalgebra::{matrix, Matrix4};
use renderer::RenderPass;
use resource::{Resource, Resources};
use swap_chain::SwapChain;
use sync_objects::SyncObjects;
use texture::{create_buffer, ImageProperties, Texture};
use uniform_buffer::UniformBufferObject;
use utility::{EntryInstance, ImageInfo, InstanceDevices};

pub mod prelude {
    pub use crate::{
        debug::{Debugger, MessageLevel, MessageType},
        CullMode, ModelTopology, Shader, TextureBuffer,
    };
}

pub struct GuiVk {
    vertex_buffer: Option<Buffer>,
    index_buffer: Option<Buffer>,
    vertex_count: u32,
    index_count: u32,
    // render_pass: RenderPass,
    // font_memory: vk::DeviceMemory,
    // frame_buffer: vk::Framebuffer,
    // resource: Resource,
    texture: Texture,
    pipeline_cache: vk::PipelineCache,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set_layout: vk::DescriptorSetLayout,
    descriptor_set: vk::DescriptorSet,
    // command_buffer: vk::CommandBuffer,
}

pub struct ImGui {
    pub context: Context,
    gui_vk: GuiVk,
}

impl ImGui {
    fn new(
        instance_devices: &InstanceDevices,
        command_pool: &vk::CommandPool,
        copy_queue: &Queue,
        render_pass: &RenderPass,
    ) -> Self {
        let mut context = Context::create();
        context.set_ini_filename(None);

        let style = context.style_mut();
        style.use_classic_colors();
        let io = context.io_mut();
        io.display_size = [300., 100.];
        io.display_framebuffer_scale = [1., 1.];

        let device = &instance_devices.devices.logical.device;

        // let render_pass = renderer::create_gui_render_pass(device);

        let width;
        let height;
        let mut data: Vec<u8> = vec![];
        {
            let mut font_atlas = context.fonts();
            let font_atlas_texture = font_atlas.build_rgba32_texture();
            width = font_atlas_texture.width;
            height = font_atlas_texture.height;
            data.extend(font_atlas_texture.data.iter())
        }
        let upload_size = width * height * 4;

        let image_properties = ImageProperties::new((width, height), data, 1, upload_size as u64);
        let texture = Texture::new(image_properties, command_pool, instance_devices);
        // panic!("blip blop");

        // let image_info = ImageInfo::new(
        //     (width, height),
        //     1,
        //     vk::SampleCountFlags::TYPE_1,
        //     vk::Format::R8G8B8A8_UNORM,
        //     vk::ImageTiling::OPTIMAL,
        //     vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST,
        //     vk::MemoryPropertyFlags::DEVICE_LOCAL,
        // );

        // let font_image = utility::create_image(image_info, instance_devices);

        // let font_view = utility::create_image_view(
        //     &font_image,
        //     vk::Format::R8G8B8A8_UNORM,
        //     vk::ImageAspectFlags::COLOR,
        //     device,
        // );

        // let staging = texture::create_buffer(
        //     upload_size as vk::DeviceSize,
        //     vk::BufferUsageFlags::TRANSFER_SRC,
        //     vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        //     instance_devices,
        // );

        // memory::map_memory(
        //     device,
        //     staging.memory,
        //     upload_size as vk::DeviceSize,
        //     data.as_slice(),
        // );

        // let command_buffer_allocate_info = vk::CommandBufferAllocateInfo::builder()
        //     .command_pool(*command_pool)
        //     .level(vk::CommandBufferLevel::PRIMARY)
        //     .command_buffer_count(1);

        // let copy_cmd = unsafe {
        //     device
        //         .allocate_command_buffers(&command_buffer_allocate_info)
        //         .expect("Could not allocate command buffers!")
        // }[0];

        // let cmd_buffer_info = vk::CommandBufferBeginInfo::default();
        // unsafe {
        //     device
        //         .begin_command_buffer(copy_cmd, &cmd_buffer_info)
        //         .expect("Could not begin command buffer!");
        // }

        // let sub_resource_range = vk::ImageSubresourceRange::builder()
        //     .aspect_mask(vk::ImageAspectFlags::COLOR)
        //     .base_mip_level(0)
        //     .level_count(1)
        //     .base_array_layer(0)
        //     .layer_count(1);
        // let mut image_memory_barrier = vk::ImageMemoryBarrier::builder()
        //     .old_layout(vk::ImageLayout::UNDEFINED)
        //     .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
        //     .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        //     .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
        //     .image(font_image.image)
        //     .subresource_range(*sub_resource_range)
        //     .src_access_mask(vk::AccessFlags::empty())
        //     .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE);
        // unsafe {
        //     device.cmd_pipeline_barrier(
        //         copy_cmd,
        //         vk::PipelineStageFlags::TOP_OF_PIPE,
        //         vk::PipelineStageFlags::TRANSFER,
        //         vk::DependencyFlags::empty(),
        //         &[],
        //         &[],
        //         std::slice::from_ref(&image_memory_barrier),
        //     )
        // }

        // let buffer_copy_region = vk::BufferImageCopy::builder()
        //     .image_subresource(
        //         vk::ImageSubresourceLayers::builder()
        //             .aspect_mask(vk::ImageAspectFlags::COLOR)
        //             .layer_count(1)
        //             .build(),
        //     )
        //     .image_extent(
        //         Extent3D::builder()
        //             .width(width)
        //             .height(height)
        //             .depth(1)
        //             .build(),
        //     );

        // unsafe {
        //     device.cmd_copy_buffer_to_image(
        //         copy_cmd,
        //         staging.buffer,
        //         font_image.image,
        //         vk::ImageLayout::TRANSFER_DST_OPTIMAL,
        //         std::slice::from_ref(&buffer_copy_region),
        //     )
        // }

        // image_memory_barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
        // image_memory_barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        // image_memory_barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
        // image_memory_barrier.dst_access_mask = vk::AccessFlags::SHADER_READ;
        // unsafe {
        //     device.cmd_pipeline_barrier(
        //         copy_cmd,
        //         vk::PipelineStageFlags::TRANSFER,
        //         vk::PipelineStageFlags::FRAGMENT_SHADER,
        //         vk::DependencyFlags::empty(),
        //         &[],
        //         &[],
        //         std::slice::from_ref(&image_memory_barrier),
        //     )
        // }

        // let submit_info =
        //     vk::SubmitInfo::builder().command_buffers(std::slice::from_ref(&copy_cmd));
        // let fence_info = vk::FenceCreateInfo::default();
        // unsafe {
        //     device
        //         .end_command_buffer(copy_cmd)
        //         .expect("Could not end command buffer!");
        //     let fence = device
        //         .create_fence(&fence_info, None)
        //         .expect("Could not create fence!");
        //     device
        //         .queue_submit(*copy_queue, std::slice::from_ref(&submit_info), fence)
        //         .expect("Could not submit queue!");
        //     device
        //         .wait_for_fences(std::slice::from_ref(&fence), true, 100_000_000_000)
        //         .expect("Wait for fences failed!");
        //     device.destroy_fence(fence, None);
        //     device.free_command_buffers(*command_pool, std::slice::from_ref(&copy_cmd));

        //     device.destroy_buffer(staging.buffer, None);
        //     device.free_memory(staging.memory, None);
        // };

        // let sampler_info = vk::SamplerCreateInfo::builder()
        //     .max_anisotropy(1.)
        //     .mag_filter(vk::Filter::LINEAR)
        //     .min_filter(vk::Filter::LINEAR)
        //     .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        //     .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        //     .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        //     .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
        //     .border_color(vk::BorderColor::FLOAT_OPAQUE_WHITE);
        // let sampler = unsafe {
        //     device
        //         .create_sampler(&sampler_info, None)
        //         .expect("Could not create sampler!")
        // };

        // Descriptor Pool
        let pool_sizes = vk::DescriptorPoolSize::builder()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1);
        let descriptor_pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(std::slice::from_ref(&pool_sizes))
            .max_sets(2);
        let descriptor_pool = unsafe {
            device
                .create_descriptor_pool(&descriptor_pool_info, None)
                .expect("Could not create descriptor pool!")
        };

        // Descriptor Set Layout
        let set_layout_bindings = vk::DescriptorSetLayoutBinding::builder()
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .binding(0)
            .descriptor_count(1);
        let descriptor_layout = vk::DescriptorSetLayoutCreateInfo::builder()
            .bindings(std::slice::from_ref(&set_layout_bindings));
        let descriptor_set_layout = unsafe {
            device
                .create_descriptor_set_layout(&descriptor_layout, None)
                .expect("Could not create descriptor set layout!")
        };

        // Descriptor Set
        let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::builder()
            .descriptor_pool(descriptor_pool)
            .set_layouts(std::slice::from_ref(&descriptor_set_layout));
        let descriptor_set = unsafe {
            device
                .allocate_descriptor_sets(&descriptor_set_alloc_info)
                .expect("Could not allocate descriptor set!")[0]
        };
        let font_descriptor = vk::DescriptorImageInfo::builder()
            .sampler(texture.sampler)
            .image_view(texture.image_view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        let write_descriptor_set = vk::WriteDescriptorSet::builder()
            .dst_set(descriptor_set)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .dst_binding(0)
            .image_info(std::slice::from_ref(&font_descriptor));
        unsafe { device.update_descriptor_sets(std::slice::from_ref(&write_descriptor_set), &[]) }

        // Pipeline Cache
        let pipeline_cache_create_info = vk::PipelineCacheCreateInfo::default();
        let pipeline_cache = unsafe {
            device
                .create_pipeline_cache(&pipeline_cache_create_info, None)
                .expect("Could not create pipeline cache")
        };

        let push_constant_range = vk::PushConstantRange::builder()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            // .size(std::mem::size_of::<Push>().try_into().unwrap())
            .size(64)
            .offset(0);
        let pipeline_layout_create_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(std::slice::from_ref(&descriptor_set_layout))
            .push_constant_ranges(std::slice::from_ref(&push_constant_range));
        let pipeline_layout = unsafe {
            device
                .create_pipeline_layout(&pipeline_layout_create_info, None)
                .expect("Could not create pipeline layout!")
        };

        // Graphics Pipeline
        let input_assembly_state = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST)
            .primitive_restart_enable(false);
        let rasterization_state = vk::PipelineRasterizationStateCreateInfo::builder()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_clamp_enable(false)
            .line_width(1.);

        let blend_attachment_state = vk::PipelineColorBlendAttachmentState::builder()
            .blend_enable(true)
            .color_write_mask(vk::ColorComponentFlags::RGBA)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let colour_blend_state = vk::PipelineColorBlendStateCreateInfo::builder()
            .attachments(std::slice::from_ref(&blend_attachment_state));

        let depth_stencil_state = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(false)
            .depth_write_enable(false)
            .depth_compare_op(vk::CompareOp::LESS_OR_EQUAL)
            .back(
                vk::StencilOpState::builder()
                    .compare_op(vk::CompareOp::ALWAYS)
                    .build(),
            );

        let view_port_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(&[Default::default()])
            .scissors(&[Default::default()])
            .build();

        let multi_sample_state = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);

        let dynamic_state_enables = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic_state =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_state_enables);

        let vertex_input_bindings = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<DrawVert>().try_into().unwrap())
            .input_rate(vk::VertexInputRate::VERTEX);
        let vertex_input_attributes = [
            vk::VertexInputAttributeDescription::builder()
                .location(0)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(memoffset::offset_of!(DrawVert, pos) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .location(1)
                .binding(0)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(memoffset::offset_of!(DrawVert, uv) as u32)
                .build(),
            vk::VertexInputAttributeDescription::builder()
                .location(2)
                .binding(0)
                .format(vk::Format::R8G8B8A8_UNORM)
                .offset(memoffset::offset_of!(DrawVert, col) as u32)
                .build(),
        ];
        let vertex_input_state = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_attribute_descriptions(&vertex_input_attributes)
            .vertex_binding_descriptions(std::slice::from_ref(&vertex_input_bindings));

        let shader_modules = create_shader_stages(Shader::Ui, device);

        let pipeline_create_info = vk::GraphicsPipelineCreateInfo::builder()
            .layout(pipeline_layout)
            .render_pass(render_pass.0)
            .base_pipeline_index(-1)
            .input_assembly_state(&input_assembly_state)
            .rasterization_state(&rasterization_state)
            .color_blend_state(&colour_blend_state)
            .multisample_state(&multi_sample_state)
            .viewport_state(&view_port_state)
            .depth_stencil_state(&depth_stencil_state)
            .dynamic_state(&dynamic_state)
            .stages(&shader_modules.stages)
            .vertex_input_state(&vertex_input_state);

        let pipeline = unsafe {
            device
                .create_graphics_pipelines(
                    pipeline_cache,
                    std::slice::from_ref(&pipeline_create_info),
                    None,
                )
                .expect("Could not create graphics pipeline!")[0]
        };

        unsafe { destroy_shader_modules(device, shader_modules.vert, shader_modules.frag) };

        // let frame_buffer = create_frame_buffer(&render_pass, &[font_view], device, width, height);

        Self {
            context,
            gui_vk: GuiVk {
                // sampler,
                vertex_buffer: None,
                index_buffer: None,
                vertex_count: 0,
                index_count: 0,
                // font_memory: texture.image.memory,
                // frame_buffer,
                // resource: Resource {
                //     image: font_image,
                //     view: font_view,
                // },
                texture,
                pipeline_cache,
                pipeline_layout,
                pipeline,
                descriptor_pool,
                descriptor_set_layout,
                descriptor_set,
                // command_buffer: copy_cmd,
                // render_pass,
            },
        }
    }

    #[inline]
    pub fn new_frame(ui: &Ui) {
        Window::new("Hello world")
            .size([300.0, 100.0], Condition::FirstUseEver)
            .build(ui, || {
                ui.text_wrapped("Hello world!");
                ui.text_wrapped("こんにちは世界！");

                ui.button("This...is...imgui-rs!");
                ui.separator();
                let mouse_pos = ui.io().mouse_pos;
                ui.text(format!(
                    "Mouse Position: ({:.1},{:.1})",
                    mouse_pos[0], mouse_pos[1]
                ));

                ui.show_demo_window(&mut true);
                // ui.show_metrics_window(&mut true);
            });
    }

    pub fn update_buffers(
        gui_vk: &mut GuiVk,
        draw_data: &DrawData,
        instance_devices: &InstanceDevices,
    ) {
        let device = &instance_devices.devices.logical.device;

        let vertex_buffer_size =
            draw_data.total_vtx_count as usize * std::mem::size_of::<DrawVert>();
        let index_buffer_size = draw_data.total_idx_count as usize * std::mem::size_of::<DrawIdx>();

        if vertex_buffer_size == 0 || index_buffer_size == 0 {
            return;
        }

        let total_vtx_count = draw_data.total_vtx_count;
        let total_idx_count = draw_data.total_idx_count;

        if gui_vk.vertex_buffer.is_none()
            || gui_vk.vertex_count != total_vtx_count.try_into().unwrap()
        {
            if let Some(vertex_buffer) = &gui_vk.vertex_buffer {
                unsafe {
                    device.unmap_memory(vertex_buffer.memory);
                    device.destroy_buffer(vertex_buffer.buffer, None);
                    device.free_memory(vertex_buffer.memory, None);
                }
                gui_vk.vertex_buffer = None;
            }

            gui_vk.vertex_buffer = Some(create_buffer(
                vertex_buffer_size.try_into().unwrap(),
                vk::BufferUsageFlags::VERTEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE,
                instance_devices,
            ));
            gui_vk.vertex_count = total_vtx_count.try_into().unwrap();
        }
        let vertex_data = unsafe {
            device
                .map_memory(
                    gui_vk.vertex_buffer.as_ref().unwrap().memory,
                    0,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to map memory!")
        };

        if gui_vk.index_buffer.is_none() || gui_vk.index_count < total_idx_count.try_into().unwrap()
        {
            if let Some(index_buffer) = &gui_vk.index_buffer {
                unsafe {
                    device.unmap_memory(index_buffer.memory);
                    device.destroy_buffer(index_buffer.buffer, None);
                    device.free_memory(index_buffer.memory, None);
                }
                gui_vk.index_buffer = None;
            }

            gui_vk.index_buffer = Some(create_buffer(
                index_buffer_size.try_into().unwrap(),
                vk::BufferUsageFlags::INDEX_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE,
                instance_devices,
            ));
            gui_vk.index_count = total_idx_count.try_into().unwrap();
        }
        let index_data = unsafe {
            device
                .map_memory(
                    gui_vk.index_buffer.as_ref().unwrap().memory,
                    0,
                    vk::WHOLE_SIZE,
                    vk::MemoryMapFlags::empty(),
                )
                .expect("Failed to map memory!")
        };

        let mut vertex_dst = vertex_data as *mut DrawVert;
        let mut index_dst = index_data as *mut DrawIdx;

        for draw_list in draw_data.draw_lists() {
            let vtx_buffer = draw_list.vtx_buffer();
            let idx_buffer = draw_list.idx_buffer();

            let _ = std::mem::replace(&mut vertex_dst, vtx_buffer.as_ptr() as *mut DrawVert);
            let _ = std::mem::replace(&mut index_dst, idx_buffer.as_ptr() as *mut u16);
        }

        if let Some(vertex_buffer) = gui_vk.vertex_buffer {
            let vertex_mapped_range = vk::MappedMemoryRange::builder().memory(vertex_buffer.memory);
            unsafe {
                device
                    .flush_mapped_memory_ranges(std::slice::from_ref(&vertex_mapped_range))
                    .expect("Could not flush mapped memory");
            }
        }

        if let Some(index_buffer) = gui_vk.index_buffer {
            let index_mapped_range = vk::MappedMemoryRange::builder().memory(index_buffer.memory);
            unsafe {
                device
                    .flush_mapped_memory_ranges(std::slice::from_ref(&index_mapped_range))
                    .expect("Could not flush mapped memory");
            }
        }
    }

    fn draw_frame(
        gui_vk: &GuiVk,
        draw_data: &DrawData,
        device: &Device,
        command_buffer: vk::CommandBuffer,
    ) {
        unsafe {
            device.cmd_bind_descriptor_sets(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                gui_vk.pipeline_layout,
                0,
                std::slice::from_ref(&gui_vk.descriptor_set),
                &[],
            );
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                gui_vk.pipeline,
            )
        };

        let framebuffer_width = draw_data.framebuffer_scale[0] * draw_data.display_size[0];
        let framebuffer_height = draw_data.framebuffer_scale[1] * draw_data.display_size[1];
        let viewports = [vk::Viewport::builder()
            .width(framebuffer_width)
            .height(framebuffer_height)
            .max_depth(1.0)
            .build()];

        unsafe { device.cmd_set_viewport(command_buffer, 0, &viewports) };

        // Ortho projection
        let projection = orthographic_vk(
            0.0,
            draw_data.display_size[0],
            0.0,
            -draw_data.display_size[1],
            -1.0,
            1.0,
        );
        unsafe {
            let push = any_as_u8_slice(&projection);
            device.cmd_push_constants(
                command_buffer,
                gui_vk.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                push,
            )
        };

        unsafe {
            if let Some(index_buffer) = gui_vk.index_buffer {
                device.cmd_bind_index_buffer(
                    command_buffer,
                    index_buffer.buffer,
                    0,
                    vk::IndexType::UINT16,
                );
            }

            if let Some(vertex_buffer) = gui_vk.vertex_buffer {
                device.cmd_bind_vertex_buffers(command_buffer, 0, &[vertex_buffer.buffer], &[0])
            }
        };

        let mut index_offset = 0;
        let mut vertex_offset = 0;
        let clip_offset = draw_data.display_pos;
        let clip_scale = draw_data.framebuffer_scale;
        for draw_list in draw_data.draw_lists() {
            for command in draw_list.commands() {
                match command {
                    DrawCmd::Elements {
                        count,
                        cmd_params:
                            DrawCmdParams {
                                clip_rect,
                                texture_id,
                                vtx_offset,
                                idx_offset,
                            },
                    } => {
                        unsafe {
                            let clip_x = (clip_rect[0] - clip_offset[0]) * clip_scale[0];
                            let clip_y = (clip_rect[1] - clip_offset[1]) * clip_scale[1];
                            let clip_w = (clip_rect[2] - clip_offset[0]) * clip_scale[0] - clip_x;
                            let clip_h = (clip_rect[3] - clip_offset[1]) * clip_scale[1] - clip_y;

                            let scissors = vk::Rect2D::builder()
                                .offset(vk::Offset2D {
                                    x: clip_x as _,
                                    y: clip_y as _,
                                })
                                .extent(vk::Extent2D {
                                    width: clip_w as _,
                                    height: clip_h as _,
                                });
                            device.cmd_set_scissor(
                                command_buffer,
                                0,
                                std::slice::from_ref(&scissors),
                            );
                        }

                        dbg!(432312);
                        unsafe {
                            device.cmd_draw_indexed(
                                command_buffer,
                                count as _,
                                1,
                                index_offset + idx_offset as u32,
                                vertex_offset + vtx_offset as i32,
                                0,
                            )
                        };
                    }
                    DrawCmd::ResetRenderState => {}
                    DrawCmd::RawCallback { .. } => {}
                }
            }

            index_offset += draw_list.idx_buffer().len() as u32;
            vertex_offset += draw_list.vtx_buffer().len() as i32;
        }
    }

    unsafe fn destroy(&self, device: &Device) {
        let GuiVk {
            // sampler,
            vertex_buffer,
            index_buffer,
            // render_pass,
            // font_memory,
            // resource,
            texture,
            pipeline_cache,
            pipeline_layout,
            pipeline,
            descriptor_pool,
            descriptor_set_layout,
            ..
        } = &self.gui_vk;

        if let Some(vertex_buffer) = &vertex_buffer {
            device.destroy_buffer(vertex_buffer.buffer, None);
            device.free_memory(vertex_buffer.memory, None);
        }
        if let Some(index_buffer) = &index_buffer {
            device.destroy_buffer(index_buffer.buffer, None);
            device.free_memory(index_buffer.memory, None);
        }
        device.destroy_image(texture.image.image, None);
        device.destroy_image_view(texture.image_view, None);
        device.free_memory(texture.image.memory, None);
        device.destroy_sampler(texture.sampler, None);
        device.destroy_pipeline_cache(*pipeline_cache, None);
        device.destroy_pipeline(*pipeline, None);
        device.destroy_pipeline_layout(*pipeline_layout, None);
        device.destroy_descriptor_pool(*descriptor_pool, None);
        device.destroy_descriptor_set_layout(*descriptor_set_layout, None);
        // device.destroy_render_pass(render_pass.0, None);
    }
}

pub fn orthographic_vk(
    left: f32,
    right: f32,
    bottom: f32,
    top: f32,
    near: f32,
    far: f32,
) -> Matrix4<f32> {
    let rml = right - left;
    let rpl = right + left;
    let tmb = top - bottom;
    let tpb = top + bottom;
    let fmn = far - near;
    matrix![
        2. / rml, 0., 0., 0.;
        0., -2. / tmb, 0., 0.;
        0., 0., -1. / fmn, 0.;
        -(rpl / rml), -(tpb / tmb), -(near / fmn), 1.;
    ]
}

pub(crate) unsafe fn any_as_u8_slice<T: Sized>(any: &T) -> &[u8] {
    let ptr = (any as *const T) as *const u8;
    std::slice::from_raw_parts(ptr, std::mem::size_of::<T>())
}

//
//
//
//
//
//
//
//

pub(crate) struct VulkanObjects(Vec<VulkanObject>);

pub struct Vulkan {
    pub(crate) command_buffers: CommandBuffers,
    pub(crate) command_pool: CommandPool,
    pub(crate) resources: Resources,
    pub(crate) render_pass: RenderPass,
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) surface_loader: Surface,
    pub swap_chain: SwapChain,
    pub(crate) sync_objects: SyncObjects,
    pub ubo: UniformBufferObject,
    pub(crate) debugger: Option<Debug>,
    pub(crate) frame_buffers: FrameBuffers,
    pub instance_devices: InstanceDevices,
    pub(crate) objects: VulkanObjects,
    pub gui: ImGui,
}

impl Vulkan {
    #[inline]
    pub fn wait_device_idle(&self) {
        unsafe {
            self.instance_devices
                .devices
                .logical
                .device
                .device_wait_idle()
                .expect("Failed to wait for device idle state");
        }
    }

    pub fn new(
        display: &Display,
        camera: &CameraInternal,
        geom_properties: &[GeomProperties],
        debugging: Option<Debugger>,
    ) -> Self {
        let entry_instance = EntryInstance::new(&display.window, debugging);

        let debugger = if cfg!(debug_assertions) {
            debugging.map(|debugging| debug::debugger(&entry_instance, debugging))
        } else {
            None
        };

        let surface = lambda_window::create_surface(
            &display.window,
            &entry_instance.instance,
            &entry_instance.entry,
        );

        let surface_loader = create_surface(&entry_instance);

        let devices = Devices::new(&entry_instance.instance, &surface, &surface_loader);

        let instance_devices = InstanceDevices {
            instance: entry_instance.instance,
            devices,
        };

        let swap_chain =
            SwapChain::new(&instance_devices, surface, &surface_loader, &display.window);

        let render_pass = renderer::create_render_pass(&instance_devices, &swap_chain);

        let resources = Resources::new(&swap_chain, &instance_devices);

        let frame_buffers = frame_buffer::create_frame_buffers(
            &swap_chain,
            &render_pass,
            &instance_devices.devices.logical.device,
            &resources,
        );

        let command_pool =
            command_buffer::create_command_pool(&instance_devices, &surface_loader, &surface);

        let sync_objects = SyncObjects::new(&instance_devices.devices.logical.device);

        let swap_chain_len = swap_chain.images.len() as u32;

        let objects = VulkanObjects(
            geom_properties
                .iter()
                .map(|property| {
                    VulkanObject::new(
                        &command_pool,
                        swap_chain_len,
                        &swap_chain,
                        &render_pass,
                        &instance_devices,
                        property,
                    )
                })
                .collect(),
        );

        let ubo = UniformBufferObject::new(&swap_chain.extent, camera);

        let mut gui = ImGui::new(
            &instance_devices,
            &command_pool,
            &instance_devices.devices.logical.queues.present,
            &render_pass,
        );

        let command_buffers = command_buffer::create_command_buffers(
            &command_pool,
            &swap_chain,
            &instance_devices,
            &render_pass,
            &frame_buffers,
            &objects.0,
            &mut gui,
        );

        Self {
            command_buffers,
            command_pool,
            render_pass,

            resources,
            surface,
            surface_loader,
            swap_chain,
            sync_objects,
            ubo,
            debugger,
            frame_buffers,
            instance_devices,
            objects,
            gui,
        }
    }

    #[inline]
    pub fn update_objects(&mut self, properties: &[GeomProperties]) {
        self.objects
            .0
            .iter_mut()
            .zip(properties)
            .for_each(|(object, properties)| {
                object.model = properties.model;
            });
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        swap_chain::cleanup_swap_chain(self);

        unsafe {
            self.gui
                .destroy(&self.instance_devices.devices.logical.device);

            self.objects.0.iter().for_each(|object| {
                device::recreate_drop(
                    &object.graphics_pipeline,
                    &self.instance_devices.devices.logical.device,
                );
                device::destroy(object, &self.instance_devices.devices.logical.device);
            });

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                self.instance_devices
                    .devices
                    .logical
                    .device
                    .destroy_semaphore(self.sync_objects.render_finished_semaphores[i], None);
                self.instance_devices
                    .devices
                    .logical
                    .device
                    .destroy_semaphore(self.sync_objects.image_available_semaphores[i], None);
                self.instance_devices
                    .devices
                    .logical
                    .device
                    .destroy_fence(self.sync_objects.in_flight_fences[i], None);
            }

            self.instance_devices
                .devices
                .logical
                .device
                .destroy_command_pool(*self.command_pool, None);

            dbg!("2");

            self.instance_devices
                .devices
                .logical
                .device
                .destroy_device(None);

            println!("here");

            if ENABLE_VALIDATION_LAYERS {
                if let Some(debugger) = self.debugger.take() {
                    debugger
                        .utils
                        .destroy_debug_utils_messenger(debugger.messenger, None);
                }
            }

            self.surface_loader.destroy_surface(self.surface, None);

            self.instance_devices.instance.destroy_instance(None);
        }
    }
}

#[derive(Clone, Debug, Deref, DerefMut, Default)]
pub struct TextureBuffer(pub Vec<u8>);

#[derive(Debug, new)]
pub struct GeomProperties<'a> {
    texture_buffer: &'a [u8],
    vertices_and_indices: VerticesAndIndices,
    topology: ModelTopology,
    cull_mode: CullMode,
    shader: Shader,
    indexed: bool,
    model: Matrix4<f32>,
}

#[derive(Debug)]
pub(crate) struct VulkanObject {
    vertices_and_indices: VerticesAndIndices,
    texture: Option<Texture>,
    graphics_pipeline: GraphicsPipeline,
    buffers: ModelBuffers,
    indexed: bool,
    model: Matrix4<f32>,
}

impl VulkanObject {
    fn new(
        command_pool: &CommandPool,
        command_buffer_count: u32,
        swap_chain: &SwapChain,
        render_pass: &RenderPass,
        instance_devices: &InstanceDevices,
        properties: &GeomProperties,
    ) -> Self {
        let mut texture = None;
        if !properties.texture_buffer.is_empty() {
            let image_properties =
                ImageProperties::get_image_properties_from_buffer(properties.texture_buffer);
            texture = Some(Texture::new(
                image_properties,
                command_pool,
                instance_devices,
            ));
        }

        let buffers = ModelBuffers::new(
            &properties.vertices_and_indices,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        let graphics_pipeline = GraphicsPipeline::new(
            swap_chain,
            render_pass.0,
            &texture,
            properties.topology,
            properties.cull_mode,
            instance_devices,
            properties.shader,
        );

        Self {
            vertices_and_indices: properties.vertices_and_indices.clone(),
            texture,
            graphics_pipeline,
            buffers,
            indexed: properties.indexed,
            model: properties.model,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct WindowSize(vk::Extent2D);

#[inline]
pub(crate) fn create_surface(entry_instance: &EntryInstance) -> Surface {
    Surface::new(&entry_instance.entry, &entry_instance.instance)
}

#[derive(Clone, Copy, Debug, Default)]
pub enum ModelTopology {
    LineList,
    LineListWithAdjacency,
    LineStrip,
    LineStripWithAdjacency,
    PatchList,
    PointList,
    TriangleFan,
    #[default]
    TriangleList,
    TriangleListWithAdjacency,
    TriangleStrip,
    TriangleStripWithAdjacency,
}

impl From<ModelTopology> for vk::PrimitiveTopology {
    fn from(model_topology: ModelTopology) -> Self {
        match model_topology {
            ModelTopology::LineList => vk::PrimitiveTopology::LINE_LIST,
            ModelTopology::LineListWithAdjacency => vk::PrimitiveTopology::LINE_LIST_WITH_ADJACENCY,
            ModelTopology::LineStrip => vk::PrimitiveTopology::LINE_STRIP,
            ModelTopology::LineStripWithAdjacency => {
                vk::PrimitiveTopology::LINE_STRIP_WITH_ADJACENCY
            }
            ModelTopology::PatchList => vk::PrimitiveTopology::PATCH_LIST,
            ModelTopology::PointList => vk::PrimitiveTopology::POINT_LIST,
            ModelTopology::TriangleFan => vk::PrimitiveTopology::TRIANGLE_FAN,
            ModelTopology::TriangleList => vk::PrimitiveTopology::TRIANGLE_LIST,
            ModelTopology::TriangleListWithAdjacency => {
                vk::PrimitiveTopology::TRIANGLE_LIST_WITH_ADJACENCY
            }
            ModelTopology::TriangleStrip => vk::PrimitiveTopology::TRIANGLE_STRIP,
            ModelTopology::TriangleStripWithAdjacency => {
                vk::PrimitiveTopology::TRIANGLE_STRIP_WITH_ADJACENCY
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CullMode {
    Back,
    Front,
    FrontAndBack,
    #[default]
    None,
}

impl From<CullMode> for vk::CullModeFlags {
    fn from(model_cull_model: CullMode) -> Self {
        match model_cull_model {
            CullMode::Back => vk::CullModeFlags::BACK,
            CullMode::Front => vk::CullModeFlags::FRONT,
            CullMode::FrontAndBack => vk::CullModeFlags::FRONT_AND_BACK,
            CullMode::None => vk::CullModeFlags::NONE,
        }
    }
}

const LIGHT: &str = "light";
const LIGHT_TEXTURE: &str = "light_texture";
const TEXTURE: &str = "texture";
const VERTEX: &str = "vertex";
const PUSH_CONSTANT: &str = "push_constant";
const UI: &str = "ui";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Shader {
    #[default]
    Light,
    LightTexture,
    Texture,
    Vertex,
    PushConstant,
    Ui,
}

impl From<Shader> for &str {
    fn from(texture_type: Shader) -> Self {
        match texture_type {
            Shader::Light => LIGHT,
            Shader::LightTexture => LIGHT_TEXTURE,
            Shader::Texture => TEXTURE,
            Shader::Vertex => VERTEX,
            Shader::PushConstant => PUSH_CONSTANT,
            Shader::Ui => UI,
        }
    }
}
