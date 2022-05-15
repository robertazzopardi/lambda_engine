use crate::{
    object::{
        utility::{ModelCullMode, ModelTopology},
        Buffer, Vertex,
    },
    swap_chain::SwapChain,
    texture::{self, Texture},
    uniform_buffer::UniformBufferObject,
    utility::InstanceDevices,
    Devices,
};
use ash::vk;
use memoffset::offset_of;
use std::{ffi::CString, mem};

#[derive(Clone, Default, Debug)]
pub struct Descriptor {
    pub descriptor_sets: Vec<vk::DescriptorSet>,
    pub descriptor_pool: vk::DescriptorPool,
    pub descriptor_set_layout: vk::DescriptorSetLayout,
    pub uniform_buffers: Vec<Buffer>,
}

#[derive(new, Clone, Default, Debug)]
pub struct GraphicsPipelineFeatures {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
}

#[derive(Clone, Default, Debug)]
pub struct GraphicsPipeline {
    pub features: GraphicsPipelineFeatures,
    pub descriptor_set: Descriptor,
}

impl GraphicsPipeline {
    pub fn new(
        swap_chain: &SwapChain,
        render_pass: vk::RenderPass,
        texture: &Texture,
        topology: ModelTopology,
        cull_mode: ModelCullMode,
        instance_devices: &InstanceDevices,
    ) -> Self {
        let InstanceDevices { devices, .. } = instance_devices;

        let descriptor_set_layout = create_descriptor_set_layout(devices);

        let features = create_pipeline_and_layout(
            devices,
            swap_chain,
            &descriptor_set_layout,
            render_pass,
            topology,
            cull_mode,
        );

        let descriptor_pool = create_descriptor_pool(devices, swap_chain.images.len() as u32);

        let uniform_buffers =
            create_uniform_buffers(swap_chain.images.len() as u32, instance_devices);

        let descriptor_sets = create_descriptor_sets(
            devices,
            descriptor_set_layout,
            descriptor_pool,
            swap_chain.images.len() as u32,
            texture,
            &uniform_buffers,
        );

        let descriptor_set = Descriptor {
            descriptor_sets,
            descriptor_pool,
            descriptor_set_layout,
            uniform_buffers,
        };

        Self {
            features,
            descriptor_set,
        }
    }
}

fn create_descriptor_set_layout(devices: &Devices) -> vk::DescriptorSetLayout {
    let bindings = [
        vk::DescriptorSetLayoutBinding {
            binding: 0,
            descriptor_count: 1,
            descriptor_type: vk::DescriptorType::UNIFORM_BUFFER,
            stage_flags: vk::ShaderStageFlags::VERTEX,
            ..Default::default()
        },
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
            .device
            .create_descriptor_set_layout(&layout_info, None)
            .expect("Failed to create descriptor set layout")
    }
}

fn create_descriptor_pool(devices: &Devices, swap_chain_image_count: u32) -> vk::DescriptorPool {
    let pool_sizes = &[
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: swap_chain_image_count,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
            descriptor_count: swap_chain_image_count,
        },
    ];

    let pool_info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(swap_chain_image_count);

    unsafe {
        devices
            .logical
            .device
            .create_descriptor_pool(&pool_info, None)
            .expect("Failed to create descriptor pool!")
    }
}

fn create_uniform_buffers(
    swap_chain_image_count: u32,
    instance_devices: &InstanceDevices,
) -> Vec<Buffer> {
    let mut buffers = Vec::new();

    for _i in 0..swap_chain_image_count {
        let buffer = texture::create_buffer(
            mem::size_of::<UniformBufferObject>() as u64,
            vk::BufferUsageFlags::UNIFORM_BUFFER,
            vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
            instance_devices,
        );
        buffers.push(buffer)
    }

    buffers
}

fn create_shader_module(devices: &Devices, path: &str) -> vk::ShaderModule {
    let mut file = std::fs::File::open(path).unwrap();
    let spv = ash::util::read_spv(&mut file).unwrap();

    let create_info = vk::ShaderModuleCreateInfo::builder().code(&spv);

    unsafe {
        devices
            .logical
            .device
            .create_shader_module(&create_info, None)
            .expect("Failed to create shader module!")
    }
}

