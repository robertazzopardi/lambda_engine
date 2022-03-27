use cgmath::{Point3, Rad, Vector3};
use derive_more::{Deref, DerefMut, From, Neg, Sub};
use std::ops::{AddAssign, SubAssign};

pub const VEC_ZERO: DirectionVec = DirectionVec(Vector3::new(0., 0., 0.));

#[derive(Clone, Copy, Debug, Deref, DerefMut, Sub, From)]
pub struct DirectionVec(pub(crate) Vector3<f32>);

impl DirectionVec {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vector3::new(x, y, z))
    }
}

impl From<DirectionVec> for Position {
    fn from(vec: DirectionVec) -> Self {
        Position(Point3::new(vec.x, vec.y, vec.z))
    }
}

impl From<Position> for DirectionVec {
    fn from(pos: Position) -> Self {
        DirectionVec(Vector3::new(pos.x, pos.y, pos.z))
    }
}

impl From<Point3<f32>> for DirectionVec {
    fn from(pos: Point3<f32>) -> Self {
        DirectionVec(Vector3::new(pos.x, pos.y, pos.z))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deref, DerefMut, From)]
pub struct Position(pub(crate) Point3<f32>);

impl Position {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Point3::new(x, y, z))
    }
}

impl Default for Position {
    fn default() -> Self {
        Self(Point3::new(0., 0., 0.))
    }
}

impl AddAssign<Vector3<f32>> for Position {
    fn add_assign(&mut self, rhs: Vector3<f32>) {
        self.0 += rhs
    }
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Deref, DerefMut, Neg)]
pub struct Angle(pub(crate) Rad<f32>);

impl Angle {
    pub fn new(a: f32) -> Self {
        Self(Rad(a))
    }
}

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
