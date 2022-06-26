use crate::{
    utility::{self, calculate_indices},
    VerticesAndIndices, WHITE,
};
use derive_builder::Builder;
use lambda_space::{
    space::{Coordinate3, Orientation, Vertices},
    vertex,
};
use nalgebra::{Matrix4, Point3, Vector2, Vector3};

#[derive(Builder, Default, Debug, Clone)]
#[builder(default, build_fn(skip))]
pub struct Model {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub radius: f32,
    model_path: &'static str,
    pub model: Matrix4<f32>,
}

impl ModelBuilder {
    pub fn build(&mut self) -> Model {
        Model {
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            radius: self.radius.expect("Field `Radius` expected"),
            model_path: self.model_path.take().expect("Field `model_path` expected"),
            model: Matrix4::from_axis_angle(&Vector3::x_axis(), 0.0f32.to_radians()),
        }
    }
}

impl Model {
    pub fn vertices_and_indices(&self) -> VerticesAndIndices {
        let mut vertices_and_indices = load_model_obj(self.model_path.to_string());

        // vertices_and_indices.vertices.iter_mut().for_each(|vert| {
        //     vert.pos += self.properties.position.coords;
        // });

        vertices_and_indices
            .vertices
            .chunks_mut(4)
            .for_each(|face| {
                utility::scale(face, self.radius);
            });

        vertices_and_indices
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
