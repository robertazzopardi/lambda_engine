extern crate lambda_engine;

use lambda_engine::{
    camera::Camera,
    debug::{DebugMessageProperties, MessageLevel, MessageType},
    display::Display,
    shapes::{
        l2d::ring::{Ring, RingProperties},
        l3d::{
            cube::{Cube, CubeProperties},
            sphere::{Sphere, SphereProperties},
        },
        utility::{ModelCullMode, ModelTopology},
        Object, ObjectBuilder,
    },
    space::Orientation,
    time::Time,
    Vulkan,
};

fn main() {
    let display = Display::new(1280, 720);

    let mut camera = Camera::new(1., 1., 6.);

    let cube = Cube::builder(
        CubeProperties::new(cgmath::Point3::new(0., 0., 0.), Orientation::new(), 5.).into(),
    )
    .texture_buffer(include_bytes!("../../assets/2k_saturn.jpg").to_vec())
    .topology(ModelTopology::TriangleList)
    .cull_mode(ModelCullMode::Back);

    let sphere = Sphere::builder(
        SphereProperties::new(
            cgmath::Point3::new(0., 0., 0.),
            Orientation::new(),
            0.4,
            20,
            20,
        )
        .into(),
    )
    .texture_buffer(include_bytes!("../../assets/2k_saturn.jpg").to_vec())
    .topology(ModelTopology::TriangleList)
    .cull_mode(ModelCullMode::Back);

    let ring = Ring::builder(
        RingProperties::new(
            cgmath::Point3::new(0., 0., 0.),
            Orientation::new(),
            0.5,
            1.,
            40,
        )
        .into(),
    )
    .texture_buffer(include_bytes!("../../assets/2k_saturn_ring_alpha.png").to_vec())
    .indexed(false)
    .topology(ModelTopology::TriangleStrip)
    .cull_mode(ModelCullMode::None);

    let objects: Vec<Box<dyn Object>> = vec![sphere, ring];

    let debugging = Some(DebugMessageProperties::new(
        MessageLevel::builder().error().verbose().warning(),
        MessageType::builder().performance().validation(),
    ));

    let vulkan = Vulkan::new(&display.window, &mut camera, objects, debugging);

    let mouse_pressed = false;

    let time = Time::new(60.);

    lambda_engine::run(vulkan, display, time, camera, mouse_pressed)
}
