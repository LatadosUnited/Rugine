//! Componentes: dados opcionais pendurados numa entidade.
//!
//! Cada entidade guarda a propria lista, na ordem em que os componentes foram
//! adicionados — a ordem em que o Inspector os mostra, como na Unity. Achar
//! "todas as entidades com Malha" e percorrer as entidades, o que basta para a
//! escala de um editor. Se o runtime precisar de milhares de entidades por
//! quadro, o armazenamento muda por baixo da API da `Scene`, e os componentes
//! em si nao.

use std::any::Any;
use std::fmt::Debug;

use rugine_core::math::Vec3;

use crate::settings::Color;

/// Um tipo que pode ser pendurado numa entidade.
///
/// `Any` e o que permite guardar tipos diferentes na mesma lista e recuperar
/// o tipo concreto depois. Uma entidade tem no maximo um de cada tipo: duas
/// Malhas na mesma entidade nao teriam significado.
pub trait Componente: Any + Debug {
    /// Nome mostrado no Inspector e no menu de adicionar.
    fn nome(&self) -> &'static str;
}

/// O que se desenha no lugar da entidade.
///
/// Por enquanto toda malha e o cubo unitario. A referencia a um arquivo de
/// malha entra aqui quando houver importacao de glTF.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Malha {
    pub cor: Color,
}

impl Default for Malha {
    fn default() -> Self {
        Self {
            cor: Color::new(0.62, 0.64, 0.70),
        }
    }
}

impl Componente for Malha {
    fn nome(&self) -> &'static str {
        "Malha"
    }
}

/// Forma de um colisor, no espaco local da entidade.
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum FormaColisor {
    /// Caixa com este tamanho total por eixo. `Vec3::ONE` coincide com a
    /// Malha, que e um cubo unitario.
    Caixa { tamanho: Vec3 },
    /// Esfera com este raio. Sob escala nao uniforme ela continua esfera, com
    /// o raio multiplicado pelo maior eixo da escala — o que a fisica consegue
    /// testar, e o que a Unity faz.
    Esfera { raio: f32 },
}

/// Volume para a fisica, que ainda nao existe. Por enquanto ele so e editado
/// e desenhado.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Colisor {
    pub forma: FormaColisor,
    /// Deslocamento do centro em relacao a origem da entidade, no espaco local.
    pub centro: Vec3,
}

impl Default for Colisor {
    fn default() -> Self {
        Self {
            forma: FormaColisor::Caixa { tamanho: Vec3::ONE },
            centro: Vec3::ZERO,
        }
    }
}

impl Componente for Colisor {
    fn nome(&self) -> &'static str {
        "Colisor"
    }
}
