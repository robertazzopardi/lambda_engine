use lambda_engine::prelude::*;

#[geometry(Cube)]
struct BoxGeom;

impl Behavior for BoxGeom {
    fn actions(&mut self) {
        println!("hello");
        // self.rotate()
        self.translate()
    }
}

impl Transformation for BoxGeom {
    fn rotate_x(&mut self, amount: f32) {
        let rot = scaled_axis_matrix_4(amount);
        // object.model *= rot;
    }

    fn translate(&mut self) {
        self.properties.position.z -= 0.1;
    }

    fn rotate_y(&mut self, amount: f32) {
        todo!()
    }

    fn rotate_z(&mut self, amount: f32) {
        todo!()
    }
}

#[geometry_system(BoxGeom)]
struct Geom;

fn main() {
    let cube = Geom::BoxGeom(
        BoxGeom::default()
            .properties(CubeBuilder::default().radius(0.5).build())
            .cull_mode(CullMode::Back)
            .shader(Shader::Vertex)
            .build(),
    );

    Engine::default().geometries(vec![cube]).build().run()
}
