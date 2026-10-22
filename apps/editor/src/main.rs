//! Editor da Rugine.
//!
//! O editor e so um [`App`] como qualquer outro: ele nao tem acesso privilegiado
//! a engine. Se algum painel precisar de algo que a API publica nao oferece, a
//! resposta certa e melhorar a API, nao abrir uma porta dos fundos.

mod camera;
mod docking;
mod estado;
mod gizmo;
mod paineis;
mod picking;
mod widgets;

use rugine_app::prelude::*;

use estado::{Editor, EstadoCompilacao, Ferramenta};

impl App for Editor {
    fn init(&mut self, ctx: &mut Context<'_>) {
        let (largura, altura) = ctx.renderer.size();
        log::info!(
            "editor aberto em {largura}x{altura} | {} entidades na cena",
            self.cena.len()
        );
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        self.talvez_capturar(ctx);

        // Monta a cena de GPU a partir do grafo: uma instancia por entidade
        // visivel, na sua matriz de mundo. E o que faz editar uma transform
        // no Inspector mover o cubo no viewport, em vez de o viewport mostrar
        // uma cena paralela que so coincide por acaso.
        let selecionada = self.cena.selected();
        let mut instancias = Vec::with_capacity(self.cena.len());

        for id in self.cena.entidades() {
            let Some(entidade) = self.cena.get(id) else {
                continue;
            };
            if !entidade.visible {
                continue;
            }

            // A selecionada fica laranja: e o unico retorno visual de selecao
            // enquanto nao houver contorno nem gizmo.
            let cor = if Some(id) == selecionada {
                [0.95, 0.62, 0.23, 1.0]
            } else {
                [0.62, 0.64, 0.70, 1.0]
            };

            instancias.push(Instancia::new(self.cena.world_matrix(id), cor));
        }

        // O gizmo e reconstruido todo frame a partir da selecao e da
        // camera: ele escala com a distancia, entao guardar seria guardar
        // algo que vence no frame seguinte.
        let sobrepostas = paineis::interacao::gizmo_atual(self)
            .map(|gizmo| gizmo.instancias())
            .unwrap_or_default();

        let proporcao = ctx.renderer.viewport().aspect_ratio();
        ctx.definir_cena(CenaGpu {
            view_proj: self.camera.view_proj(proporcao),
            instancias,
            grade: self.grade,
            sobrepostas,
        });

        // A "compilacao" e simulada: ela existe para que o estado da barra de
        // menu e do painel de scripts seja real, e nao um rotulo fixo. Quando
        // houver scripts de verdade, so a origem do tempo muda.
        if let EstadoCompilacao::Compilando { segundos } = &mut self.compilacao {
            *segundos += ctx.time.delta();
            if *segundos >= 3.0 {
                self.compilacao = EstadoCompilacao::Ocioso;
                log::info!("compilacao concluida");
            }
        }
    }

    fn ui(&mut self, raiz: &mut egui::Ui, ctx: &mut Context<'_>) {
        atalhos(raiz.ctx(), self);

        // A ordem de declaracao define quem fica com os cantos: a barra de
        // menu atravessa o topo inteiro, os paineis laterais vao do menu ate
        // embaixo, e o painel inferior fica entre eles. E a mesma ordem da
        // captura de referencia.
        // Mais alta quando e a nossa barra: ela acumula a funcao da barra de
        // titulo do sistema, e botoes de janela em 26px ficam apertados.
        let altura_topo = if self.barra_propria { 32.0 } else { 26.0 };

        egui::Panel::top("barra_menu")
            .exact_size(altura_topo)
            .show(raiz, |ui| {
                paineis::menu::desenhar(ui, self, ctx);
            });

        // Todo o resto da janela e o dock: cada painel e uma aba que o
        // usuario pode arrastar para outro lugar, agrupar com outra ou soltar
        // numa janela flutuante. O arranjo inicial vem de `layout_padrao`.
        //
        // O dock sai do `Editor` durante o desenho porque o visualizador
        // precisa de `&mut Editor` ao mesmo tempo que a `egui_dock` precisa de
        // `&mut DockState`.
        let mut dock = self
            .dock
            .take()
            .expect("o dock e devolvido ao fim de todo frame");

        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(raiz, |ui| {
                let estilo = egui_dock::Style::from_egui(ui.style().as_ref());
                let mut visualizador = docking::Visualizador {
                    editor: &mut *self,
                    ctx: &mut *ctx,
                };

                egui_dock::DockArea::new(&mut dock)
                    .style(estilo)
                    .show_close_buttons(false)
                    .show_add_buttons(false)
                    .show_inside(ui, &mut visualizador);
            });

        self.dock = Some(dock);

        // Por ultimo: as alcas ficam numa camada de frente e precisam ser
        // declaradas depois dos paineis que elas cobrem.
        if self.barra_propria {
            paineis::bordas::desenhar(raiz.ctx(), ctx);
        }
    }
}

