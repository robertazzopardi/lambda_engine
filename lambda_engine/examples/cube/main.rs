extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    display::Display,
    shapes::{
        l3d::cube::CubeInfoBuilder,
        utility::{ModelCullMode, ModelTopology},
        Object, ShapeBuilder,
    },
    time::Time,
    Vulkan,
};

const TEXTURE: &str = "./lambda_engine/examples/assets/textures/2k_saturn.jpg";

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(2., 1., 0.);

    let cube = ShapeBuilder::default()
        .properties(CubeInfoBuilder::default().radius(0.5).build().unwrap())
        .texture(TEXTURE)
        .topology(ModelTopology::TRIANGLE_LIST)
        .cull_mode(ModelCullMode::BACK)
        .indexed(true)
        .build()
        .unwrap();

    let objects: Vec<Box<dyn Object>> = vec![Box::new(cube)];

    let vulkan = Vulkan::new(&display.window, &mut camera, objects, None);

    let mouse_pressed = false;

    let time = Time::new(60.);

    lambda_engine::run(vulkan, display, time, camera, mouse_pressed)
}
