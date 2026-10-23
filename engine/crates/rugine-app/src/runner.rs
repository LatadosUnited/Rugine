//! O loop: traduz eventos do winit em callbacks da engine.

use std::sync::Arc;

use anyhow::{Context as _, Result};
use rugine_core::time::Time;
use rugine_render::{CenaGpu, FrameContent, Renderer};
use rugine_ui::UiLayer;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, KeyEvent, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::app::{App, Context};
use crate::janela::Decoracao;

/// Como a interface participa do frame.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum UiMode {
    /// Sem egui. A cena ocupa a janela. É o modo de um jogo publicado.
    #[default]
    Disabled,
    /// A cena ocupa a janela e a interface e desenhada por cima. Modo HUD.
    Overlay,
    /// A interface e dona da janela e a cena vai para uma textura que ela
    /// exibe dentro de um painel. Modo editor.
    Editor,
}

/// Como a janela nasce.
pub struct EngineConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    /// Fecha a janela com Esc. Conveniente durante o desenvolvimento,
    /// indesejado num jogo publicado — e perigoso num editor, onde Esc e uma
    /// tecla de cancelar.
    pub exit_on_escape: bool,
    pub ui: UiMode,
    /// Quem desenha a barra de titulo.
    pub decoracao: Decoracao,
    /// Sincronizar a apresentacao com o monitor.
    ///
    /// Ligado por padrao: sem isso a GPU desenha frames que ninguem ve.
    /// Desligar serve para medir o custo real do frame, ja que com vsync o
    /// contador de fps mede a taxa do monitor. `RUGINE_VSYNC=0` sobrepoe sem
    /// recompilar.
    pub vsync: bool,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            title: "Rugine".to_owned(),
            width: 1280,
            height: 720,
            exit_on_escape: true,
            ui: UiMode::Disabled,
            decoracao: Decoracao::Sistema,
            vsync: true,
        }
    }
}

/// Abre a janela e roda a aplicacao ate ela terminar.
///
/// Bloqueia a thread chamadora: em macOS e iOS o loop de eventos *exige* a
/// thread principal, entao esta e a unica forma portavel de estruturar isto.
pub fn run<A: App>(config: EngineConfig, app: A) -> Result<()> {
    let event_loop = EventLoop::new().context("falha ao criar o loop de eventos")?;

    // Poll, nao Wait: um jogo desenha continuamente. `Wait` dormiria ate o
    // proximo evento de input, o que e o certo para um editor parado, mas
    // impediria qualquer animacao.
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut runner = Runner {
        config,
        app,
        state: None,
        requested_viewport_size: None,
        error: None,
    };

    event_loop.run_app(&mut runner)?;

    // Falhas dentro dos callbacks do winit nao podem ser retornadas de la
    // (as assinaturas do trait retornam `()`), entao sao guardadas e
    // repropagadas aqui, depois que o loop encerra.
    if let Some(error) = runner.error.take() {
        return Err(error);
    }

    Ok(())
}

/// Ajustes de janela que so existem numa plataforma.
///
/// Duas funcoes com `cfg`, e nao um `cfg` em volta de um bloco no meio do
/// codigo: dentro de uma funcao, o bloco some no Linux e deixa para tras o
/// `mut` de que ele era o unico usuario — e `unused_mut` vira erro sob
/// `-D warnings`. Foi assim que a primeira execucao do CI quebrou, num aviso
/// que nao aparece no Windows.
#[cfg(target_os = "windows")]
fn extras_de_plataforma(
    attributes: winit::window::WindowAttributes,
    decoracao: Decoracao,
) -> winit::window::WindowAttributes {
    use winit::platform::windows::WindowAttributesExtWindows as _;

    if decoracao == Decoracao::Propria {
        // Sem a decoracao do sistema, o Windows tira tambem a sombra e a borda
        // da janela, e ela fica um retangulo chapado colado no fundo. Isto
        // devolve as duas coisas sem devolver a barra de titulo.
        attributes.with_undecorated_shadow(true)
    } else {
        attributes
    }
}

#[cfg(not(target_os = "windows"))]
fn extras_de_plataforma(
    attributes: winit::window::WindowAttributes,
    _decoracao: Decoracao,
) -> winit::window::WindowAttributes {
    attributes
}

