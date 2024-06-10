use crate::{
    buffer::Buffer,
    device::Devices,
    swap_chain::SwapChain,
    texture::{self, Texture},
    uniform_buffer::UniformBufferObject,
    utility::InstanceDevices,
    CullMode, ModelTopology, Shader,
};
use ash::{vk, Device};
use memoffset::offset_of;
use smallvec::{smallvec, SmallVec};
use std::{ffi::CStr, mem};
use wave_space::space::Vertex;

#[derive(Default, Debug, Clone)]
pub struct Descriptor {
    pub sets: Vec<vk::DescriptorSet>,
    pub pool: vk::DescriptorPool,
    pub set_layout: vk::DescriptorSetLayout,
}

#[derive(Default, Debug, Clone)]
pub struct GraphicsPipelineFeatures {
    pub pipeline: vk::Pipeline,
    pub layout: vk::PipelineLayout,
    pub cache: Option<vk::PipelineCache>,
}

impl GraphicsPipelineFeatures {
    pub fn new(
        pipeline: vk::Pipeline,
        layout: vk::PipelineLayout,
        cache: Option<vk::PipelineCache>,
    ) -> Self {
        Self {
            pipeline,
            layout,
            cache,
        }
    }
}

#[derive(Default, Debug, Clone)]
pub struct GraphicsPipeline {
    pub features: GraphicsPipelineFeatures,
    pub descriptors: Descriptor,
    pub topology: ModelTopology,
    pub cull_mode: CullMode,
    pub shader_type: Shader,
    pub uniform_buffers: Vec<Buffer>,
}

impl GraphicsPipeline {
    pub fn new(
        swap_chain: &SwapChain,
        render_pass: vk::RenderPass,
        texture: &Option<Texture>,
        topology: ModelTopology,
        cull_mode: CullMode,
        instance_devices: &InstanceDevices,
        shader_type: Shader,
    ) -> Self {
        let InstanceDevices { devices, .. } = instance_devices;

        let descriptor_set_layout =
            create_descriptor_set_layout(&devices.logical.device, shader_type);

        let features = create_pipeline_and_layout(
            devices,
            swap_chain,
            &descriptor_set_layout,
            render_pass,
            topology,
            cull_mode,
            shader_type,
        );

        let descriptor_pool =
            create_descriptor_pool(&devices.logical.device, swap_chain.images.len() as u32);

        let uniform_buffers =
            create_uniform_buffers(swap_chain.images.len() as u32, instance_devices);

        let descriptor_sets = if shader_type == Shader::PushConstant {
            create_descriptor_set(
                &devices.logical.device,
                descriptor_pool,
                texture.as_ref().unwrap(),
            )
        } else {
            create_descriptor_sets(
                &devices.logical.device,
                descriptor_set_layout,
                descriptor_pool,
                swap_chain.images.len(),
                texture,
                &uniform_buffers,
            )
        };

        let descriptors = Descriptor {
            sets: descriptor_sets,
            pool: descriptor_pool,
            set_layout: descriptor_set_layout,
        };

        Self {
            features,
            descriptors,
            topology,
            cull_mode,
            shader_type,
            uniform_buffers,
        }
    }

    pub fn recreate(
        &self,
        instance_devices: &InstanceDevices,
        swap_chain: &SwapChain,
        render_pass: vk::RenderPass,
        texture: &Option<Texture>,
    ) -> Self {
        let InstanceDevices { devices, .. } = instance_devices;

        let descriptor_set_layout =
            create_descriptor_set_layout(&devices.logical.device, self.shader_type);

        let features = create_pipeline_and_layout(
            devices,
            swap_chain,
            &descriptor_set_layout,
            render_pass,
            self.topology,
            self.cull_mode,
            self.shader_type,
        );

        let descriptor_pool =
            create_descriptor_pool(&devices.logical.device, swap_chain.images.len() as u32);

        let uniform_buffers =
            create_uniform_buffers(swap_chain.images.len() as u32, instance_devices);

        let descriptor_sets = create_descriptor_sets(
            &devices.logical.device,
            descriptor_set_layout,
            descriptor_pool,
            swap_chain.images.len(),
            texture,
            &uniform_buffers,
        );

        let descriptor_set = Descriptor {
            sets: descriptor_sets,
            pool: descriptor_pool,
            set_layout: descriptor_set_layout,
        };

        Self {
            features,
            uniform_buffers,
            descriptors: descriptor_set,
            topology: self.topology,
            cull_mode: self.cull_mode,
            shader_type: self.shader_type,
        }
    }
}

