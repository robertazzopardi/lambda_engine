#[macro_export]
macro_rules! vertex {
    ($pos:expr, $col:expr, $norm:expr, $tex:expr) => {
        wave_space::space::Vertex {
            pos: $pos,
            colour: $col,
            normal: $norm,
            tex_coord: $tex,
        }
    };
}
