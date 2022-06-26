use crate::{vector2, VerticesAndIndices, WHITE};
use derive_builder::Builder;
use lambda_space::{
    space::{Orientation, Vertex, Vertices},
    vertex,
};
use nalgebra::{Matrix4, Point3, Vector3};

const SQUARE_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

#[derive(Builder, Default, Debug, Clone)]
#[builder(default, build_fn(skip))]
pub struct Plane {
    pub position: Point3<f32>,
    pub orientation: Orientation,
    pub radius: f32,
    pub has_depth: bool,
    pub model: Matrix4<f32>,
}

impl PlaneBuilder {
    pub fn build(&mut self) -> Plane {
        Plane {
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            radius: self.radius.expect("Field `Radius` expected"),
            has_depth: self.has_depth.unwrap_or_default(),
            model: Matrix4::from_axis_angle(&Vector3::x_axis(), 0.0f32.to_radians()),
        }
    }
}

impl Plane {
    pub fn vertices_and_indices(&self) -> VerticesAndIndices {
        let mut vertices = square_from_vertices(&[
            [-0.5, -0.5, 0.5],
            [0.5, -0.5, 0.5],
            [0.5, 0.5, 0.5],
            [-0.5, 0.5, 0.5],
        ]);

        vertices.iter_mut().for_each(|vert| {
            vert.pos += self.position.coords;
        });

        VerticesAndIndices::new(vertices, SQUARE_INDICES.to_vec().into())
    }
}

pub(crate) fn square_from_vertices(vertices: &[[f32; 3]]) -> Vertices {
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
