use lambda_engine::prelude::*;

const VIKING_MODEL: &str = "./examples/assets/models/viking_room_model/viking_room.obj";
const VIKING_MODEL_TEXTURE: &str = "./examples/assets/models/viking_room_model/viking_room.png";

fn main() {
    let viking_model = Model::new(
        GeometryBuilder::default()
            .properties(
                ModelBuilder::default()
                    .radius(0.5)
                    .model_path(VIKING_MODEL.to_owned())
                    .build(),
            )
            .texture(VIKING_MODEL_TEXTURE)
            .shader(Shader::LightTexture)
            .cull_mode(CullMode::None)
            .build(),
    );

    let objects: Geometries = vec![viking_model.into()];

    Engine::default().geometries(objects).build().run()
}
