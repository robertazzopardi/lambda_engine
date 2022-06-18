use crate::{utility, Behavior, GeomBuilder, Geometry, VerticesAndIndices};
use derive_builder::Builder;
use derive_more::{Deref, DerefMut};
use lambda_space::space::{Coordinate3, Orientation};
use lambda_vulkan::GeomProperties;
use nalgebra::Vector2;

#[derive(Builder, Default, Debug, Clone)]
#[builder(default, build_fn(skip))]
#[builder(name = "RingBuilder")]
pub struct RingInfo {
    pub position: Coordinate3,
    pub orientation: Orientation,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub sector_count: u32,
}

impl RingBuilder {
    pub fn build(&mut self) -> RingInfo {
        RingInfo {
            position: self.position.unwrap_or_default(),
            orientation: self.orientation.unwrap_or_default(),
            inner_radius: self.inner_radius.unwrap_or_default(),
            outer_radius: self.outer_radius.unwrap_or_default(),
            sector_count: self.sector_count.unwrap_or_default(),
        }
    }
}

#[derive(new, Deref, DerefMut, Debug, Clone)]
pub struct Ring(Geometry<RingInfo>);

impl Behavior for Ring {
    fn actions(&mut self) {
        todo!()
    }
}

impl GeomBuilder for Ring {
    fn vertices_and_indices(&self) -> VerticesAndIndices {
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

        VerticesAndIndices::new(
            vertices.into(),
            utility::spherical_indices(self.properties.sector_count, 2),
        )
    }

    fn features(&self) -> GeomProperties {
        GeomProperties::new(
            &self.texture,
            self.vertices_and_indices(),
            self.topology,
            self.cull_mode,
            self.shader,
            *self.indexed,
        )
    }
}
