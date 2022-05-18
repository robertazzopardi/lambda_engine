use lambda_engine::{
    camera::Camera,
    display::{Display, Resolution},
    object::{
        l3d::model::ModelInfoBuilder,
        utility::{ModelCullMode, ShaderType},
        ObjectBuilder, Shapes,
    },
    Engine,
};

const CUBE_MODEL: &str = "./examples/assets/models/monkey_model/monkey_head.obj";
const SATURN_TEXTURE: &str = "./examples/assets/textures/2k_saturn.jpg";

fn main() {
    let display = Display::new(Resolution::ResHD);

    let mut camera = Camera::new(2., 0., 0.);

    let cube_model = ObjectBuilder::default()
        .properties(
            ModelInfoBuilder::default()
                .radius(0.3)
                .model_path(CUBE_MODEL)
                .build()
                .unwrap(),
        )
        .texture(SATURN_TEXTURE)
        .shader(ShaderType::LightTexture)
        .cull_mode(ModelCullMode::Back)
        .indexed()
        .build()
        .unwrap();

    let objects: Shapes = vec![cube_model];

    let engine = Engine::new(&display, &mut camera, objects, None);

    engine.run(display, camera)
}
