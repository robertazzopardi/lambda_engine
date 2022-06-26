use crate::{utility, VerticesAndIndices};
use derive_builder::Builder;
use lambda_space::space::{Coordinate3, Orientation};
use nalgebra::{Matrix4, Vector2, Vector3};

#[derive(Builder, Default, Debug, Clone)]
#[builder(default, build_fn(skip))]
pub struct Ring {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub sector_count: u32,
    pub model: Matrix4<f32>,
}

impl RingBuilder {
    pub fn build(&mut self) -> Ring {
        Ring {
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            inner_radius: self.inner_radius.unwrap_or_default(),
            outer_radius: self.outer_radius.unwrap_or_default(),
            sector_count: self.sector_count.unwrap_or_default(),
            model: Matrix4::from_axis_angle(&Vector3::x_axis(), 0.0f32.to_radians()),
        }
    }
}

impl Ring {
    pub fn vertices_and_indices(&self) -> VerticesAndIndices {
        assert!(
            self.inner_radius <= self.outer_radius,
            "Ring inner radius mut be smaller or equal to its outer radius"
        );

        let mut angle = 0.;
        let angle_step = 180. / self.sector_count as f32;
        let length = 1.;

        let pos = self.position;

        let mut vertices = Vec::new();

        for _ in 0..=self.sector_count {
            vertices.push(utility::make_point(
                &mut angle,
                self.outer_radius,
                angle_step,
                length,
                Vector2::zeros(),
                &pos,
            ));
            vertices.push(utility::make_point(
                &mut angle,
                self.inner_radius,
                angle_step,
                length,
                Vector2::new(1., 1.),
                &pos,
            ));
        }

        VerticesAndIndices::new(
            vertices.into(),
            utility::spherical_indices(self.sector_count, 2),
        )
    }
}
