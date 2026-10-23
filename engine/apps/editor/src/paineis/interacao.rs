//! Clique e arrasto no viewport: selecao e manipulacao pelo gizmo.

use rugine_app::prelude::{Mat4, Quat, Raio, Vec3, Vec4, egui};
use rugine_scene::EntityId;

use crate::estado::{Arrasto, Editor, Espaco, Ferramenta};
use crate::gizmo::{Alca, Eixo, Gizmo};
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
        _ if editor.arrasto.is_some() => editor.arrasto.as_ref().map(|a| a.alca),
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
) -> Option<Alca> {
    if Gizmo::usa_aneis(gizmo.ferramenta) {
        anel_sob(gizmo, view_proj, ponto, tamanho).map(Alca::Eixo)
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

/// Onde um ponto do mundo cai na tela, em pixels do viewport.
///
/// O inverso de `Raio::da_tela`.
fn na_tela(view_proj: Mat4, p: Vec3, tamanho: (f32, f32)) -> (f32, f32) {
    let clip = view_proj * Vec4::new(p.x, p.y, p.z, 1.0);
    let ndc = clip.truncate() / clip.w;
    (
        (ndc.x + 1.0) * 0.5 * tamanho.0,
        (1.0 - ndc.y) * 0.5 * tamanho.1,
    )
}

/// Quantos pixels o comprimento do gizmo ocupa na tela.
fn pixels_por_comprimento(
    gizmo: &Gizmo,
    posicao_camera: Vec3,
    view_proj: Mat4,
    tamanho: (f32, f32),
) -> f32 {
    // Um vetor qualquer perpendicular a visada: de frente para a camera, todo
    // vetor nesse plano projeta com o mesmo tamanho. Um eixo do gizmo nao
    // serviria, porque pode estar apontado para a camera e medir zero.
    let lado = (gizmo.origem - posicao_camera)
        .normalize_or_zero()
        .any_orthonormal_vector();
    let a = na_tela(view_proj, gizmo.origem, tamanho);
    let b = na_tela(view_proj, gizmo.origem + lado * gizmo.comprimento, tamanho);

    // Piso de um pixel: dividir pelo resultado nao pode explodir.
    (b.0 - a.0).hypot(b.1 - a.1).max(1.0)
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
    let Some(alca) = alca_sob(gizmo, view_proj, ponto, tamanho) else {
        return;
    };
    let Some(id) = editor.cena.selected() else {
        return;
    };
    let Some(entidade) = editor.cena.get(id) else {
        return;
    };

    let transform = entidade.transform;

    let (agarre, angulo_agarre) = match alca {
        // O centro mede o arrasto na tela, a partir de `ponto_inicial`.
        Alca::Centro => (0.0, 0.0),
        Alca::Eixo(eixo) if Gizmo::usa_aneis(gizmo.ferramenta) => {
            let Some(angulo) = angulo_no_anel(gizmo, eixo, view_proj, ponto, tamanho) else {
                return;
            };
            (0.0, angulo)
        }
        Alca::Eixo(eixo) => {
            let raio = Raio::da_tela(view_proj, ponto, tamanho);
            let Some(t) = picking::ponto_no_eixo(&raio, gizmo.origem, gizmo.eixo_mundo(eixo))
            else {
                return;
            };
            (t, 0.0)
        }
    };

    // Guardamos o estado do inicio do gesto e aplicamos sempre o
    // deslocamento total. Acumular diferenca quadro a quadro faria o objeto
    // escorregar: cada quadro arredonda, e o erro soma.
    editor.arrasto = Some(Arrasto {
        alca,
        gizmo: *gizmo,
        agarre,
        ponto_inicial: ponto,
        pixels_por_comprimento: pixels_por_comprimento(
            gizmo,
            editor.camera.posicao(),
            view_proj,
            tamanho,
        ),
        translacao_inicial: transform.translation,
        escala_inicial: transform.scale,
        rotacao_inicial: transform.rotation,
        angulo_acumulado: 0.0,
        angulo_anterior: angulo_agarre,
    });
}

fn continuar(editor: &mut Editor, view_proj: Mat4, ponto: (f32, f32), tamanho: (f32, f32)) {
    let Some(gizmo) = editor.arrasto.as_ref().map(|a| a.gizmo) else {
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
    let Some(eixo_anel) = arrasto.alca.eixo() else {
        return;
    };
    let Some(angulo) = angulo_no_anel(gizmo, eixo_anel, view_proj, ponto, tamanho) else {
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

    let eixo = gizmo.eixo_mundo(eixo_anel);
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
    let eixo = match arrasto.alca {
        Alca::Eixo(eixo) => eixo,
        Alca::Centro => {
            escalar_uniforme(editor, id, ponto);
            return;
        }
    };

    let raio = Raio::da_tela(view_proj, ponto, tamanho);
    let direcao = gizmo.eixo_mundo(eixo);

    let Some(t) = picking::ponto_no_eixo(&raio, gizmo.origem, direcao) else {
        return;
    };

    let mut deslocamento = t - arrasto.agarre;
    if snap {
        deslocamento = (deslocamento / passo_snap).round() * passo_snap;
    }

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

/// Escala os tres eixos juntos, pelo arrasto do centro.
///
/// Medido em pixels na tela, e nao ao longo de um eixo do mundo: o centro nao
/// tem direcao, e qualquer eixo escolhido ficaria de ponta para a camera em
/// algum angulo. Para a direita ou para cima aumenta, como na Unity, e
/// arrastar o tamanho de uma haste dobra a escala.
fn escalar_uniforme(editor: &mut Editor, id: EntityId, ponto: (f32, f32)) {
    let snap = editor.snap;
    let passo_snap = editor.tamanho_snap;

    let Some(arrasto) = editor.arrasto.as_ref() else {
        return;
    };

    // O `y` da tela cresce para baixo, entao subir e `-dy`.
    let dx = ponto.0 - arrasto.ponto_inicial.0;
    let dy = ponto.1 - arrasto.ponto_inicial.1;
    let mut aumento = (dx - dy) / arrasto.pixels_por_comprimento;
    if snap {
        aumento = (aumento / passo_snap).round() * passo_snap;
    }

    // Multiplicar, e nao somar a cada eixo: um objeto com escala (1, 3, 1)
    // continua tres vezes mais alto que largo. Somar o mesmo valor aos tres
    // mudaria a proporcao dele.
    let fator = (1.0 + aumento).max(0.01);
    let escala = (arrasto.escala_inicial * fator).max(Vec3::splat(0.01));

    if let Some(entidade) = editor.cena.get_mut(id) {
        entidade.transform.scale = escala;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TAMANHO: (f32, f32) = (800.0, 600.0);

    fn editor_com_selecao(ferramenta: Ferramenta, espaco: Espaco) -> (Editor, EntityId, Mat4) {
        let mut editor = Editor::default();
        let id = editor.cena.roots()[0];
        editor.cena.select(Some(id));
        editor.ferramenta = ferramenta;
        editor.espaco = espaco;
        let view_proj = editor.camera.view_proj(TAMANHO.0 / TAMANHO.1);
        (editor, id, view_proj)
    }

    /// O mouse parado num ponto tem que deixar o objeto parado. Antes, o
    /// deslocamento era medido contra a origem do quadro atual, que ja tinha
    /// andado com o objeto, e ele alternava entre dois lugares.
    #[test]
    fn mover_com_mouse_parado_nao_pisca() {
        let (mut editor, id, view_proj) = editor_com_selecao(Ferramenta::Mover, Espaco::Global);
        let gizmo = gizmo_atual(&editor).unwrap();
        let inicial = editor.cena.get(id).unwrap().transform.translation;

        let agarre = gizmo.origem + Vec3::X * (gizmo.comprimento * 0.5);
        iniciar(
            &mut editor,
            &Some(gizmo),
            view_proj,
            na_tela(view_proj, agarre, TAMANHO),
            TAMANHO,
        );
        assert_eq!(
            editor.arrasto.as_ref().map(|a| a.alca),
            Some(Alca::Eixo(Eixo::X))
        );

        let destino = na_tela(view_proj, agarre + Vec3::X, TAMANHO);
        let mut posicoes = Vec::new();
        for _ in 0..4 {
            continuar(&mut editor, view_proj, destino, TAMANHO);
            posicoes.push(editor.cena.get(id).unwrap().transform.translation);
        }

        for p in &posicoes {
            assert!((*p - (inicial + Vec3::X)).length() < 1e-3, "{posicoes:?}");
        }
    }

    /// O mesmo para a rotacao em modo Local, onde a base do anel gira junto
    /// com o objeto.
    #[test]
    fn rotacionar_local_com_mouse_parado_nao_pisca() {
        let (mut editor, id, view_proj) = editor_com_selecao(Ferramenta::Rotacionar, Espaco::Local);
        let gizmo = gizmo_atual(&editor).unwrap();
        let (u, v) = gizmo.base_mundo(Eixo::X);
        let r = gizmo.raio_anel();

        let no_anel = |graus: f32| {
            let a = graus.to_radians();
            na_tela(
                view_proj,
                gizmo.origem + (u * a.cos() + v * a.sin()) * r,
                TAMANHO,
            )
        };

        // 45 graus, e nao 0: em 0 o ponto `u * r` fica tambem sobre o anel
        // vizinho, onde os dois se cruzam, e o clique pode pegar o outro.
        iniciar(&mut editor, &Some(gizmo), view_proj, no_anel(45.0), TAMANHO);
        assert_eq!(
            editor.arrasto.as_ref().map(|a| a.alca),
            Some(Alca::Eixo(Eixo::X))
        );

        let angulo = 30f32.to_radians();
        let destino = no_anel(75.0);

        let mut rotacoes = Vec::new();
        for _ in 0..4 {
            continuar(&mut editor, view_proj, destino, TAMANHO);
            rotacoes.push(editor.cena.get(id).unwrap().transform.rotation);
        }

        for q in &rotacoes {
            assert!(q.angle_between(rotacoes[0]) < 1e-3, "{rotacoes:?}");
        }
        let girado = rotacoes[0].angle_between(gizmo.orientacao);
        assert!(
            (girado - angulo).abs() < 1e-2,
            "girou {} graus",
            girado.to_degrees()
        );
    }

    /// Arrastar o centro escala os tres eixos pelo mesmo fator, mantendo a
    /// proporcao, e o mouse parado deixa a escala parada.
    #[test]
    fn centro_escala_os_tres_eixos() {
        let (mut editor, id, view_proj) = editor_com_selecao(Ferramenta::Escalar, Espaco::Local);
        editor.cena.get_mut(id).unwrap().transform.scale = Vec3::new(1.0, 3.0, 2.0);
        let gizmo = gizmo_atual(&editor).unwrap();

        let meio = na_tela(view_proj, gizmo.origem, TAMANHO);
        iniciar(&mut editor, &Some(gizmo), view_proj, meio, TAMANHO);
        assert_eq!(editor.arrasto.as_ref().map(|a| a.alca), Some(Alca::Centro));

        // Meia haste para a direita: fator 1,5.
        let regua = editor.arrasto.as_ref().unwrap().pixels_por_comprimento;
        let destino = (meio.0 + regua * 0.5, meio.1);

        for _ in 0..4 {
            continuar(&mut editor, view_proj, destino, TAMANHO);
            let escala = editor.cena.get(id).unwrap().transform.scale;
            assert!(
                (escala - Vec3::new(1.5, 4.5, 3.0)).length() < 1e-3,
                "escala {escala}"
            );
        }

        // Para a esquerda do ponto de partida, encolhe.
        continuar(
            &mut editor,
            view_proj,
            (meio.0 - regua * 0.5, meio.1),
            TAMANHO,
        );
        let escala = editor.cena.get(id).unwrap().transform.scale;
        assert!(
            (escala - Vec3::new(0.5, 1.5, 1.0)).length() < 1e-3,
            "escala {escala}"
        );
    }

    /// Com o centro agarravel, a haste continua pegando o seu eixo.
    #[test]
    fn haste_da_escala_ainda_pega_o_eixo() {
        let (mut editor, _, view_proj) = editor_com_selecao(Ferramenta::Escalar, Espaco::Local);
        let gizmo = gizmo_atual(&editor).unwrap();

        let na_haste = gizmo.origem + gizmo.eixo_mundo(Eixo::Y) * (gizmo.comprimento * 0.6);
        let ponto = na_tela(view_proj, na_haste, TAMANHO);
        iniciar(&mut editor, &Some(gizmo), view_proj, ponto, TAMANHO);

        assert_eq!(
            editor.arrasto.as_ref().map(|a| a.alca),
            Some(Alca::Eixo(Eixo::Y))
        );
    }
}
