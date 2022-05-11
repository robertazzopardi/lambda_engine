extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    display::Display,
    shapes::{
        l3d::model::ModelInfoBuilder,
        utility::{ModelCullMode, ModelTopology},
        Object, ShapeBuilder,
    },
    time::Time,
    Vulkan,
};

const CUBE_MODEL: &str = "./lambda_engine/examples/assets/models/cube_model/cube.obj";
const SATURN_TEXTURE: &str = "./lambda_engine/examples/assets/textures/2k_saturn.jpg";

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(2., 1., 0.);

    let cube_model = ShapeBuilder::default()
        .properties(
            ModelInfoBuilder::default()
                .radius(0.5)
                .model_path(CUBE_MODEL)
                .build()
                .unwrap(),
        )
        .texture(SATURN_TEXTURE)
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
