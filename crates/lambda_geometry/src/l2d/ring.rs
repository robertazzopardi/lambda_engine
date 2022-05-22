use crate::{utility, InternalObject, Object, VerticesAndIndices};
use derive_builder::Builder;
use lambda_space::space::{Coordinate3, Orientation};
use nalgebra::Vector2;

pub type Ring = Object<RingInfo>;

#[derive(Builder, Default, Debug, Clone, new)]
#[builder(default)]
pub struct RingInfo {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub sector_count: u32,
}

impl InternalObject for Ring {
    fn vertices_and_indices(&mut self) -> &VerticesAndIndices {
        assert!(
            self.properties.inner_radius <= self.properties.outer_radius,
            "Ring inner radius mut be smaller or equal to its outer radius"
        );

        let mut angle = 0.;
        let angle_step = 180. / self.properties.sector_count as f32;
        let length = 1.;

        let pos = self.properties.position;

        let mut vertices = Vec::new();

        for _ in 0..=self.properties.sector_count {
            vertices.push(utility::make_point(
                &mut angle,
                self.properties.outer_radius,
                angle_step,
                length,
                Vector2::zeros(),
                &pos,
            ));
            vertices.push(utility::make_point(
                &mut angle,
                self.properties.inner_radius,
                angle_step,
                length,
                Vector2::new(1., 1.),
                &pos,
            ));
        }

        self.vulkan_object.vertices_and_indices = Some(VerticesAndIndices::new(
            vertices.into(),
            utility::spherical_indices(self.properties.sector_count, 2),
        ));

        self.vulkan_object.vertices_and_indices.as_ref().unwrap()
    }
}