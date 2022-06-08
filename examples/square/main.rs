use lambda_engine::prelude::*;

fn main() {
    let mut display = Display::new(Resolution::ResHD);

    let mut camera = Camera::new(-2., 1., 0.);

    let square = Square::new(
        GeometryBuilder::default()
            .properties(SquareBuilder::default().radius(0.5).build())
            .cull_mode(CullMode::None)
            .shader(Shader::Vertex)
            .indexed()
            .build(),
    );

    let objects: Geometries = vec![square.into()];

    let mut engine = Engine::new(&display, &mut camera, objects, None);

    engine.run(&mut display, camera);
}
