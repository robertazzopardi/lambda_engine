use crate::{
    object::{utility, Indices, InternalObject, Object, Vertices, VerticesAndIndices, WHITE},
    space::{Coordinate3, Orientation},
    vertex,
};
use derive_builder::Builder;
use nalgebra::{Point3, Vector2, Vector3};

pub type Model<'a> = Object<ModelInfo<'a>>;

#[derive(Builder, Default, Debug, Clone, Copy, new)]
#[builder(default)]
pub struct ModelInfo<'a> {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub radius: f32,
    model_path: &'a str,
}

impl InternalObject for Model<'_> {
    fn vertices_and_indices(&mut self) {
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

        self.vertices_and_indices = Some(vertices_and_indices);
    }
}

fn load_model_obj(model_path: String) -> VerticesAndIndices {
    let (models, _materials) =
        tobj::load_obj(model_path, &tobj::GPU_LOAD_OPTIONS).expect("Failed to OBJ load file");

    let mut vertices = Vertices(Vec::new());
    let mut indices = Indices(Vec::new());

    for model in models {
        let mesh = &model.mesh;

        for index in mesh.indices.iter() {
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
            indices.push(*index as u16);
        }
    }

    VerticesAndIndices::new(vertices, indices)
}
