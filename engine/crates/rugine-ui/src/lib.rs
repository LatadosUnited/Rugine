//! Integracao da egui com a Rugine.
//!
//! Junta as duas metades da egui: a `egui-winit`, que traduz eventos de janela
//! em input, e a `egui-wgpu`, que pinta o resultado. O resto da engine nao
//! precisa conhecer nenhuma das duas.

use egui_wgpu::wgpu;
use rugine_render::{RenderTarget, Renderer, SurfacePass};
use winit::window::Window;

pub use egui;

/// Camada de UI: input, contexto da egui e pintura.
pub struct UiLayer {
    state: egui_winit::State,
    painter: egui_wgpu::Renderer,

    /// Id pelo qual a egui conhece a textura do viewport.
    ///
    /// Registrado uma vez e *atualizado* a cada resize, em vez de registrado de
    /// novo: registrar criaria um id novo por resize, vazando bind group e
    /// sampler a cada pixel que o usuario arrasta num divisor de painel.
    viewport_texture: egui::TextureId,

    /// Saida do ultimo `run`, aguardando pintura.
    pending: Option<PendingFrame>,

    /// Quantas primitivas recortadas a UI gerou no ultimo frame.
    ///
    /// E o numero de draws que a interface custa, e o contador que diz se a UI
    /// esta realmente produzindo geometria: zero com paineis declarados
    /// significa que algo os engoliu antes da tesselacao.
    primitivas: usize,
}

/// O que `run` produz e `paint` consome.
struct PendingFrame {
    primitives: Vec<egui::ClippedPrimitive>,
    textures_delta: egui::TexturesDelta,
    pixels_per_point: f32,
}

impl UiLayer {
    pub fn new(window: &Window, renderer: &mut Renderer) -> Self {
        let context = egui::Context::default();

        // O padrao da egui e seguir o tema do sistema, e a `egui-winit`
        // alimenta esse tema a partir da janela. Sem fixar aqui, um Windows em
        // modo claro troca a paleta inteira do editor e `set_visuals` abaixo e
        // ignorado — porque ele vale para o tema *corrente*, que acabou de
        // virar o claro.
        //
        // Um editor fixar o tema escuro e decisao de produto, nao limitacao:
        // seguir o sistema de novo e trocar esta linha por
        // `ThemePreference::System` e fornecer visuals para os dois temas.
        context.set_theme(egui::ThemePreference::Dark);
        context.set_visuals_of(egui::Theme::Dark, tema_escuro());

        let state = egui_winit::State::new(
            context,
            egui::ViewportId::ROOT,
            window,
            // `None`: a escala vem da janela, respeitando o DPI do sistema.
            None,
            // Escuro fixo, de novo: passar `window.theme()` reintroduziria o
            // tema do sistema por outra porta.
            Some(winit::window::Theme::Dark),
            // Limite de textura do device, para a egui nao tentar atlas
            // maiores do que a GPU aceita.
            Some(renderer.device().limits().max_texture_dimension_2d as usize),
        );

        let mut painter = egui_wgpu::Renderer::new(
            renderer.device(),
            // Linear, nao sRGB: a egui aplica a propria correcao de gama.
            renderer.ui_format(),
            egui_wgpu::RendererOptions::default(),
        );

        let viewport_texture = painter.register_native_texture(
            renderer.device(),
            renderer.viewport().view(),
            // Linear: o viewport e redimensionado para caber no painel, e
            // Nearest deixaria serrilhado visivel ao arrastar o divisor.
            wgpu::FilterMode::Linear,
        );

        Self {
            state,
            painter,
            viewport_texture,
            pending: None,
            primitivas: 0,
        }
    }

    /// Draws que a interface custou no ultimo frame.
    pub fn primitive_count(&self) -> usize {
        self.primitivas
    }

    /// Id da textura do viewport, para o editor exibir como imagem.
    pub fn viewport_texture(&self) -> egui::TextureId {
        self.viewport_texture
    }

    /// Reaponta o id do viewport para a textura nova apos um resize.
    pub fn refresh_viewport_texture(&mut self, device: &wgpu::Device, target: &RenderTarget) {
        self.painter.update_egui_texture_from_wgpu_texture(
            device,
            target.view(),
            wgpu::FilterMode::Linear,
            self.viewport_texture,
        );
    }

    /// Entrega um evento de janela a egui.
    ///
    /// Retorna `true` se a egui consumiu o evento. Quem chama deve respeitar
    /// isso: sem essa checagem, clicar num botao do editor tambem giraria a
    /// camera da cena atras dele.
    pub fn on_window_event(&mut self, window: &Window, event: &winit::event::WindowEvent) -> bool {
        self.state.on_window_event(window, event).consumed
    }

