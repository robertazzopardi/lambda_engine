extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    debug::{DebugMessageProperties, MessageLevel, MessageType},
    display::Display,
    shapes::{
        l2d::ring::Ring,
        l3d::{cube::Cube, sphere::Sphere},
        utility::{ModelCullMode, ModelTopology},
        Object, ObjectBuilder, Shape,
    },
    space::{Coordinate3d, Orientation},
    time::Time,
    Vulkan,
};

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(1., 1., 6.);

    let cube: Shape<Cube> = ObjectBuilder::default()
        .properties(Cube::new(
            Coordinate3d::default(),
            Orientation::default(),
            3.,
        ))
        .texture_buffer(Some(include_bytes!("../../assets/2k_saturn.jpg").to_vec()))
        .topology(ModelTopology::TRIANGLE_LIST)
        .cull_mode(ModelCullMode::BACK)
        .indexed(true)
        .build()
        .unwrap();

    let sphere: Shape<Sphere> = ObjectBuilder::default()
        .properties(Sphere::new(
            Coordinate3d::new(2.5, 1., 5.5),
            Orientation::default(),
            0.4,
            50,
            50,
        ))
        .texture_buffer(Some(include_bytes!("../../assets/2k_saturn.jpg").to_vec()))
        .topology(ModelTopology::TRIANGLE_LIST)
        .cull_mode(ModelCullMode::BACK)
        .indexed(true)
        .build()
        .unwrap();

    let ring: Shape<Ring> = ObjectBuilder::default()
        .properties(Ring::new(
            Coordinate3d::new(2.5, 1., 5.5),
            Orientation::default(),
            0.5,
            1.,
            50,
        ))
        .texture_buffer(Some(
            include_bytes!("../../assets/2k_saturn_ring_alpha.png").to_vec(),
        ))
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
