extern crate ash;
#[macro_use]
extern crate derive_new;
extern crate derive_builder;

pub mod buffer;
pub mod command_buffer;
pub mod debug;
pub mod device;
pub mod frame_buffer;
pub mod graphics_pipeline;
pub mod memory;
pub mod renderer;
pub mod resource;
pub mod swap_chain;
pub mod sync_objects;
pub mod texture;
pub mod uniform_buffer;
pub mod utility;

use ash::{extensions::khr::Surface, vk};
use buffer::ModelBuffers;
use command_buffer::{CommandPool, VkCommander};
use debug::{Debug, DebugMessageProperties};
use device::Devices;
use frame_buffer::FrameBuffers;
use graphics_pipeline::GraphicsPipeline;
use lambda_camera::prelude::Camera;
use lambda_space::space::VerticesAndIndices;
use lambda_window::prelude::Display;
use nalgebra::{Matrix4, Vector3};
use resource::Resources;
use swap_chain::SwapChain;
use sync_objects::SyncObjects;
use texture::Texture;
use uniform_buffer::UniformBufferObject;
use utility::{EntryInstance, InstanceDevices};

use crate::{debug::ENABLE_VALIDATION_LAYERS, sync_objects::MAX_FRAMES_IN_FLIGHT};

pub type VkTop = vk::PrimitiveTopology;
pub type VkCull = vk::CullModeFlags;
pub type Fence = vk::Fence;

pub mod prelude {
    pub use crate::{
        debug::{Debug, DebugMessageProperties, MessageLevel, MessageType},
        CullMode, ModelTopology, Shader,
    };
}

pub type VulkanObjects = Vec<VulkanObject>;

#[derive(Clone)]
pub struct Vulkan {
    pub commander: VkCommander,
    pub render_pass: RenderPass,
    pub resources: Resources,
    pub surface: vk::SurfaceKHR,
    pub surface_loader: Surface,
    pub swap_chain: SwapChain,
    pub sync_objects: SyncObjects,
    pub ubo: UniformBufferObject,
    pub debugger: Option<Debug>,
    pub frame_buffers: FrameBuffers,
    pub instance_devices: InstanceDevices,
    pub objects: VulkanObjects,
}