fn create_descriptor_set_layout(device: &Device, shader_type: Shader) -> vk::DescriptorSetLayout {
    let mut bindings: SmallVec<[vk::DescriptorSetLayoutBinding; 2]> = smallvec![
        vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .stage_flags(vk::ShaderStageFlags::VERTEX),
        vk::DescriptorSetLayoutBinding::default()
            .binding(1)
            .descriptor_count(1)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT),
    ];
    if shader_type == Shader::PushConstant {
        bindings = smallvec![vk::DescriptorSetLayoutBinding::default()
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .binding(0)
            .descriptor_count(1)];
    };

    let layout_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings);

    unsafe {
        device
            .create_descriptor_set_layout(&layout_info, None)
            .expect("Failed to create descriptor set layout")
    }
}

fn create_descriptor_pool(device: &Device, swap_chain_image_count: u32) -> vk::DescriptorPool {
    let pool_sizes = &[
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::UNIFORM_BUFFER)
            .descriptor_count(swap_chain_image_count),
        vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(swap_chain_image_count),
    ];

    let pool_info = vk::DescriptorPoolCreateInfo::default()
        .pool_sizes(pool_sizes)
        .max_sets(swap_chain_image_count);

    unsafe {
        device
            .create_descriptor_pool(&pool_info, None)
            .expect("Failed to create descriptor pool!")
    }
}

fn create_uniform_buffers(
    swap_chain_image_count: u32,
    instance_devices: &InstanceDevices,
) -> Vec<Buffer> {
    let mut buffers = Vec::new();

    let buffer = texture::create_buffer(
        mem::size_of::<UniformBufferObject>().try_into().unwrap(),
        vk::BufferUsageFlags::UNIFORM_BUFFER,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
        instance_devices,
    );

    for _ in 0..swap_chain_image_count {
        buffers.push(buffer)
    }

    buffers
}

