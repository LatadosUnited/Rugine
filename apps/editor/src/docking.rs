//! Paineis encaixaveis.
//!
//! Cada painel e uma aba que pode ser arrastada para qualquer lugar: virar aba
//! ao lado de outra, abrir uma divisao nova ou sair numa janela flutuante.
//!
//! Quem faz o trabalho pesado e a `egui_dock`. Escrever um sistema de docking
//! que preste — arrastar entre nos, divisoes aninhadas, previa de encaixe — e
//! um projeto em si, e este ja existe testado.

use egui_dock::{DockState, NodeIndex};
use rugine_app::prelude::{Context, egui};

use crate::estado::Editor;
use crate::paineis;

/// Um painel encaixavel.
///
/// `Hash` e `Eq` sao exigidos pela `egui_dock` para dar identidade estavel a
/// cada aba entre frames.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Aba {
    Hierarquia,
    Cena,
    Inspector,
    Viewport,
    Projeto,
    Scripts,
}

impl Aba {
    pub fn titulo(self) -> &'static str {
        match self {
            Self::Hierarquia => "Hierarquia",
            Self::Cena => "Cena",
            Self::Inspector => "Inspector",
            Self::Viewport => "Viewport",
            Self::Projeto => "Projeto",
            Self::Scripts => "Scripts",
        }
    }
}

/// O arranjo inicial, reproduzindo o editor de referencia.
///
/// O `fraction` de cada divisao e quanto o no *antigo* fica — nao o novo.
/// Trocar isso de lugar e o erro mais facil de cometer aqui, e o resultado e
/// um painel lateral ocupando tres quartos da tela.
pub fn layout_padrao() -> DockState<Aba> {
    let mut dock = DockState::new(vec![Aba::Viewport]);
    let superficie = dock.main_surface_mut();

    // Coluna esquerda com 20%.
    let [centro, esquerda] = superficie.split_left(NodeIndex::root(), 0.8, vec![Aba::Hierarquia]);

    // Cena ocupando a metade de baixo da coluna esquerda.
    superficie.split_below(esquerda, 0.5, vec![Aba::Cena]);

    // Inspector a direita com 22%.
    let [centro, _] = superficie.split_right(centro, 0.78, vec![Aba::Inspector]);

    // Projeto e Scripts como duas abas do mesmo no, embaixo.
    superficie.split_below(centro, 0.72, vec![Aba::Projeto, Aba::Scripts]);

    dock
}

/// Liga cada aba ao painel que a desenha.
///
/// Guarda emprestimos em vez de dados: o estado continua morando no `Editor`,
/// e esta struct existe so pela duracao de um frame.
pub struct Visualizador<'a, 'b> {
    pub editor: &'a mut Editor,
    pub ctx: &'a mut Context<'b>,
}

impl egui_dock::TabViewer for Visualizador<'_, '_> {
    type Tab = Aba;

    fn id(&mut self, aba: &mut Self::Tab) -> egui::Id {
        egui::Id::new(*aba)
    }

    fn title(&mut self, aba: &mut Self::Tab) -> egui::WidgetText {
        aba.titulo().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, aba: &mut Self::Tab) {
        match aba {
            Aba::Hierarquia => paineis::hierarquia::desenhar(ui, self.editor),
            Aba::Cena => paineis::cena::desenhar(ui, self.editor),
            Aba::Inspector => paineis::inspector::desenhar(ui, self.editor),
            Aba::Viewport => paineis::viewport::desenhar(ui, self.editor, self.ctx),
            Aba::Projeto => paineis::projeto::projeto(ui),
            Aba::Scripts => paineis::projeto::scripts(ui, self.editor),
        }
    }

    /// Nenhum painel pode ser fechado.
    ///
    /// Enquanto nao houver um menu para reabrir um painel fechado, permitir
    /// fechar so cria um beco sem saida: o painel some e nao ha como traze-lo
    /// de volta sem reiniciar o editor.
    fn is_closeable(&self, _aba: &Self::Tab) -> bool {
        false
    }

    /// O viewport desenha a propria imagem ate a borda.
    ///
    /// Sem isto, a moldura da aba deixaria uma faixa do fundo em volta da cena.
    fn clear_background(&self, aba: &Self::Tab) -> bool {
        *aba != Aba::Viewport
    }
}
