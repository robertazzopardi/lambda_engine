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
use ash::{khr::surface, vk, Device};
use buffer::{Buffer, ModelBuffers};
use command_buffer::{CommandBuffers, CommandPool};
use debug::{Debug, Debugger};
use derive_more::{Deref, DerefMut};
use device::Devices;
use frame_buffer::FrameBuffers;
use graphics_pipeline::GraphicsPipeline;
use nalgebra::{matrix, Matrix4, Vector3};
use renderer::RenderPass;
use resource::Resources;
use swap_chain::{recreate_swap_chain, SwapChain};
use sync_objects::SyncObjects;
use texture::{create_buffer, ImageProperties, Texture};
use uniform_buffer::{update_uniform_buffers, UniformBufferObject};
use utility::{EntryInstance, ImageInfo, InstanceDevices};
use wave_camera::prelude::CameraInternal;
use wave_space::space::{Vertex, VerticesAndIndices};
use wave_window::{prelude::Display, window::RenderBackend};
use winit::window::Window;

pub mod prelude {
    pub use crate::{
        debug::{Debugger, MessageLevel, MessageType},
        CullMode, ModelTopology, Shader, TextureBuffer,
    };
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
    pub(crate) surface_loader: surface::Instance,
    pub swap_chain: SwapChain,
    pub(crate) sync_objects: SyncObjects,
    pub ubo: UniformBufferObject,
    pub(crate) debugger: Option<Debug>,
    pub(crate) frame_buffers: FrameBuffers,
    pub instance_devices: InstanceDevices,
    pub(crate) objects: VulkanObjects,
}

impl RenderBackend for Vulkan {
    fn create(window: &Window) -> Self
    where
        Self: Sized,
    {
        Self::new(window, &[], None)
    }

    fn destroy(&self) {
        self.wait_device_idle();
    }

    fn update(&mut self, view: Matrix4<f32>) {
        self.ubo.update(&self.swap_chain.extent, view);
    }

    fn render(&mut self, window: &Window, current_frame: &mut usize, resized: &mut bool, dt: f32) {
        let device = &mut self.instance_devices.devices.logical.device;

        let image_available_semaphore =
            self.sync_objects.image_available_semaphores[*current_frame];
        let in_flight_fence = self.sync_objects.in_flight_fences[*current_frame];
        let render_finished_semaphore =
            self.sync_objects.render_finished_semaphores[*current_frame];

        unsafe {
            device
                .wait_for_fences(
                    &self.sync_objects.in_flight_fences,
                    true,
                    vk::DeviceSize::MAX,
                )
                .expect("Failed to wait for Fence!");

            let (image_index, _is_sub_optimal) = {
                let result = self.swap_chain.swap_chain.acquire_next_image(
                    self.swap_chain.swap_chain_khr,
                    vk::DeviceSize::MAX,
                    image_available_semaphore,
                    vk::Fence::null(),
                );
                match result {
                    Ok(image_index) => image_index,
                    Err(vk_result) => match vk_result {
                        vk::Result::ERROR_OUT_OF_DATE_KHR => {
                            recreate_swap_chain(self, window);
                            return;
                        }
                        _ => panic!("Failed to acquire Swap Chain vk::Image!"),
                    },
                }
            };

            update_uniform_buffers(
                device,
                &mut self.objects,
                &self.ubo,
                image_index.try_into().unwrap(),
                dt,
            );

            if self.sync_objects.images_in_flight[image_index as usize] != vk::Fence::null() {
                device
                    .wait_for_fences(
                        std::slice::from_ref(
                            &self.sync_objects.images_in_flight[image_index as usize],
                        ),
                        true,
                        vk::DeviceSize::MAX,
                    )
                    .expect("Could not wait for images in flight");
            }

            self.sync_objects.images_in_flight[image_index as usize] = in_flight_fence;

            let submit_infos = vk::SubmitInfo::default()
                .wait_semaphores(std::slice::from_ref(&image_available_semaphore))
                .wait_dst_stage_mask(std::slice::from_ref(
                    &vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                ))
                .command_buffers(std::slice::from_ref(
                    &self.command_buffers[image_index as usize],
                ))
                .signal_semaphores(std::slice::from_ref(&render_finished_semaphore));

            device
                .reset_fences(std::slice::from_ref(&in_flight_fence))
                .expect("Failed to reset Fence!");

            device
                .queue_submit(
                    self.instance_devices.devices.logical.queues.present,
                    std::slice::from_ref(&submit_infos),
                    in_flight_fence,
                )
                .expect("Failed to execute queue submit.");

            let present_info = vk::PresentInfoKHR::default()
                .wait_semaphores(std::slice::from_ref(&render_finished_semaphore))
                .swapchains(std::slice::from_ref(&self.swap_chain.swap_chain_khr))
                .image_indices(std::slice::from_ref(&image_index));

            let result = self.swap_chain.swap_chain.queue_present(
                self.instance_devices.devices.logical.queues.present,
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
                recreate_swap_chain(self, window);
            }

            *current_frame = (*current_frame + 1) % MAX_FRAMES_IN_FLIGHT;
        }
    }
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
        window: &Window,
        geom_properties: &[GeomProperties],
        debugging: Option<Debugger>,
    ) -> Self {
        let entry_instance = EntryInstance::new(window, debugging);

        let debugger = if cfg!(debug_assertions) {
            debugging.map(|debugging| debug::debugger(&entry_instance, debugging))
        } else {
            None
        };

        let surface =
            wave_window::create_surface(window, &entry_instance.instance, &entry_instance.entry);

        let surface_loader = create_surface(&entry_instance);

        let devices = Devices::new(&entry_instance.instance, &surface, &surface_loader);

        let instance_devices = InstanceDevices {
            instance: entry_instance.instance,
            devices,
        };

        let swap_chain = SwapChain::new(&instance_devices, surface, &surface_loader, window);

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

        let ubo = UniformBufferObject::default();

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

#[derive(Debug, Clone)]
pub struct GeomProperties {
    texture_buffer: Vec<u8>,
    vertices_and_indices: VerticesAndIndices,
    topology: ModelTopology,
    cull_mode: CullMode,
    shader: Shader,
    indexed: bool,
    model: Matrix4<f32>,
}

impl GeomProperties {
    pub fn new(
        texture_buffer: &[u8],
        vertices_and_indices: VerticesAndIndices,
        topology: ModelTopology,
        cull_mode: CullMode,
        shader: Shader,
        indexed: bool,
        model: Matrix4<f32>,
    ) -> Self {
        Self {
            texture_buffer: texture_buffer.to_vec(),
            vertices_and_indices,
            topology,
            cull_mode,
            shader,
            indexed,
            model,
        }
    }

    fn create_texture(
        &self,
        command_pool: &CommandPool,
        instance_devices: &InstanceDevices,
    ) -> Option<Texture> {
        let mut texture = None;
        if !self.texture_buffer.is_empty() {
            let image_properties =
                ImageProperties::get_image_properties_from_buffer(&self.texture_buffer);
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
pub(crate) fn create_surface(entry_instance: &EntryInstance) -> surface::Instance {
    surface::Instance::new(&entry_instance.entry, &entry_instance.instance)
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
