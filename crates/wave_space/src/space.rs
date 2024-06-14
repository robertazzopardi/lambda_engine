use derive_more::{Deref, DerefMut, From};
use nalgebra::{vector, Point3, Vector2, Vector3};

pub trait Pos {}

#[derive(Clone, Copy, Debug, PartialEq, Deref, DerefMut, From)]
pub struct Coordinate2(pub(crate) Vector2<f32>);

impl Coordinate2 {
    #[must_use]
    pub const fn new(x: f32, y: f32) -> Self {
        Self(vector![x, y])
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deref, DerefMut, From, Default)]
pub struct Pos3(pub Vector3<f32>);

impl Pos3 {
    #[must_use]
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self(vector![x, y, z])
    }

    #[must_use]
    pub fn x() -> Self {
        Self(Vector3::x())
    }

    #[must_use]
    pub const fn from_x(x: f32) -> Self {
        Self(vector![x, 0., 0.])
    }

    #[must_use]
    pub fn y() -> Self {
        Self(Vector3::y())
    }

    #[must_use]
    pub const fn from_y(y: f32) -> Self {
        Self(vector![0., y, 0.])
    }

    #[must_use]
    pub fn z() -> Self {
        Self(Vector3::z())
    }

    #[must_use]
    pub const fn from_z(z: f32) -> Self {
        Self(vector![0., 0., z])
    }
}

impl Pos for Pos3 {}

impl std::ops::AddAssign<Vector3<f32>> for Pos3 {
    fn add_assign(&mut self, rhs: Vector3<f32>) {
        self.0 += rhs;
    }
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
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
#[derive(Deref, DerefMut, Default, Debug, PartialEq, Clone, Copy, Eq)]
pub struct LookDirection([i8; 6]);

impl LookDirection {
    pub fn set_up(&mut self, value: i8) {
        self[0] += value;
    }
    pub fn set_down(&mut self, value: i8) {
        self[1] += value;
    }
    pub fn set_left(&mut self, value: i8) {
        self[2] += value;
    }
    pub fn set_right(&mut self, value: i8) {
        self[3] += value;
    }
    pub fn set_forward(&mut self, value: i8) {
        self[4] += value;
    }
    pub fn set_back(&mut self, value: i8) {
        self[5] += value;
    }
    #[must_use]
    pub fn x(&self) -> i8 {
        self[3] - self[2]
    }
    #[must_use]
    pub fn y(&self) -> i8 {
        self[0] - self[1]
    }
    #[must_use]
    pub fn z(&self) -> i8 {
        self[4] - self[5]
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Vertex {
    pub pos: Point3<f32>,
    pub colour: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
}

impl Vertex {
    #[must_use]
    pub const fn new(
        pos: Point3<f32>,
        colour: Vector3<f32>,
        normal: Vector3<f32>,
        tex_coord: Vector2<f32>,
    ) -> Self {
        Self {
            pos,
            colour,
            normal,
            tex_coord,
        }
    }
}

#[derive(Clone, Default, Debug, From, Deref, DerefMut)]
pub struct Vertices(Vec<Vertex>);

#[derive(Clone, Default, Debug, From, Deref, DerefMut)]
pub struct Indices(Vec<u16>);

#[derive(Clone, Default, Debug)]
pub struct VerticesAndIndices {
    pub vertices: Vertices,
    pub indices: Indices,
}

impl VerticesAndIndices {
    #[must_use]
    pub const fn new(vertices: Vertices, indices: Indices) -> Self {
        Self { vertices, indices }
    }
}
