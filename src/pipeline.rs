use std::{ffi::CString, mem::size_of};

use ash::{vk, Instance};
use memoffset::offset_of;

use crate::{model::Vertex, swapchain::SwapChain, texture::{Texture, self}, Devices, UniformBufferObject};

pub struct LambdaDescriptorSet {
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub uniform_buffers: Vec<vk::Buffer>,
    pub uniform_buffers_memory: Vec<vk::DeviceMemory>,
}

pub struct GraphicsPipeline {
    pub topology: vk::PrimitiveTopology,
    pub cull_mode: vk::CullModeFlags,
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub descriptor_set: LambdaDescriptorSet,
}

impl GraphicsPipeline {
    pub fn new(
        instance: &Instance,
        topology: Option<vk::PrimitiveTopology>,
        cull_mode: Option<vk::CullModeFlags>,
        devices: &Devices,
        swapchain: &SwapChain,
        render_pass: vk::RenderPass,
        texture_image_view: vk::ImageView,
        sampler: vk::Sampler,
    ) -> Self {
        let topology = match topology {
            Some(topology) => topology,
            None => vk::PrimitiveTopology::TRIANGLE_LIST,
        };

        let cull_mode = match cull_mode {
            Some(cull_mode) => cull_mode,
            None => vk::CullModeFlags::BACK,
        };

        let descriptor_set_layout = Self::create_descriptor_set_layout(devices);

        let (pipeline, layout) = Self::create_pipeline_and_layout(
            devices,
            topology,
            cull_mode,
            swapchain,
            &descriptor_set_layout,
            render_pass,
        );

        let descriptor_pool = Self::create_descriptor_pool(devices, swapchain.images.len() as u32);

        let (uniform_buffers, uniform_buffers_memory) = GraphicsPipeline::create_uniform_buffers(
            instance,
            devices,
            swapchain.images.len() as u32,
        );

        let descriptor_sets = Self::create_descriptor_sets(
            devices,
            descriptor_set_layout,
            descriptor_pool,
            swapchain.images.len() as u32,
            texture_image_view,
            sampler,
            &uniform_buffers,
        );

        Self {
            topology,
            cull_mode,
            pipeline,
            layout,
            descriptor_set: LambdaDescriptorSet {
                descriptor_sets,
                descriptor_pool,
                descriptor_set_layout,
                uniform_buffers,
                uniform_buffers_memory,
            },
        }
    }

    fn create_shader_module(devices: &Devices, code: &[u32]) -> vk::ShaderModule {
        let create_info = vk::ShaderModuleCreateInfo::builder().code(code);

        unsafe {
            devices
                .logical
                .create_shader_module(&create_info, None)
                .expect("Failed to create shader module!")
        }
    }

    fn create_pipeline_and_layout(
        devices: &Devices,
        topology: vk::PrimitiveTopology,
        cull_mode: vk::CullModeFlags,
        swapchain: &SwapChain,
        descriptor_set_layout: &vk::DescriptorSetLayout,
        render_pass: vk::RenderPass,
    ) -> (vk::Pipeline, vk::PipelineLayout) {
        let entry_point = CString::new("main").unwrap();

        let mut vertex_file =
            std::fs::File::open("/Users/rob/_CODE/C/vulkan-tmp/src/shaders/light_texture/vert.spv")
                .unwrap();
        let vertex_spv = ash::util::read_spv(&mut vertex_file).unwrap();
        let vert_shader_module = Self::create_shader_module(devices, &vertex_spv);

        let mut frag_file =
            std::fs::File::open("/Users/rob/_CODE/C/vulkan-tmp/src/shaders/light_texture/frag.spv")
                .unwrap();
        let frag_spv = ash::util::read_spv(&mut frag_file).unwrap();
        let frag_shader_module = Self::create_shader_module(devices, &frag_spv);

        let shader_stages = [
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::VERTEX,
                module: vert_shader_module,
                p_name: entry_point.as_ptr(),
                ..Default::default()
            },
            vk::PipelineShaderStageCreateInfo {
                stage: vk::ShaderStageFlags::FRAGMENT,
                module: frag_shader_module,
                p_name: entry_point.as_ptr(),
                ..Default::default()
            },
        ];

        let binding_description = vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(size_of::<Vertex>().try_into().unwrap())
            .input_rate(vk::VertexInputRate::VERTEX);

