use lambda_engine::prelude::*;

#[geometry(CubeInfo)]
struct BoxGeom;

impl Behavior for BoxGeom {
    fn actions(&mut self) {}
}

// #[geometry_system]
// struct _Geom;

fn main() {
    let cube = BoxGeom::default()
        .properties(CubeBuilder::default().radius(0.5).build())
        .cull_mode(CullMode::Back)
        .shader(Shader::Vertex);

    //

    let box_cube = Cube::new(
        GeometryBuilder::default()
            .properties(CubeBuilder::default().radius(0.5).build())
            .cull_mode(CullMode::Back)
            .shader(Shader::Vertex)
            .build(),
    );

    let objects = vec![box_cube.into()];

    // DebugMessageProperties::new(
    //     MessageLevel::builder().error().verbose().warning(),
    //     MessageType::builder().performance().validation(),
    // );

    Engine::default().geometries(objects).build().run()
}
