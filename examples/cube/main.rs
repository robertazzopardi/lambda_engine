use lambda_engine::prelude::*;

#[geometry(Cube)]
struct BoxGeom;

impl Behavior for BoxGeom {
    fn actions(&mut self) {
        // println!("hello");
        self.rotate_x(1.1);
        // self.translate()
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
