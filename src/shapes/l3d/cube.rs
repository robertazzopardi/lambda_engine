use crate::{
    pos3d,
    shapes::{utility, Object, Shape, Vertex, Vertices, VerticesAndIndices, WHITE},
    space::{Coordinate3d, Orientation, VEC_ZERO},
    vector2, vertex,
};

const CUBE_VERTICES: [[Vertex; 4]; 6] = [
    [
        vertex!(pos3d!(-0.5, -0.5, 0.5), WHITE, VEC_ZERO, vector2!(1., 0.)),
        vertex!(pos3d!(0.5, -0.5, 0.5), WHITE, VEC_ZERO, vector2!(0., 0.)),
        vertex!(pos3d!(0.5, 0.5, 0.5), WHITE, VEC_ZERO, vector2!(0., 1.)),
        vertex!(pos3d!(-0.5, 0.5, 0.5), WHITE, VEC_ZERO, vector2!(1., 1.)),
    ],
    [
        vertex!(pos3d!(-0.5, 0.5, -0.5), WHITE, VEC_ZERO, vector2!(1., 0.)),
        vertex!(pos3d!(0.5, 0.5, -0.5), WHITE, VEC_ZERO, vector2!(0., 0.)),
        vertex!(pos3d!(0.5, -0.5, -0.5), WHITE, VEC_ZERO, vector2!(0., 1.)),
        vertex!(pos3d!(-0.5, -0.5, -0.5), WHITE, VEC_ZERO, vector2!(1., 1.)),
    ],
    [
        vertex!(pos3d!(-0.5, 0.5, 0.5), WHITE, VEC_ZERO, vector2!(1., 0.)),
        vertex!(pos3d!(-0.5, 0.5, -0.5), WHITE, VEC_ZERO, vector2!(0., 0.)),
        vertex!(pos3d!(-0.5, -0.5, -0.5), WHITE, VEC_ZERO, vector2!(0., 1.)),
        vertex!(pos3d!(-0.5, -0.5, 0.5), WHITE, VEC_ZERO, vector2!(1., 1.)),
    ],
    [
        vertex!(pos3d!(0.5, -0.5, 0.5), WHITE, VEC_ZERO, vector2!(1., 0.)),
        vertex!(pos3d!(0.5, -0.5, -0.5), WHITE, VEC_ZERO, vector2!(0., 0.)),
        vertex!(pos3d!(0.5, 0.5, -0.5), WHITE, VEC_ZERO, vector2!(0., 1.)),
        vertex!(pos3d!(0.5, 0.5, 0.5), WHITE, VEC_ZERO, vector2!(1., 1.)),
    ],
    [
        vertex!(pos3d!(0.5, 0.5, 0.5), WHITE, VEC_ZERO, vector2!(1., 0.)),
        vertex!(pos3d!(0.5, 0.5, -0.5), WHITE, VEC_ZERO, vector2!(0., 0.)),
        vertex!(pos3d!(-0.5, 0.5, -0.5), WHITE, VEC_ZERO, vector2!(0., 1.)),
        vertex!(pos3d!(-0.5, 0.5, 0.5), WHITE, VEC_ZERO, vector2!(1., 1.)),
    ],
    [
        vertex!(pos3d!(0.5, -0.5, -0.5), WHITE, VEC_ZERO, vector2!(1., 0.)),
        vertex!(pos3d!(0.5, -0.5, 0.5), WHITE, VEC_ZERO, vector2!(0., 0.)),
        vertex!(pos3d!(-0.5, -0.5, 0.5), WHITE, VEC_ZERO, vector2!(0., 1.)),
        vertex!(pos3d!(-0.5, -0.5, -0.5), WHITE, VEC_ZERO, vector2!(1., 1.)),
    ],
];

const CUBE_INDICES: [u16; 36] = [
    0, 1, 2, 2, 3, 0, // top
    4, 5, 6, 6, 7, 4, // bottom
    8, 9, 10, 8, 10, 11, // right
    12, 13, 14, 12, 14, 15, // left
    16, 17, 18, 16, 18, 19, // front
    20, 21, 22, 20, 22, 23, // back
];

pub type Cube<'a> = Shape<'a, CubeInfo>;

#[derive(Default, Debug, Clone, new)]
pub struct CubeInfo {
    pub position: Coordinate3d,
    pub orientation: Orientation,
    pub radius: f32,
}

impl Object for Cube<'_> {
    fn vertices_and_indices(&mut self) {
        let mut cube = CUBE_VERTICES;

        cube.iter_mut().for_each(|face| {
            utility::calculate_normals(face);
            utility::scale(face, self.properties.radius);
        });

        let vertices = Vertices(cube.into_iter().flatten().collect());

        self.vertices_and_indices = VerticesAndIndices::new(vertices, CUBE_INDICES.to_vec().into());
    }
}
