extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    debug::{DebugMessageProperties, MessageLevel, MessageType},
    display::Display,
    shapes::{
        l2d::ring::Ring,
        l3d::sphere::Sphere,
        utility::{ModelCullMode, ModelTopology},
        Object, ObjectBuilder, Shape,
    },
    space::{Orientation, Position},
    time::Time,
    Vulkan,
};

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(1., 1., 6.);

    // let cube: Shape<Cube> = ObjectBuilder::default()
    //     .properties(Cube::new(Position::default(), Orientation::default(), 3.))
    //     .texture_buffer(Some(include_bytes!("../../assets/2k_saturn.jpg").to_vec()))
    //     .topology(ModelTopology::TriangleList)
    //     .cull_mode(ModelCullMode::Back)
    //     .indexed(true)
    //     .build()
    //     .unwrap();

    let sphere: Shape<Sphere> = ObjectBuilder::default()
        .properties(Sphere::new(
            Position::default(),
            Orientation::default(),
            0.4,
            20,
            20,
        ))
        .texture_buffer(Some(include_bytes!("../../assets/2k_saturn.jpg").to_vec()))
        .topology(ModelTopology::TRIANGLE_LIST)
        .cull_mode(ModelCullMode::BACK)
        .indexed(true)
        .build()
        .unwrap();

    let ring: Shape<Ring> = ObjectBuilder::default()
        .properties(Ring::new(
            Position::default(),
            Orientation::default(),
            0.5,
            1.,
            40,
        ))
        .texture_buffer(Some(
            include_bytes!("../../assets/2k_saturn_ring_alpha.png").to_vec(),
        ))
        .topology(ModelTopology::TRIANGLE_STRIP)
        .cull_mode(ModelCullMode::NONE)
        .build()
        .unwrap();

    let objects: Vec<Box<dyn Object>> = vec![Box::new(ring), Box::new(sphere)];

    let debugging = Some(DebugMessageProperties::new(
        MessageLevel::builder().error().verbose().warning(),
        MessageType::builder().performance().validation(),
    ));

    let vulkan = Vulkan::new(&display.window, &mut camera, objects, debugging);

    let mouse_pressed = false;

    let time = Time::new(60.);

    lambda_engine::run(vulkan, display, time, camera, mouse_pressed)
}
