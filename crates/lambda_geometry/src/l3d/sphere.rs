use crate::{utility::spherical_indices, VerticesAndIndices, WHITE};
use derive_builder::Builder;
use lambda_space::{
    space::{Coordinate3, Orientation, Vertices},
    vertex,
};
use nalgebra::{Point3, Vector2};
use std::ops::Mul;

#[derive(Builder, Default, Debug, Clone, Copy)]
#[builder(default, build_fn(skip))]
pub struct Sphere {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub radius: f32,
    pub sector_count: u32,
    pub stack_count: u32,
}

impl SphereBuilder {
    pub fn build(&mut self) -> Sphere {
        Sphere {
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            radius: self.radius.expect("Field `Radius` expected"),
            sector_count: self.sector_count.unwrap_or_default(),
            stack_count: self.stack_count.unwrap_or_default(),
        }
    }
}

impl Sphere {
    pub fn vertices_and_indices(&self) -> VerticesAndIndices {
        let length = 1. / self.radius;

        let sector_step = 2. * std::f32::consts::PI / self.sector_count as f32;
        let stack_step = std::f32::consts::PI / self.stack_count as f32;

        let pos = self.position;

        let mut vertices = Vertices::default();

        for i in 0..=self.stack_count {
            let stack_angle = std::f32::consts::FRAC_PI_2 - i as f32 * stack_step;
            let xy = self.radius * stack_angle.cos();

            let mut vec = Point3::new(0., 0., (self.radius * stack_angle.sin()) + pos.z);

            for j in 0..=self.sector_count {
                let sector_angle = j as f32 * sector_step;

                vec.x = (xy * sector_angle.cos()) + pos.x;
                vec.y = (xy * sector_angle.sin()) + pos.y;

                let normal = vec.mul(length);

                let tex_coord = Vector2::new(
                    j as f32 / self.sector_count as f32,
                    i as f32 / self.stack_count as f32,
                );

                vertices.push(vertex!(vec, WHITE, normal.coords, tex_coord));
            }
        }

        VerticesAndIndices::new(
            vertices,
            spherical_indices(self.sector_count, self.stack_count),
        )
    }
}
