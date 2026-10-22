//! Janela, loop de eventos e o ponto de entrada da aplicacao.
//!
//! Este crate e a fronteira entre a engine e o que roda sobre ela: ele e dono
//! da janela, do loop e da camada de UI, e chama de volta o [`App`] uma vez
//! por frame.

mod app;
mod janela;
mod runner;

pub use app::{App, Context};
pub use janela::{Borda, Decoracao};
pub use runner::{EngineConfig, UiMode, run};

/// Reexports para que um projeto precise de uma unica dependencia.
///
/// Inclui `egui`, `log` e `wgpu` de proposito: tudo isso e usado por quem
/// escreve um jogo ou painel, e reexportar e o que garante a *mesma* versao
/// da engine. Com declaracoes proprias no Cargo.toml do projeto, duas versoes
/// de wgpu ou de egui no mesmo binario dao erros de tipo incompreensiveis.
pub mod prelude {
    pub use crate::{App, Borda, Context, Decoracao, EngineConfig, UiMode, run};
    pub use log;
    pub use rugine_core::prelude::*;
    pub use rugine_render::{RenderStats, Renderer, wgpu};
    pub use rugine_ui::egui;
    pub use winit::keyboard::KeyCode;
}
