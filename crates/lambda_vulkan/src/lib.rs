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

use crate::debug::Debug;
use ash::{extensions::khr::Surface, vk};
use buffer::ModelBuffers;
use command_buffer::VkCommander;
use frame_buffer::FrameBuffers;
use graphics_pipeline::GraphicsPipeline;
use lambda_space::space::VerticesAndIndices;
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
        ModelCullMode, ModelTopology, ShaderType,
    };
}

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
}

#[derive(Default, Debug, Clone, new)]
pub struct RenderPass(pub vk::RenderPass);

#[derive(Default, Debug, Clone, new)]
pub struct VulkanObject {
    pub vertices_and_indices: Option<VerticesAndIndices>,
    pub texture_buffer: Option<Texture>,
    pub graphics_pipeline: Option<GraphicsPipeline>,
    pub buffers: Option<ModelBuffers>,
    pub indexed: bool,
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
pub enum ModelCullMode {
    Back,
    Front,
    FrontAndBack,
    None,
}

impl From<ModelCullMode> for VkCull {
    fn from(model_cull_model: ModelCullMode) -> Self {
        match model_cull_model {
            ModelCullMode::Back => VkCull::BACK,
            ModelCullMode::Front => VkCull::FRONT,
            ModelCullMode::FrontAndBack => VkCull::FRONT_AND_BACK,
            ModelCullMode::None => VkCull::NONE,
        }
    }
}

impl Default for ModelCullMode {
    fn default() -> Self {
        Self::None
    }
}

const LIGHT: &str = "light";
const LIGHT_TEXTURE: &str = "light_texture";
const TEXTURE: &str = "texture";
const VERTEX: &str = "vertex";

#[derive(Debug, Clone, Copy)]
pub enum ShaderType {
    Light,
    LightTexture,
    Texture,
    Vertex,
}

impl Default for ShaderType {
    fn default() -> Self {
        Self::Light
    }
}

impl From<ShaderType> for &str {
    fn from(texture_type: ShaderType) -> Self {
        match texture_type {
            ShaderType::Light => LIGHT,
            ShaderType::LightTexture => LIGHT_TEXTURE,
            ShaderType::Texture => TEXTURE,
            ShaderType::Vertex => VERTEX,
        }
    }
}
