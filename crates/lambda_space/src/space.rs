use derive_more::{AddAssign, Deref, DerefMut, From, Neg};
use imgui::DrawVert;
use nalgebra::{point, vector, Point3, Vector2, Vector3};

pub trait Pos {}

#[derive(Clone, Copy, Debug, PartialEq, Deref, DerefMut, From)]
pub struct Coordinate2(pub(crate) Vector2<f32>);

impl Coordinate2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self(vector![x, y])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deref, DerefMut, From, Default)]
pub struct Pos3(pub Vector3<f32>);

impl Pos3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(vector![x, y, z])
    }

    pub fn x() -> Self {
        Self(Vector3::x())
    }

    pub fn from_x(x: f32) -> Self {
        Self(vector![x, 0., 0.])
    }

    pub fn y() -> Self {
        Self(Vector3::y())
    }

    pub fn from_y(y: f32) -> Self {
        Self(vector![0., y, 0.])
    }

    pub fn z() -> Self {
        Self(Vector3::z())
    }

    pub fn from_z(z: f32) -> Self {
        Self(vector![0., 0., z])
    }
}

impl Pos for Pos3 {}

impl std::ops::AddAssign<Vector3<f32>> for Pos3 {
    fn add_assign(&mut self, rhs: Vector3<f32>) {
        self.0 += rhs
    }
}

#[derive(Default, Debug, PartialEq, Clone, Copy, new)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
}

/// Look direction to map mouse movement and scroll
///
/// Stored as: \
/// up \
/// down \
/// left \
/// right \
/// forward (forward scroll) \
/// back (back scroll)
#[derive(Deref, DerefMut, Default, Debug, PartialEq, Clone, Copy)]
pub struct LookDirection([i8; 6]);

impl LookDirection {
    pub fn set_up(&mut self, value: i8) {
        self[0] = value
    }
    pub fn set_down(&mut self, value: i8) {
        self[1] = value
    }
    pub fn set_left(&mut self, value: i8) {
        self[2] = value
    }
    pub fn set_right(&mut self, value: i8) {
        self[3] = value
    }
    pub fn set_forward(&mut self, value: i8) {
        self[4] = value
    }
    pub fn set_back(&mut self, value: i8) {
        self[5] = value
    }
    pub fn x(&self) -> i8 {
        self[3] - self[2]
    }
    pub fn y(&self) -> i8 {
        self[0] - self[1]
    }
    pub fn z(&self) -> i8 {
        self[4] - self[5]
    }
}

#[derive(Clone, Copy, Debug, new)]
pub struct Vertex {
    pub pos: Point3<f32>,
    pub colour: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

impl From<DrawVert> for Vertex {
    fn from(vert: DrawVert) -> Self {
        let DrawVert { pos, uv, col } = vert;
        // dbg!(point![pos[0] / 10., pos[1] / 10., 1.]);
        Self {
            pos: point![pos[0] / 100., pos[1] / 100., 0.],
            colour: vector![col[0] as f32, col[1] as f32, col[2] as f32],
            normal: vector![uv[0], uv[1], 0.],
            tex_coord: vector![0., 1.],
        }
    }
}

#[derive(new, Clone, Default, Debug, From, Deref, DerefMut)]
pub struct Vertices(Vec<Vertex>);

#[derive(new, Clone, Default, Debug, From, Deref, DerefMut)]
pub struct Indices(Vec<u16>);

#[derive(new, Clone, Default, Debug)]
pub struct VerticesAndIndices {
    pub vertices: Vertices,
    pub indices: Indices,
}
