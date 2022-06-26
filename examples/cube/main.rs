use lambda_engine::prelude::*;

#[geometry(Cube)]
struct BoxGeom;

impl Behavior for BoxGeom {
    fn actions(&mut self) {
        self.rotate_y(0.001);
    }
}

#[geometry_system(BoxGeom)]
struct Geom;

fn main() {
    let cube = Geom::BoxGeom(
        BoxGeom::default()
            .properties(CubeBuilder::default().radius(0.5).build())
            .cull_mode(CullMode::Back)
            .shader(Shader::Vertex)
            .build(),
    );

    Engine::default().geometries(vec![cube]).build().run()
}
