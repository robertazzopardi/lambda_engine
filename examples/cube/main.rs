use lambda_engine::prelude::*;

fn main() {
    let mut display = Display::new(Resolution::ResHD);

    let mut camera = Camera::new(-2., 1., 0.);

    let cube = GeometryBuilder::default()
        .properties(CubeInfoBuilder::default().radius(0.5).build().unwrap())
        .cull_mode(ModelCullMode::Back)
        .shader(ShaderType::Vertex)
        .indexed()
        .build()
        .unwrap();

    let objects: Shapes = vec![cube];

    let mut engine = Engine::new(&display, &mut camera, objects, None);

    engine.run(&mut display, camera);
}
