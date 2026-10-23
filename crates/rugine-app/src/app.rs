//! O contrato entre a engine e o jogo.

use rugine_core::time::Time;
use rugine_render::{CenaGpu, Renderer};
use rugine_ui::egui;
use winit::window::Window;

use crate::janela::Borda;

/// O que um jogo (ou o editor) implementa.
///
/// Todos os metodos tem corpo vazio por padrao, para que um projeto novo
/// compile implementando so o que precisa — e para que adicionar um callback
/// na engine depois nao quebre quem ja implementou o trait.
pub trait App: 'static {
    /// Chamado uma vez, depois que a janela e a GPU existem.
    ///
    /// Carregar recursos aqui, e nao no construtor do seu tipo: no construtor
    /// ainda nao existe `Renderer`, logo nao existe `Device` para criar
    /// buffers ou texturas.
    fn init(&mut self, _ctx: &mut Context<'_>) {}

    /// Chamado uma vez por frame, antes do desenho.
    fn update(&mut self, _ctx: &mut Context<'_>) {}

    /// Chamado uma vez por frame para construir a interface.
    ///
    /// Recebe a `Ui` raiz, que e de onde os paineis de topo pendem. O
    /// `Context` sai de `ui.ctx()` quando for preciso.
    ///
    /// Só roda se `EngineConfig::ui` nao for [`UiMode::Disabled`]. A egui e
    /// imediata: tudo o que deve aparecer na tela precisa ser declarado aqui a
    /// cada frame, nao criado uma vez e guardado.
    fn ui(&mut self, _ui: &mut egui::Ui, _ctx: &mut Context<'_>) {}

    /// Chamado quando a janela muda de tamanho, em pixels fisicos.
    ///
    /// O `Renderer` ja reconfigurou a surface quando isto roda; serve para o
    /// jogo recalcular o que depende de proporcao, como matriz de projecao.
    fn resized(&mut self, _width: u32, _height: u32) {}
}

/// O que o jogo recebe a cada callback.
///
/// Passar um `Context` em vez de varios parametros significa que a engine pode
/// expor mais sistemas (input, assets, audio) sem mudar a assinatura do trait.
pub struct Context<'a> {
    pub time: &'a Time,
    pub renderer: &'a mut Renderer,

    /// A textura onde a cena foi desenhada, para exibir dentro de um painel.
    ///
    /// Só tem conteudo util em [`UiMode::Editor`]; nos outros modos a cena vai
    /// direto para a janela.
    pub viewport_texture: egui::TextureId,

    /// Quantos draws a interface custou no frame anterior.
    ///
    /// Do frame anterior, e nao deste: no momento em que a UI e construida,
    /// o custo dela ainda nao foi medido.
    pub ui_draws: u32,

    pub(crate) cena: &'a mut CenaGpu,
    pub(crate) window: &'a Window,
    pub(crate) requested_viewport_size: &'a mut Option<[u32; 2]>,
    pub(crate) exit_requested: &'a mut bool,
}

impl Context<'_> {
    /// Entrega ao renderer o que desenhar neste frame.
    ///
    /// Chamado em `update`. Sem isto, o frame desenha o que foi definido no
    /// anterior — nada, no primeiro.
    pub fn definir_cena(&mut self, cena: CenaGpu) {
        *self.cena = cena;
    }

    /// Pede o encerramento ordenado da aplicacao.
    ///
    /// O frame atual ainda termina; o loop sai depois dele.
    pub fn exit(&mut self) {
        *self.exit_requested = true;
    }

    /// Pede que a textura do viewport passe a ter este tamanho.
    ///
    /// Aplicado no *proximo* frame, de proposito. A UI imediata so descobre
    /// quanto espaco o painel tem enquanto o constroi, e nesse momento a cena
    /// deste frame ja foi desenhada. Um frame de atraso ao arrastar um divisor
    /// e imperceptivel; recriar a textura no meio do frame nao e opcao.
    pub fn request_viewport_size(&mut self, width: u32, height: u32) {
        *self.requested_viewport_size = Some([width, height]);
    }

    /// Move a janela com o mouse ate o botao ser solto.
    ///
    /// Isto entrega o controle ao laco de movimento do sistema, e e o que faz
    /// o encaixe de janelas (snap) continuar funcionando numa barra de titulo
    /// desenhada por nos.
    ///
    /// So tem efeito se chamado logo apos o botao esquerdo ser pressionado.
    pub fn arrastar_janela(&self) {
        if let Err(erro) = self.window.drag_window() {
            log::debug!("arrastar janela nao suportado aqui: {erro}");
        }
    }

    /// Redimensiona a janela arrastando uma borda.
    ///
    /// Sem efeito no macOS, onde a winit nao implementa isto — motivo pelo
    /// qual [`crate::Decoracao::recomendada`] mantem a barra nativa la.
    pub fn redimensionar_janela(&self, borda: Borda) {
        if let Err(erro) = self.window.drag_resize_window(borda.into()) {
            log::debug!("redimensionar janela nao suportado aqui: {erro}");
        }
    }

    pub fn minimizar_janela(&self) {
        self.window.set_minimized(true);
    }

    pub fn janela_maximizada(&self) -> bool {
        self.window.is_maximized()
    }

    pub fn alternar_maximizada(&self) {
        self.window.set_maximized(!self.window.is_maximized());
    }

    /// A janela como dona de dialogos nativos, como os de abrir e salvar.
    ///
    /// So os handles crus, e nao a `Window` da winit: o que um dialogo
    /// precisa e saber sobre qual janela ficar modal, e expor a `Window`
    /// inteira deixaria a aplicacao mexer na janela por fora da engine.
    pub fn janela_dona(
        &self,
    ) -> &(impl raw_window_handle::HasWindowHandle + raw_window_handle::HasDisplayHandle) {
        self.window
    }
}
