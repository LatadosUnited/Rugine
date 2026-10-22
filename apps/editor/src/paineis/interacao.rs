//! Clique e arrasto no viewport: selecao e manipulacao pelo gizmo.

use rugine_app::prelude::{Mat4, Raio, Vec3, egui};

use crate::estado::{Arrasto, Editor, Ferramenta};
use crate::gizmo::{Eixo, Gizmo};
use crate::picking;

/// Trata o clique esquerdo dentro do viewport.
///
/// `ponto` e a posicao do mouse relativa ao canto do viewport, em pixels.
pub fn tratar(editor: &mut Editor, resposta: &egui::Response, rect: egui::Rect, escala: f32) {
    let tamanho = (rect.width() * escala, rect.height() * escala);
    if tamanho.0 < 1.0 || tamanho.1 < 1.0 {
        return;
    }

    let proporcao = tamanho.0 / tamanho.1;
    let view_proj = editor.camera.view_proj(proporcao);

    let ponto = |pos: egui::Pos2| ((pos.x - rect.min.x) * escala, (pos.y - rect.min.y) * escala);

    // Soltar o botao encerra qualquer arrasto, mesmo que o ponteiro tenha
    // saido do viewport no meio.
    if resposta.drag_stopped() {
        editor.arrasto = None;
    }

    let gizmo = gizmo_atual(editor);

    // Qual alca esta sob o mouse, para destacar antes mesmo de clicar.
    editor.alca_sob_mouse = match (&gizmo, resposta.hover_pos()) {
        (Some(gizmo), Some(pos)) if editor.arrasto.is_none() => {
            picking::alca_sob(gizmo, view_proj, ponto(pos), tamanho)
        }
        _ => editor.arrasto.as_ref().map(|a| a.eixo),
    };

    if resposta.drag_started_by(egui::PointerButton::Primary)
        && let Some(pos) = resposta.interact_pointer_pos()
    {
        iniciar_arrasto(editor, &gizmo, view_proj, ponto(pos), tamanho);
    }

    if resposta.dragged_by(egui::PointerButton::Primary)
        && let Some(pos) = resposta.interact_pointer_pos()
    {
        continuar_arrasto(editor, view_proj, ponto(pos), tamanho);
    }

    // Clique sem arrasto: seleciona. Depois do arrasto, para que mover uma
    // alca nao mude a selecao ao soltar.
    if resposta.clicked_by(egui::PointerButton::Primary)
        && editor.arrasto.is_none()
        && let Some(pos) = resposta.interact_pointer_pos()
    {
        let acerto = picking::selecionar(&editor.cena, view_proj, ponto(pos), tamanho);
        editor.cena.select(acerto.map(|a| a.entidade));
    }
}

/// O gizmo do frame, se ha selecao e a ferramenta o desenha.
pub fn gizmo_atual(editor: &Editor) -> Option<Gizmo> {
    if !Gizmo::visivel(editor.ferramenta) {
        return None;
    }

    let id = editor.cena.selected()?;
    if !editor.cena.contains(id) {
        return None;
    }

    let origem = editor.cena.world_matrix(id).transform_point3(Vec3::ZERO);

    Some(Gizmo::new(
        origem,
        editor.camera.posicao(),
        editor.ferramenta,
        editor.alca_sob_mouse,
    ))
}

fn iniciar_arrasto(
    editor: &mut Editor,
    gizmo: &Option<Gizmo>,
    view_proj: Mat4,
    ponto: (f32, f32),
    tamanho: (f32, f32),
) {
    let Some(gizmo) = gizmo else {
        return;
    };
    let Some(eixo) = picking::alca_sob(gizmo, view_proj, ponto, tamanho) else {
        return;
    };
    let Some(id) = editor.cena.selected() else {
        return;
    };
    let Some(entidade) = editor.cena.get(id) else {
        return;
    };

    let raio = Raio::da_tela(view_proj, ponto, tamanho);
    let Some(t) = picking::ponto_no_eixo(&raio, gizmo.origem, eixo.direcao()) else {
        return;
    };

    // Guardamos o estado inicial e o ponto de agarre, e aplicamos sempre o
    // deslocamento total em relacao a eles. Acumular diferenca quadro a
    // quadro faria o objeto escorregar: cada quadro arredonda, e o erro
    // soma.
    editor.arrasto = Some(Arrasto {
        eixo,
        agarre: t,
        translacao_inicial: entidade.transform.translation,
        escala_inicial: entidade.transform.scale,
    });
}

fn continuar_arrasto(editor: &mut Editor, view_proj: Mat4, ponto: (f32, f32), tamanho: (f32, f32)) {
    let Some(arrasto) = editor.arrasto.as_ref() else {
        return;
    };
    let Some(id) = editor.cena.selected() else {
        return;
    };

    // A origem do gizmo vem da matriz de mundo do *pai*: arrastar move a
    // translacao local, e usar a posicao de mundo atual como referencia
    // realimentaria o proprio movimento.
    let origem = editor.cena.world_matrix(id).transform_point3(Vec3::ZERO);
    let raio = Raio::da_tela(view_proj, ponto, tamanho);

    let Some(t) = picking::ponto_no_eixo(&raio, origem, arrasto.eixo.direcao()) else {
        return;
    };

    let mut deslocamento = t - arrasto.agarre;
    if editor.snap {
        deslocamento = (deslocamento / editor.tamanho_snap).round() * editor.tamanho_snap;
    }

    let eixo = arrasto.eixo;
    let ferramenta = editor.ferramenta;
    let translacao_inicial = arrasto.translacao_inicial;
    let escala_inicial = arrasto.escala_inicial;

    let Some(entidade) = editor.cena.get_mut(id) else {
        return;
    };

    match ferramenta {
        Ferramenta::Mover => {
            let delta = eixo.direcao() * deslocamento;
            entidade.transform.translation = translacao_inicial + delta;
        }
        Ferramenta::Escalar => {
            let indice = match eixo {
                Eixo::X => 0,
                Eixo::Y => 1,
                Eixo::Z => 2,
            };
            let mut escala = escala_inicial.to_array();
            // Escala nao pode chegar a zero: uma matriz com escala nula nao
            // tem inversa, e o picking (que inverte a matriz) pararia de
            // funcionar naquele objeto.
            escala[indice] = (escala_inicial[indice] + deslocamento).max(0.01);
            entidade.transform.scale = Vec3::from_array(escala);
        }
        _ => {}
    }
}
