use crate::{
    utility::{self, calculate_indices},
    GeomBehavior, Geometry, VerticesAndIndices, WHITE,
};
use derive_builder::Builder;
use derive_more::{Deref, DerefMut};
use lambda_space::{
    space::{Coordinate3, Orientation, Vertices},
    vertex,
};
use lambda_vulkan::{
    buffer::ModelBuffers, command_buffer::CommandPool, graphics_pipeline::GraphicsPipeline,
    swap_chain::SwapChain, texture::Texture, utility::InstanceDevices, RenderPass, VulkanObject,
};
use nalgebra::{Point3, Vector2, Vector3};

#[derive(Builder, Default, Debug, Clone, new)]
#[builder(default, build_fn(skip))]
#[builder(name = "ModelBuilder")]
pub struct ModelInfo {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub radius: f32,
    model_path: String,
}

impl ModelBuilder {
    pub fn build(&mut self) -> ModelInfo {
        let radius = self.radius.expect("Field `Radius` expected");
        let model_path = self.model_path.take().expect("Field `model_path` expected");

        ModelInfo {
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            radius,
            model_path,
        }
    }
}

#[derive(new, Deref, DerefMut, Debug, Clone)]
pub struct Model(Geometry<ModelInfo>);

impl GeomBehavior for Model {
    fn vertices_and_indices(&mut self) -> VerticesAndIndices {
        let mut vertices_and_indices = load_model_obj(self.properties.model_path.to_string());

        // vertices_and_indices.vertices.iter_mut().for_each(|vert| {
        //     vert.pos += self.properties.position.coords;
        // });

        vertices_and_indices
            .vertices
            .chunks_mut(4)
            .for_each(|face| {
                utility::scale(face, self.properties.radius);
            });

        vertices_and_indices
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

fn load_model_obj(model_path: String) -> VerticesAndIndices {
    let (models, _materials) =
        tobj::load_obj(model_path, &tobj::GPU_LOAD_OPTIONS).expect("Failed to OBJ load file");

    let mut vertices = Vertices::default();

    models.into_iter().for_each(|tobj::Model { mesh, .. }| {
        mesh.indices.iter().for_each(|index| {
            let i = *index as usize;

            let pos = Point3::new(
                mesh.positions[i * 3],
                mesh.positions[i * 3 + 1],
                mesh.positions[i * 3 + 2],
            );

            let normal = Vector3::new(
                mesh.normals[i * 3],
                mesh.normals[i * 3 + 1],
                mesh.normals[i * 3 + 2],
            );

            let texcoord = Vector2::new(mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]);

            let vertex = vertex!(pos, WHITE, normal, texcoord);

            vertices.push(vertex);
        });
    });

    let indices = calculate_indices(&vertices);

    VerticesAndIndices::new(vertices, indices)
}
