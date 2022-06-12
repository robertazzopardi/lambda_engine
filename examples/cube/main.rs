use lambda_engine::prelude::*;

fn actions() {
    println!("hello")
}

fn main() {
    let cube = Cube::new(
        GeometryBuilder::default()
            .properties(CubeBuilder::default().radius(0.5).build())
            .cull_mode(CullMode::Back)
            .shader(Shader::Vertex)
            .behavior(actions)
            .indexed()
            .build(),
    );

    let objects: Geometries = vec![cube.into()];

    let debugging = Some(DebugMessageProperties::default());

    Engine::new(Resolution::ResHD, objects, debugging).run()
}
