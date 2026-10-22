//! Decoracao e controles de janela.
//!
//! Uma janela sem decoracao do sistema precisa que a aplicacao forneca tudo o
//! que a barra nativa dava: mover, redimensionar, minimizar, maximizar e
//! fechar. Estes tipos sao a parte portavel disso — o editor desenha, a engine
//! fala com a plataforma.

/// Quem desenha a barra de titulo.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Decoracao {
    /// A barra do sistema operacional.
    #[default]
    Sistema,
    /// Sem barra do sistema: a aplicacao desenha a sua e usa os metodos de
    /// [`crate::Context`] para mover e redimensionar.
    Propria,
}

impl Decoracao {
    /// A escolha que funciona na plataforma atual.
    ///
    /// Propria no Windows e no Linux (X11 e Wayland), onde a winit implementa
    /// `drag_resize_window`. No macOS ela retorna sempre `NotSupported`, entao
    /// uma janela sem decoracao ficaria **sem nenhuma forma de ser
    /// redimensionada** — ali a barra nativa nao e preferencia, e requisito.
    pub fn recomendada() -> Self {
        if cfg!(target_os = "macos") {
            Self::Sistema
        } else {
            Self::Propria
        }
    }
}

/// Qual borda da janela esta sendo arrastada.
///
/// Espelha a `ResizeDirection` da winit, mas em nomes de posicao em vez de
/// pontos cardeais, e mantem o winit fora da API publica da engine.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Borda {
    Cima,
    Baixo,
    Esquerda,
    Direita,
    CimaEsquerda,
    CimaDireita,
    BaixoEsquerda,
    BaixoDireita,
}

impl From<Borda> for winit::window::ResizeDirection {
    fn from(borda: Borda) -> Self {
        match borda {
            Borda::Cima => Self::North,
            Borda::Baixo => Self::South,
            Borda::Esquerda => Self::West,
            Borda::Direita => Self::East,
            Borda::CimaEsquerda => Self::NorthWest,
            Borda::CimaDireita => Self::NorthEast,
            Borda::BaixoEsquerda => Self::SouthWest,
            Borda::BaixoDireita => Self::SouthEast,
        }
    }
}
