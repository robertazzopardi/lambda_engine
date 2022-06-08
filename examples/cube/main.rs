use lambda_engine::prelude::*;

fn actions() {}

fn main() {
    let mut display = Display::new(Resolution::ResHD);

    let mut camera = Camera::new(-2., 1., 0.);

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

    let mut engine = Engine::new(&display, &mut camera, objects, None);

    engine.run(&mut display, camera);
}
