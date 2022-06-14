use lambda_engine::prelude::*;

fn main() {
    let cube = GeometryBuilder::default()
        .properties(CubeBuilder::default().radius(0.5).build())
        .cull_mode(CullMode::Back)
        .shader(Shader::Vertex)
        .indexed()
        .build();

    let objects: Geometries = vec![cube.into()];

    let debugging = Some(DebugMessageProperties::default());

    Engine::new(Resolution::ResHD, objects, debugging).run()
}
