//! Bordas de redimensionamento de uma janela sem decoracao.
//!
//! Sem a moldura do sistema, nao ha mais onde agarrar para redimensionar. Isto
//! devolve as oito alcas, invisiveis, por cima de tudo.

use rugine_app::prelude::{Borda, Context, egui};

/// Espessura da alca nas laterais, em pontos.
///
/// Fina o bastante para nao roubar cliques do conteudo colado na borda, larga
/// o bastante para o mouse acertar sem precisar de pontaria.
const ESPESSURA: f32 = 5.0;

/// Lado da alca nos cantos, onde se redimensiona em dois eixos.
const CANTO: f32 = 12.0;

pub fn desenhar(egui: &egui::Context, ctx: &mut Context<'_>) {
    // Uma janela maximizada nao se redimensiona arrastando; manter as alcas
    // ativas so produziria arrastos que nao fazem nada.
    if ctx.janela_maximizada() {
        return;
    }

    // `viewport_rect`, nao `content_rect`: queremos a janela inteira. O
    // content_rect desconta recortes e barras de status do sistema, e as alcas
    // precisam ficar exatamente na borda.
    let tela = egui.viewport_rect();

    // Os cantos primeiro: declarados antes, ficam por cima das laterais, que e
    // o que faz a diagonal vencer na quina em vez do eixo unico.
    let alcas: [(&str, egui::Rect, Borda, egui::CursorIcon); 8] = [
        (
            "canto_ce",
            egui::Rect::from_min_size(tela.left_top(), egui::vec2(CANTO, CANTO)),
            Borda::CimaEsquerda,
            egui::CursorIcon::ResizeNorthWest,
        ),
        (
            "canto_cd",
            egui::Rect::from_min_size(
                tela.right_top() - egui::vec2(CANTO, 0.0),
                egui::vec2(CANTO, CANTO),
            ),
            Borda::CimaDireita,
            egui::CursorIcon::ResizeNorthEast,
        ),
        (
            "canto_be",
            egui::Rect::from_min_size(
                tela.left_bottom() - egui::vec2(0.0, CANTO),
                egui::vec2(CANTO, CANTO),
            ),
            Borda::BaixoEsquerda,
            egui::CursorIcon::ResizeSouthWest,
        ),
        (
            "canto_bd",
            egui::Rect::from_min_size(
                tela.right_bottom() - egui::vec2(CANTO, CANTO),
                egui::vec2(CANTO, CANTO),
            ),
            Borda::BaixoDireita,
            egui::CursorIcon::ResizeSouthEast,
        ),
        (
            "borda_cima",
            egui::Rect::from_min_size(tela.left_top(), egui::vec2(tela.width(), ESPESSURA)),
            Borda::Cima,
            egui::CursorIcon::ResizeNorth,
        ),
        (
            "borda_baixo",
            egui::Rect::from_min_size(
                tela.left_bottom() - egui::vec2(0.0, ESPESSURA),
                egui::vec2(tela.width(), ESPESSURA),
            ),
            Borda::Baixo,
            egui::CursorIcon::ResizeSouth,
        ),
        (
            "borda_esq",
            egui::Rect::from_min_size(tela.left_top(), egui::vec2(ESPESSURA, tela.height())),
            Borda::Esquerda,
            egui::CursorIcon::ResizeWest,
        ),
        (
            "borda_dir",
            egui::Rect::from_min_size(
                tela.right_top() - egui::vec2(ESPESSURA, 0.0),
                egui::vec2(ESPESSURA, tela.height()),
            ),
            Borda::Direita,
            egui::CursorIcon::ResizeEast,
        ),
    ];

    for (nome, rect, borda, cursor) in alcas {
        alca(egui, ctx, nome, rect, borda, cursor);
    }
}

/// Uma alca invisivel sobre a janela.
fn alca(
    egui: &egui::Context,
    ctx: &mut Context<'_>,
    nome: &str,
    rect: egui::Rect,
    borda: Borda,
    cursor: egui::CursorIcon,
) {
    // `Area` em camada de frente: as alcas precisam ficar acima dos paineis,
    // senao qualquer painel encostado na borda come o arrasto.
    let resposta = egui::Area::new(egui::Id::new(("alca_janela", nome)))
        .order(egui::Order::Foreground)
        .fixed_pos(rect.min)
        .interactable(true)
        .show(egui, |ui| {
            ui.allocate_response(rect.size(), egui::Sense::click_and_drag())
        })
        .inner;

    if resposta.hovered() || resposta.is_pointer_button_down_on() {
        egui.set_cursor_icon(cursor);
    }

    // Igual ao arrastar da barra: quem conduz o resto do gesto e o sistema.
    if resposta.drag_started() {
        ctx.redimensionar_janela(borda);
    }
}