fn create_shader_module(device: &Device, path: &str) -> vk::ShaderModule {
    let mut file = std::fs::File::open(path).unwrap();
    let spv = ash::util::read_spv(&mut file).unwrap();

    let create_info = vk::ShaderModuleCreateInfo::default().code(&spv);

    unsafe {
        device
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
    cull_mode: CullMode,
    shader_type: Shader,
) -> GraphicsPipelineFeatures {
    let device = &devices.logical.device;

    let shader_modules = create_shader_stages(shader_type, device);

    let binding_description = vk::VertexInputBindingDescription::default()
        .binding(0)
        .stride(mem::size_of::<Vertex>().try_into().unwrap())
        .input_rate(vk::VertexInputRate::VERTEX);

    let mut attribute_descriptions: SmallVec<[vk::VertexInputAttributeDescription; 2]> = smallvec![
        vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(Vertex, pos) as u32),
        vk::VertexInputAttributeDescription::default()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(offset_of!(Vertex, colour) as u32),
    ];

    if shader_type != Shader::Vertex && shader_type != Shader::PushConstant {
        attribute_descriptions.extend([
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(2)
                .format(vk::Format::R32G32B32_SFLOAT)
                .offset(offset_of!(Vertex, normal) as u32),
            vk::VertexInputAttributeDescription::default()
                .binding(0)
                .location(3)
                .format(vk::Format::R32G32_SFLOAT)
                .offset(offset_of!(Vertex, tex_coord) as u32),
        ]);
    }

    let vertex_input_info = vk::PipelineVertexInputStateCreateInfo::default()
        .vertex_binding_descriptions(std::slice::from_ref(&binding_description))
        .vertex_attribute_descriptions(&attribute_descriptions);

    let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
        .topology(topology.into())
        .primitive_restart_enable(false);

    let view_port = vk::Viewport::default()
        .x(0.)
        .y(0.)
        .width(swap_chain.extent.width as f32)
        .height(swap_chain.extent.height as f32)
        .min_depth(0.)
        .max_depth(1.);

    let scissor = vk::Rect2D::default()
        .offset(vk::Offset2D::default())
        .extent(swap_chain.extent);

    let view_port_state = vk::PipelineViewportStateCreateInfo::default()
        .viewports(std::slice::from_ref(&view_port))
        .scissors(std::slice::from_ref(&scissor));

    let rasterizer = vk::PipelineRasterizationStateCreateInfo::default()
        .depth_clamp_enable(false)
        .rasterizer_discard_enable(false)
        .polygon_mode(vk::PolygonMode::FILL)
        // .polygon_mode(vk::PolygonMode::LINE)
        // .polygon_mode(vk::PolygonMode::POINT)
        // .polygon_mode(vk::PolygonMode::FILL_RECTANGLE_NV)
        .line_width(1.)
        .cull_mode(cull_mode.into())
        .front_face(vk::FrontFace::COUNTER_CLOCKWISE)
        .depth_bias_enable(false);

    let multi_sampling = vk::PipelineMultisampleStateCreateInfo::default()
        .rasterization_samples(devices.physical.samples)
        .sample_shading_enable(true)
        .min_sample_shading(0.2)
        .alpha_to_coverage_enable(false)
        .alpha_to_one_enable(false);

    let stencil_state = vk::StencilOpState::default()
        .fail_op(vk::StencilOp::KEEP)
        .pass_op(vk::StencilOp::KEEP)
        .depth_fail_op(vk::StencilOp::KEEP)
        .compare_op(vk::CompareOp::ALWAYS)
        .compare_mask(0)
        .write_mask(0)
        .reference(0);

    let depth_stencil = vk::PipelineDepthStencilStateCreateInfo::default()
        .depth_test_enable(true)
        .depth_write_enable(true)
        .depth_compare_op(vk::CompareOp::LESS)
        .depth_bounds_test_enable(false)
        .stencil_test_enable(false)
        .front(stencil_state)
        .back(stencil_state)
        .min_depth_bounds(0.)
        .max_depth_bounds(1.);

    let color_blend_attachment = vk::PipelineColorBlendAttachmentState::default()
        .color_write_mask(vk::ColorComponentFlags::RGBA)
        .blend_enable(true)
        .src_color_blend_factor(vk::BlendFactor::SRC_ALPHA)
        .dst_color_blend_factor(vk::BlendFactor::ONE_MINUS_SRC_ALPHA)
        .color_blend_op(vk::BlendOp::ADD)
        .src_alpha_blend_factor(vk::BlendFactor::ONE)
        .dst_alpha_blend_factor(vk::BlendFactor::ZERO)
        .alpha_blend_op(vk::BlendOp::ADD);

    let color_blending = vk::PipelineColorBlendStateCreateInfo::default()
        .logic_op_enable(false)
        .logic_op(vk::LogicOp::COPY)
        .attachments(std::slice::from_ref(&color_blend_attachment))
        .blend_constants([0., 0., 0., 0.]);

    let mut pipeline_layout_info = vk::PipelineLayoutCreateInfo::default()
        .set_layouts(std::slice::from_ref(descriptor_set_layout));

    let push_constant_range = vk::PushConstantRange::default()
        .stage_flags(vk::ShaderStageFlags::VERTEX)
        // .size(std::mem::size_of::<Push>().try_into().unwrap())
        .size(64)
        .offset(0);
    if shader_type == Shader::PushConstant {
        pipeline_layout_info =
            pipeline_layout_info.push_constant_ranges(std::slice::from_ref(&push_constant_range));
    }

    let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];

    let dynamic_state_create_info =
        vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);

    unsafe {
        let layout = device
            .create_pipeline_layout(&pipeline_layout_info, None)
            .expect("Failed to create pipeline layout!");

        let entry_point = &CStr::from_bytes_with_nul(b"main\0").unwrap();
        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(shader_modules.vert)
                .name(entry_point),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(shader_modules.frag)
                .name(entry_point),
        ];

        let pipeline_info = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
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

        let mut pipeline_cache = None;
        if shader_type == Shader::PushConstant {
            let pipeline_cache_create_info = vk::PipelineCacheCreateInfo::default();
            pipeline_cache = Some(
                device
                    .create_pipeline_cache(&pipeline_cache_create_info, None)
                    .expect("Could not create pipeline cache"),
            );
        }

        let pipeline = if let Some(cache) = pipeline_cache {
            device
                .create_graphics_pipelines(cache, std::slice::from_ref(&pipeline_info), None)
                .expect("Failed to create graphics pipeline!")
        } else {
            device
                .create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    std::slice::from_ref(&pipeline_info),
                    None,
                )
                .expect("Failed to create graphics pipeline!")
        };

        destroy_shader_modules(device, shader_modules.vert, shader_modules.frag);

        GraphicsPipelineFeatures::new(pipeline[0], layout, pipeline_cache)
    }
}

