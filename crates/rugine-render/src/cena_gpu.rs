//! O que o renderer desenha num frame.
//!
//! Separado do `Renderer` porque e dado, nao estado de GPU: quem monta isto e
//! o jogo ou o editor, a cada frame, a partir da sua propria cena.

use rugine_core::math::Mat4;

use crate::mesh::Instancia;

/// A descricao de um frame.
pub struct CenaGpu {
    /// Matriz view-projection ja combinada.
    pub view_proj: Mat4,
    /// Uma entrada por objeto a desenhar.
    pub instancias: Vec<Instancia>,
    /// Se a grade do chao aparece.
    pub grade: bool,
}

impl Default for CenaGpu {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY,
            instancias: Vec::new(),
            grade: true,
        }
    }
}
