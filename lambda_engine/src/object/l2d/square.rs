use crate::{
    object::{InternalObject, Object, Vertex, Vertices, VerticesAndIndices, WHITE},
    space::Orientation,
    vector2, vertex,
};
use derive_builder::Builder;
use nalgebra::{Point3, Vector3};

const SQUARE_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub type Square = Object<SquareInfo>;

#[derive(Builder, Default, Debug, Clone, new)]
#[builder(default)]
pub struct SquareInfo {
    pub position: Point3<f32>,
    pub orientation: Orientation,
    pub radius: f32,
    pub has_depth: bool,
}

impl InternalObject for Square {
    fn vertices_and_indices(&mut self) {
        let mut vertices = square_from_vertices(&[
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
        ]);

        vertices.iter_mut().for_each(|vert| {
            vert.pos += self.properties.position.coords;
        });

        self.vertices_and_indices = Some(VerticesAndIndices::new(
            vertices,
            SQUARE_INDICES.to_vec().into(),
        ));
    }
}

pub fn square_from_vertices(verts: &[[f32; 3]]) -> Vertices {
    let tex_coord = vec![
        vector2!(1., 0.),
        vector2!(0., 0.),
        vector2!(0., 1.),
        vector2!(1., 1.),
    ];

    let mut tex_coords = Vec::new();
    for _ in 0..(verts.len() / 4) {
        tex_coords.extend(tex_coord.clone());
    }

    Vertices(
        verts
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
