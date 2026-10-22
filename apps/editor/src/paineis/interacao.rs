//! Clique e arrasto no viewport: selecao e manipulacao pelo gizmo.

use rugine_app::prelude::{Mat4, Quat, Raio, Vec3, egui};
use rugine_scene::EntityId;

use crate::estado::{Arrasto, Editor, Espaco, Ferramenta};
use crate::gizmo::{Eixo, Gizmo};
use crate::picking;

/// Trata clique e arrasto do botao esquerdo dentro do viewport.
pub fn tratar(editor: &mut Editor, resposta: &egui::Response, rect: egui::Rect, escala: f32) {
    let tamanho = (rect.width() * escala, rect.height() * escala);
    if tamanho.0 < 1.0 || tamanho.1 < 1.0 {
        return;
    }

    let view_proj = editor.camera.view_proj(tamanho.0 / tamanho.1);
    let ponto = |pos: egui::Pos2| ((pos.x - rect.min.x) * escala, (pos.y - rect.min.y) * escala);

    // Soltar o botao encerra o arrasto, mesmo que o ponteiro tenha saido do
    // viewport no meio do gesto.
    if resposta.drag_stopped() {
        editor.arrasto = None;
    }

    let gizmo = gizmo_atual(editor);

    // Qual alca esta sob o mouse, para destacar antes de clicar. Durante um
    // arrasto o destaque fica preso na alca agarrada.
    editor.alca_sob_mouse = match (&gizmo, resposta.hover_pos()) {
        _ if editor.arrasto.is_some() => editor.arrasto.as_ref().map(|a| a.eixo),
        (Some(gizmo), Some(pos)) => alca_sob(gizmo, view_proj, ponto(pos), tamanho),
        _ => None,
    };

    if resposta.drag_started_by(egui::PointerButton::Primary)
        && let Some(pos) = resposta.interact_pointer_pos()
    {
        iniciar(editor, &gizmo, view_proj, ponto(pos), tamanho);
    }

    if resposta.dragged_by(egui::PointerButton::Primary)
        && let Some(pos) = resposta.interact_pointer_pos()
    {
        continuar(editor, view_proj, ponto(pos), tamanho);
    }

    // Clique limpo (sem arrasto) seleciona. Depois do tratamento de arrasto,
    // para que soltar uma alca nao troque a selecao.
    if resposta.clicked_by(egui::PointerButton::Primary)
        && editor.arrasto.is_none()
        && let Some(pos) = resposta.interact_pointer_pos()
    {
        let acerto = picking::selecionar(&editor.cena, view_proj, ponto(pos), tamanho);
        editor.cena.select(acerto.map(|a| a.entidade));
    }
}

/// Qual alca esta sob o ponto, conforme o tipo de gizmo.
fn alca_sob(
    gizmo: &Gizmo,
    view_proj: Mat4,
    ponto: (f32, f32),
    tamanho: (f32, f32),
) -> Option<Eixo> {
    if Gizmo::usa_aneis(gizmo.ferramenta) {
        anel_sob(gizmo, view_proj, ponto, tamanho)
    } else {
        picking::alca_sob(gizmo, view_proj, ponto, tamanho)
    }
}

/// Qual anel de rotacao esta sob o ponto.
///
/// Nao ha teste raio-toro aqui: cruza-se o plano do anel e mede-se a
/// distancia ao centro. Para um anel fino o resultado e o mesmo, e a conta e
/// uma divisao em vez de uma equacao de quarto grau.
fn anel_sob(
    gizmo: &Gizmo,
    view_proj: Mat4,
    ponto: (f32, f32),
    tamanho: (f32, f32),
) -> Option<Eixo> {
    let raio = Raio::da_tela(view_proj, ponto, tamanho);
    let raio_anel = gizmo.raio_anel();

    // Tolerancia proporcional: de longe o anel ocupa menos pixels, mas em
    // unidades de mundo ele tambem e maior, entao a faixa acompanha.
    let tolerancia = raio_anel * 0.14;

    let mut melhor: Option<(f32, Eixo)> = None;

    for eixo in Eixo::TODOS {
        let normal = gizmo.eixo_mundo(eixo);

        let Some(t) = raio.intersecta_plano(gizmo.origem, normal) else {
            continue;
        };

        let distancia = (raio.em(t) - gizmo.origem).length();
        if (distancia - raio_anel).abs() > tolerancia {
            continue;
        }

        if melhor.is_none_or(|(atual, _)| t < atual) {
            melhor = Some((t, eixo));
        }
    }

    melhor.map(|(_, eixo)| eixo)
}

