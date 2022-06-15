use lambda_engine::prelude::*;

fn main() {
    let cube = Cube::new(
        GeometryBuilder::default()
            .properties(CubeBuilder::default().radius(0.5).build())
            .cull_mode(CullMode::Back)
            .shader(Shader::Vertex)
            .build(),
    );

    let objects: Geometries = vec![cube.into()];

    Engine::new(Resolution::ResHD, objects, None).run()
}
