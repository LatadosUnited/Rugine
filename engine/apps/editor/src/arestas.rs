//! Arestas finas no viewport: colisores e o contorno da selecao.
//!
//! As arestas sao barrinhas instanciadas, como os aneis do gizmo, e entram na
//! lista da cena com teste de profundidade: o que esta atras de um objeto fica
//! escondido por ele. O gizmo e o oposto, porque precisa ser agarravel sempre.

use rugine_app::prelude::{Instancia, Mat4, Quat, Vec3};
use rugine_scene::{Colisor, FormaColisor};

/// Verde, a cor de colisor na maioria dos editores.
const COR_COLISOR: [f32; 4] = [0.45, 0.95, 0.50, 1.0];

/// Laranja da selecao.
const COR_SELECAO: [f32; 4] = [0.95, 0.62, 0.23, 1.0];

/// Segmentos de cada circulo da esfera.
const SEGMENTOS: usize = 32;

/// Instancias das arestas de um colisor.
///
/// `mundo` e a matriz de mundo da entidade. `espessura` vem em unidades de
/// mundo, e quem chama a ajusta pela distancia da camera para a linha ter a
/// mesma largura na tela perto e longe.
pub fn colisor(colisor: &Colisor, mundo: Mat4, espessura: f32, saida: &mut Vec<Instancia>) {
    let centro = colisor.centro;
    match colisor.forma {
        FormaColisor::Caixa { tamanho } => {
            caixa(centro, tamanho, mundo, espessura, COR_COLISOR, saida);
        }
        FormaColisor::Esfera { raio } => esfera(centro, raio, mundo, espessura, saida),
    }
}

/// Contorno da entidade selecionada, nas arestas da Malha.
///
/// Contorno, e nao pintar o objeto de laranja: pintado, ele escondia a
/// propria cor, e mudar a cor da Malha nao mostrava nada justo enquanto a
/// entidade estava selecionada para ser editada.
pub fn selecao(mundo: Mat4, espessura: f32, saida: &mut Vec<Instancia>) {
    // A Malha e sempre o cubo unitario por enquanto. Com malhas de glTF, o
    // contorno passa a ser a caixa envolvente delas.
    caixa(Vec3::ZERO, Vec3::ONE, mundo, espessura, COR_SELECAO, saida);
}

fn caixa(
    centro: Vec3,
    tamanho: Vec3,
    mundo: Mat4,
    espessura: f32,
    cor: [f32; 4],
    saida: &mut Vec<Instancia>,
) {
    // Os 8 cantos: o bit 0 do indice escolhe o sinal de X, o 1 o de Y, o 2 o
    // de Z. Levados ao mundo pela matriz inteira, a caixa gira e escala com a
    // entidade.
    let canto = |indice: usize| {
        let sinal = |bit: usize| if indice & (1 << bit) == 0 { -0.5 } else { 0.5 };
        let local = centro + tamanho * Vec3::new(sinal(0), sinal(1), sinal(2));
        mundo.transform_point3(local)
    };

    // Uma aresta liga dois cantos que diferem num bit so. Para cada canto,
    // ligar so aos vizinhos com o bit ligado conta cada aresta uma vez: 12.
    for a in 0..8 {
        for bit in 0..3 {
            if a & (1 << bit) == 0 {
                saida.extend(barra(canto(a), canto(a | (1 << bit)), espessura, cor));
            }
        }
    }
}

fn esfera(centro: Vec3, raio: f32, mundo: Mat4, espessura: f32, saida: &mut Vec<Instancia>) {
    let (escala, rotacao, _) = mundo.to_scale_rotation_translation();
    let centro = mundo.transform_point3(centro);
    // Esfera continua esfera sob escala nao uniforme: vale o maior eixo.
    let raio = raio * escala.abs().max_element();

    // Tres circulos, um em cada plano local, girados com a entidade.
    for (u, v) in [(Vec3::Y, Vec3::Z), (Vec3::Z, Vec3::X), (Vec3::X, Vec3::Y)] {
        let (u, v) = (rotacao * u, rotacao * v);
        let ponto = |passo: usize| {
            let a = passo as f32 / SEGMENTOS as f32 * std::f32::consts::TAU;
            centro + (u * a.cos() + v * a.sin()) * raio
        };

        for passo in 0..SEGMENTOS {
            saida.extend(barra(
                ponto(passo),
                ponto(passo + 1),
                espessura,
                COR_COLISOR,
            ));
        }
    }
}