    /// Roda um frame de UI, guardando o resultado para `paint`.
    ///
    /// `build` e `FnMut` porque a egui pode rodar mais de um passe no mesmo
    /// frame — quando um layout descobre que precisa de outro tamanho, ele
    /// descarta o passe e refaz. O codigo de UI precisa tolerar isso, o que na
    /// pratica significa nao ter efeito colateral que conte quantas vezes rodou.
    pub fn run(&mut self, window: &Window, build: impl FnMut(&mut egui::Ui)) {
        let input = self.state.take_egui_input(window);
        // `run_ui` entrega a `Ui` raiz. Na egui 0.37 os paineis pendem de uma
        // `Ui`, nao do `Context` — e a raiz e a unica `Ui` em que um painel de
        // topo pode ser aberto.
        let output = self.state.egui_ctx().clone().run_ui(input, build);

        // Cursor, clipboard, links abertos: tudo que a UI pediu ao sistema.
        self.state
            .handle_platform_output(window, output.platform_output);

        let pixels_per_point = output.pixels_per_point;
        let primitives = self
            .state
            .egui_ctx()
            .tessellate(output.shapes, pixels_per_point);

        // Se o frame anterior nunca chegou a ser pintado (janela ocultada, por
        // exemplo), seus deltas tem que ser zerados antes do descarte, senao o
        // `Drop` da `TexturesDelta` entra em panico.
        if let Some(mut antigo) = self.pending.take() {
            antigo.textures_delta.clear();
        }

        self.primitivas = primitives.len();
        if log::log_enabled!(log::Level::Debug) {
            let contexto = self.state.egui_ctx();
            let tema = contexto.theme();
            let estilo = contexto.style_of(tema);
            log::debug!(
                "UI: {} primitivas | tema={tema:?} | fundo do painel={:?}",
                self.primitivas,
                estilo.visuals.panel_fill,
            );
        }

        self.pending = Some(PendingFrame {
            primitives,
            textures_delta: output.textures_delta,
            pixels_per_point,
        });
    }

    /// Pinta o frame guardado por `run` na janela.
    ///
    /// Devolve os command buffers que a egui gerou para upload — eles precisam
    /// ser submetidos antes do encoder principal, por isso sobem para quem
    /// chamou em vez de serem submetidos aqui.
    /// `background` `Some` limpa a janela antes de pintar (modo editor, em que
    /// a UI e dona da tela); `None` preserva o que ja foi desenhado (modo HUD,
    /// em que limpar apagaria a cena).
    pub fn paint(
        &mut self,
        pass: SurfacePass<'_>,
        background: Option<wgpu::Color>,
    ) -> Vec<wgpu::CommandBuffer> {
        let Some(mut frame) = self.pending.take() else {
            return Vec::new();
        };

        // Texturas novas (atlas de fonte, imagens) tem que existir antes do
        // draw que as usa.
        // Uma textura pode ter varios deltas num frame (o atlas de fonte cresce
        // em etapas), e a ordem entre eles importa: sao regioes aplicadas em
        // sequencia sobre a mesma textura.
        for (id, deltas) in &frame.textures_delta.set {
            for delta in deltas {
                self.painter
                    .update_texture(pass.device, pass.queue, *id, delta);
            }
        }

        // Aplicar os deltas nao os remove da struct, e o `Drop` de
        // `TexturesDelta` entra em panico se ela for destruida com deltas
        // pendentes — e a forma que a epaint usa para pegar integracoes que
        // esquecem de aplicar texturas. Zeramos depois de consumir.
        let liberar: Vec<egui::TextureId> = frame.textures_delta.free.iter().copied().collect();
        frame.textures_delta.clear();

        let descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: pass.size_in_pixels,
            pixels_per_point: frame.pixels_per_point,
        };

        let buffers = self.painter.update_buffers(
            pass.device,
            pass.queue,
            pass.encoder,
            &frame.primitives,
            &descriptor,
        );

        {
            let render_pass = pass.encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rugine::ui_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: pass.view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: match background {
                            Some(color) => wgpu::LoadOp::Clear(color),
                            None => wgpu::LoadOp::Load,
                        },
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            // A egui exige um render pass de lifetime estatico. `forget_lifetime`
            // e a saida que ela mesma documenta: o pass continua preso ao
            // encoder, so o verificador de emprestimos deixa de acompanhar.
            self.painter.render(
                &mut render_pass.forget_lifetime(),
                &frame.primitives,
                &descriptor,
            );
        }

        // So liberamos texturas depois de pintar, porque o draw acima ainda
        // pode estar referenciando alguma delas.
        for id in &liberar {
            self.painter.free_texture(id);
        }

        buffers
    }
}

/// Tema escuro da Rugine.
///
/// Os valores vem do visual de referencia do editor: cinzas frios, cantos
/// pouco arredondados e azul so no que esta ativo.
fn tema_escuro() -> egui::Visuals {
    let mut visuals = egui::Visuals::dark();

    visuals.panel_fill = egui::Color32::from_rgb(34, 34, 38);
    visuals.window_fill = egui::Color32::from_rgb(40, 40, 45);
    visuals.extreme_bg_color = egui::Color32::from_rgb(24, 24, 27);
    visuals.faint_bg_color = egui::Color32::from_rgb(44, 44, 50);

    visuals.widgets.noninteractive.bg_fill = egui::Color32::from_rgb(40, 40, 45);
    visuals.widgets.inactive.bg_fill = egui::Color32::from_rgb(52, 52, 58);
    visuals.widgets.hovered.bg_fill = egui::Color32::from_rgb(64, 64, 72);
    visuals.widgets.active.bg_fill = egui::Color32::from_rgb(52, 110, 190);

    visuals.selection.bg_fill = egui::Color32::from_rgb(45, 95, 165);
    visuals.selection.stroke = egui::Stroke::new(1.0, egui::Color32::from_rgb(190, 210, 240));

    visuals
}
