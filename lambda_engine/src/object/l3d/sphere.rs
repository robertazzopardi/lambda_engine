use crate::{
    object::{utility, Object, Shape, VerticesAndIndices, WHITE},
    space::{Coordinate3, Orientation},
    vertex,
};
use derive_builder::Builder;
use nalgebra::{Point3, Vector2};
use std::ops::Mul;

pub type Sphere = Shape<SphereInfo>;

#[derive(Builder, Default, Debug, Clone, Copy, new)]
#[builder(default)]
pub struct SphereInfo {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub radius: f32,
    pub sector_count: u32,
    pub stack_count: u32,
}

impl Object for Sphere {
    fn vertices_and_indices(&mut self) {
        let length = 1. / self.properties.radius;

        let sector_step = 2. * std::f32::consts::PI / self.properties.sector_count as f32;
        let stack_step = std::f32::consts::PI / self.properties.stack_count as f32;

        let pos = self.properties.position;

        let mut vertices = Vec::new();

        for i in 0..=self.properties.stack_count {
            let stack_angle = std::f32::consts::FRAC_PI_2 - i as f32 * stack_step;
            let xy = self.properties.radius * stack_angle.cos();

            let mut vec = Point3::new(0., 0., (self.properties.radius * stack_angle.sin()) + pos.z);

            for j in 0..=self.properties.sector_count {
                let sector_angle = j as f32 * sector_step;

                vec.x = (xy * sector_angle.cos()) + pos.x;
                vec.y = (xy * sector_angle.sin()) + pos.y;

                let normal = vec.mul(length);

                let tex_coord = Vector2::new(
                    j as f32 / self.properties.sector_count as f32,
                    i as f32 / self.properties.stack_count as f32,
                );

                vertices.push(vertex!(vec, WHITE, normal.coords, tex_coord));
            }
        }

        self.vertices_and_indices = Some(VerticesAndIndices::new(
            vertices.into(),
            utility::spherical_indices(self.properties.sector_count, self.properties.stack_count),
        ));
    }
}
