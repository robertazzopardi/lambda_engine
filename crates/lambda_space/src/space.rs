use derive_more::{AddAssign, Deref, DerefMut, From, Neg};
use nalgebra::{vector, Point3, Vector2, Vector3};

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

#[derive(
    Clone, Copy, Debug, Default, PartialEq, PartialOrd, AddAssign, Deref, DerefMut, Neg, From, new,
)]
pub struct Angle(pub f32);

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Orientation {
    pub yaw: Angle,
    pub pitch: Angle,
    pub roll: Angle,
}

#[derive(Default, Debug, PartialEq, Clone, Copy, new)]
pub struct Rotation {
    pub x: f32,
    pub y: f32,
}

#[derive(Default, Debug, PartialEq, Clone, Copy)]
pub struct LookDirection {
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
    pub forward: f32,
    pub backward: f32,
}

#[derive(Clone, Copy, Debug, new)]
pub struct Vertex {
    pub pos: Point3<f32>,
    pub colour: Vector3<f32>,
    pub normal: Vector3<f32>,
    pub tex_coord: Vector2<f32>,
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
