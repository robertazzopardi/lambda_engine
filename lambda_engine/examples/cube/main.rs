use lambda_engine::{
    camera::Camera,
    display::{Display, Resolution},
    object::{
        l3d::cube::CubeInfoBuilder,
        utility::{ModelCullMode, ShaderType},
        ObjectBuilder, Shapes,
    },
    Engine,
};

fn main() {
    let display = Display::new(Resolution::ResHD);

    let mut camera = Camera::new(-2., 1., 0.);

    let cube = ObjectBuilder::default()
        .properties(CubeInfoBuilder::default().radius(0.5).build().unwrap())
        .cull_mode(ModelCullMode::BACK)
        .shader(ShaderType::Vertex)
        .indexed()
        .build()
        .unwrap();

    let objects: Shapes = vec![cube];

    let engine = Engine::new(&display, &mut camera, objects, None);

    lambda_engine::run(engine, display, camera)
}
