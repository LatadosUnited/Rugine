//! Renderizacao da Rugine.
//!
//! Camada fina sobre wgpu. O que ela resolve e o que wgpu deixa na sua mao:
//! o ciclo de vida da surface (criar, reconfigurar, lidar com frames perdidos)
//! e a aquisicao/apresentacao de frames.

mod error;
mod hello_triangle;
mod renderer;
mod target;
mod vertex;

pub use error::RenderError;
pub use renderer::{FrameContent, RenderStats, Renderer, SurfacePass};
pub use target::RenderTarget;
pub use vertex::Vertex;

/// Reexportado para que o codigo cliente nao precise declarar wgpu como
/// dependencia direta — e, principalmente, nao consiga declarar uma versao
/// diferente da que a engine usa.
pub use wgpu;
