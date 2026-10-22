//! Selecao por clique no viewport.
//!
//! Raycast na CPU contra as caixas das entidades, e nao um buffer de ids na
//! GPU. Toda geometria da cena e um cubo unitario transformado, entao o teste
//! analitico e exato — e dispensa um passe extra de render, uma textura de
//! ids e uma leitura de volta assincrona, que so se pagariam com malhas
//! arbitrarias e milhares de objetos.

use rugine_app::prelude::{Mat4, Raio, Vec3};
use rugine_scene::{EntityId, Scene};

/// O que o raio acertou.
pub struct Acerto {
    pub entidade: EntityId,
    /// Distancia ao longo do raio, em unidades da direcao dele.
    pub t: f32,
}

/// Entidade visivel mais proxima sob o ponto da tela.
///
/// `ponto` e `tamanho` em pixels do viewport, origem no canto superior
/// esquerdo.
pub fn selecionar(
    cena: &Scene,
    view_proj: Mat4,
    ponto: (f32, f32),
    tamanho: (f32, f32),
) -> Option<Acerto> {
    let raio = Raio::da_tela(view_proj, ponto, tamanho);
    mais_proxima(cena, &raio)
}

/// A entidade mais proxima atingida pelo raio.
fn mais_proxima(cena: &Scene, raio: &Raio) -> Option<Acerto> {
    let mut melhor: Option<Acerto> = None;

    for entidade in cena.entidades() {
        // Invisivel nao e clicavel: o que nao esta na tela nao pode ser
        // alvo de um clique na tela.
        let Some(dados) = cena.get(entidade) else {
            continue;
        };
        if !dados.visible {
            continue;
        }

        let Some(t) = raio.intersecta_caixa(cena.world_matrix(entidade)) else {
            continue;
        };

        // O menor `t` e o que esta na frente. Sem esta comparacao, a selecao
        // pegaria a ultima da lista em vez da visivel.
        if melhor.as_ref().is_none_or(|atual| t < atual.t) {
            melhor = Some(Acerto { entidade, t });
        }
    }

    melhor
}

/// Qual alca do gizmo esta sob o ponto, se alguma.
///
/// Testada antes das entidades: o gizmo e desenhado por cima de tudo, entao
/// tem que capturar o clique antes, senao clicar numa alca sobre um objeto
/// selecionaria o objeto.
pub fn alca_sob(
    gizmo: &crate::gizmo::Gizmo,
    view_proj: Mat4,
    ponto: (f32, f32),
    tamanho: (f32, f32),
) -> Option<crate::gizmo::Eixo> {
    let raio = Raio::da_tela(view_proj, ponto, tamanho);

    let mut melhor: Option<(f32, crate::gizmo::Eixo)> = None;

    for (eixo, modelo) in gizmo.alcas() {
        let Some(t) = raio.intersecta_caixa(modelo) else {
            continue;
        };
        if melhor.is_none_or(|(atual, _)| t < atual) {
            melhor = Some((t, eixo));
        }
    }

    melhor.map(|(_, eixo)| eixo)
}

/// Ponto do eixo mais proximo do raio, para arrastar uma alca.
///
/// Resolve a aproximacao entre duas retas: a do eixo e a que sai do mouse. O
/// mouse se move num plano e o objeto num eixo, entao nao ha intersecao de
/// verdade — o que se quer e o ponto do eixo que passa mais perto do raio.
///
/// Retorna `None` quando as retas sao quase paralelas: ali a resposta tende
/// ao infinito, e arrastar jogaria o objeto para longe.
pub fn ponto_no_eixo(raio: &Raio, origem_eixo: Vec3, direcao_eixo: Vec3) -> Option<f32> {
    let d1 = direcao_eixo;
    let d2 = raio.direcao;
    let r = origem_eixo - raio.origem;

    let a = d1.dot(d1);
    let b = d1.dot(d2);
    let c = d2.dot(d2);
    let d = d1.dot(r);
    let e = d2.dot(r);

    let denominador = a * c - b * b;
    if denominador.abs() < 1e-6 {
        return None;
    }

    Some((b * e - c * d) / denominador)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Arrastar ao longo de um eixo: o parametro devolvido tem que ser a
    /// coordenada do ponto do eixo mais proximo do raio.
    #[test]
    fn ponto_no_eixo_acha_a_coordenada() {
        // Raio apontando para -Z, passando por x = 3.
        let raio = Raio::new(Vec3::new(3.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));

        let t = ponto_no_eixo(&raio, Vec3::ZERO, Vec3::X).unwrap();

        assert!((t - 3.0).abs() < 1e-4, "esperado 3.0, deu {t}");
    }

    /// Com o eixo visto de topo, o raio fica paralelo a ele e a resposta
    /// tenderia ao infinito. Tem que recusar, nao devolver um numero enorme.
    #[test]
    fn ponto_no_eixo_recusa_raio_paralelo() {
        let raio = Raio::new(Vec3::new(0.0, 2.0, 0.0), Vec3::X);
        assert_eq!(ponto_no_eixo(&raio, Vec3::ZERO, Vec3::X), None);
    }
}
