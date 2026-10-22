//! Fundamentos da Rugine: o que todo outro crate pode depender.
//!
//! Regra de camada: `rugine-core` nao depende de nada da engine. Se algo aqui
//! precisar de GPU, janela ou assets, esta na camada errada.

pub mod math;
pub mod time;

pub mod prelude {
    pub use crate::math::{Mat4, Quat, Vec2, Vec3, Vec4};
    pub use crate::time::Time;
}