#[inline]
pub unsafe fn destroy_shader_modules(
    device: &Device,
    vert_shader_module: vk::ShaderModule,
    frag_shader_module: vk::ShaderModule,
) {
    device.destroy_shader_module(vert_shader_module, None);
    device.destroy_shader_module(frag_shader_module, None);
}

pub(crate) struct ShaderModules {
    pub vert: vk::ShaderModule,
    pub frag: vk::ShaderModule,
}

pub(crate) fn create_shader_stages(shader_type: Shader, device: &Device) -> ShaderModules {
    let shader_folder: &str = shader_type.into();

    let vert = create_shader_module(
        device,
        &format!("./crates/wave_internal/src/shaders/{shader_folder}/vert.spv"),
    );
    let frag = create_shader_module(
        device,
        &format!("./crates/wave_internal/src/shaders/{shader_folder}/frag.spv"),
    );

    ShaderModules { vert, frag }
}

fn create_descriptor_set(
    device: &Device,
    descriptor_pool: vk::DescriptorPool,
    texture: &Texture,
) -> Vec<vk::DescriptorSet> {
    // Descriptor Set Layout
    let set_layout_bindings = vk::DescriptorSetLayoutBinding::default()
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .stage_flags(vk::ShaderStageFlags::FRAGMENT)
        .binding(0)
        .descriptor_count(1);
    let descriptor_layout = vk::DescriptorSetLayoutCreateInfo::default()
        .bindings(std::slice::from_ref(&set_layout_bindings));
    let descriptor_set_layout = unsafe {
        device
            .create_descriptor_set_layout(&descriptor_layout, None)
            .expect("Could not create descriptor set layout!")
    };

    // Descriptor Set
    let descriptor_set_alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(std::slice::from_ref(&descriptor_set_layout));
    let descriptor_set = unsafe {
        device
            .allocate_descriptor_sets(&descriptor_set_alloc_info)
            .expect("Could not allocate descriptor set!")[0]
    };
    let font_descriptor = vk::DescriptorImageInfo::default()
        .sampler(texture.sampler)
        .image_view(texture.view)
        .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    let write_descriptor_set = vk::WriteDescriptorSet::default()
        .dst_set(descriptor_set)
        .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
        .dst_binding(0)
        .image_info(std::slice::from_ref(&font_descriptor));
    unsafe { device.update_descriptor_sets(std::slice::from_ref(&write_descriptor_set), &[]) };

    std::slice::from_ref(&descriptor_set).to_vec()
}

fn create_descriptor_sets(
    device: &Device,
    descriptor_layout: vk::DescriptorSetLayout,
    descriptor_pool: vk::DescriptorPool,
    swap_chain_image_count: usize,
    texture: &Option<Texture>,
    uniform_buffers: &[Buffer],
) -> Vec<vk::DescriptorSet> {
    let layouts = vec![descriptor_layout; swap_chain_image_count];

    let alloc_info = vk::DescriptorSetAllocateInfo::default()
        .descriptor_pool(descriptor_pool)
        .set_layouts(&layouts);

    let image_info = texture.as_ref().map(|texture| {
        vk::DescriptorImageInfo::default()
            .sampler(texture.sampler)
            .image_view(texture.view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
    });

    let descriptor_sets = unsafe {
        device
            .allocate_descriptor_sets(&alloc_info)
            .expect("Failed to allocate descriptor sets!")
    };

    for i in 0..swap_chain_image_count {
        let buffer_info = vk::DescriptorBufferInfo::default()
            .buffer(uniform_buffers[i].buffer)
            .offset(0)
            .range(mem::size_of::<UniformBufferObject>().try_into().unwrap());

        let mut descriptor_writes: SmallVec<[vk::WriteDescriptorSet; 2]> =
            smallvec![vk::WriteDescriptorSet::default()
                .dst_set(descriptor_sets[i])
                .dst_binding(0)
                .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
                .buffer_info(std::slice::from_ref(&buffer_info))];

        if let Some(ref image_info) = image_info {
            descriptor_writes.push(
                vk::WriteDescriptorSet::default()
                    .dst_set(descriptor_sets[i])
                    .dst_binding(1)
                    .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
                    .image_info(std::slice::from_ref(&image_info)),
            )
        }

        unsafe {
            device.update_descriptor_sets(&descriptor_writes, &[]);
        }
    }

    descriptor_sets
}
