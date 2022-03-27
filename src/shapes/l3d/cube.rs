use crate::{
    shapes::{utility, DirectionVec, Object, Shape, Vertex, VerticesAndIndices, VEC3_ZERO, WHITE},
    space::{Orientation, Position},
};
use cgmath::{Point3, Vector2};

const CUBE_VERTICES: [[Vertex; 4]; 6] = [
    [
        Vertex {
            pos: Position(Point3::new(-0.5, -0.5, 0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(1., 0.),
        },
        Vertex {
            pos: Position(Point3::new(0.5, -0.5, 0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(0., 0.),
        },
        Vertex {
            pos: Position(Point3::new(0.5, 0.5, 0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(0., 1.),
        },
        Vertex {
            pos: Position(Point3::new(-0.5, 0.5, 0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(1., 1.),
        },
    ],
    [
        Vertex {
            pos: Position(Point3::new(-0.5, 0.5, -0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(1., 0.),
        },
        Vertex {
            pos: Position(Point3::new(0.5, 0.5, -0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(0., 0.),
        },
        Vertex {
            pos: Position(Point3::new(0.5, -0.5, -0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(0., 1.),
        },
        Vertex {
            pos: Position(Point3::new(-0.5, -0.5, -0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(1., 1.),
        },
    ],
    [
        Vertex {
            pos: Position(Point3::new(-0.5, 0.5, 0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(1., 0.),
        },
        Vertex {
            pos: Position(Point3::new(-0.5, 0.5, -0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(0., 0.),
        },
        Vertex {
            pos: Position(Point3::new(-0.5, -0.5, -0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(0., 1.),
        },
        Vertex {
            pos: Position(Point3::new(-0.5, -0.5, 0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(1., 1.),
        },
    ],
    [
        Vertex {
            pos: Position(Point3::new(0.5, -0.5, 0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(1., 0.),
        },
        Vertex {
            pos: Position(Point3::new(0.5, -0.5, -0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(0., 0.),
        },
        Vertex {
            pos: Position(Point3::new(0.5, 0.5, -0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(0., 1.),
        },
        Vertex {
            pos: Position(Point3::new(0.5, 0.5, 0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(1., 1.),
        },
    ],
    [
        Vertex {
            pos: Position(Point3::new(0.5, 0.5, 0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(1., 0.),
        },
        Vertex {
            pos: Position(Point3::new(0.5, 0.5, -0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(0., 0.),
        },
        Vertex {
            pos: Position(Point3::new(-0.5, 0.5, -0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(0., 1.),
        },
        Vertex {
            pos: Position(Point3::new(-0.5, 0.5, 0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(1., 1.),
        },
    ],
    [
        Vertex {
            pos: Position(Point3::new(0.5, -0.5, -0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(1., 0.),
        },
        Vertex {
            pos: Position(Point3::new(0.5, -0.5, 0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(0., 0.),
        },
        Vertex {
            pos: Position(Point3::new(-0.5, -0.5, 0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(0., 1.),
        },
        Vertex {
            pos: Position(Point3::new(-0.5, -0.5, -0.5)),
            colour: WHITE,
            normal: DirectionVec(VEC3_ZERO),
            tex_coord: Vector2::new(1., 1.),
        },
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

#[derive(Default, Debug, Clone, new)]
pub struct Cube {
    pub position: Position,
    pub orientation: Orientation,
    pub radius: f32,
}

impl Object for Shape<Cube> {
    fn vertices_and_indices(&mut self) {
        let mut cube = CUBE_VERTICES;

        cube.map(|_| utility::calculate_normals);

        for face in cube.iter_mut() {
            utility::scale_from_origin(face, self.properties.radius);
        }

        let vertices = cube.into_iter().flatten().collect::<Vec<Vertex>>();

        self.vertices_and_indices = Some(VerticesAndIndices::new(vertices, CUBE_INDICES.to_vec()));
    }
}
