use crate::{
    shapes::{Object, Shape, Vertices, VerticesAndIndices, WHITE},
    space::{Coordinate3, Orientation, VEC_ZERO},
    vector2, vertex,
};

const SQUARE_INDICES: [u16; 6] = [0, 1, 2, 2, 3, 0];

pub type Square<'a> = Shape<'a, SquareInfo>;

#[derive(Default, Debug, Clone, new)]
pub struct SquareInfo {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub radius: f32,
    pub has_depth: bool,
}

impl Object for Square<'_> {
    fn vertices_and_indices(&mut self) {
        let mut vertices = Vertices(vec![
            vertex!(
                Coordinate3::new(-0.5, -0.5, 0.5),
                WHITE,
                VEC_ZERO,
                vector2!(1., 0.)
            ),
            vertex!(
                Coordinate3::new(0.5, -0.5, 0.5),
                WHITE,
                VEC_ZERO,
                vector2!(0., 0.)
            ),
            vertex!(
                Coordinate3::new(0.5, 0.5, 0.5),
                WHITE,
                VEC_ZERO,
                vector2!(0., 1.)
            ),
            vertex!(
                Coordinate3::new(-0.5, 0.5, 0.5),
                WHITE,
                VEC_ZERO,
                vector2!(1., 1.)
            ),
        ]);

        vertices.iter_mut().for_each(|vert| {
            vert.pos += self.properties.position;
        });

        self.vertices_and_indices =
            VerticesAndIndices::new(vertices, SQUARE_INDICES.to_vec().into());
    }
}
