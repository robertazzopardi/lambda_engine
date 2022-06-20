#[macro_use]
extern crate derive_new;

pub mod macros;
pub mod space;

pub mod prelude {
    pub use crate::space::VerticesAndIndices;
}