        let attribute_descriptions = [
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 0,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, pos) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 1,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, colour) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 2,
                format: vk::Format::R32G32B32_SFLOAT,
                offset: offset_of!(Vertex, normal) as u32,
            },
            vk::VertexInputAttributeDescription {
                binding: 0,
                location: 3,
                format: vk::Format::R32G32_SFLOAT,
                offset: offset_of!(Vertex, tex_coord) as u32,
            },
        ];

        let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::builder()
            .vertex_binding_descriptions(std::slice::from_ref(&binding_description))
            .vertex_attribute_descriptions(&attribute_descriptions);

        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::builder()
            .topology(topology)
            .primitive_restart_enable(false);

        let view_port = vk::Viewport::builder()
            .x(0.)
            .y(0.)
            .width(swapchain.extent.width as f32)
            .height(swapchain.extent.height as f32)
            .min_depth(0.)
            .max_depth(1.);

        let scissor = vk::Rect2D::builder()
            .offset(vk::Offset2D { x: 0, y: 0 })
            .extent(swapchain.extent);

        let view_port_state = vk::PipelineViewportStateCreateInfo::builder()
            .viewports(std::slice::from_ref(&view_port))
            .scissors(std::slice::from_ref(&scissor));

        let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
            .depth_clamp_enable(false)
            .rasterizer_discard_enable(false)
            .polygon_mode(vk::PolygonMode::FILL)
            // .polygon_mode(vk::PolygonMode::LINE)
            // .polygon_mode(vk::PolygonMode::POINT)
            .line_width(1.)
            .cull_mode(cull_mode)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
            .depth_bias_enable(false);

        let multisampling = vk::PipelineMultisampleStateCreateInfo::builder()
            .rasterization_samples(devices.msaa_samples)
            .sample_shading_enable(true)
            .min_sample_shading(0.2)
            .alpha_to_coverage_enable(false)
            .alpha_to_one_enable(false);

        let stencil_state = vk::StencilOpState {
            fail_op: vk::StencilOp::KEEP,
            pass_op: vk::StencilOp::KEEP,
            depth_fail_op: vk::StencilOp::KEEP,
            compare_op: vk::CompareOp::ALWAYS,
            compare_mask: 0,
            write_mask: 0,
            reference: 0,
        };

        let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::builder()
            .depth_test_enable(true)
            .depth_write_enable(true)
            .depth_compare_op(vk::CompareOp::LESS)
            .depth_bounds_test_enable(false)
            .stencil_test_enable(false)
            .front(stencil_state)
            .back(stencil_state)
            .min_depth_bounds(0.0)
            .max_depth_bounds(1.0);

        let color_blend_attachment = vk::PipelineColorBlendAttachmentState::builder()
            .color_write_mask(
                vk::ColorComponentFlags::R
                    | vk::ColorComponentFlags::G
                    | vk::ColorComponentFlags::B
                    | vk::ColorComponentFlags::A,
            )
            .blend_enable(true)
            .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
            .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
            .color_blend_op(vk::BlendOp::ADD)
            .src_alpha_blend_factor(vk::BlendFactor::ONE)
            .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
            .alpha_blend_op(vk::BlendOp::ADD);

        let color_blending = vk::PipelineColorBlendStateCreateInfo::builder()
            .logic_op_enable(false)
            .logic_op(vk::LogicOp::COPY)
            .attachments(std::slice::from_ref(&color_blend_attachment))
            .blend_constants([0., 0., 0., 0.]);

        let pipeline_layout_info = vk::PipelineLayoutCreateInfo::builder()
            .set_layouts(std::slice::from_ref(descriptor_set_layout));

        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

        let dynamic_state_create_info =
            vk::PipelineDynamicStateCreateInfo::builder().dynamic_states(&dynamic_states);

        unsafe {
            let layout = devices
                .logical
                .create_pipeline_layout(&pipeline_layout_info, None)
                .expect("Failed to create pipeline layout!");

            let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
                .stages(&shader_stages)
                .vertex_input_state(&vertex_input_info)
                .input_assembly_state(&input_assembly)
                .viewport_state(&view_port_state)
                .rasterization_state(&rasterizer)
                .multisample_state(&multisampling)
                .dynamic_state(&dynamic_state_create_info)
                .color_blend_state(&color_blending)
                .layout(layout)
                .render_pass(render_pass)
                .subpass(0)
                .base_pipeline_handle(vk::Pipeline::null())
                .depth_stencil_state(&depth_stencil);

            let pipeline = devices
                .logical
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_info),
                    None,
                )
                .expect("Failed to create graphics pipeline!");

            devices
                .logical
                .destroy_shader_module(vert_shader_module, None);
            devices
                .logical
                .destroy_shader_module(frag_shader_module, None);

            (pipeline[0], layout)
        }
    }

    fn fun_name() -> vk::DescriptorSetLayoutBinding {
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            ..Default::default()
        }
    }

    fn create_descriptor_set_layout(devices: &Devices) -> vk::DescriptorSetLayout {
        let bindings = [
            Self::fun_name(),
            vk::DescriptorSetLayoutBinding {
                binding: 1,
                descriptor_count: 1,
                descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                stage_flags: vk::ShaderStageFlags::FRAGMENT,
                ..Default::default()
            },
        ];

        let layout_info = vk::DescriptorSetLayoutCreateInfo::builder().bindings(&bindings);

        unsafe {
            devices
                .logical
                .create_descriptor_set_layout(&layout_info, None)
                .expect("Failed to create descriptor set layout")
        }
    }

    fn create_descriptor_pool(devices: &Devices, swapchain_image_count: u32) -> vk::DescriptorPool {
        let pool_sizes = &[
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: swapchain_image_count,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: swapchain_image_count,
            },
        ];

        let pool_info = vk::DescriptorPoolCreateInfo::builder()
            .pool_sizes(pool_sizes)
            .max_sets(swapchain_image_count);

        unsafe {
            devices
                .logical
                .create_descriptor_pool(&pool_info, None)
                .expect("Failed to create descriptor pool!")
        }
    }

    fn create_uniform_buffers(
        instance: &Instance,
        devices: &Devices,
        swapchain_image_count: u32,
    ) -> (Vec<vk::Buffer>, Vec<vk::DeviceMemory>) {
        let mut uniform_buffers = Vec::new();
        let mut uniform_buffer_memory = Vec::new();

        for _i in 0..swapchain_image_count {
            let (buffer, memory) = texture::create_buffer(
                instance,
                devices,
                size_of::<UniformBufferObject>() as u64,
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            );
            uniform_buffers.push(buffer);
            uniform_buffer_memory.push(memory);
        }

        (uniform_buffers, uniform_buffer_memory)
    }

    fn create_descriptor_sets(
        devices: &Devices,
        descriptor_layout: vk::DescriptorSetLayout,
        descriptor_pool: vk::DescriptorPool,
        swapchain_image_count: u32,
        texture_image_view: vk::ImageView,
        sampler: vk::Sampler,
        uniform_buffers: &[vk::Buffer],
    ) -> Vec<vk::DescriptorSet> {
        let layouts = vec![descriptor_layout; swapchain_image_count as usize];

        let alloc_info = vk::DescriptorSetAllocateInfo {
            descriptor_pool,
            descriptor_set_count: swapchain_image_count,
            p_set_layouts: layouts.as_slice().as_ptr(),
            ..Default::default()
        };

        let image_info = vk::DescriptorImageInfo {
            sampler,
            image_view: texture_image_view,
            image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        };

        unsafe {
            let descriptor_sets = devices
                .logical
                .allocate_descriptor_sets(&alloc_info)
                .expect("Failed to allocate descriptor sets!");

            for i in 0..swapchain_image_count as usize {
                let buffer_info = vk::DescriptorBufferInfo {
                    buffer: uniform_buffers[i],
                    offset: 0,
                    range: size_of::<UniformBufferObject>() as u64,
                };

                let descriptor_writes = [
                    vk::WriteDescriptorSet {
                        dst_set: descriptor_sets[i],
                        dst_binding: 0,
                        descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
                        p_buffer_info: std::slice::from_ref(&buffer_info).as_ptr(),
                        descriptor_count: 1,
                        ..Default::default()
                    },
                    vk::WriteDescriptorSet {
                        dst_set: descriptor_sets[i],
                        dst_binding: 1,
                        descriptor_type: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                        p_image_info: std::slice::from_ref(&image_info).as_ptr(),
                        descriptor_count: 1,
                        ..Default::default()
                    },
                ];

                devices
                    .logical
                    .update_descriptor_sets(&descriptor_writes, &[]);
            }

            descriptor_sets
        }
    }
}
