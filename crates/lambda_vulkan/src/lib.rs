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
use command_buffer::VkCommander;
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
    // pub objects: VulkanObjects,
}

impl Vulkan {
    // pub fn new(
    //     display: &Display,
    //     camera: &Camera,
    //     vulkan_objects: &VulkanObjects,
    //     debugging: Option<DebugMessageProperties>,
    // ) -> Self {
    //     let entry_instance = EntryInstance::new(&display.window, &debugging);

    //     let debugger =
    //         debugging.map(|debug_properties| debug::debugger(&entry_instance, debug_properties));

    //     let surface = lambda_window::create_surface(
    //         &display.window,
    //         &entry_instance.instance,
    //         &entry_instance.entry,
    //     );

    //     let surface_loader = create_surface(&entry_instance);

    //     let devices = Devices::new(&entry_instance.instance, &surface, &surface_loader);

    //     let instance_devices = InstanceDevices::new(entry_instance.instance, devices);

    //     let swap_chain =
    //         SwapChain::new(&instance_devices, surface, &surface_loader, &display.window);

    //     let render_pass = renderer::create_render_pass(&instance_devices, &swap_chain);

    //     let resources = Resources::new(&swap_chain, &instance_devices);

    //     let frame_buffers = frame_buffer::create_frame_buffers(
    //         &swap_chain,
    //         &render_pass,
    //         &instance_devices,
    //         &resources,
    //     );

    //     let command_pool =
    //         command_buffer::create_command_pool(&instance_devices, &surface_loader, &surface);

    //     let sync_objects = SyncObjects::new(&instance_devices);

    //     let swap_chain_len = swap_chain.images.len() as u32;

    //     let vulkan_objects = models
    //         .iter_mut()
    //         .map(|property| {
    //             // dbg!(property.clone());

    //             property.deferred_build(
    //                 &command_pool,
    //                 swap_chain_len,
    //                 &swap_chain,
    //                 &render_pass,
    //                 &instance_devices,
    //             );

    //             property.vulkan_object()
    //         })
    //         .collect::<Vec<&VulkanObject>>();

    //     let command_buffers = command_buffer::create_command_buffers(
    //         &command_pool,
    //         &swap_chain,
    //         &instance_devices,
    //         &render_pass,
    //         &frame_buffers,
    //         &vulkan_objects,
    //     );

    //     let commander = VkCommander::new(command_buffers, command_pool);

    //     let ubo = UniformBufferObject::new(&swap_chain.extent, camera);

    //     Vulkan {
    //         commander,
    //         render_pass,
    //         resources,
    //         surface,
    //         surface_loader,
    //         swap_chain,
    //         sync_objects,
    //         ubo,
    //         debugger,
    //         frame_buffers,
    //         instance_devices,
    //     }
    // }
}

#[derive(Default, Debug, new)]
pub struct RenderPass(pub vk::RenderPass);

#[derive(Debug)]
pub struct VulkanObject {
    pub vertices_and_indices: VerticesAndIndices,
    pub texture: Option<Texture>,
    pub graphics_pipeline: GraphicsPipeline,
    pub buffers: ModelBuffers,
    pub indexed: bool,
    model: Matrix4<f32>,
}

impl Default for VulkanObject {
    fn default() -> Self {
        Self {
            vertices_and_indices: Default::default(),
            texture: Default::default(),
            graphics_pipeline: Default::default(),
            buffers: Default::default(),
            indexed: Default::default(),
            model: Matrix4::from_axis_angle(&Vector3::x_axis(), 0.0f32.to_radians()),
        }
    }
}

impl VulkanObject {
    pub fn new(indexed: bool) -> Self {
        Self {
            indexed,
            ..Default::default()
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

#[derive(Clone, Copy, Debug)]
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

#[derive(Debug, Clone, Copy)]
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