/// Estado que so existe depois que a plataforma nos deu uma janela.
///
/// Em Android a aplicacao pode ser suspensa e retomada, destruindo e
/// recriando a janela — por isso isto e `Option` no runner, e nao campos
/// diretos: nao existe janela antes de `resumed`.
struct State {
    window: Arc<Window>,
    renderer: Renderer,
    ui: Option<UiLayer>,
    time: Time,
    /// O que desenhar, preenchido pela aplicacao em `update`.
    cena: CenaGpu,
}

struct Runner<A: App> {
    config: EngineConfig,
    app: A,
    state: Option<State>,
    /// Tamanho de viewport pedido pela UI no frame anterior.
    requested_viewport_size: Option<[u32; 2]>,
    error: Option<anyhow::Error>,
}

impl<A: App> Runner<A> {
    /// Interrompe o loop guardando o erro para `run` repropagar.
    fn fail(&mut self, event_loop: &ActiveEventLoop, error: anyhow::Error) {
        self.error = Some(error);
        event_loop.exit();
    }

    /// Um frame completo.
    fn frame(&mut self, event_loop: &ActiveEventLoop) {
        // Desestruturado para que `app`, `state` e `requested_viewport_size`
        // sejam emprestados em separado: sem isto, chamar um metodo de `self`
        // travaria o acesso aos outros campos.
        let Runner {
            config,
            app,
            state,
            requested_viewport_size,
            ..
        } = self;

        let Some(State {
            window,
            renderer,
            ui,
            time,
            cena,
        }) = state.as_mut()
        else {
            return;
        };

        time.tick();

        // O tamanho que a UI pediu no frame anterior so pode ser aplicado
        // agora, antes de desenhar a cena.
        if let Some([width, height]) = requested_viewport_size.take()
            && renderer.resize_viewport(width, height)
            && let Some(ui) = ui.as_mut()
        {
            // A textura foi recriada; o id da egui precisa reapontar.
            ui.refresh_viewport_texture(renderer.device(), renderer.viewport());
        }

        let viewport_texture = ui
            .as_ref()
            .map(|ui| ui.viewport_texture())
            .unwrap_or_default();
        let ui_draws = ui
            .as_ref()
            .map(|ui| ui.primitive_count() as u32)
            .unwrap_or(0);

        let mut exit_requested = false;
        let mut requested_size = None;

        // `update` vem antes de qualquer desenho porque e ele que monta a
        // cena: inverter a ordem desenharia sempre o frame anterior.
        app.update(&mut Context {
            time,
            renderer,
            viewport_texture,
            ui_draws,
            cena,
            window,
            requested_viewport_size: &mut requested_size,
            exit_requested: &mut exit_requested,
        });

        if config.ui == UiMode::Editor {
            // Cena para a textura offscreen, antes da UI, que vai exibi-la.
            renderer.draw_viewport(cena);
        }

        match ui.as_mut() {
            None => {
                // Sem UI: a cena vai direto para a janela.
                window.pre_present_notify();
                renderer.render(cena);
            }
            Some(ui) => {
                ui.run(window, |raiz| {
                    app.ui(
                        raiz,
                        &mut Context {
                            time,
                            renderer,
                            viewport_texture,
                            ui_draws,
                            cena,
                            window,
                            requested_viewport_size: &mut requested_size,
                            exit_requested: &mut exit_requested,
                        },
                    );
                });

                let (content, background) = match config.ui {
                    // No editor a UI e dona da tela e pode limpar.
                    UiMode::Editor => (
                        FrameContent::Ui,
                        Some(rugine_render::wgpu::Color {
                            r: 0.08,
                            g: 0.08,
                            b: 0.09,
                            a: 1.0,
                        }),
                    ),
                    // No HUD, limpar apagaria a cena desenhada logo antes.
                    _ => (FrameContent::SceneThenUi, None),
                };

                window.pre_present_notify();
                renderer.present(content, cena, |pass| ui.paint(pass, background));
            }
        }

        *requested_viewport_size = requested_size;

        if exit_requested {
            event_loop.exit();
        }
    }
}

