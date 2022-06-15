use crate::{
    l2d::square::square_from_vertices,
    utility::{self, calculate_indices},
    GeomBehavior, Geometry, VerticesAndIndices,
};
use derive_builder::Builder;
use derive_more::{Deref, DerefMut};
use lambda_space::space::{Coordinate3, Orientation};
use lambda_vulkan::GeomProperties;

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
#[builder(name = "CubeBuilder")]
pub struct CubeInfo {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub radius: f32,
}

impl CubeBuilder {
    pub fn build(&mut self) -> CubeInfo {
        CubeInfo {
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            radius: self.radius.expect("Field `Radius` Expected"),
        }
    }
}

#[derive(new, Deref, DerefMut, Debug)]
pub struct Cube(Geometry<CubeInfo>);

impl GeomBehavior for Cube {
    fn vertices_and_indices(&self) -> VerticesAndIndices {
        let mut vertices = square_from_vertices(&CUBE_VERTICES);

        vertices.chunks_mut(4).for_each(|face| {
            utility::calculate_normals(face);
            utility::scale(face, self.properties.radius);
        });

        let indices = calculate_indices(&vertices);

        VerticesAndIndices::new(vertices, indices)
    }

    fn features(&self) -> GeomProperties {
        GeomProperties::new(
            &self.texture,
            self.vertices_and_indices(),
            self.topology,
            self.cull_mode,
            self.shader,
            *self.indexed,
        )
    }
}
