use lambda_engine::prelude::*;

fn main() {
    let box_cube = Cube::new(
        GeometryBuilder::default()
            .properties(CubeBuilder::default().radius(0.5).build())
            .cull_mode(CullMode::Back)
            .shader(Shader::Vertex)
            .build(),
    );

    let objects: Geometries = vec![box_cube.into()];

    // DebugMessageProperties::new(
    //     MessageLevel::builder().error().verbose().warning(),
    //     MessageType::builder().performance().validation(),
    // );

    Engine::default().geometries(objects).build().run()
}
