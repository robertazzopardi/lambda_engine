use lambda_engine::prelude::*;
use lambda_engine::{
    camera::Camera,
    debug::{DebugMessageProperties, MessageLevel, MessageType},
    display::{Display, Resolution},
    object::{
        l2d::ring::RingInfoBuilder,
        l3d::sphere::SphereInfoBuilder,
        utility::{ModelCullMode, ModelTopology, ShaderType},
        ObjectBuilder, Shapes,
    },
};

const SATURN_TEXTURE: &str = "./examples/assets/textures/2k_saturn.jpg";
const RING_TEXTURE: &str = "./examples/assets/textures/2k_saturn_ring_alpha.png";

fn main() {
    let display = Display::new(Resolution::ResHD);

    let mut camera = Camera::new(2., 1., 0.);

    let sections = 50;

    let sphere = ObjectBuilder::default()
        .properties(
            SphereInfoBuilder::default()
                .radius(0.4)
                .sector_count(sections)
                .stack_count(sections)
                .build()
                .unwrap(),
        )
        .texture(SATURN_TEXTURE)
        .shader(ShaderType::LightTexture)
        .cull_mode(ModelCullMode::Back)
        .indexed()
        .build()
        .unwrap();

    let ring = ObjectBuilder::default()
        .properties(
            RingInfoBuilder::default()
                .inner_radius(0.5)
                .outer_radius(1.)
                .sector_count(sections)
                .build()
                .unwrap(),
        )
        .texture(RING_TEXTURE)
        .shader(ShaderType::LightTexture)
        .topology(ModelTopology::TriangleStrip)
        .cull_mode(ModelCullMode::None)
        .build()
        .unwrap();

    let objects: Shapes = vec![sphere, ring];

    let debugging = Some(DebugMessageProperties::new(
        MessageLevel::builder().error().verbose().warning(),
        MessageType::builder().performance().validation(),
    ));

    let engine = Engine::new(&display, &mut camera, objects, debugging);

    engine.run(display, camera)
}
