//! Renderizacao da Rugine.
//!
//! Camada fina sobre wgpu. O que ela resolve e o que wgpu deixa na sua mao:
//! o ciclo de vida da surface (criar, reconfigurar, lidar com frames
//! perdidos), a aquisicao de frames e o desenho do mundo.

mod camera;
mod cena_gpu;
mod depth;
mod error;
mod mesh;
mod pipelines;
mod renderer;
mod target;

pub use cena_gpu::CenaGpu;
pub use depth::DepthBuffer;
pub use error::RenderError;
pub use mesh::{Instancia, Mesh, Vertex, VertexLinha};
pub use renderer::{FrameContent, RenderStats, Renderer, SurfacePass};
pub use target::RenderTarget;

/// Reexportado para que o codigo cliente nao precise declarar wgpu como
/// dependencia direta — e, principalmente, nao consiga declarar uma versao
/// diferente da que a engine usa.
pub use wgpu;
