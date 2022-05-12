use crate::{
    object::{
        l2d::square::square_from_vertices, utility, Object, Shape, Vertices, VerticesAndIndices,
    },
    space::{Coordinate3, Orientation},
};
use derive_builder::Builder;

lazy_static! {
    static ref CUBE_VERTICES: Vertices = square_from_vertices(vec![
        // face 1
        [-0.5, -0.5, 0.5],
        [0.5, -0.5, 0.5],
        [0.5, 0.5, 0.5],
        [-0.5, 0.5, 0.5],
        // face 2
        [-0.5, 0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, -0.5, -0.5],
        [-0.5, -0.5, -0.5],
        // face 3
        [-0.5, 0.5, 0.5],
        [-0.5, 0.5, -0.5],
        [-0.5, -0.5, -0.5],
        [-0.5, -0.5, 0.5],
        // face 4
        [0.5, -0.5, 0.5],
        [0.5, -0.5, -0.5],
        [0.5, 0.5, -0.5],
        [0.5, 0.5, 0.5],
        // face 5
        [0.5, 0.5, 0.5],
        [0.5, 0.5, -0.5],
        [-0.5, 0.5, -0.5],
        [-0.5, 0.5, 0.5],
        // face 6
        [0.5, -0.5, -0.5],
        [0.5, -0.5, 0.5],
        [-0.5, -0.5, 0.5],
        [-0.5, -0.5, -0.5],
    ]);
}

const CUBE_INDICES: [u16; 36] = [
    0, 1, 2, 2, 3, 0, // top
    4, 5, 6, 6, 7, 4, // bottom
    8, 9, 10, 8, 10, 11, // right
    12, 13, 14, 12, 14, 15, // left
    16, 17, 18, 16, 18, 19, // front
    20, 21, 22, 20, 22, 23, // back
];

pub type Cube = Shape<CubeInfo>;

#[derive(Builder, Default, Debug, Clone, new)]
#[builder(default)]
pub struct CubeInfo {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub radius: f32,
}

impl Object for Cube {
    fn vertices_and_indices(&mut self) {
        let mut vertices = CUBE_VERTICES.clone();

        vertices.chunks_mut(4).for_each(|face| {
            utility::calculate_normals(face);
            utility::scale(face, self.properties.radius);
        });

        self.vertices_and_indices = Some(VerticesAndIndices::new(
            vertices,
            CUBE_INDICES.to_vec().into(),
        ));
    }
}
