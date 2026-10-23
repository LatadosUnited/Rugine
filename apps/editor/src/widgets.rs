//! Pecas visuais reusadas pelos paineis.
//!
//! Centralizadas aqui porque sao o que da unidade ao editor: se cada painel
//! desenhar seu proprio cabecalho, eles divergem no primeiro ajuste.

use rugine_app::prelude::egui;

/// Azul de destaque, usado so no que esta ativo.
pub const DESTAQUE: egui::Color32 = egui::Color32::from_rgb(52, 110, 190);

/// Laranja de atencao (compilando, avisos).
pub const ATENCAO: egui::Color32 = egui::Color32::from_rgb(222, 145, 58);

// O cabecalho de painel que existia aqui saiu: quem desenha o titulo agora e a
// aba do dock, e manter os dois daria dois titulos empilhados.

/// Rotulo de secao dentro de um painel, com um traco a esquerda.
///
/// O traco e desenhado, nao escrito: o travessao tipografico tem o mesmo
/// problema das setas, pode nao existir na fonte empacotada.
pub fn secao(ui: &mut egui::Ui, texto: &str) {
    ui.add_space(3.0);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(egui::vec2(10.0, 8.0), egui::Sense::hover());
        ui.painter().hline(
            rect.left()..=rect.right(),
            rect.center().y,
            egui::Stroke::new(1.0, egui::Color32::from_gray(110)),
        );
        ui.label(
            egui::RichText::new(texto)
                .size(11.0)
                .color(egui::Color32::from_gray(150)),
        );
    });
    ui.add_space(3.0);
}

/// Qual simbolo um botao de janela desenha.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum IconeJanela {
    Minimizar,
    Maximizar,
    Restaurar,
    Fechar,
}

/// Vermelho do botao de fechar sob o mouse.
///
/// Fechar ficar vermelho no hover e convencao em toda plataforma: e o unico
/// botao da barra cuja acao perde trabalho.
const VERMELHO_FECHAR: egui::Color32 = egui::Color32::from_rgb(196, 43, 48);

/// Botao da barra de titulo.
///
/// Os simbolos sao desenhados, nao escritos. Os caracteres convencionais para
/// isto (`\u{2014}`, `\u{25A1}`, `\u{2715}`) sofrem do mesmo problema das
/// setas: podem nao existir na fonte e virar quadrado vazio.
pub fn botao_janela(ui: &mut egui::Ui, icone: IconeJanela) -> egui::Response {
    let tamanho = egui::vec2(42.0, ui.available_height());
    let (rect, resposta) = ui.allocate_exact_size(tamanho, egui::Sense::click());

    let destacado = resposta.hovered() || resposta.is_pointer_button_down_on();

    if destacado {
        let fundo = if icone == IconeJanela::Fechar {
            VERMELHO_FECHAR
        } else {
            egui::Color32::from_rgb(62, 62, 70)
        };
        ui.painter().rect_filled(rect, 0.0, fundo);
    }

    let cor = if destacado {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_gray(190)
    };

    // Traco fino e nitido: 1px em tela nao escalada, e o que faz o simbolo
    // parecer de sistema em vez de desenhado.
    let traco = egui::Stroke::new(1.0, cor);
    let pintor = ui.painter();
    let centro = rect.center();
    let meio = 5.0;

    match icone {
        IconeJanela::Minimizar => {
            pintor.hline((centro.x - meio)..=(centro.x + meio), centro.y + 0.5, traco);
        }
        IconeJanela::Maximizar => {
            pintor.rect_stroke(
                egui::Rect::from_center_size(centro, egui::vec2(meio * 2.0, meio * 2.0)),
                0.0,
                traco,
                egui::StrokeKind::Inside,
            );
        }
        IconeJanela::Restaurar => {
            // Dois retangulos deslocados: o de tras indica a janela voltando
            // ao tamanho anterior.
            let frente =
                egui::Rect::from_center_size(centro + egui::vec2(-1.5, 1.5), egui::vec2(9.0, 9.0));
            let tras =
                egui::Rect::from_center_size(centro + egui::vec2(1.5, -1.5), egui::vec2(9.0, 9.0));
            pintor.rect_stroke(tras, 0.0, traco, egui::StrokeKind::Inside);
            pintor.rect_filled(frente, 0.0, ui.visuals().panel_fill);
            pintor.rect_stroke(frente, 0.0, traco, egui::StrokeKind::Inside);
        }
        IconeJanela::Fechar => {
            pintor.line_segment(
                [
                    centro + egui::vec2(-meio, -meio),
                    centro + egui::vec2(meio, meio),
                ],
                traco,
            );
            pintor.line_segment(
                [
                    centro + egui::vec2(meio, -meio),
                    centro + egui::vec2(-meio, meio),
                ],
                traco,
            );
        }
    }

    resposta
}

