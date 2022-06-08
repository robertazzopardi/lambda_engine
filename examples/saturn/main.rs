use lambda_engine::prelude::*;

const SATURN_TEXTURE: &str = "./examples/assets/textures/2k_saturn.jpg";
const RING_TEXTURE: &str = "./examples/assets/textures/2k_saturn_ring_alpha.png";

fn main() {
    let mut display = Display::new(Resolution::ResHD);

    let mut camera = Camera::new(2., 1., 0.);

    let sections = 50;

    let sphere = Sphere::new(
        GeometryBuilder::default()
            .properties(
                SphereBuilder::default()
                    .radius(0.4)
                    .sector_count(sections)
                    .stack_count(sections)
                    .build(),
            )
            .texture(SATURN_TEXTURE)
            .shader(Shader::LightTexture)
            .cull_mode(CullMode::Back)
            .indexed()
            .build(),
    );

    let ring = Ring::new(
        GeometryBuilder::default()
            .properties(
                RingBuilder::default()
                    .inner_radius(0.5)
                    .outer_radius(1.)
                    .sector_count(sections)
                    .build(),
            )
            .texture(RING_TEXTURE)
            .shader(Shader::LightTexture)
            .topology(ModelTopology::TriangleStrip)
            .cull_mode(CullMode::None)
            .build(),
    );

    let objects: Geometries = vec![sphere.into(), ring.into()];

    let debugging = Some(DebugMessageProperties::new(
        MessageLevel::builder().error().verbose().warning(),
        MessageType::builder().performance().validation(),
    ));

    let mut engine = Engine::new(&display, &mut camera, objects, debugging);

    engine.run(&mut display, camera)
}
