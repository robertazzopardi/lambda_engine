use derive_more::{AddAssign, Deref, DerefMut, From, Neg};
use nalgebra::{Point2, Point3, Vector3};

#[derive(Clone, Copy, Debug, PartialEq, Deref, DerefMut, From)]
pub struct Coordinate2d(pub(crate) Point2<f32>);

impl Coordinate2d {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Point2::new(x, y))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deref, DerefMut, From)]
pub struct Coordinate3(pub(crate) Point3<f32>);

impl Coordinate3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Point3::new(x, y, z))
    }
}

impl Default for Coordinate3 {
    fn default() -> Self {
        Self::new(0., 0., 0.)
    }
}

impl std::ops::AddAssign<Vector3<f32>> for Coordinate3 {
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
    pub horizontal: f32,
    pub vertical: f32,
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
