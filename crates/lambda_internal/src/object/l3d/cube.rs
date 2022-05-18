use crate::{
    object::{
        l2d::square::square_from_vertices,
        utility::{self, calculate_indices},
        InternalObject, Object, VerticesAndIndices,
    },
    space::{Coordinate3, Orientation},
};
use derive_builder::Builder;

const CUBE_VERTICES: [[f32; 3]; 36] = [
    [1.0, 1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, 1.0, 1.0],
    [1.0, 1.0, -1.0],
    [-1.0, 1.0, 1.0],
    [1.0, 1.0, 1.0],
    [1.0, -1.0, 1.0],
    [1.0, 1.0, 1.0],
    [-1.0, 1.0, 1.0],
    [1.0, -1.0, 1.0],
    [-1.0, 1.0, 1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, 1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, 1.0],
    [-1.0, 1.0, -1.0],
    [-1.0, -1.0, -1.0],
    [-1.0, -1.0, -1.0],
    [1.0, -1.0, -1.0],
    [1.0, -1.0, 1.0],
    [-1.0, -1.0, -1.0],
    [1.0, -1.0, 1.0],
    [-1.0, -1.0, 1.0],
    [1.0, -1.0, -1.0],
    [1.0, 1.0, -1.0],
    [1.0, 1.0, 1.0],
    [1.0, -1.0, -1.0],
    [1.0, 1.0, 1.0],
    [1.0, -1.0, 1.0],
    [-1.0, -1.0, -1.0],
    [-1.0, 1.0, -1.0],
    [1.0, 1.0, -1.0],
    [-1.0, -1.0, -1.0],
    [1.0, 1.0, -1.0],
    [1.0, -1.0, -1.0],
];

pub type Cube = Object<CubeInfo>;

#[derive(Builder, Default, Debug, Clone, new)]
#[builder(default)]
pub struct CubeInfo {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub radius: f32,
}

impl InternalObject for Cube {
    fn vertices_and_indices(&mut self) -> &VerticesAndIndices {
        let mut vertices = square_from_vertices(&CUBE_VERTICES);

        vertices.chunks_mut(4).for_each(|face| {
            utility::calculate_normals(face);
            utility::scale(face, self.properties.radius);
        });

        let indices = calculate_indices(&vertices);

        self.vertices_and_indices = Some(VerticesAndIndices::new(vertices, indices));
        self.vertices_and_indices.as_ref().unwrap()
    }
}
