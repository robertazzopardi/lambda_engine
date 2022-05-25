use lambda_engine::prelude::*;

fn main() {
    let mut display = Display::new(Resolution::ResHD);

    let mut camera = Camera::new(-2., 1., 0.);

    let square = Square::new(
        GeometryBuilder::default()
            .properties(SquareInfoBuilder::default().radius(0.5).build().unwrap())
            .cull_mode(ModelCullMode::None)
            .shader(ShaderType::Vertex)
            .indexed()
            .build()
            .unwrap(),
    );

    let objects: Geometries = vec![square.into()];

    let mut engine = Engine::new(&display, &mut camera, objects, None);

    engine.run(&mut display, camera);
}
