//! Fundamentos da Rugine: o que todo outro crate pode depender.
//!
//! Regra de camada: `rugine-core` nao depende de nada da engine. Se algo aqui
//! precisar de GPU, janela ou assets, esta na camada errada.

pub mod math;
pub mod raio;
pub mod time;

pub mod prelude {
    // `camera` entra inteiro: todo jogo precisa montar view e projecao, e e
    // o modulo que fixa as convencoes de NDC. Deixar de fora obrigaria cada
    // projeto a declarar `rugine-core` so para alcanca-lo.
    pub use crate::math::camera;
    pub use crate::math::{Mat4, Quat, Vec2, Vec3, Vec4};
    pub use crate::raio::Raio;
    pub use crate::time::Time;
}
