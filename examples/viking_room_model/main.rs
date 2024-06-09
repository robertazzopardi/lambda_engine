use wave_engine::prelude::*;

const VIKING_MODEL: &str = "./examples/assets/models/viking_room_model/viking_room.obj";
const VIKING_MODEL_TEXTURE: &str = "./examples/assets/models/viking_room_model/viking_room.png";

#[geometry(Model)]
struct ModelGeom;

impl Behavior for ModelGeom {
    fn actions(&mut self) {}
}

#[geometry_system(ModelGeom)]
struct Geom;

fn main() {
    let viking_model = Geom::ModelGeom(
        ModelGeom::default()
            .properties(
                ModelBuilder::default()
                    .radius(0.5)
                    .model_path(VIKING_MODEL)
                    .build(),
            )
            .texture(VIKING_MODEL_TEXTURE)
            .shader(Shader::LightTexture)
            .cull_mode(CullMode::None)
            .build(),
    );

    Engine::default()
        .geometries(vec![viking_model])
        .build()
        .run()
}
