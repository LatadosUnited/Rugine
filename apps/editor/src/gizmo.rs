//! Alcas de manipulacao no viewport.
//!
//! As alcas sao cubos alongados, desenhados pelo mesmo pipeline instanciado
//! das entidades. A consequencia util e que selecionar uma alca usa
//! exatamente a mesma intersecao raio-caixa que selecionar um objeto: nao ha
//! um segundo caminho de picking para manter em sincronia com o primeiro.

use rugine_app::prelude::{Instancia, Mat4, Quat, Vec3};

use crate::estado::Ferramenta;

/// Eixo de uma alca.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Eixo {
    X,
    Y,
    Z,
}

impl Eixo {
    pub const TODOS: [Self; 3] = [Self::X, Self::Y, Self::Z];

    pub fn direcao(self) -> Vec3 {
        match self {
            Self::X => Vec3::X,
            Self::Y => Vec3::Y,
            Self::Z => Vec3::Z,
        }
    }

    /// Vermelho/verde/azul para X/Y/Z, a convencao de todo editor 3D.
    pub fn cor(self) -> [f32; 4] {
        match self {
            Self::X => [0.85, 0.25, 0.28, 1.0],
            Self::Y => [0.35, 0.80, 0.30, 1.0],
            Self::Z => [0.25, 0.45, 0.90, 1.0],
        }
    }

    pub fn cor_destacada(self) -> [f32; 4] {
        let [r, g, b, a] = self.cor();
        [
            (r + 0.35).min(1.0),
            (g + 0.35).min(1.0),
            (b + 0.35).min(1.0),
            a,
        ]
    }
}

/// O gizmo de um frame.
pub struct Gizmo {
    /// Onde ele esta, em coordenadas de mundo.
    pub origem: Vec3,
    /// Comprimento de uma alca no mundo, ja ajustado pela distancia.
    pub comprimento: f32,
    pub ferramenta: Ferramenta,
    /// Alca sob o mouse ou sendo arrastada.
    pub destacado: Option<Eixo>,
}

impl Gizmo {
    /// Fracao da distancia da camera que o gizmo ocupa.
    ///
    /// O gizmo escala com a distancia para manter tamanho aparente constante
    /// na tela. Sem isso ele vira um ponto em cenas grandes e engole a tela
    /// em objetos proximos — em ambos os casos, impossivel de agarrar.
    const FRACAO_DA_TELA: f32 = 0.13;

    /// Espessura da alca, como fracao do comprimento.
    const ESPESSURA: f32 = 0.055;

    pub fn new(
        origem: Vec3,
        posicao_camera: Vec3,
        ferramenta: Ferramenta,
        destacado: Option<Eixo>,
    ) -> Self {
        let distancia = (origem - posicao_camera).length().max(0.001);

        Self {
            origem,
            comprimento: distancia * Self::FRACAO_DA_TELA,
            ferramenta,
            destacado,
        }
    }

    /// Se esta ferramenta desenha alcas.
    pub fn visivel(ferramenta: Ferramenta) -> bool {
        matches!(ferramenta, Ferramenta::Mover | Ferramenta::Escalar)
    }

