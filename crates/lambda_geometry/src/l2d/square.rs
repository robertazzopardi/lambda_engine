use crate::{vector2, GeomBehavior, Geometry, VerticesAndIndices, WHITE};
use derive_builder::Builder;
use derive_more::{Deref, DerefMut};
use lambda_space::{
    space::{Orientation, Vertex, Vertices},
    vertex,
};
use lambda_vulkan::GeomProperties;
use nalgebra::{Point3, Vector3};

const SQUARE_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

#[derive(Builder, Default, Debug, Clone)]
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
        SquareInfo {
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            radius: self.radius.expect("Field `Radius` expected"),
            has_depth: self.has_depth.unwrap_or_default(),
        }
    }
}

#[derive(new, Deref, DerefMut, Debug)]
pub struct Square(Geometry<SquareInfo>);

impl GeomBehavior for Square {
    fn vertices_and_indices(&self) -> VerticesAndIndices {
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

    fn features(&self) -> GeomProperties {
        GeomProperties::new(
            &self.texture,
            self.vertices_and_indices(),
            self.topology,
            self.cull_mode,
            self.shader,
            self.indexed,
        )
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
