//! Painel Viewport: a cena renderizada, com barra de ferramentas e
//! estatisticas sobrepostas.

use rugine_app::prelude::{Context, RenderStats, Vec3, egui};

use crate::estado::{Editor, Espaco, ExibirColliders, Ferramenta};
use crate::widgets;

pub fn desenhar(ui: &mut egui::Ui, editor: &mut Editor, ctx: &mut Context<'_>) {
    let disponivel = ui.available_size();

    // A textura e pedida em pixels fisicos, nao em pontos logicos: num monitor
    // com escala 150%, usar pontos renderizaria a cena a dois tercos da
    // resolucao e ela sairia borrada.
    let escala = ui.ctx().pixels_per_point();
    let largura_px = (disponivel.x * escala).round().max(1.0) as u32;
    let altura_px = (disponivel.y * escala).round().max(1.0) as u32;

    editor.viewport_px = [largura_px, altura_px];
    ctx.request_viewport_size(largura_px, altura_px);

    // A imagem da cena preenche o painel; a barra e as estatisticas sao
    // desenhadas por cima, no mesmo retangulo.
    let (rect, resposta) = ui.allocate_exact_size(disponivel, egui::Sense::click_and_drag());

    navegar(ui, editor, &resposta, ctx.time.delta());

    ui.painter().image(
        ctx.viewport_texture,
        rect,
        egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
        egui::Color32::WHITE,
    );

    let stats = ctx.renderer.stats();
    let fps = ctx.time.fps();

    // Conteudo sobreposto, ancorado no topo do retangulo da imagem.
    let mut sobreposto = ui.new_child(
        egui::UiBuilder::new()
            .max_rect(rect.shrink(6.0))
            .layout(egui::Layout::top_down(egui::Align::Min)),
    );

    if editor.exibir_estatisticas {
        barra_estatisticas(&mut sobreposto, editor, stats, fps, ctx.ui_draws);
    }

    sobreposto.add_space(4.0);
    barra_ferramentas(&mut sobreposto, editor);
}

/// Traduz mouse e teclado em movimento de camera, no esquema do Unity.
///
/// | Gesto                  | Acao                                  |
/// |------------------------|---------------------------------------|
/// | Botao direito          | Olhar em volta (camera parada)        |
/// | Direito + WASD / QE    | Voar; Shift acelera                   |
/// | Alt + botao esquerdo   | Orbitar em torno do pivo              |
/// | Botao do meio          | Deslocar                              |
/// | Roda                   | Aproximar                             |
/// | F                      | Enquadrar a selecao                   |
///
/// O botao esquerdo sozinho fica livre de proposito: no Unity ele seleciona, e
/// e o que o picking vai usar.
fn navegar(ui: &egui::Ui, editor: &mut Editor, resposta: &egui::Response, delta_tempo: f32) {
    let (alt, shift) = ui.input(|e| (e.modifiers.alt, e.modifiers.shift));
    let direito_pressionado = ui.input(|e| e.pointer.secondary_down());

    // O modo de voo e um estado, nao um gesto: ele comeca quando o direito
    // desce sobre o viewport e so termina quando o botao sobe, mesmo que o
    // ponteiro saia do painel no meio do caminho. Sem guardar isso, soltar o
    // botao fora do viewport deixaria a camera presa voando.
    if resposta.hovered() && ui.input(|e| e.pointer.button_pressed(egui::PointerButton::Secondary))
    {
        editor.voando = true;
    }
    if !direito_pressionado {
        editor.voando = false;
    }

    if editor.voando {
        // Delta global do ponteiro, nao `drag_delta`: o arrasto continua
        // valendo mesmo com o ponteiro fora do retangulo do viewport.
        let delta = ui.input(|e| e.pointer.delta());
        editor.camera.olhar(delta.x, delta.y);

        let mut direcao = Vec3::ZERO;
        ui.input(|e| {
            if e.key_down(egui::Key::W) {
                direcao.z += 1.0;
            }
            if e.key_down(egui::Key::S) {
                direcao.z -= 1.0;
            }
            if e.key_down(egui::Key::D) {
                direcao.x += 1.0;
            }
            if e.key_down(egui::Key::A) {
                direcao.x -= 1.0;
            }
            if e.key_down(egui::Key::E) {
                direcao.y += 1.0;
            }
            if e.key_down(egui::Key::Q) {
                direcao.y -= 1.0;
            }
        });

        editor.camera.voar(direcao, delta_tempo, shift);
        return;
    }

    let delta = resposta.drag_delta();
    if delta != egui::Vec2::ZERO {
        if alt && resposta.dragged_by(egui::PointerButton::Primary) {
            editor.camera.orbitar(delta.x, delta.y);
        } else if resposta.dragged_by(egui::PointerButton::Middle) {
            editor.camera.deslocar(delta.x, delta.y);
        }
    }

    // A roda so age com o ponteiro sobre o viewport: sem essa checagem,
    // rolar a lista da Hierarquia tambem aproximaria a camera.
    if resposta.hovered() {
        let roda = ui.input(|e| e.smooth_scroll_delta.y);
        if roda != 0.0 {
            editor.camera.aproximar(roda * 0.01);
        }
    }
}

