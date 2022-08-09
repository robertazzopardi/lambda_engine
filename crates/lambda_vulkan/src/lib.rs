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
use ash::{extensions::khr::Surface, vk, Device};
use buffer::{Buffer, ModelBuffers};
use command_buffer::{CommandBuffers, CommandPool};
use debug::{Debug, Debugger};
use derive_more::{Deref, DerefMut};
use device::Devices;
use frame_buffer::FrameBuffers;
use graphics_pipeline::GraphicsPipeline;
use imgui::{Condition, Context, DrawCmd, DrawCmdParams, DrawData, DrawIdx, DrawVert, Ui, Window};
use lambda_camera::prelude::CameraInternal;
use lambda_space::space::{Vertex, VerticesAndIndices};
use lambda_window::prelude::Display;
use nalgebra::{matrix, Matrix4, Vector3};
use renderer::RenderPass;
use resource::Resources;
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
    texture: Option<Texture>,
    // pipeline_cache: vk::PipelineCache,
    // pipeline_layout: vk::PipelineLayout,
    // pipeline: vk::Pipeline,
    // descriptor_pool: vk::DescriptorPool,
    // descriptor_set_layout: vk::DescriptorSetLayout,
    // descriptor_set: vk::DescriptorSet,
    graphics_pipeline: GraphicsPipeline,
    // command_buffer: vk::CommandBuffer,
}

pub struct ImGui {
    pub context: Context,
    gui_vk: GuiVk,
}

impl ImGui {
    fn object(
        command_pool: &CommandPool,
        swap_chain_len: u32,
        swap_chain: &SwapChain,
        render_pass: &RenderPass,
        instance_devices: &InstanceDevices,
    ) -> VulkanObject {
        let mut context = Context::create();
        context.set_ini_filename(None);

        let style = context.style_mut();
        style.use_classic_colors();
        let io = context.io_mut();
        io.display_size = [300., 100.];
        io.display_framebuffer_scale = [1., 1.];

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

        let image_properties = ImageProperties::new((width, height), &data, 1, upload_size as u64);
        let image_info = ImageInfo::new(
            (width, height),
            1,
            vk::SampleCountFlags::TYPE_1,
            vk::Format::R8G8B8A8_UNORM,
            vk::ImageTiling::OPTIMAL,
            vk::ImageUsageFlags::SAMPLED
                | vk::ImageUsageFlags::TRANSFER_DST
                | vk::ImageUsageFlags::COLOR_ATTACHMENT,
            vk::MemoryPropertyFlags::DEVICE_LOCAL,
        );
        let texture = Some(Texture::new(
            image_properties,
            command_pool,
            instance_devices,
            vk::Format::R8G8B8A8_UNORM,
            image_info,
        ));

        let ui = context.frame();
        ImGui::new_frame(&ui);
        let draw_data = ui.render();

        let mut vertices: Vec<Vertex> = vec![];
        let mut indices = vec![];
        for draw_list in draw_data.draw_lists() {
            let vtx_buffer = draw_list.vtx_buffer();
            let idx_buffer = draw_list.idx_buffer();

            for vtx in vtx_buffer.iter() {
                let m = *vtx;
                vertices.push(m.into())
            }
            indices.extend(idx_buffer);
        }

        let vertices_and_indices = VerticesAndIndices::new(vertices.into(), indices.into());

        VulkanObject::new(
            command_pool,
            swap_chain_len,
            swap_chain,
            render_pass,
            instance_devices,
            &GeomProperties {
                texture_buffer: &data,
                vertices_and_indices,
                topology: ModelTopology::TriangleList,
                cull_mode: CullMode::None,
                shader: Shader::Vertex,
                indexed: true,
                // model: orthographic_vk(
                //     0.0,
                //     draw_data.display_size[0],
                //     0.0,
                //     -draw_data.display_size[1],
                //     -1.0,
                //     1.0,
                // ),
                model: Matrix4::from_axis_angle(&Vector3::x_axis(), 0.0f32.to_radians()),
            },
            texture,
        )
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

        // dbg!(draw_data.draw_lists_count());
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
                gui_vk.graphics_pipeline.features.layout,
                0,
                std::slice::from_ref(&gui_vk.graphics_pipeline.descriptors.sets[0]),
                &[],
            );
            device.cmd_bind_pipeline(
                command_buffer,
                vk::PipelineBindPoint::GRAPHICS,
                gui_vk.graphics_pipeline.features.pipeline,
            )
        };

        let framebuffer_width = draw_data.framebuffer_scale[0] * draw_data.display_size[0];
        let framebuffer_height = draw_data.framebuffer_scale[1] * draw_data.display_size[1];
        let viewports = vk::Viewport::builder()
            .width(framebuffer_width)
            .height(framebuffer_height)
            .max_depth(1.0);

        unsafe { device.cmd_set_viewport(command_buffer, 0, std::slice::from_ref(&viewports)) };

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
                gui_vk.graphics_pipeline.features.layout,
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
            vertex_buffer,
            index_buffer,
            texture,
            graphics_pipeline,
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
        if let Some(texture) = texture {
            device.destroy_image(texture.image.image, None);
            device.destroy_image_view(texture.view, None);
            device.free_memory(texture.image.memory, None);
            device.destroy_sampler(texture.sampler, None);
        }
        if let Some(cache) = graphics_pipeline.features.cache {
            device.destroy_pipeline_cache(cache, None);
        }
        device.destroy_pipeline(graphics_pipeline.features.pipeline, None);
        device.destroy_pipeline_layout(graphics_pipeline.features.layout, None);
        device.destroy_descriptor_pool(graphics_pipeline.descriptors.pool, None);
        device.destroy_descriptor_set_layout(graphics_pipeline.descriptors.set_layout, None);
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

