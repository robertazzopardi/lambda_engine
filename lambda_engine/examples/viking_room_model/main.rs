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

const VIKING_MODEL: &str =
    "./lambda_engine/examples/assets/models/viking_room_model/viking_room.obj";
const VIKING_MODEL_TEXTURE: &str =
    "./lambda_engine/examples/assets/models/viking_room_model/viking_room.png";

fn main() {
    let display = Display::new(Resolution::ResHD);

    let mut camera = Camera::new(2., 1., 0.);

    let cube_model = ObjectBuilder::default()
        .properties(
            ModelInfoBuilder::default()
                .radius(0.5)
                .model_path(VIKING_MODEL)
                .build()
                .unwrap(),
        )
        .texture(VIKING_MODEL_TEXTURE)
        .shader(ShaderType::LightTexture)
        .cull_mode(ModelCullMode::None)
        .build()
        .unwrap();

    let objects: Shapes = vec![cube_model];

    let engine = Engine::new(&display, &mut camera, objects, None);

    engine.run(display, camera)
}