/// Linha de contadores do frame.
fn barra_estatisticas(
    ui: &mut egui::Ui,
    editor: &Editor,
    stats: RenderStats,
    fps: f32,
    ui_draws: u32,
) {
    let [largura, altura] = editor.viewport_px;

    moldura(ui).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 6.0;

            // Abaixo de 30 fps o numero fica laranja: e o limiar em que a
            // interacao no viewport comeca a parecer presa.
            let cor_fps = if fps < 30.0 {
                widgets::ATENCAO
            } else {
                egui::Color32::from_gray(200)
            };
            ui.label(
                egui::RichText::new(format!("{fps:.0} FPS"))
                    .size(11.0)
                    .color(cor_fps),
            );

            separador(ui);
            ui.label(
                egui::RichText::new(format!("{} draws", stats.draw_calls))
                    .size(11.0)
                    .color(egui::Color32::from_gray(185)),
            );

            separador(ui);
            ui.label(
                egui::RichText::new(format!("{} tris", stats.triangles))
                    .size(11.0)
                    .color(egui::Color32::from_gray(185)),
            );

            separador(ui);
            ui.label(
                egui::RichText::new(format!("UI: {ui_draws} draws"))
                    .size(11.0)
                    .color(egui::Color32::from_gray(185)),
            );

            separador(ui);
            ui.label(
                egui::RichText::new(format!("{largura}x{altura}"))
                    .size(11.0)
                    .color(egui::Color32::from_gray(150)),
            );
        });
    });
}

/// Barra com ferramenta, espaco, snap e alternancias de exibicao.
fn barra_ferramentas(ui: &mut egui::Ui, editor: &mut Editor) {
    moldura(ui).show(ui, |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = 3.0;

            for ferramenta in Ferramenta::TODAS {
                let ativa = editor.ferramenta == ferramenta;
                let resposta = widgets::botao_alternavel(ui, ativa, ferramenta.rotulo())
                    .on_hover_text(format!("Atalho: {}", ferramenta.atalho()));
                if resposta.clicked() {
                    editor.ferramenta = ferramenta;
                }
            }

            separador(ui);

            let rotulo_espaco = editor.espaco.rotulo();
            if widgets::botao_alternavel(ui, false, rotulo_espaco)
                .on_hover_text("Alterna entre referencial global e local")
                .clicked()
            {
                editor.espaco = match editor.espaco {
                    Espaco::Global => Espaco::Local,
                    Espaco::Local => Espaco::Global,
                };
            }

            if widgets::botao_alternavel(ui, editor.snap, "Snap").clicked() {
                editor.snap = !editor.snap;
            }

            if editor.snap {
                ui.add_sized(
                    [48.0, 18.0],
                    egui::DragValue::new(&mut editor.tamanho_snap)
                        .speed(0.05)
                        .range(0.01..=10.0)
                        .fixed_decimals(2),
                );
            }

            if widgets::botao_alternavel(ui, editor.grade, "Grade").clicked() {
                editor.grade = !editor.grade;
            }

            separador(ui);

            egui::ComboBox::from_id_salt("colliders")
                .selected_text(format!("Colliders: {}", editor.colliders.rotulo()))
                .show_ui(ui, |ui| {
                    for modo in ExibirColliders::TODOS {
                        ui.selectable_value(&mut editor.colliders, modo, modo.rotulo());
                    }
                });

            if widgets::botao_alternavel(ui, editor.exibir_ui, "UI").clicked() {
                editor.exibir_ui = !editor.exibir_ui;
            }

            if widgets::botao_alternavel(ui, editor.exibir_estatisticas, "Estatisticas").clicked() {
                editor.exibir_estatisticas = !editor.exibir_estatisticas;
            }
        });
    });
}

/// Moldura escura translucida, para o texto sobreposto ficar legivel sobre
/// qualquer cena.
fn moldura(ui: &egui::Ui) -> egui::Frame {
    egui::Frame::new()
        .fill(egui::Color32::from_black_alpha(165))
        .corner_radius(3.0)
        .inner_margin(egui::Margin::symmetric(6, 3))
        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
}

fn separador(ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new("|")
            .size(11.0)
            .color(egui::Color32::from_gray(90)),
    );
}
