use lambda_engine::prelude::*;

const CUBE_MODEL: &str = "./examples/assets/models/cube_model/cube.obj";
const SATURN_TEXTURE: &str = "./examples/assets/textures/2k_saturn.jpg";

#[geometry(Model)]
struct ModelGeom;

impl Behavior for ModelGeom {
    fn actions(&mut self) {}
}

#[geometry_system(ModelGeom)]
struct Geom;

fn main() {
    let model = Geom::ModelGeom(
        ModelGeom::default()
            .properties(
                ModelBuilder::default()
                    .radius(0.3)
                    .model_path(CUBE_MODEL.to_owned())
                    .build(),
            )
            .texture(SATURN_TEXTURE)
            .shader(Shader::LightTexture)
            .cull_mode(CullMode::Back)
            .build(),
    );

    // DebugMessageProperties::new(
    //     MessageLevel::builder().error().verbose().warning(),
    //     MessageType::builder().performance().validation(),
    // );

    Engine::default().geometries(vec![model]).build().run()
}