    /// Matriz de cada alca, para desenho e para picking.
    ///
    /// Uma funcao so para os dois usos de proposito: se o desenho e o teste
    /// de clique calculassem a caixa em lugares diferentes, a alca acabaria
    /// clicavel num ponto e visivel em outro.
    pub fn alcas(&self) -> impl Iterator<Item = (Eixo, Mat4)> + '_ {
        Eixo::TODOS.into_iter().map(move |eixo| {
            let direcao = eixo.direcao();
            let espessura = self.comprimento * Self::ESPESSURA;

            // A haste nasce na origem e vai para o eixo, entao o centro dela
            // fica na metade do caminho.
            let centro = self.origem + direcao * (self.comprimento * 0.5);

            // O cubo unitario e esticado ao longo de X e depois girado para o
            // eixo certo. Girar em vez de escalar cada eixo direto mantem uma
            // forma so para as tres alcas.
            let escala = Vec3::new(self.comprimento, espessura, espessura);
            let rotacao = match eixo {
                Eixo::X => Quat::IDENTITY,
                Eixo::Y => Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
                Eixo::Z => Quat::from_rotation_y(-std::f32::consts::FRAC_PI_2),
            };

            (
                eixo,
                Mat4::from_scale_rotation_translation(escala, rotacao, centro),
            )
        })
    }

    /// Instancias para desenhar: as hastes mais a ponta de cada eixo.
    pub fn instancias(&self) -> Vec<Instancia> {
        let mut saida = Vec::with_capacity(7);

        for (eixo, modelo) in self.alcas() {
            let cor = if self.destacado == Some(eixo) {
                eixo.cor_destacada()
            } else {
                eixo.cor()
            };
            saida.push(Instancia::new(modelo, cor));

            // Ponta: cubo em Escalar, bloco menor em Mover. A forma da ponta
            // e o que diz qual ferramenta esta ativa sem olhar a barra.
            let ponta = self.comprimento
                * if self.ferramenta == Ferramenta::Escalar {
                    0.16
                } else {
                    0.12
                };
            let centro_ponta = self.origem + eixo.direcao() * self.comprimento;
            saida.push(Instancia::new(
                Mat4::from_scale_rotation_translation(
                    Vec3::splat(ponta),
                    Quat::IDENTITY,
                    centro_ponta,
                ),
                cor,
            ));
        }

        // Cubo central branco: marca a origem do gizmo e ajuda a ver que ele
        // esta sobre a entidade certa.
        saida.push(Instancia::new(
            Mat4::from_scale_rotation_translation(
                Vec3::splat(self.comprimento * 0.07),
                Quat::IDENTITY,
                self.origem,
            ),
            [0.9, 0.9, 0.92, 1.0],
        ));

        saida
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rugine_app::prelude::Raio;

    fn gizmo() -> Gizmo {
        // Camera a 10 de distancia, entao o comprimento sai de FRACAO_DA_TELA.
        Gizmo::new(
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 10.0),
            Ferramenta::Mover,
            None,
        )
    }

    #[test]
    fn tres_alcas_uma_por_eixo() {
        let eixos: Vec<Eixo> = gizmo().alcas().map(|(eixo, _)| eixo).collect();
        assert_eq!(eixos, vec![Eixo::X, Eixo::Y, Eixo::Z]);
    }

    /// Cada alca tem que se estender ao longo do seu proprio eixo. Se a
    /// rotacao estiver errada, a alca de Z aponta para X e arrastar move o
    /// objeto na direcao errada.
    #[test]
    fn cada_alca_aponta_para_o_seu_eixo() {
        let gizmo = gizmo();
        let meio = gizmo.comprimento * 0.5;

        for (eixo, modelo) in gizmo.alcas() {
            let centro = modelo.transform_point3(Vec3::ZERO);
            let esperado = eixo.direcao() * meio;
            assert!(
                (centro - esperado).length() < 1e-4,
                "alca {eixo:?} centrada em {centro}, esperado {esperado}"
            );

            // A ponta local +X da caixa, levada ao mundo, tem que cair sobre
            // o eixo do gizmo.
            let ponta = modelo.transform_point3(Vec3::new(0.5, 0.0, 0.0));
            let esperada = eixo.direcao() * gizmo.comprimento;
            assert!(
                (ponta - esperada).length() < 1e-4,
                "ponta da alca {eixo:?} em {ponta}, esperado {esperada}"
            );
        }
    }

    /// Um raio mirado na alca de X tem que selecionar X, nao outra.
    #[test]
    fn raio_na_alca_x_seleciona_x() {
        let gizmo = gizmo();
        let alvo = Vec3::new(gizmo.comprimento * 0.5, 0.0, 0.0);

        let raio = Raio::new(alvo + Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));

        let acertos: Vec<Eixo> = gizmo
            .alcas()
            .filter(|(_, modelo)| raio.intersecta_caixa(*modelo).is_some())
            .map(|(eixo, _)| eixo)
            .collect();

        assert_eq!(acertos, vec![Eixo::X], "deveria acertar so a alca de X");
    }

    #[test]
    fn escalar_e_mover_desenham_alcas_e_selecionar_nao() {
        assert!(Gizmo::visivel(Ferramenta::Mover));
        assert!(Gizmo::visivel(Ferramenta::Escalar));
        assert!(!Gizmo::visivel(Ferramenta::Selecionar));
    }

    /// O gizmo cresce com a distancia, para manter tamanho aparente.
    #[test]
    fn comprimento_acompanha_a_distancia() {
        let perto = Gizmo::new(
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 5.0),
            Ferramenta::Mover,
            None,
        );
        let longe = Gizmo::new(
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, 50.0),
            Ferramenta::Mover,
            None,
        );
        assert!(longe.comprimento > perto.comprimento * 9.0);
    }
}
