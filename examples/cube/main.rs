use lambda_engine::prelude::*;

#[geometry(Cube)]
struct BoxGeom;

impl Behavior for BoxGeom {
    fn actions(&mut self) {
        // println!("hello");
        self.rotate_x(1.1);
        // self.translate()
    }
}

impl Transformation for BoxGeom {
    fn rotate_x(&mut self, amount: f32) {
        let rot = scaled_axis_matrix_4(amount);
        // dbg!(rot);
        self.properties.model *= rot;

        println!("outer {:?}", self.properties.model);
    }

    fn rotate_y(&mut self, amount: f32) {
        todo!()
    }

    fn rotate_z(&mut self, amount: f32) {
        todo!()
    }

    fn translate(&mut self) {
        // self.properties.position.z -= 0.1;
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
