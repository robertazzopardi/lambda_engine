use crate::{vector2, GeomBehavior, Geometry, VerticesAndIndices, WHITE};
use derive_builder::Builder;
use derive_more::{Deref, DerefMut};
use lambda_space::{
    space::{Orientation, Vertex, Vertices},
    vertex,
};
use lambda_vulkan::{
    buffer::ModelBuffers, command_buffer::CommandPool, graphics_pipeline::GraphicsPipeline,
    swap_chain::SwapChain, texture::Texture, utility::InstanceDevices, RenderPass, VulkanObject,
};
use nalgebra::{Point3, Vector3};

const SQUARE_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

#[derive(Builder, Default, Debug, Clone, new)]
#[builder(default, build_fn(skip))]
#[builder(name = "SquareBuilder")]
pub struct SquareInfo {
    pub position: Point3<f32>,
    pub orientation: Orientation,
    pub radius: f32,
    pub has_depth: bool,
}

impl SquareBuilder {
    pub fn build(&mut self) -> SquareInfo {
        let radius = self.radius.expect("Field `Radius` expected");

        SquareInfo {
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            radius,
            has_depth: self.has_depth.unwrap_or_default(),
        }
    }
}

#[derive(new, Deref, DerefMut, Debug, Clone)]
pub struct Square(Geometry<SquareInfo>);

impl GeomBehavior for Square {
    fn vertices_and_indices(&mut self) -> VerticesAndIndices {
        let mut vertices = square_from_vertices(&[
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
        ]);

        vertices.iter_mut().for_each(|vert| {
            vert.pos += self.properties.position.coords;
        });

        VerticesAndIndices::new(vertices, SQUARE_INDICES.to_vec().into())
    }

    fn vulkan_object(&self) -> VulkanObject {
        self.vulkan_object.clone()
    }

    fn deferred_build(
        &mut self,
        command_pool: &CommandPool,
        command_buffer_count: u32,
        swap_chain: &SwapChain,
        render_pass: &RenderPass,
        instance_devices: &InstanceDevices,
    ) {
        if !self.texture.is_empty() {
            self.vulkan_object.texture =
                Some(Texture::new(&self.texture, command_pool, instance_devices));
        }

        let vertices_and_indices = self.vertices_and_indices();

        self.vulkan_object.buffers = ModelBuffers::new(
            &vertices_and_indices,
            command_pool,
            command_buffer_count,
            instance_devices,
        );

        self.vulkan_object.graphics_pipeline = GraphicsPipeline::new(
            swap_chain,
            render_pass.0,
            &self.vulkan_object.texture,
            self.topology,
            self.cull_mode,
            instance_devices,
            self.shader,
        );

        self.vulkan_object.vertices_and_indices = vertices_and_indices;
    }
}

pub fn square_from_vertices(vertices: &[[f32; 3]]) -> Vertices {
    let tex_coord = [
        vector2!(1., 0.),
        vector2!(0., 0.),
        vector2!(0., 1.),
        vector2!(1., 1.),
    ];

    let mut tex_coords = Vec::new();
    for _ in 0..(vertices.len() / 4) {
        tex_coords.extend(tex_coord);
    }

    Vertices::new(
        vertices
            .iter()
            .enumerate()
            .map(|(index, vert)| {
                vertex!(
                    Point3::new(vert[0], vert[1], vert[2]),
                    WHITE,
                    Vector3::zeros(),
                    tex_coords[index]
                )
            })
            .collect::<Vec<Vertex>>(),
    )
}