fn create_pipeline_and_layout(
    devices: &Devices,
    swap_chain: &SwapChain,
    descriptor_set_layout: &vk::DescriptorSetLayout,
    render_pass: vk::RenderPass,
    topology: ModelTopology,
    cull_mode: ModelCullMode,
) -> GraphicsPipelineFeatures {
    let entry_point = CString::new("main").unwrap();

    let vert_shader_module = create_shader_module(
        devices,
        "./lambda_engine/src/shaders/light_texture/vert.spv",
    );

    let frag_shader_module = create_shader_module(
        devices,
        "./lambda_engine/src/shaders/light_texture/frag.spv",
    );

    let shader_stages = [
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::VERTEX)
            .module(vert_shader_module)
            .name(&entry_point)
            .build(),
        vk::PipelineShaderStageCreateInfo::builder()
            .stage(vk::ShaderStageFlags::FRAGMENT)
            .module(frag_shader_module)
            .name(&entry_point)
            .build(),
    ];

    let binding_description = vk::VertexInputBindingDescription::builder()
        .binding(0)
        .stride(mem::size_of::<Vertex>().try_into().unwrap())
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
        .topology(topology.0)
        .primitive_restart_enable(false);

    let view_port = vk::Viewport::builder()
        .x(0.)
        .y(0.)
        .width(swap_chain.extent.width as f32)
        .height(swap_chain.extent.height as f32)
        .min_depth(0.)
        .max_depth(1.);

    let scissor = vk::Rect2D::builder()
        .offset(vk::Offset2D { x: 0, y: 0 })
        .extent(swap_chain.extent);

    let view_port_state = vk::PipelineViewportStateCreateInfo::builder()
        .viewports(std::slice::from_ref(&view_port))
        .scissors(std::slice::from_ref(&scissor));

    let rasterizer = vk::PipelineRasterizationStateCreateInfo::builder()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        // .polygon_mode(vk::PolygonMode::LINE)
        // .polygon_mode(vk::PolygonMode::POINT)
        // .polygon_mode(vk::PolygonMode::FILL_RECTANGLE_NV)
        .line_width(1.)
        .cull_mode(cull_mode.0)
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);

    let multi_sampling = vk::PipelineMultisampleStateCreateInfo::builder()
        .rasterization_samples(devices.physical.samples)
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
        .min_depth_bounds(0.)
        .max_depth_bounds(1.);

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
            .device
            .create_pipeline_layout(&pipeline_layout_info, None)
            .expect("Failed to create pipeline layout!");

        let pipeline_info = vk::GraphicsPipelineCreateInfo::builder()
            .stages(&shader_stages)
            .vertex_input_state(&vertex_input_info)
            .input_assembly_state(&input_assembly)
            .viewport_state(&view_port_state)
            .rasterization_state(&rasterizer)
            .multisample_state(&multi_sampling)
            .dynamic_state(&dynamic_state_create_info)
            .color_blend_state(&color_blending)
            .layout(layout)
            .render_pass(render_pass)
            .subpass(0)
            .base_pipeline_handle(vk::Pipeline::null())
            .depth_stencil_state(&depth_stencil);

        let pipeline = devices
            .logical
            .device
            .create_graphics_pipelines(
                vk::PipelineCache::null(),
                std::slice::from_ref(&pipeline_info),
                None,
            )
            .expect("Failed to create graphics pipeline!");

        devices
            .logical
            .device
            .destroy_shader_module(vert_shader_module, None);
        devices
            .logical
            .device
            .destroy_shader_module(frag_shader_module, None);

        GraphicsPipelineFeatures::new(pipeline[0], layout)
    }
}

fn create_descriptor_sets(
    devices: &Devices,
    descriptor_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    swap_chain_image_count: u32,
    texture: &Texture,
    uniform_buffers: &[Buffer],
) -> Vec<vk::DescriptorSet> {
    let layouts = vec![descriptor_layout; swap_chain_image_count as usize];

    let alloc_info = vk::DescriptorSetAllocateInfo {
        descriptor_pool,
        descriptor_set_count: swap_chain_image_count,
        p_set_layouts: layouts.as_slice().as_ptr(),
        ..Default::default()
    };

    let image_info = vk::DescriptorImageInfo {
        sampler: texture.sampler,
        image_view: texture.image_view,
        image_layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    };

    unsafe {
        let descriptor_sets = devices
            .logical
            .device
            .allocate_descriptor_sets(&alloc_info)
            .expect("Failed to allocate descriptor sets!");

        for i in 0..swap_chain_image_count as usize {
            let buffer_info = vk::DescriptorBufferInfo {
                buffer: uniform_buffers[i].buffer,
                offset: 0,
                range: mem::size_of::<UniformBufferObject>() as u64,
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
                .device
                .update_descriptor_sets(&descriptor_writes, &[]);
        }

        descriptor_sets
    }
}
