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

    /// Base ortonormal do plano perpendicular a este eixo.
    ///
    /// `u x v` aponta para o proprio eixo. Essa escolha nao e estetica: e o
    /// que faz `atan2(v, u)` crescer no sentido anti-horario *visto do lado
    /// positivo do eixo*, ou seja, no mesmo sentido de uma rotacao positiva
    /// pela regra da mao direita. Trocar `u` por `v` inverteria o giro.
    pub fn base(self) -> (Vec3, Vec3) {
        match self {
            Self::X => (Vec3::Y, Vec3::Z),
            Self::Y => (Vec3::Z, Vec3::X),
            Self::Z => (Vec3::X, Vec3::Y),
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

/// Uma parte agarravel do gizmo.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Alca {
    /// A haste ou o anel de um eixo.
    Eixo(Eixo),
    /// O cubo do meio, que no gizmo de escala escala os tres eixos juntos.
    Centro,
}

impl Alca {
    /// O eixo da alca, se ela tem um.
    pub fn eixo(self) -> Option<Eixo> {
        match self {
            Self::Eixo(eixo) => Some(eixo),
            Self::Centro => None,
        }
    }
}

/// O gizmo de um frame.
#[derive(Clone, Copy)]
pub struct Gizmo {
    /// Onde ele esta, em coordenadas de mundo.
    pub origem: Vec3,
    /// Para onde os eixos apontam.
    ///
    /// Identidade no modo Global; a rotacao da entidade no modo Local.
    pub orientacao: Quat,
    /// Comprimento de uma alca no mundo, ja ajustado pela distancia.
    pub comprimento: f32,
    pub ferramenta: Ferramenta,
    /// Alca sob o mouse ou sendo arrastada.
    pub destacado: Option<Alca>,
}

impl Gizmo {
    /// Fracao da distancia da camera que o gizmo ocupa.
    ///
    /// O gizmo escala com a distancia para manter tamanho aparente constante
    /// na tela. Sem isso ele vira um ponto em cenas grandes e engole a tela
    /// em objetos proximos — em ambos os casos, impossivel de agarrar.
    const FRACAO_DA_TELA: f32 = 0.13;

    /// Espessura da haste, como fracao do comprimento.
    const ESPESSURA: f32 = 0.055;

    /// Espessura do anel, como fracao do comprimento.
    ///
    /// Mais grosso que a haste: a haste e uma barra longa e chama atencao
    /// pelo comprimento, enquanto o anel so tem a espessura para se fazer
    /// notar.
    const ESPESSURA_ANEL: f32 = 0.085;

    /// Quantos segmentos formam cada anel.
    ///
    /// 28 e o ponto em que o anel ja parece curvo sem o custo virar um
    /// problema: sao 28 cubos por eixo, 84 instancias so de aneis.
    const SEGMENTOS: usize = 28;

    pub fn new(
        origem: Vec3,
        orientacao: Quat,
        posicao_camera: Vec3,
        ferramenta: Ferramenta,
        destacado: Option<Alca>,
    ) -> Self {
        let distancia = (origem - posicao_camera).length().max(0.001);

        Self {
            origem,
            orientacao,
            comprimento: distancia * Self::FRACAO_DA_TELA,
            ferramenta,
            destacado,
        }
    }

    /// Direcao do eixo ja orientada pelo modo Global/Local.
    pub fn eixo_mundo(&self, eixo: Eixo) -> Vec3 {
        (self.orientacao * eixo.direcao()).normalize_or_zero()
    }

    /// Base do plano de um anel, ja orientada.
    pub fn base_mundo(&self, eixo: Eixo) -> (Vec3, Vec3) {
        let (u, v) = eixo.base();
        (
            (self.orientacao * u).normalize_or_zero(),
            (self.orientacao * v).normalize_or_zero(),
        )
    }

    /// Raio dos aneis de rotacao.
    pub fn raio_anel(&self) -> f32 {
        // Maior que a haste do gizmo de mover: o anel precisa sobrar para
        // fora da silhueta do objeto para ser agarravel.
        self.comprimento * 1.05
    }

    /// Se esta ferramenta desenha alcas.
    pub fn visivel(ferramenta: Ferramenta) -> bool {
        matches!(
            ferramenta,
            Ferramenta::Mover | Ferramenta::Escalar | Ferramenta::Rotacionar
        )
    }

    /// Se a ferramenta usa aneis em vez de hastes.
    pub fn usa_aneis(ferramenta: Ferramenta) -> bool {
        ferramenta == Ferramenta::Rotacionar
    }

    /// Matriz de cada alca, para desenho e para picking.
    ///
    /// Uma funcao so para os dois usos de proposito: se o desenho e o teste
    /// de clique calculassem a caixa em lugares diferentes, a alca acabaria
    /// clicavel num ponto e visivel em outro.
    pub fn alcas(&self) -> impl Iterator<Item = (Eixo, Mat4)> + '_ {
        Eixo::TODOS.into_iter().map(move |eixo| {
            let direcao = self.eixo_mundo(eixo);
            let espessura = self.comprimento * Self::ESPESSURA;

            // A haste nasce na origem e vai para o eixo, entao o centro dela
            // fica na metade do caminho.
            let centro = self.origem + direcao * (self.comprimento * 0.5);

            // O cubo unitario e esticado ao longo de X e depois girado para o
            // eixo certo. Girar em vez de escalar cada eixo direto mantem uma
            // forma so para as tres alcas.
            let escala = Vec3::new(self.comprimento, espessura, espessura);
            let rotacao = self.orientacao
                * match eixo {
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

    /// Se a ferramenta tem o cubo central agarravel.
    ///
    /// So a escala: no mover, arrastar o centro precisaria de um plano para
    /// deslizar, e no girar de uma rotacao livre — gestos que nao existem
    /// ainda.
    pub fn tem_centro(ferramenta: Ferramenta) -> bool {
        ferramenta == Ferramenta::Escalar
    }

    /// Matriz do cubo central, para desenho e para picking.
    ///
    /// Quando agarravel ele cresce: o marcador de 7% do comprimento basta
    /// para ver a origem, mas e pequeno demais para mirar.
    pub fn centro(&self) -> Mat4 {
        let lado = if Self::tem_centro(self.ferramenta) {
            0.2
        } else {
            0.07
        };
        Mat4::from_scale_rotation_translation(
            Vec3::splat(self.comprimento * lado),
            self.orientacao,
            self.origem,
        )
    }

    fn cor_centro(&self) -> [f32; 4] {
        if self.destacado == Some(Alca::Centro) {
            [1.0, 0.92, 0.35, 1.0]
        } else {
            [0.9, 0.9, 0.92, 1.0]
        }
    }

    fn cor_eixo(&self, eixo: Eixo) -> [f32; 4] {
        if self.destacado == Some(Alca::Eixo(eixo)) {
            eixo.cor_destacada()
        } else {
            eixo.cor()
        }
    }

    /// Instancias para desenhar.
    pub fn instancias(&self) -> Vec<Instancia> {
        if Self::usa_aneis(self.ferramenta) {
            return self.instancias_aneis();
        }

        let mut saida = Vec::with_capacity(7);

        for (eixo, modelo) in self.alcas() {
            let cor = self.cor_eixo(eixo);
            saida.push(Instancia::new(modelo, cor));

            // Ponta: cubo em Escalar, bloco menor em Mover. A forma da ponta
            // e o que diz qual ferramenta esta ativa sem olhar a barra.
            let ponta = self.comprimento
                * if self.ferramenta == Ferramenta::Escalar {
                    0.16
                } else {
                    0.12
                };
            // `eixo_mundo`, e nao `direcao`: com o gizmo girado (sempre, na
            // escala de um objeto girado) a ponta tem que acompanhar a haste.
            let centro_ponta = self.origem + self.eixo_mundo(eixo) * self.comprimento;
            saida.push(Instancia::new(
                Mat4::from_scale_rotation_translation(
                    Vec3::splat(ponta),
                    self.orientacao,
                    centro_ponta,
                ),
                cor,
            ));
        }

        // Cubo central: marca a origem do gizmo e ajuda a ver que ele esta
        // sobre a entidade certa. Na escala, e tambem a alca dos tres eixos.
        saida.push(Instancia::new(self.centro(), self.cor_centro()));

        saida
    }

    /// Um ponto do anel de um eixo, no angulo dado.
    pub fn ponto_do_anel(&self, eixo: Eixo, angulo: f32) -> Vec3 {
        let (u, v) = self.base_mundo(eixo);
        let raio = self.raio_anel();
        self.origem + (u * angulo.cos() + v * angulo.sin()) * raio
    }

    /// Aneis desenhados como cubinhos ao longo da circunferencia.
    ///
    /// Cubinhos, e nao linhas, para reaproveitar o mesmo pipeline
    /// instanciado de tudo mais. Um anel de linhas exigiria um buffer
    /// dinamico so para o gizmo, e nao ficaria mais legivel.
    fn instancias_aneis(&self) -> Vec<Instancia> {
        let mut saida = Vec::with_capacity(Eixo::TODOS.len() * Self::SEGMENTOS + 1);
        let espessura = self.comprimento * Self::ESPESSURA_ANEL;

        for eixo in Eixo::TODOS {
            let cor = self.cor_eixo(eixo);

            for passo in 0..Self::SEGMENTOS {
                let a = passo as f32 / Self::SEGMENTOS as f32 * std::f32::consts::TAU;
                let b = (passo + 1) as f32 / Self::SEGMENTOS as f32 * std::f32::consts::TAU;

                let inicio = self.ponto_do_anel(eixo, a);
                let corda = self.ponto_do_anel(eixo, b) - inicio;

                let comprimento_segmento = corda.length();
                if comprimento_segmento < 1e-6 {
                    continue;
                }

                saida.push(Instancia::new(
                    Mat4::from_scale_rotation_translation(
                        // Um pouco mais longa que a corda, para os cantos se
                        // encostarem em vez de deixar vao entre segmentos.
                        Vec3::new(comprimento_segmento * 1.08, espessura, espessura),
                        // O cubo nasce esticado em X; girar X ate a tangente
                        // encaixa a barra no anel.
                        Quat::from_rotation_arc(Vec3::X, corda / comprimento_segmento),
                        inicio + corda * 0.5,
                    ),
                    cor,
                ));
            }
        }

        saida.push(Instancia::new(self.centro(), self.cor_centro()));

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
            Quat::IDENTITY,
            Vec3::new(0.0, 0.0, 10.0),
            Ferramenta::Mover,
            None,
        )
    }

    fn gizmo_rotacao(orientacao: Quat) -> Gizmo {
        Gizmo::new(
            Vec3::ZERO,
            orientacao,
            Vec3::new(0.0, 0.0, 10.0),
            Ferramenta::Rotacionar,
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
    fn so_selecionar_nao_desenha_alcas() {
        assert!(Gizmo::visivel(Ferramenta::Mover));
        assert!(Gizmo::visivel(Ferramenta::Escalar));
        assert!(Gizmo::visivel(Ferramenta::Rotacionar));
        assert!(!Gizmo::visivel(Ferramenta::Selecionar));
    }

    /// O gizmo cresce com a distancia, para manter tamanho aparente.
    #[test]
    fn comprimento_acompanha_a_distancia() {
        let perto = Gizmo::new(
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::new(0.0, 0.0, 5.0),
            Ferramenta::Mover,
            None,
        );
        let longe = Gizmo::new(
            Vec3::ZERO,
            Quat::IDENTITY,
            Vec3::new(0.0, 0.0, 50.0),
            Ferramenta::Mover,
            None,
        );
        assert!(longe.comprimento > perto.comprimento * 9.0);
    }

    /// `u x v` tem que apontar para o proprio eixo. Se a base estiver
    /// trocada, o angulo cresce ao contrario e girar vai para o lado errado.
    #[test]
    fn base_do_anel_segue_a_mao_direita() {
        for eixo in Eixo::TODOS {
            let (u, v) = eixo.base();
            let produto = u.cross(v);
            assert!(
                (produto - eixo.direcao()).length() < 1e-5,
                "base de {eixo:?} da {produto}, esperado {}",
                eixo.direcao()
            );
        }
    }

    /// O ponto em angulo zero tem que cair sobre o primeiro vetor da base, e
    /// a um raio de distancia do centro.
    #[test]
    fn ponto_do_anel_comeca_na_base() {
        let gizmo = gizmo_rotacao(Quat::IDENTITY);

        for eixo in Eixo::TODOS {
            let (u, _) = eixo.base();
            let ponto = gizmo.ponto_do_anel(eixo, 0.0);
            let esperado = u * gizmo.raio_anel();

            assert!(
                (ponto - esperado).length() < 1e-4,
                "anel de {eixo:?} em angulo 0 deu {ponto}, esperado {esperado}"
            );
        }
    }

    /// Todo ponto do anel fica a um raio do centro, em qualquer angulo.
    #[test]
    fn anel_e_circular() {
        let gizmo = gizmo_rotacao(Quat::IDENTITY);
        let raio = gizmo.raio_anel();

        for passo in 0..12 {
            let angulo = passo as f32 / 12.0 * std::f32::consts::TAU;
            for eixo in Eixo::TODOS {
                let distancia = (gizmo.ponto_do_anel(eixo, angulo) - gizmo.origem).length();
                assert!(
                    (distancia - raio).abs() < 1e-4,
                    "{eixo:?} em {angulo}: distancia {distancia}, raio {raio}"
                );
            }
        }
    }

    /// No modo Local os eixos acompanham a rotacao da entidade.
    #[test]
    fn orientacao_local_gira_os_eixos() {
        let meia_volta_em_z = Quat::from_rotation_z(std::f32::consts::FRAC_PI_2);
        let gizmo = gizmo_rotacao(meia_volta_em_z);

        // Girados 90 graus em Z, o eixo X do objeto aponta para +Y no mundo.
        let x = gizmo.eixo_mundo(Eixo::X);
        assert!(
            (x - Vec3::Y).length() < 1e-5,
            "X local deveria apontar para +Y, apontou {x}"
        );

        // E Z nao muda, por ser o eixo do giro.
        let z = gizmo.eixo_mundo(Eixo::Z);
        assert!(
            (z - Vec3::Z).length() < 1e-5,
            "Z deveria ficar parado, deu {z}"
        );
    }

    #[test]
    fn rotacionar_usa_aneis_e_as_outras_nao() {
        assert!(Gizmo::usa_aneis(Ferramenta::Rotacionar));
        assert!(!Gizmo::usa_aneis(Ferramenta::Mover));
        assert!(!Gizmo::usa_aneis(Ferramenta::Escalar));
    }

    /// O gizmo de rotacao tem que produzir instancias de verdade: 28 por
    /// anel, tres aneis, mais o centro.
    #[test]
    fn rotacao_gera_instancias() {
        let instancias = gizmo_rotacao(Quat::IDENTITY).instancias();
        assert_eq!(instancias.len(), Gizmo::SEGMENTOS * 3 + 1);
    }

    #[test]
    fn mover_gera_instancias() {
        assert_eq!(gizmo().instancias().len(), 7);
    }

    fn gizmo_escala(orientacao: Quat) -> Gizmo {
        Gizmo::new(
            Vec3::ZERO,
            orientacao,
            Vec3::new(0.0, 0.0, 10.0),
            Ferramenta::Escalar,
            None,
        )
    }

    /// Com o objeto girado, cada ponta tem que ficar no fim da sua haste, e
    /// nao no eixo do mundo.
    #[test]
    fn pontas_acompanham_a_orientacao() {
        let gizmo = gizmo_escala(Quat::from_rotation_z(std::f32::consts::FRAC_PI_2));
        let instancias = gizmo.instancias();

        for (i, eixo) in Eixo::TODOS.into_iter().enumerate() {
            // Pares haste/ponta, na ordem dos eixos.
            let ponta = instancias[i * 2 + 1].modelo();
            let centro = ponta.transform_point3(Vec3::ZERO);
            let esperado = gizmo.eixo_mundo(eixo) * gizmo.comprimento;
            assert!(
                (centro - esperado).length() < 1e-4,
                "ponta de {eixo:?} em {centro}, esperado {esperado}"
            );
        }
    }

    /// So a escala tem centro agarravel, e ele e maior que o marcador.
    #[test]
    fn centro_agarravel_so_na_escala() {
        assert!(Gizmo::tem_centro(Ferramenta::Escalar));
        assert!(!Gizmo::tem_centro(Ferramenta::Mover));
        assert!(!Gizmo::tem_centro(Ferramenta::Rotacionar));

        let lado = |g: &Gizmo| g.centro().to_scale_rotation_translation().0.x;
        assert!(lado(&gizmo_escala(Quat::IDENTITY)) > lado(&gizmo()) * 2.0);
    }
}