#[derive(Debug, Deref, DerefMut)]
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
}

impl Vulkan {
    #[inline]
    pub fn wait_device_idle(&self) {
        let device = &self.instance_devices.devices.logical.device;
        unsafe {
            device
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

        let mut objects = VulkanObjects(
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
                        property.create_texture(&command_pool, &instance_devices),
                    )
                })
                .collect(),
        );

        let ubo = UniformBufferObject::new(&swap_chain.extent, camera);

        objects.0.push(ImGui::object(
            &command_pool,
            swap_chain_len,
            &swap_chain,
            &render_pass,
            &instance_devices,
        ));

        // dbg!(&objects);

        let command_buffers = command_buffer::create_command_buffers(
            &command_pool,
            &swap_chain,
            &instance_devices,
            &render_pass,
            &frame_buffers,
            &objects.0,
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
        }
    }

    #[inline]
    pub fn update_objects(&mut self, properties: &[GeomProperties]) {
        self.objects
            .0
            .iter_mut()
            .zip(properties)
            .for_each(|(object, properties)| object.model = properties.model);
    }
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        swap_chain::cleanup_swap_chain(self);

        let device = &self.instance_devices.devices.logical.device;

        unsafe {
            self.objects.0.iter().for_each(|object| {
                device::recreate_drop(&object.graphics_pipeline, device);
                device::destroy(object, device);
            });

            for i in 0..MAX_FRAMES_IN_FLIGHT {
                device.destroy_semaphore(self.sync_objects.render_finished_semaphores[i], None);
                device.destroy_semaphore(self.sync_objects.image_available_semaphores[i], None);
                device.destroy_fence(self.sync_objects.in_flight_fences[i], None);
            }

            device.destroy_command_pool(*self.command_pool, None);

            dbg!("2");

            device.destroy_device(None);

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

impl<'a> GeomProperties<'a> {
    fn create_texture(
        &self,
        command_pool: &CommandPool,
        instance_devices: &InstanceDevices,
    ) -> Option<Texture> {
        let mut texture = None;
        if !self.texture_buffer.is_empty() {
            let image_properties =
                ImageProperties::get_image_properties_from_buffer(self.texture_buffer);
            let image_info = ImageInfo::new(
                image_properties.image_dimensions,
                image_properties.mip_levels,
                vk::SampleCountFlags::TYPE_1,
                vk::Format::R8G8B8A8_SRGB,
                vk::ImageTiling::OPTIMAL,
                vk::ImageUsageFlags::TRANSFER_SRC
                    | vk::ImageUsageFlags::TRANSFER_DST
                    | vk::ImageUsageFlags::SAMPLED,
                vk::MemoryPropertyFlags::DEVICE_LOCAL,
            );
            texture = Some(Texture::new(
                image_properties,
                command_pool,
                instance_devices,
                vk::Format::R8G8B8A8_SRGB,
                image_info,
            ));
        }
        texture
    }
}

#[derive(Debug)]
pub(crate) struct VulkanObject {
    vertices_and_indices: VerticesAndIndices,
    texture: Option<Texture>,
    graphics_pipeline: GraphicsPipeline,
    buffers: ModelBuffers,
    indexed: bool,
    model: Matrix4<f32>,
    shader: Shader,
}

impl VulkanObject {
    fn new(
        command_pool: &CommandPool,
        command_buffer_count: u32,
        swap_chain: &SwapChain,
        render_pass: &RenderPass,
        instance_devices: &InstanceDevices,
        properties: &GeomProperties,
        texture: Option<Texture>,
    ) -> Self {
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
            shader: properties.shader,
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
const UI: &str = "ui";
// const PUSH_CONSTANT: &str = "push_constant";
/// For now ui and push constant will be the same
const PUSH_CONSTANT: &str = UI;

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
