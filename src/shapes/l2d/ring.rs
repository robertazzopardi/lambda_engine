use crate::{
    shapes::{utility, Object, Shape, VerticesAndIndices},
    space::{Coordinate3d, Orientation},
};
use cgmath::{Vector2, Zero};

pub type Ring<'a> = Shape<'a, RingInfo>;

#[derive(Default, Debug, Clone, new)]
pub struct RingInfo {
    pub position: Coordinate3d,
    pub orientation: Orientation,
    pub inner_radius: f32,
    pub outer_radius: f32,
    pub sector_count: u32,
}

impl Object for Ring<'_> {
    fn vertices_and_indices(&mut self) {
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
                Vector2::zero(),
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

        self.vertices_and_indices = VerticesAndIndices::new(
            vertices.into(),
            utility::spherical_indices(self.properties.sector_count, 2),
        );
    }
}