/// Angulo do ponteiro no plano de um anel, em radianos.
fn angulo_no_anel(
    gizmo: &Gizmo,
    eixo: Eixo,
    view_proj: Mat4,
    ponto: (f32, f32),
    tamanho: (f32, f32),
) -> Option<f32> {
    let raio = Raio::da_tela(view_proj, ponto, tamanho);
    let normal = gizmo.eixo_mundo(eixo);
    let t = raio.intersecta_plano(gizmo.origem, normal)?;

    let deslocamento = raio.em(t) - gizmo.origem;
    let (u, v) = gizmo.base_mundo(eixo);

    Some(deslocamento.dot(v).atan2(deslocamento.dot(u)))
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

    let mundo = editor.cena.world_matrix(id);
    let origem = mundo.transform_point3(Vec3::ZERO);

    // No modo Local os eixos seguem a rotacao da entidade; no Global ficam
    // nos eixos do mundo. `to_scale_rotation_translation` extrai a rotacao
    // ja sem a escala, que senao entortaria os eixos do gizmo.
    //
    // Escalar e sempre local: escala vive no espaco do objeto, e uma alca de
    // escala num eixo de mundo nao teria como ser aplicada a um objeto
    // girado.
    let orientacao = if editor.espaco == Espaco::Local || editor.ferramenta == Ferramenta::Escalar {
        mundo.to_scale_rotation_translation().1
    } else {
        Quat::IDENTITY
    };

    Some(Gizmo::new(
        origem,
        orientacao,
        editor.camera.posicao(),
        editor.ferramenta,
        editor.alca_sob_mouse,
    ))
}

fn iniciar(
    editor: &mut Editor,
    gizmo: &Option<Gizmo>,
    view_proj: Mat4,
    ponto: (f32, f32),
    tamanho: (f32, f32),
) {
    let Some(gizmo) = gizmo else {
        return;
    };
    let Some(eixo) = alca_sob(gizmo, view_proj, ponto, tamanho) else {
        return;
    };
    let Some(id) = editor.cena.selected() else {
        return;
    };
    let Some(entidade) = editor.cena.get(id) else {
        return;
    };

    let transform = entidade.transform;

    let (agarre, angulo_agarre) = if Gizmo::usa_aneis(gizmo.ferramenta) {
        let Some(angulo) = angulo_no_anel(gizmo, eixo, view_proj, ponto, tamanho) else {
            return;
        };
        (0.0, angulo)
    } else {
        let raio = Raio::da_tela(view_proj, ponto, tamanho);
        let Some(t) = picking::ponto_no_eixo(&raio, gizmo.origem, gizmo.eixo_mundo(eixo)) else {
            return;
        };
        (t, 0.0)
    };

    // Guardamos o estado do inicio do gesto e aplicamos sempre o
    // deslocamento total. Acumular diferenca quadro a quadro faria o objeto
    // escorregar: cada quadro arredonda, e o erro soma.
    editor.arrasto = Some(Arrasto {
        eixo,
        agarre,
        translacao_inicial: transform.translation,
        escala_inicial: transform.scale,
        rotacao_inicial: transform.rotation,
        angulo_acumulado: 0.0,
        angulo_anterior: angulo_agarre,
    });
}

fn continuar(editor: &mut Editor, view_proj: Mat4, ponto: (f32, f32), tamanho: (f32, f32)) {
    if editor.arrasto.is_none() {
        return;
    }
    let Some(gizmo) = gizmo_atual(editor) else {
        return;
    };
    let Some(id) = editor.cena.selected() else {
        return;
    };

    if Gizmo::usa_aneis(editor.ferramenta) {
        rotacionar(editor, &gizmo, id, view_proj, ponto, tamanho);
    } else {
        mover_ou_escalar(editor, &gizmo, id, view_proj, ponto, tamanho);
    }
}

