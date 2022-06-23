use crate::{
    l2d::plane::square_from_vertices,
    utility::{self, calculate_indices},
    VerticesAndIndices,
};
use derive_builder::Builder;
use lambda_space::space::{Coordinate3, Orientation};

pub const CUBE_VERTICES: [[f32; 3]; 36] = [
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

#[derive(Builder, Default, Debug, Clone, Copy)]
#[builder(default, build_fn(skip))]
pub struct Cube {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub radius: f32,
}

impl CubeBuilder {
    pub fn build(&mut self) -> Cube {
        Cube {
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            radius: self.radius.expect("Field `Radius` Expected"),
        }
    }
}

impl Cube {
    pub fn vertices_and_indices(&self) -> VerticesAndIndices {
        let mut vertices = square_from_vertices(&CUBE_VERTICES);

        vertices.chunks_mut(4).for_each(|face| {
            utility::calculate_normals(face);
            utility::scale(face, self.radius);
        });

        let indices = calculate_indices(&vertices);

        VerticesAndIndices::new(vertices, indices)
    }
}
