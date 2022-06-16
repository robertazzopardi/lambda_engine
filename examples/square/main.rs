use lambda_engine::prelude::*;

fn main() {
    let square = Square::new(
        GeometryBuilder::default()
            .properties(SquareBuilder::default().radius(0.5).build())
            .cull_mode(CullMode::None)
            .shader(Shader::Vertex)
            .build(),
    );

    let objects: Geometries = vec![square.into()];

    Engine::default().geometries(objects).build().run()
}
