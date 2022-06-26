use lambda_engine::prelude::*;

const SATURN_TEXTURE: &str = "./examples/assets/textures/2k_saturn.jpg";
const RING_TEXTURE: &str = "./examples/assets/textures/2k_saturn_ring_alpha.png";

#[geometry(Sphere)]
struct SphereGeom;

impl Behavior for SphereGeom {
    fn actions(&mut self) {}
}

#[geometry(Ring)]
struct RingGeom;

impl Behavior for RingGeom {
    fn actions(&mut self) {}
}

#[geometry_system(SphereGeom, RingGeom)]
struct Geom;

fn main() {
    let sections = 50;

    let sphere = Geom::SphereGeom(
        SphereGeom::default()
            .properties(
                SphereBuilder::default()
                    .radius(0.4)
                    .sector_count(sections)
                    .stack_count(sections)
                    .build(),
            )
            .texture(SATURN_TEXTURE)
            .shader(Shader::LightTexture)
            .cull_mode(CullMode::Back)
            .build(),
    );

    let ring = Geom::RingGeom(
        RingGeom::default()
            .properties(
                RingBuilder::default()
                    .inner_radius(0.5)
                    .outer_radius(1.)
                    .sector_count(sections)
                    .build(),
            )
            .texture(RING_TEXTURE)
            .shader(Shader::LightTexture)
            .topology(ModelTopology::TriangleStrip)
            .cull_mode(CullMode::None)
            .no_index()
            .build(),
    );

    Engine::default()
        .geometries(vec![sphere, ring])
        .camera(Camera::default().build())
        .build()
        .run()
}
