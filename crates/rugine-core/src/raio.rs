//! Raios e intersecoes, para selecao no viewport.

use crate::math::{Mat4, Vec3, Vec4};

/// Semirreta no espaco de mundo.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Raio {
    pub origem: Vec3,
    /// Nao precisa ser unitaria. O parametro `t` devolvido pelas intersecoes
    /// e sempre em unidades desta direcao, e manter o comprimento original e
    /// o que faz `t` significar a mesma coisa antes e depois de transformar
    /// o raio para o espaco local de um objeto.
    pub direcao: Vec3,
}

impl Raio {
    pub fn new(origem: Vec3, direcao: Vec3) -> Self {
        Self { origem, direcao }
    }

    pub fn em(&self, t: f32) -> Vec3 {
        self.origem + self.direcao * t
    }

    /// Raio que sai da camera e passa por um ponto da tela.
    ///
    /// `ponto` em pixels, com a origem no canto superior esquerdo do viewport
    /// — a convencao da egui e da maioria das APIs de janela. `tamanho` e o
    /// do viewport, nos mesmos pixels.
    ///
    /// O raio e montado desprojetando dois pontos, no plano proximo e no
    /// distante, em vez de montar a direcao a partir do angulo de abertura:
    /// assim a conta vale igual para projecao em perspectiva e ortografica,
    /// sem saber qual das duas esta em uso.
    pub fn da_tela(view_proj: Mat4, ponto: (f32, f32), tamanho: (f32, f32)) -> Self {
        let inversa = view_proj.inverse();

        // Pixels para NDC. O Y inverte porque a tela cresce para baixo e o
        // NDC cresce para cima.
        let x = 2.0 * (ponto.0 / tamanho.0.max(1.0)) - 1.0;
        let y = 1.0 - 2.0 * (ponto.1 / tamanho.1.max(1.0));

        // Z = 0 e o plano proximo e Z = 1 o distante, que e a convencao de
        // profundidade do wgpu (e a que `rugine_core::math::camera` fixa).
        let perto = desprojetar(inversa, x, y, 0.0);
        let longe = desprojetar(inversa, x, y, 1.0);

        Self::new(perto, longe - perto)
    }

    /// Intersecao com uma caixa unitaria centrada na origem, depois de
    /// transformada por `modelo`.
    ///
    /// Retorna o `t` da primeira entrada, ou `None` se o raio passa longe ou
    /// se a caixa esta inteiramente atras da origem do raio.
    ///
    /// Em vez de transformar os oito cantos da caixa para o mundo, leva o
    /// raio para o espaco local da caixa: la ela e um cubo alinhado aos
    /// eixos, e o teste vira o algoritmo das fatias, que sao seis divisoes.
    pub fn intersecta_caixa(&self, modelo: Mat4) -> Option<f32> {
        let inversa = modelo.inverse();

        // `w = 1` para ponto (sofre translacao), `w = 0` para direcao (nao
        // sofre). Trocar os dois e o erro classico aqui, e ele se manifesta
        // como selecao funcionando so em objetos na origem.
        let origem = inversa.transform_point3(self.origem);
        let direcao = inversa.transform_vector3(self.direcao);

        intersecta_cubo_unitario(origem, direcao)
    }
}

fn desprojetar(inversa: Mat4, x: f32, y: f32, z: f32) -> Vec3 {
    let p = inversa * Vec4::new(x, y, z, 1.0);
    // A divisao perspectiva: sem ela, o ponto fica errado em qualquer
    // projecao que nao seja ortografica.
    p.truncate() / p.w
}

