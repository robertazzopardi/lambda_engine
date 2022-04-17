extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    display::Display,
    shapes::{
        l3d::cube::CubeInfo,
        utility::{ModelCullMode, ModelTopology},
        Object, ObjectBuilder,
    },
    space::{Coordinate3, Orientation},
    time::Time,
    Vulkan,
};

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(2., 1., 0.);

    let saturn_texture = include_bytes!("../assets/2k_saturn.jpg");

    let cube = ObjectBuilder::default()
        .properties(CubeInfo::new(
            Coordinate3::default(),
            Orientation::default(),
            0.5,
        ))
        .texture(saturn_texture)
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
