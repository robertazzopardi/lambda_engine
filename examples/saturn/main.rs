extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    display::Display,
    model::ModelProperties,
    model::{ModelCullMode, ModelTopology, ModelType},
    time::Time,
    SceneProperties, Vulkan,
};

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(3., 1., 2.);

    let models = vec![
        ModelProperties {
            texture: include_bytes!("../../assets/2k_saturn.jpg").to_vec(),
            model_type: ModelType::Sphere,
            indexed: true,
            topology: ModelTopology::TriangleList,
            cull_mode: ModelCullMode::Back,
        },
        ModelProperties {
            texture: include_bytes!("../../assets/2k_saturn_ring_alpha.png").to_vec(),
            model_type: ModelType::Ring,
            indexed: false,
            topology: ModelTopology::TriangleStrip,
            cull_mode: ModelCullMode::None,
        },
    ];

    let scene_properties = SceneProperties { models };

    let vulkan: Vulkan = Vulkan::new(&display.window, &mut camera, scene_properties);

    let mouse_pressed = false;

    let time = Time::new(60.);

    lambda_engine::run(vulkan, display, time, camera, mouse_pressed)
}
