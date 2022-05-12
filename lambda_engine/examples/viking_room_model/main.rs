extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    display::Display,
    object::{
        l3d::model::ModelInfoBuilder,
        utility::{ModelCullMode, ModelTopology},
        ShapeBuilder, Shapes,
    },
    time::Time,
    Engine,
};

const VIKING_MODEL: &str =
    "./lambda_engine/examples/assets/models/viking_room_model/viking_room.obj";
const VIKING_MODEL_TEXTURE: &str =
    "./lambda_engine/examples/assets/models/viking_room_model/viking_room.png";

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(2., 1., 0.);

    let cube_model = ShapeBuilder::default()
        .properties(
            ModelInfoBuilder::default()
                .radius(0.5)
                .model_path(VIKING_MODEL)
                .build()
                .unwrap(),
        )
        .texture(VIKING_MODEL_TEXTURE)
        .topology(ModelTopology::TRIANGLE_LIST)
        .cull_mode(ModelCullMode::NONE)
        .build()
        .unwrap();

    let objects: Shapes = vec![cube_model];

    let vulkan = Engine::new(&display.window, &mut camera, objects, None);

    let time = Time::new(60.);

    lambda_engine::run(vulkan, display, time, camera)
}