/// Uma barra fina de `p` a `q`.
fn barra(p: Vec3, q: Vec3, espessura: f32, cor: [f32; 4]) -> Option<Instancia> {
    let direcao = q - p;
    let comprimento = direcao.length();
    if comprimento < 1e-6 {
        return None;
    }

    Some(Instancia::new(
        Mat4::from_scale_rotation_translation(
            // Uma espessura a mais no comprimento fecha os cantos, que senao
            // ficariam com um vao do tamanho da propria espessura.
            Vec3::new(comprimento + espessura, espessura, espessura),
            Quat::from_rotation_arc(Vec3::X, direcao / comprimento),
            (p + q) * 0.5,
        ),
        cor,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn centros(instancias: &[Instancia]) -> Vec<Vec3> {
        instancias
            .iter()
            .map(|i| i.modelo().transform_point3(Vec3::ZERO))
            .collect()
    }

    /// 12 arestas, e o meio de cada uma de uma caixa unitaria tem exatamente
    /// uma coordenada zero e as outras em +-0,5.
    #[test]
    fn caixa_tem_as_doze_arestas() {
        let mut saida = Vec::new();
        colisor(&Colisor::default(), Mat4::IDENTITY, 0.01, &mut saida);
        assert_eq!(saida.len(), 12);

        let mut vistos = Vec::new();
        for c in centros(&saida) {
            let zeros = c.to_array().iter().filter(|v| v.abs() < 1e-5).count();
            let bordas = c
                .to_array()
                .iter()
                .filter(|v| (v.abs() - 0.5).abs() < 1e-5)
                .count();
            assert_eq!((zeros, bordas), (1, 2), "meio de aresta fora do lugar: {c}");
            assert!(
                !vistos.iter().any(|v: &Vec3| (*v - c).length() < 1e-5),
                "aresta repetida em {c}"
            );
            vistos.push(c);
        }
    }

    /// O colisor acompanha a matriz de mundo e o proprio `centro`.
    #[test]
    fn caixa_segue_o_mundo_e_o_centro() {
        let alvo = Colisor {
            forma: FormaColisor::Caixa { tamanho: Vec3::ONE },
            centro: Vec3::new(0.0, 1.0, 0.0),
        };
        let mundo = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));

        let mut saida = Vec::new();
        colisor(&alvo, mundo, 0.01, &mut saida);

        let soma: Vec3 = centros(&saida).into_iter().sum();
        let media = soma / saida.len() as f32;
        assert!(
            (media - Vec3::new(5.0, 1.0, 0.0)).length() < 1e-4,
            "media {media}"
        );
    }

    /// Sob escala nao uniforme, a esfera usa o maior eixo.
    #[test]
    fn esfera_usa_o_maior_eixo_da_escala() {
        let alvo = Colisor {
            forma: FormaColisor::Esfera { raio: 0.5 },
            centro: Vec3::ZERO,
        };
        let mundo = Mat4::from_scale(Vec3::new(1.0, 4.0, 1.0));

        let mut saida = Vec::new();
        colisor(&alvo, mundo, 0.01, &mut saida);
        assert_eq!(saida.len(), SEGMENTOS * 3);

        // O meio de um segmento fica um pouco para dentro do circulo.
        let mais_longe = centros(&saida)
            .into_iter()
            .map(Vec3::length)
            .fold(0.0, f32::max);
        assert!((mais_longe - 2.0).abs() < 0.02, "raio {mais_longe}");
    }

    /// O contorno fica nas arestas da Malha e acompanha a matriz de mundo.
    #[test]
    fn selecao_contorna_a_malha() {
        let mundo = Mat4::from_scale_rotation_translation(
            Vec3::new(2.0, 1.0, 1.0),
            Quat::IDENTITY,
            Vec3::new(0.0, 3.0, 0.0),
        );

        let mut saida = Vec::new();
        selecao(mundo, 0.01, &mut saida);
        assert_eq!(saida.len(), 12);

        // Toda aresta fica na casca da caixa de mundo: x em +-1 ou y em 3+-0,5.
        for c in centros(&saida) {
            let na_casca = (c.x.abs() - 1.0).abs() < 1e-4 || ((c.y - 3.0).abs() - 0.5).abs() < 1e-4;
            assert!(na_casca, "aresta fora da casca: {c}");
        }
    }
}
