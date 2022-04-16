use crate::{
    shapes::{Object, Shape, Vertices, VerticesAndIndices, WHITE},
    space::Orientation,
    vector2, vertex,
};
use nalgebra::{Point3, Vector3};

const SQUARE_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub type Square<'a> = Shape<'a, SquareInfo>;

#[derive(Default, Debug, Clone, new)]
pub struct SquareInfo {
    pub position: Point3<f32>,
    pub orientation: Orientation,
    pub radius: f32,
    pub has_depth: bool,
}

impl Object for Square<'_> {
    fn vertices_and_indices(&mut self) {
        let mut vertices = Vertices(vec![
            vertex!(
                Point3::new(-0.5, -0.5, 0.5),
                *WHITE,
                Vector3::zeros(),
                vector2!(1., 0.)
            ),
            vertex!(
                Point3::new(0.5, -0.5, 0.5),
                *WHITE,
                Vector3::zeros(),
                vector2!(0., 0.)
            ),
            vertex!(
                Point3::new(0.5, 0.5, 0.5),
                *WHITE,
                Vector3::zeros(),
                vector2!(0., 1.)
            ),
            vertex!(
                Point3::new(-0.5, 0.5, 0.5),
                *WHITE,
                Vector3::zeros(),
                vector2!(1., 1.)
            ),
        ]);

        vertices.iter_mut().for_each(|vert| {
            vert.pos += self.properties.position.coords;
        });

        self.vertices_and_indices =
            VerticesAndIndices::new(vertices, SQUARE_INDICES.to_vec().into());
    }
}