impl Editor {
    /// Salva o viewport em PNG quando `RUGINE_CAPTURA` aponta um caminho.
    ///
    /// Existe porque "nenhum erro de validacao" nao e o mesmo que "apareceu
    /// na tela": sem olhar o resultado, um gizmo pode ser gerado, enviado e
    /// desenhado sem nunca ficar visivel.
    ///
    /// Espera alguns frames antes de capturar: o primeiro frame ainda nao
    /// tem o viewport no tamanho final, porque a UI so informa o tamanho
    /// depois de se construir uma vez.
    fn talvez_capturar(&mut self, ctx: &mut Context<'_>) {
        let frame_da_captura: u64 = std::env::var("RUGINE_CAPTURA_FRAME")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(30);

        let Some(caminho) = std::env::var("RUGINE_CAPTURA").ok() else {
            return;
        };
        if ctx.time.frames() != frame_da_captura {
            return;
        }

        let Some(captura) = ctx.renderer.capturar_viewport() else {
            log::error!("captura falhou");
            return;
        };

        // Conta pixels fortemente coloridos: e o indicador de que o gizmo
        // chegou a tela, sem precisar abrir a imagem.
        let coloridos = captura.contar(|[r, g, b, _]| {
            let maior = r.max(g).max(b) as i32;
            let menor = r.min(g).min(b) as i32;
            maior > 90 && maior - menor > 50
        });

        log::info!(
            "captura {}x{}: {coloridos} pixels coloridos",
            captura.largura,
            captura.altura
        );

        if let Err(erro) = salvar_png(&caminho, &captura) {
            log::error!("nao consegui salvar {caminho}: {erro}");
        } else {
            log::info!("captura salva em {caminho}");
        }

        ctx.exit();
    }
}

/// Grava a captura como PNG.
fn salvar_png(caminho: &str, captura: &rugine_render::Captura) -> std::io::Result<()> {
    let arquivo = std::fs::File::create(caminho)?;
    let mut encoder = png::Encoder::new(
        std::io::BufWriter::new(arquivo),
        captura.largura,
        captura.altura,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);

    let mut escritor = encoder.write_header()?;
    escritor.write_image_data(&captura.rgba)?;
    Ok(())
}

/// Atalhos globais do editor.
///
/// Só agem quando nenhum campo de texto tem foco: sem essa checagem, digitar
/// "move" no campo de busca trocaria de ferramenta no meio da palavra.
fn atalhos(egui: &egui::Context, editor: &mut Editor) {
    if egui.egui_wants_keyboard_input() {
        return;
    }

    // Durante o voo, WASDQE pilota a camera. Sem esta saida, voar para a
    // frente tambem trocaria a ferramenta para Mover, e subir trocaria para
    // Rotacionar.
    if editor.voando {
        return;
    }

    egui.input(|entrada| {
        for (tecla, ferramenta) in [
            (egui::Key::Q, Ferramenta::Selecionar),
            (egui::Key::W, Ferramenta::Mover),
            (egui::Key::E, Ferramenta::Rotacionar),
            (egui::Key::R, Ferramenta::Escalar),
        ] {
            if entrada.key_pressed(tecla) {
                editor.ferramenta = ferramenta;
            }
        }

        if entrada.key_pressed(egui::Key::F7)
            && !matches!(editor.compilacao, EstadoCompilacao::Compilando { .. })
        {
            editor.compilacao = EstadoCompilacao::Compilando { segundos: 0.0 };
        }

        // F enquadra a selecao, como na maioria dos editores 3D.
        if entrada.key_pressed(egui::Key::F)
            && let Some(id) = editor.cena.selected()
        {
            let posicao = editor.cena.world_matrix(id).transform_point3(Vec3::ZERO);
            editor.camera.enquadrar(posicao);
        }

        if entrada.key_pressed(egui::Key::Delete)
            && let Some(id) = editor.cena.selected()
        {
            editor.cena.despawn(id);
        }
    });
}

fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // No macOS isto devolve `Sistema`: la a winit nao implementa
    // `drag_resize_window`, e uma janela sem decoracao ficaria sem como ser
    // redimensionada. No Windows e no Linux a barra e nossa.
    let decoracao = Decoracao::recomendada();

    let editor = Editor {
        barra_propria: decoracao == Decoracao::Propria,
        ..Default::default()
    };

    log::info!("decoracao de janela: {decoracao:?}");

    run(
        EngineConfig {
            title: "Rugine — editor".to_owned(),
            width: 1600,
            height: 900,
            // Esc nao fecha um editor: ali ele e tecla de cancelar, e fechar a
            // aplicacao por engano custa trabalho perdido.
            exit_on_escape: false,
            ui: UiMode::Editor,
            decoracao,
            // Vsync ligado: um editor passa a maior parte do tempo parado, e
            // desenhar sem limite so esquentaria a maquina a toa. O menu Ver
            // desliga quando se quer medir.
            vsync: true,
        },
        editor,
    )
}
