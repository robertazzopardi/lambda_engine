extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    debug::{DebugMessageProperties, MessageLevel, MessageType},
    display::Display,
    shapes::{
        l2d::{ring::RingInfo, square::SquareInfo},
        l3d::{cube::CubeInfo, sphere::SphereInfo},
        utility::{ModelCullMode, ModelTopology},
        Object, ObjectBuilder,
    },
    space::{Coordinate3d, Orientation},
    time::Time,
    Vulkan,
};

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(1., 1., 6.);

    let saturn_texture = include_bytes!("../assets/2k_saturn.jpg");
    let saturn_ring_textures = include_bytes!("../assets/2k_saturn_ring_alpha.png");

    let square = ObjectBuilder::default()
        .properties(SquareInfo::new(
            Coordinate3d::default(),
            Orientation::default(),
            3.,
            true,
        ))
        .texture(saturn_texture)
        .topology(ModelTopology::TRIANGLE_LIST)
        .cull_mode(ModelCullMode::NONE)
        .indexed(true)
        .build()
        .unwrap();

    let cube = ObjectBuilder::default()
        .properties(CubeInfo::new(
            Coordinate3d::default(),
            Orientation::default(),
            3.,
        ))
        .texture(saturn_texture)
        .topology(ModelTopology::TRIANGLE_LIST)
        .cull_mode(ModelCullMode::BACK)
        .indexed(true)
        .build()
        .unwrap();

    let sphere = ObjectBuilder::default()
        .properties(SphereInfo::new(
            Coordinate3d::new(2.5, 1., 5.5),
            Orientation::default(),
            0.4,
            50,
            50,
        ))
        .texture(saturn_texture)
        .topology(ModelTopology::TRIANGLE_LIST)
        .cull_mode(ModelCullMode::BACK)
        .indexed(true)
        .build()
        .unwrap();

    let ring = ObjectBuilder::default()
        .properties(RingInfo::new(
            Coordinate3d::new(2.5, 1., 5.5),
            Orientation::default(),
            0.5,
            1.,
            50,
        ))
        .texture(saturn_ring_textures)
        .topology(ModelTopology::TRIANGLE_STRIP)
        .cull_mode(ModelCullMode::NONE)
        .build()
        .unwrap();

    let objects: Vec<Box<dyn Object>> = vec![Box::new(square), Box::new(sphere), Box::new(ring)];

    let debugging = Some(DebugMessageProperties::new(
        MessageLevel::builder().error().verbose().warning(),
        MessageType::builder().performance().validation(),
    ));

    let vulkan = Vulkan::new(&display.window, &mut camera, objects, debugging);

    let mouse_pressed = false;

    let time = Time::new(60.);

    lambda_engine::run(vulkan, display, time, camera, mouse_pressed)
}