impl Vulkan {
    pub fn new(
        display: &Display,
        camera: &Camera,
        geom_properties: Vec<GeomProperties>,
        debugging: Option<DebugMessageProperties>,
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

        let instance_devices = InstanceDevices::new(entry_instance.instance, devices);

        let swap_chain =
            SwapChain::new(&instance_devices, surface, &surface_loader, &display.window);

        let render_pass = renderer::create_render_pass(&instance_devices, &swap_chain);

        let resources = Resources::new(&swap_chain, &instance_devices);

        let frame_buffers = frame_buffer::create_frame_buffers(
            &swap_chain,
            &render_pass,
            &instance_devices,
            &resources,
        );

        let command_pool =
            command_buffer::create_command_pool(&instance_devices, &surface_loader, &surface);

        let sync_objects = SyncObjects::new(&instance_devices);

        let swap_chain_len = swap_chain.images.len() as u32;

        let objects = geom_properties
            .into_iter()
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
            .collect::<Vec<VulkanObject>>();

        let command_buffers = command_buffer::create_command_buffers(
            &command_pool,
            &swap_chain,
            &instance_devices,
            &render_pass,
            &frame_buffers,
            &objects,
        );

        let commander = VkCommander::new(command_buffers, command_pool);

        let ubo = UniformBufferObject::new(&swap_chain.extent, camera);

        Self {
            commander,
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
}

impl Drop for Vulkan {
    fn drop(&mut self) {
        swap_chain::cleanup_swap_chain(self);

        unsafe {
            self.objects.iter().for_each(|object| {
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
                .destroy_command_pool(*self.commander.pool, None);

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

#[derive(Default, Debug, Clone, new)]
pub struct RenderPass(pub vk::RenderPass);

#[derive(Debug, Clone, new)]
pub struct GeomProperties<'a> {
    texture_buffer: &'a [u8],
    vertices_and_indices: VerticesAndIndices,
    topology: ModelTopology,
    cull_mode: CullMode,
    shader: Shader,
    indexed: bool,
}

#[derive(Debug, Clone)]
pub struct VulkanObject {
    pub vertices_and_indices: VerticesAndIndices,
    pub texture: Option<Texture>,
    pub graphics_pipeline: GraphicsPipeline,
    pub buffers: ModelBuffers,
    pub indexed: bool,
    model: Matrix4<f32>,
}

impl VulkanObject {
    fn new(
        command_pool: &CommandPool,
        command_buffer_count: u32,
        swap_chain: &SwapChain,
        render_pass: &RenderPass,
        instance_devices: &InstanceDevices,
        properties: GeomProperties,
    ) -> Self {
        let mut texture = None;
        if !properties.texture_buffer.is_empty() {
            texture = Some(Texture::new(
                properties.texture_buffer,
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
            vertices_and_indices: properties.vertices_and_indices,
            texture,
            graphics_pipeline,
            buffers,
            indexed: properties.indexed,
            model: Matrix4::from_axis_angle(&Vector3::x_axis(), 0.0f32.to_radians()),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct WindowSize(pub vk::Extent2D);

pub fn create_surface(entry_instance: &EntryInstance) -> Surface {
    Surface::new(&entry_instance.entry, &entry_instance.instance)
}

#[derive(Clone, Copy, Debug)]
pub enum ModelTopology {
    LineList,
    LineListWithAdjacency,
    LineStrip,
    LineStripWithAdjacency,
    PatchList,
    PointList,
    TriangleFan,
    TriangleList,
    TriangleListWithAdjacency,
    TriangleStrip,
    TriangleStripWithAdjacency,
}

impl From<ModelTopology> for VkTop {
    fn from(model_topology: ModelTopology) -> Self {
        match model_topology {
            ModelTopology::LineList => VkTop::LINE_LIST,
            ModelTopology::LineListWithAdjacency => VkTop::LINE_LIST_WITH_ADJACENCY,
            ModelTopology::LineStrip => VkTop::LINE_STRIP,
            ModelTopology::LineStripWithAdjacency => VkTop::LINE_STRIP_WITH_ADJACENCY,
            ModelTopology::PatchList => VkTop::PATCH_LIST,
            ModelTopology::PointList => VkTop::POINT_LIST,
            ModelTopology::TriangleFan => VkTop::TRIANGLE_FAN,
            ModelTopology::TriangleList => VkTop::TRIANGLE_LIST,
            ModelTopology::TriangleListWithAdjacency => VkTop::TRIANGLE_LIST_WITH_ADJACENCY,
            ModelTopology::TriangleStrip => VkTop::TRIANGLE_STRIP,
            ModelTopology::TriangleStripWithAdjacency => VkTop::TRIANGLE_STRIP_WITH_ADJACENCY,
        }
    }
}

impl Default for ModelTopology {
    fn default() -> Self {
        Self::TriangleList
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CullMode {
    Back,
    Front,
    FrontAndBack,
    None,
}

impl From<CullMode> for VkCull {
    fn from(model_cull_model: CullMode) -> Self {
        match model_cull_model {
            CullMode::Back => VkCull::BACK,
            CullMode::Front => VkCull::FRONT,
            CullMode::FrontAndBack => VkCull::FRONT_AND_BACK,
            CullMode::None => VkCull::NONE,
        }
    }
}

impl Default for CullMode {
    fn default() -> Self {
        Self::None
    }
}

const LIGHT: &str = "light";
const LIGHT_TEXTURE: &str = "light_texture";
const TEXTURE: &str = "texture";
const VERTEX: &str = "vertex";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Shader {
    Light,
    LightTexture,
    Texture,
    Vertex,
}

impl Default for Shader {
    fn default() -> Self {
        Self::Light
    }
}

impl From<Shader> for &str {
    fn from(texture_type: Shader) -> Self {
        match texture_type {
            Shader::Light => LIGHT,
            Shader::LightTexture => LIGHT_TEXTURE,
            Shader::Texture => TEXTURE,
            Shader::Vertex => VERTEX,
        }
    }
}