fn rotacionar(
    editor: &mut Editor,
    gizmo: &Gizmo,
    id: EntityId,
    view_proj: Mat4,
    ponto: (f32, f32),
    tamanho: (f32, f32),
) {
    /// Degrau do snap de rotacao, em graus.
    ///
    /// Fixo, e nao o `tamanho_snap` da barra: aquele e em unidades de mundo
    /// e nao faz sentido como angulo. 15 graus e o degrau que divide a volta
    /// em 24 e inclui 30, 45 e 90.
    const PASSO_GRAUS: f32 = 15.0;

    let snap = editor.snap;

    let Some(arrasto) = editor.arrasto.as_mut() else {
        return;
    };
    let Some(angulo) = angulo_no_anel(gizmo, arrasto.eixo, view_proj, ponto, tamanho) else {
        return;
    };

    // `atan2` salta de +pi para -pi ao cruzar meia volta. Normalizar a
    // diferenca para `-pi..pi` e soma-la ao acumulado e o que permite girar
    // varias voltas sem o objeto dar um pinote a cada cruzamento.
    let mut passo = angulo - arrasto.angulo_anterior;
    if passo > std::f32::consts::PI {
        passo -= std::f32::consts::TAU;
    } else if passo < -std::f32::consts::PI {
        passo += std::f32::consts::TAU;
    }

    arrasto.angulo_acumulado += passo;
    arrasto.angulo_anterior = angulo;

    let mut total = arrasto.angulo_acumulado;
    if snap {
        let degrau = PASSO_GRAUS.to_radians();
        total = (total / degrau).round() * degrau;
    }

    let eixo = gizmo.eixo_mundo(arrasto.eixo);
    let rotacao_inicial = arrasto.rotacao_inicial;

    let Some(entidade) = editor.cena.get_mut(id) else {
        return;
    };

    // Pre-multiplicar aplica o giro no espaco em que o gizmo esta desenhado.
    // Pos-multiplicar giraria no espaco do objeto, e a alca nao acompanharia
    // o mouse. `normalize` porque multiplicar quaternions acumula erro, e um
    // quaternion fora da norma distorce a escala do objeto.
    entidade.transform.rotation =
        (Quat::from_axis_angle(eixo, total) * rotacao_inicial).normalize();
}

fn mover_ou_escalar(
    editor: &mut Editor,
    gizmo: &Gizmo,
    id: EntityId,
    view_proj: Mat4,
    ponto: (f32, f32),
    tamanho: (f32, f32),
) {
    let snap = editor.snap;
    let passo_snap = editor.tamanho_snap;
    let ferramenta = editor.ferramenta;

    let Some(arrasto) = editor.arrasto.as_ref() else {
        return;
    };

    let raio = Raio::da_tela(view_proj, ponto, tamanho);
    let direcao = gizmo.eixo_mundo(arrasto.eixo);

    let Some(t) = picking::ponto_no_eixo(&raio, gizmo.origem, direcao) else {
        return;
    };

    let mut deslocamento = t - arrasto.agarre;
    if snap {
        deslocamento = (deslocamento / passo_snap).round() * passo_snap;
    }

    let eixo = arrasto.eixo;
    let translacao_inicial = arrasto.translacao_inicial;
    let escala_inicial = arrasto.escala_inicial;

    let Some(entidade) = editor.cena.get_mut(id) else {
        return;
    };

    match ferramenta {
        Ferramenta::Mover => {
            entidade.transform.translation = translacao_inicial + direcao * deslocamento;
        }
        Ferramenta::Escalar => {
            let indice = match eixo {
                Eixo::X => 0,
                Eixo::Y => 1,
                Eixo::Z => 2,
            };
            let mut escala = escala_inicial.to_array();
            // Escala nao pode chegar a zero: matriz com escala nula nao tem
            // inversa, e o picking (que inverte a matriz) pararia de
            // funcionar naquele objeto.
            escala[indice] = (escala_inicial[indice] + deslocamento).max(0.01);
            entidade.transform.scale = Vec3::from_array(escala);
        }
        _ => {}
    }
}
