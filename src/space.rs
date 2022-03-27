use std::ops::{AddAssign, SubAssign};

use cgmath::{Point3, Rad, Vector3};
use derive_more::{Deref, DerefMut, From, Neg};

#[derive(Clone, Copy, Debug, PartialEq, Deref, DerefMut, From)]
pub struct Position(pub(crate) Point3<f32>);

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
pub struct Angle(pub Rad<f32>);

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
