use cgmath::{Point2, Point3, Rad, Vector3};
use derive_more::{Deref, DerefMut, From, Neg, Sub};
use std::ops::{AddAssign, SubAssign};

pub const VEC_ZERO: DirectionVector = DirectionVector(Vector3::new(0., 0., 0.));

#[derive(Clone, Copy, Debug, Deref, DerefMut, Sub, From)]
pub struct DirectionVector(pub(crate) Vector3<f32>);

impl DirectionVector {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vector3::new(x, y, z))
    }
}

impl From<DirectionVector> for Coordinate3d {
    fn from(vec: DirectionVector) -> Self {
        Self::new(vec.x, vec.y, vec.z)
    }
}

impl From<Coordinate3d> for DirectionVector {
    fn from(pos: Coordinate3d) -> Self {
        Self::new(pos.x, pos.y, pos.z)
    }
}

impl From<Point3<f32>> for DirectionVector {
    fn from(pos: Point3<f32>) -> Self {
        Self::new(pos.x, pos.y, pos.z)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deref, DerefMut, From)]
pub struct Coordinate2d(pub(crate) Point2<f32>);

impl Coordinate2d {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Point2::new(0., 0.))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deref, DerefMut, From)]
pub struct Coordinate3d(pub(crate) Point3<f32>);

impl Coordinate3d {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Point3::new(x, y, z))
    }
}

impl Default for Coordinate3d {
    fn default() -> Self {
        Self::new(0., 0., 0.)
    }
}

impl AddAssign<Vector3<f32>> for Coordinate3d {
    fn add_assign(&mut self, rhs: Vector3<f32>) {
        self.0 += rhs
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Deref, DerefMut, Neg, From, new)]
pub struct Angle(pub(crate) Rad<f32>);

impl Default for Angle {
    fn default() -> Self {
        Self(Rad(0.))
    }
}

impl AddAssign<Rad<f32>> for Angle {
    fn add_assign(&mut self, rhs: Rad<f32>) {
        self.0 += rhs
    }
}

impl SubAssign<Rad<f32>> for Angle {
    fn sub_assign(&mut self, rhs: Rad<f32>) {
        self.0 -= rhs
    }
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub struct Orientation {
    pub yaw: Angle,
    pub pitch: Angle,
    pub _roll: Angle,
}

#[derive(Default, Debug, PartialEq)]
pub struct Rotation {
    pub horizontal: f32,
    pub vertical: f32,
}

#[derive(Default, Debug, PartialEq)]
pub struct LookDirection {
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
    pub forward: f32,
    pub backward: f32,
}
