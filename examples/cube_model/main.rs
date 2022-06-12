use lambda_engine::prelude::*;

const CUBE_MODEL: &str = "./examples/assets/models/cube_model/cube.obj";
const SATURN_TEXTURE: &str = "./examples/assets/textures/2k_saturn.jpg";

fn main() {
    let cube_model = Model::new(
        GeometryBuilder::default()
            .properties(
                ModelBuilder::default()
                    .radius(0.3)
                    .model_path(CUBE_MODEL.to_owned())
                    .build(),
            )
            .texture(SATURN_TEXTURE)
            .shader(Shader::LightTexture)
            .cull_mode(CullMode::Back)
            .indexed()
            .build(),
    );

    let objects: Geometries = vec![cube_model.into()];

    Engine::new(Resolution::ResHD, objects, None).run()
}