impl<A: App> ApplicationHandler for Runner<A> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        // `resumed` pode disparar mais de uma vez no ciclo de vida de um app
        // movel. Recriar a GPU aqui vazaria o device anterior.
        if self.state.is_some() {
            return;
        }

        let attributes = Window::default_attributes()
            .with_title(&self.config.title)
            .with_inner_size(PhysicalSize::new(self.config.width, self.config.height))
            .with_decorations(self.config.decoracao == Decoracao::Sistema);

        let attributes = extras_de_plataforma(attributes, self.config.decoracao);

        let window = match event_loop.create_window(attributes) {
            Ok(window) => Arc::new(window),
            Err(error) => {
                return self.fail(
                    event_loop,
                    anyhow::Error::new(error).context("falha ao criar a janela"),
                );
            }
        };

        // O tamanho real vem da janela, nao da config: o gerenciador de janelas
        // tem a palavra final, e em telas com escala o tamanho fisico difere do
        // logico que pedimos.
        let size = window.inner_size();

        // `block_on` aqui e deliberado: a inicializacao da GPU e assincrona na
        // API do wgpu (por causa do WebGPU), mas no desktop ela resolve de
        // imediato e nao ha runtime async para hospedar. Em wasm isto viraria
        // uma inicializacao em duas fases.
        let mut renderer = match pollster::block_on(Renderer::new(
            window.clone(),
            size.width,
            size.height,
            self.config.vsync,
        )) {
            Ok(renderer) => renderer,
            Err(error) => {
                return self.fail(
                    event_loop,
                    anyhow::Error::new(error).context("falha ao inicializar a GPU"),
                );
            }
        };

        let ui = match self.config.ui {
            UiMode::Disabled => None,
            _ => Some(UiLayer::new(&window, &mut renderer)),
        };

        log::info!(
            "janela {}x{} pronta | ui {:?}",
            size.width,
            size.height,
            self.config.ui
        );

        let mut state = State {
            window,
            renderer,
            ui,
            time: Time::new(),
            cena: CenaGpu::default(),
        };

        let viewport_texture = state
            .ui
            .as_ref()
            .map(|ui| ui.viewport_texture())
            .unwrap_or_default();

        let mut exit_requested = false;
        let mut requested_size = None;

        self.app.init(&mut Context {
            time: &state.time,
            renderer: &mut state.renderer,
            viewport_texture,
            ui_draws: 0,
            cena: &mut state.cena,
            window: &state.window,
            requested_viewport_size: &mut requested_size,
            exit_requested: &mut exit_requested,
        });

        if exit_requested {
            event_loop.exit();
        }

        self.state = Some(state);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        // A egui ve o evento primeiro. Se ela o consumiu, o resto da engine
        // nao deve reagir: sem isso, clicar num botao do editor tambem agiria
        // sobre a cena atras dele, e digitar num campo de texto acionaria
        // atalhos de teclado.
        let consumido = match self.state.as_mut() {
            Some(state) => match state.ui.as_mut() {
                Some(ui) => ui.on_window_event(&state.window, &event),
                None => false,
            },
            None => false,
        };

        let Some(state) = self.state.as_mut() else {
            return;
        };

        match event {
            // Fechar e redimensionar nunca sao "consumidos": sao da janela,
            // nao da interface.
            WindowEvent::CloseRequested => {
                log::info!("fechamento pedido pela janela");
                event_loop.exit();
            }

            WindowEvent::Resized(size) => {
                // Só avisamos o jogo quando a surface realmente mudou: ele usa
                // isto para recalcular matriz de projecao e alvos de render, e
                // refazer isso por evento redundante e desperdicio.
                if state.renderer.resize(size.width, size.height) {
                    self.app.resized(size.width, size.height);
                }
            }

            WindowEvent::RedrawRequested => self.frame(event_loop),

            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } if self.config.exit_on_escape && !consumido => {
                event_loop.exit();
            }

            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Com ControlFlow::Poll, pedir redraw aqui e o que mantem a aplicacao
        // desenhando continuamente.
        if let Some(state) = self.state.as_ref() {
            state.window.request_redraw();
        }
    }
}
