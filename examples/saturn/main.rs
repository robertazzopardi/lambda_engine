extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    display::Display,
    model::{
        self,
        utility::{ModelCullMode, ModelTopology},
        ModelProperties,
    },
    time::Time,
    VkArray, Vulkan,
};

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(1., 1., 1.);

    let models = VkArray {
        objects: [
            ModelProperties {
                texture: include_bytes!("../../assets/2k_saturn.jpg").to_vec(),
                indexed: true,
                topology: ModelTopology::TriangleList,
                cull_mode: ModelCullMode::Back,
                vertices_and_indices: model::sphere(0.4, 20, 20),
            },
            ModelProperties {
                texture: include_bytes!("../../assets/2k_saturn_ring_alpha.png").to_vec(),
                indexed: false,
                topology: ModelTopology::TriangleStrip,
                cull_mode: ModelCullMode::None,
                vertices_and_indices: model::ring(0.5, 1., 40),
            },
        ],
    };

    let vulkan: Vulkan = Vulkan::new(&display.window, &mut camera, models);

    let mouse_pressed = false;

    let time = Time::new(60.);

    lambda_engine::run(vulkan, display, time, camera, mouse_pressed)
}
