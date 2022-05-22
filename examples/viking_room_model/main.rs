use lambda_engine::prelude::*;

const VIKING_MODEL: &str = "./examples/assets/models/viking_room_model/viking_room.obj";
const VIKING_MODEL_TEXTURE: &str = "./examples/assets/models/viking_room_model/viking_room.png";

fn main() {
    let mut display = Display::new(Resolution::ResHD);

    let mut camera = Camera::new(2., 1., 0.);

    let cube_model = GeometryBuilder::default()
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

    let mut engine = Engine::new(&display, &mut camera, objects, None);

    engine.run(&mut display, camera)
}
