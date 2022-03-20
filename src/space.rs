use cgmath::Rad;

#[derive(new, Clone, Copy, Debug, PartialEq)]
pub struct Orientation {
    #[new(value = "Rad(0.0)")]
    pub yaw: Rad<f32>,
    #[new(value = "Rad(0.0)")]
    pub pitch: Rad<f32>,
    #[new(value = "Rad(0.0)")]
    pub _roll: Rad<f32>,
}

#[derive(Default, Debug, PartialEq)]
pub struct Rotation {
    pub horizontal: f32,
    pub vertical: f32,
}

#[derive(Default, Debug, PartialEq)]
pub struct Direction {
    pub left: f32,
    pub right: f32,
    pub up: f32,
    pub down: f32,
    pub forward: f32,
    pub backward: f32,
}
