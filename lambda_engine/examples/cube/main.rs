use lambda_engine::{
    camera::Camera,
    display::Display,
    object::{
        l3d::cube::CubeInfoBuilder,
        utility::{ModelCullMode, ModelTopology, ShaderType},
        ObjectBuilder, Shapes,
    },
    Engine,
};

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(-2., 1., 0.);

    let cube = ObjectBuilder::default()
        .properties(CubeInfoBuilder::default().radius(0.5).build().unwrap())
        .topology(ModelTopology::TRIANGLE_LIST)
        .cull_mode(ModelCullMode::BACK)
        .shader(ShaderType::Vertex)
        .indexed()
        .build()
        .unwrap();

    let objects: Shapes = vec![cube];

    let engine = Engine::new(&display.window, &mut camera, objects, None);

    lambda_engine::run(engine, display, camera)
}