/// Botao que fica azul quando ativo.
///
/// `selectable_label` da egui so muda o fundo sutilmente; a barra do viewport
/// precisa que o modo ativo seja obvio a distancia.
pub fn botao_alternavel(ui: &mut egui::Ui, ativo: bool, texto: &str) -> egui::Response {
    let botao = egui::Button::new(egui::RichText::new(texto).color(if ativo {
        egui::Color32::WHITE
    } else {
        egui::Color32::from_gray(200)
    }))
    .fill(if ativo {
        DESTAQUE
    } else {
        egui::Color32::from_rgb(48, 48, 54)
    })
    .corner_radius(3.0);

    ui.add(botao)
}

/// Linha de propriedade: rotulo a esquerda com largura fixa, controle a direita.
///
/// A coluna do rotulo tem a mesma largura em todas as linhas, o que mantem os
/// controles alinhados entre si no Inspector, em vez de cada linha comecar
/// numa coluna diferente conforme o tamanho do rotulo.
///
/// A coluna encolhe em painel estreito, e o rotulo que nao couber termina em
/// reticencias: sem isso, ele empurraria os controles para fora do painel.
pub fn propriedade<R>(
    ui: &mut egui::Ui,
    rotulo: &str,
    conteudo: impl FnOnce(&mut egui::Ui) -> R,
) -> R {
    let coluna = (ui.available_width() * 0.28).clamp(52.0, 84.0);

    ui.horizontal(|ui| {
        ui.add_sized(
            [coluna, 18.0],
            egui::Label::new(
                egui::RichText::new(rotulo)
                    .size(12.0)
                    .color(egui::Color32::from_gray(165)),
            )
            .halign(egui::Align::LEFT)
            .truncate(),
        )
        .on_hover_text(rotulo);
        conteudo(ui)
    })
    .inner
}

/// Tres campos numericos X/Y/Z, com a cor convencional por eixo.
///
/// Vermelho/verde/azul para X/Y/Z e convencao de praticamente todo editor 3D;
/// quebrar isso custa mais do que qualquer ganho estetico.
pub fn campos_xyz(ui: &mut egui::Ui, valores: &mut [f32; 3], velocidade: f32) -> bool {
    const CORES: [egui::Color32; 3] = [
        egui::Color32::from_rgb(205, 90, 92),
        egui::Color32::from_rgb(120, 180, 95),
        egui::Color32::from_rgb(88, 135, 205),
    ];
    const NOMES: [&str; 3] = ["X", "Y", "Z"];

    /// Largura reservada para cada letra de eixo.
    const LETRA: f32 = 9.0;
    const ESPACO: f32 = 3.0;

    let mut mudou = false;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = ESPACO;
        // Seis itens (letra e campo, tres vezes) e cinco espacos entre eles.
        // A letra entra na conta com largura fixa: medida pelo texto, ela
        // variava com a fonte e a soma passava da borda.
        let largura = ((ui.available_width() - 3.0 * LETRA - 5.0 * ESPACO) / 3.0).max(18.0);
        // O `DragValue` tem largura minima de `interact_size.x`; sem baixar o
        // minimo, os campos nao encolhem e a linha estoura em painel estreito.
        ui.spacing_mut().interact_size.x = largura;

        for eixo in 0..3 {
            ui.add_sized(
                [LETRA, 18.0],
                egui::Label::new(
                    egui::RichText::new(NOMES[eixo])
                        .size(11.0)
                        .color(CORES[eixo]),
                ),
            );
            let resposta = ui.add_sized(
                [largura, 18.0],
                egui::DragValue::new(&mut valores[eixo])
                    .speed(velocidade)
                    .fixed_decimals(2),
            );
            mudou |= resposta.changed();
        }
    });

    mudou
}

/// Seletor de cor mostrando os tres canais em 0..255, como no editor de
/// referencia, mais um quadrado clicavel que abre o seletor completo.
pub fn cor_rgb(ui: &mut egui::Ui, cor: &mut rugine_scene::settings::Color) -> bool {
    let mut bytes = cor.to_u8();
    let mut mudou = false;

    const ESPACO: f32 = 3.0;

    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = ESPACO;
        // Tres campos, o botao de cor e tres espacos entre os quatro. O botao
        // tem a largura de `interact_size` da egui, e nao um numero chutado:
        // chutado, a linha passava da borda e o painel ganhava rolagem lateral.
        let botao = ui.spacing().interact_size.x;
        let largura = ((ui.available_width() - botao - 3.0 * ESPACO) / 3.0).max(18.0);

        // O minimo do `DragValue` baixa so para os campos; o botao de cor usa o
        // mesmo `interact_size`, entao ele e restaurado antes do botao.
        ui.spacing_mut().interact_size.x = largura;
        for canal in &mut bytes {
            let resposta = ui.add_sized([largura, 18.0], egui::DragValue::new(canal).speed(1.0));
            mudou |= resposta.changed();
        }
        ui.spacing_mut().interact_size.x = botao;

        let mut rgb = [cor.r, cor.g, cor.b];
        if ui.color_edit_button_rgb(&mut rgb).changed() {
            *cor = rugine_scene::settings::Color::new(rgb[0], rgb[1], rgb[2]);
            return;
        }

        if mudou {
            *cor = rugine_scene::settings::Color::from_u8(bytes[0], bytes[1], bytes[2]);
        }
    });

    mudou
}
