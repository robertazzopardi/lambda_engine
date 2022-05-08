extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    display::Display,
    shapes::{
        l3d::model::ModelInfo,
        utility::{ModelCullMode, ModelTopology},
        Object, ShapeBuilder,
    },
    space::{Coordinate3, Orientation},
    time::Time,
    Vulkan,
};

const VIKING_MODEL: &str =
    "./lambda_engine/examples/assets/models/viking_room_model/viking_room.obj";
const VIKING_MODEL_TEXTURE: &str =
    "./lambda_engine/examples/assets/models/viking_room_model/viking_room.png";

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(2., 1., 0.);

    let cube_model = ShapeBuilder::default()
        .properties(ModelInfo::new(
            Coordinate3::default(),
            Orientation::default(),
            0.5,
            VIKING_MODEL,
        ))
        .texture(VIKING_MODEL_TEXTURE)
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