/// Algoritmo das fatias contra a caixa `[-0.5, 0.5]` nos tres eixos.
fn intersecta_cubo_unitario(origem: Vec3, direcao: Vec3) -> Option<f32> {
    const MEIO: f32 = 0.5;

    let mut t_entrada = f32::NEG_INFINITY;
    let mut t_saida = f32::INFINITY;

    for eixo in 0..3 {
        let o = origem[eixo];
        let d = direcao[eixo];

        if d.abs() < 1e-7 {
            // Raio paralelo a este par de faces: ou ja esta entre elas, ou
            // nunca vai estar. Dividir aqui daria infinito e envenenaria a
            // comparacao abaixo.
            if !(-MEIO..=MEIO).contains(&o) {
                return None;
            }
            continue;
        }

        let inverso = 1.0 / d;
        let mut t1 = (-MEIO - o) * inverso;
        let mut t2 = (MEIO - o) * inverso;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }

        t_entrada = t_entrada.max(t1);
        t_saida = t_saida.min(t2);

        if t_entrada > t_saida {
            return None;
        }
    }

    // Caixa inteiramente atras da origem do raio.
    if t_saida < 0.0 {
        return None;
    }

    // Com a origem dentro da caixa, `t_entrada` e negativo; o primeiro ponto
    // a frente e a propria origem.
    Some(if t_entrada < 0.0 { 0.0 } else { t_entrada })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{Quat, camera};

    fn raio_para_frente() -> Raio {
        Raio::new(Vec3::new(0.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0))
    }

    #[test]
    fn acerta_caixa_na_origem() {
        let t = raio_para_frente().intersecta_caixa(Mat4::IDENTITY);
        // A caixa vai de z = 0.5 a z = -0.5, entao a entrada esta a 4.5.
        assert!((t.unwrap() - 4.5).abs() < 1e-4, "deu {t:?}");
    }

    #[test]
    fn erra_caixa_deslocada_para_o_lado() {
        let modelo = Mat4::from_translation(Vec3::new(5.0, 0.0, 0.0));
        assert_eq!(raio_para_frente().intersecta_caixa(modelo), None);
    }

    #[test]
    fn ignora_caixa_atras_do_raio() {
        let modelo = Mat4::from_translation(Vec3::new(0.0, 0.0, 20.0));
        assert_eq!(raio_para_frente().intersecta_caixa(modelo), None);
    }

    /// Se ponto e direcao forem transformados do mesmo jeito, a selecao passa
    /// a funcionar so em objetos na origem. Este teste prende isso.
    #[test]
    fn respeita_translacao_da_caixa() {
        let modelo = Mat4::from_translation(Vec3::new(0.0, 0.0, -10.0));
        let t = raio_para_frente().intersecta_caixa(modelo);
        // Caixa centrada em z = -10, face em z = -9.5, raio parte de z = 5.
        assert!((t.unwrap() - 14.5).abs() < 1e-4, "deu {t:?}");
    }

    #[test]
    fn respeita_escala_da_caixa() {
        // Raio passa a 2 unidades do centro: erra o cubo unitario...
        let raio = Raio::new(Vec3::new(2.0, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(raio.intersecta_caixa(Mat4::IDENTITY), None);

        // ...mas acerta se a caixa for esticada ate ele.
        let esticada = Mat4::from_scale(Vec3::new(6.0, 1.0, 1.0));
        assert!(raio.intersecta_caixa(esticada).is_some());
    }

    #[test]
    fn respeita_rotacao_da_caixa() {
        // Caixa fina em X, alta em Y, girada 90 graus em Z: a parte fina
        // passa a ser vertical, e um raio deslocado em Y erra.
        let modelo = Mat4::from_scale_rotation_translation(
            Vec3::new(4.0, 0.1, 1.0),
            Quat::from_rotation_z(std::f32::consts::FRAC_PI_2),
            Vec3::ZERO,
        );

        let no_eixo_y = Raio::new(Vec3::new(0.0, 1.5, 5.0), Vec3::new(0.0, 0.0, -1.0));
        assert!(
            no_eixo_y.intersecta_caixa(modelo).is_some(),
            "depois de girar, a caixa deveria se estender em Y"
        );

        let no_eixo_x = Raio::new(Vec3::new(1.5, 0.0, 5.0), Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(
            no_eixo_x.intersecta_caixa(modelo),
            None,
            "depois de girar, a caixa nao deveria mais se estender em X"
        );
    }

    #[test]
    fn origem_dentro_da_caixa_da_zero() {
        let raio = Raio::new(Vec3::ZERO, Vec3::new(0.0, 0.0, -1.0));
        assert_eq!(raio.intersecta_caixa(Mat4::IDENTITY), Some(0.0));
    }

    /// O raio do centro da tela tem que apontar para onde a camera olha.
    #[test]
    fn raio_do_centro_aponta_para_a_frente() {
        let olho = Vec3::new(0.0, 0.0, 8.0);
        let view = camera::look_at(olho, Vec3::ZERO, Vec3::Y);
        let proj = camera::perspective(60f32.to_radians(), 16.0 / 9.0, 0.1, 100.0);

        let raio = Raio::da_tela(proj * view, (640.0, 360.0), (1280.0, 720.0));

        assert!(
            (raio.origem - olho).length() < 0.2,
            "deveria partir da camera, partiu de {}",
            raio.origem
        );

        let direcao = raio.direcao.normalize();
        assert!(
            (direcao - Vec3::new(0.0, 0.0, -1.0)).length() < 1e-3,
            "deveria apontar para -Z, apontou para {direcao}"
        );

        assert!(raio.intersecta_caixa(Mat4::IDENTITY).is_some());
    }

    /// Clicar no alto da tela tem que mirar acima do centro, nao abaixo.
    /// E o teste que pega a inversao de Y entre tela e NDC.
    #[test]
    fn topo_da_tela_mira_para_cima() {
        let view = camera::look_at(Vec3::new(0.0, 0.0, 8.0), Vec3::ZERO, Vec3::Y);
        let proj = camera::perspective(60f32.to_radians(), 1.0, 0.1, 100.0);

        let raio = Raio::da_tela(proj * view, (320.0, 40.0), (640.0, 640.0));

        assert!(
            raio.direcao.normalize().y > 0.0,
            "um clique no topo deveria mirar para cima, mirou {}",
            raio.direcao.normalize()
        );
    }
}
