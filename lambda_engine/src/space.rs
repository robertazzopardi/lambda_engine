use derive_more::{AddAssign, Deref, DerefMut, From, Neg, Sub};

pub const VEC_ZERO: DirectionVector = DirectionVector(cgmath::Vector3::new(0., 0., 0.));

#[derive(Clone, Copy, Debug, Deref, DerefMut, Sub, From)]
pub struct DirectionVector(pub(crate) cgmath::Vector3<f32>);

impl DirectionVector {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(cgmath::Vector3::new(x, y, z))
    }
}

impl From<Coordinate3> for DirectionVector {
    fn from(pos: Coordinate3) -> Self {
        Self::new(pos.x, pos.y, pos.z)
    }
}

impl From<cgmath::Point3<f32>> for DirectionVector {
    fn from(pos: cgmath::Point3<f32>) -> Self {
        Self::new(pos.x, pos.y, pos.z)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deref, DerefMut, From)]
pub struct Coordinate2d(pub(crate) cgmath::Point2<f32>);

impl Coordinate2d {
    pub fn new(x: f32, y: f32) -> Self {
        Self(cgmath::Point2::new(x, y))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Deref, DerefMut, From)]
pub struct Coordinate3(pub(crate) cgmath::Point3<f32>);

impl Coordinate3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(cgmath::Point3::new(x, y, z))
    }
}

impl Default for Coordinate3 {
    fn default() -> Self {
        Self::new(0., 0., 0.)
    }
}

impl std::ops::AddAssign<cgmath::Vector3<f32>> for Coordinate3 {
    fn add_assign(&mut self, rhs: cgmath::Vector3<f32>) {
        self.0 += rhs
    }
}

impl std::ops::AddAssign<Self> for Coordinate3 {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
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
