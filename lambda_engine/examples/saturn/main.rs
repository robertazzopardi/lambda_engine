extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    debug::{DebugMessageProperties, MessageLevel, MessageType},
    display::Display,
    shapes::{
        l2d::ring::RingInfo,
        l3d::sphere::SphereInfo,
        utility::{ModelCullMode, ModelTopology},
        Object, ShapeBuilder,
    },
    space::{Coordinate3, Orientation},
    time::Time,
    Vulkan,
};

const SATURN_TEXTURE: &str = "./lambda_engine/examples/assets/textures/2k_saturn.jpg";
const RING_TEXTURE: &str = "./lambda_engine/examples/assets/textures/2k_saturn_ring_alpha.png";

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(2., 1., 0.);

    let sections = 50;

    let sphere = ShapeBuilder::default()
        .properties(SphereInfo::new(
            Coordinate3::default(),
            Orientation::default(),
            0.4,
            sections,
            sections,
        ))
        .texture(SATURN_TEXTURE)
        .topology(ModelTopology::TRIANGLE_LIST)
        .cull_mode(ModelCullMode::BACK)
        .indexed(true)
        .build()
        .unwrap();

    let ring = ShapeBuilder::default()
        .properties(RingInfo::new(
            Coordinate3::default(),
            Orientation::default(),
            0.5,
            1.,
            sections,
        ))
        .texture(RING_TEXTURE)
        .topology(ModelTopology::TRIANGLE_STRIP)
        .cull_mode(ModelCullMode::NONE)
        .build()
        .unwrap();

    let objects: Vec<Box<dyn Object>> = vec![Box::new(sphere), Box::new(ring)];

    let debugging = Some(DebugMessageProperties::new(
        MessageLevel::builder().error().verbose().warning(),
        MessageType::builder().performance().validation(),
    ));

    let vulkan = Vulkan::new(&display.window, &mut camera, objects, debugging);

    let mouse_pressed = false;

    let time = Time::new(60.);

    lambda_engine::run(vulkan, display, time, camera, mouse_pressed)
}
