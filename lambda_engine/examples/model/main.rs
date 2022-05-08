extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    display::Display,
    shapes::{
        model::ModelInfo,
        utility::{ModelCullMode, ModelTopology},
        Object, ObjectBuilder,
    },
    time::Time,
    Vulkan,
};

const CUBE_MODEL: &str = "./lambda_engine/examples/assets/models/cube.obj";

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(2., 1., 0.);

    let cube_model = ObjectBuilder::default()
        .properties(ModelInfo::new(CUBE_MODEL))
        .texture(include_bytes!("../assets/textures/2k_saturn.jpg"))
        .topology(ModelTopology::TRIANGLE_LIST)
        .cull_mode(ModelCullMode::BACK)
        .indexed(true)
        .build()
        .unwrap();

    let objects: Vec<Box<dyn Object>> = vec![Box::new(cube_model)];

    let vulkan = Vulkan::new(&display.window, &mut camera, objects, None);

    let mouse_pressed = false;

    let time = Time::new(60.);

    lambda_engine::run(vulkan, display, time, camera, mouse_pressed)
}
