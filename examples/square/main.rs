use lambda_engine::prelude::*;

#[geometry(Plane)]
struct PlaneGeom;

impl Behavior for PlaneGeom {
    fn actions(&mut self) {}
}

#[geometry_system(PlaneGeom)]
struct Geom;

fn main() {
    let plane = Geom::PlaneGeom(
        PlaneGeom::default()
            .properties(PlaneBuilder::default().radius(0.5).build())
            .cull_mode(CullMode::None)
            .shader(Shader::Vertex)
            .build(),
    );

    let mut engine = Engine::default();
    engine.add_geometry(&[plane]);
    engine.run();
}
