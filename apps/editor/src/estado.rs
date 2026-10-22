//! Estado do editor: o que os paineis leem e escrevem.

use egui_dock::DockState;
use rugine_scene::Scene;

use crate::docking::{Aba, layout_padrao};

/// Ferramenta de manipulacao ativa na barra do viewport.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Ferramenta {
    #[default]
    Selecionar,
    Mover,
    Rotacionar,
    Escalar,
}

impl Ferramenta {
    pub const TODAS: [Self; 4] = [
        Self::Selecionar,
        Self::Mover,
        Self::Rotacionar,
        Self::Escalar,
    ];

    pub fn rotulo(self) -> &'static str {
        match self {
            Self::Selecionar => "Selecionar",
            Self::Mover => "Mover",
            Self::Rotacionar => "Rotacionar",
            Self::Escalar => "Escalar",
        }
    }

    /// Atalho de teclado, na convencao que a maioria dos editores usa.
    pub fn atalho(self) -> &'static str {
        match self {
            Self::Selecionar => "Q",
            Self::Mover => "W",
            Self::Rotacionar => "E",
            Self::Escalar => "R",
        }
    }
}

/// Em que referencial a ferramenta opera.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Espaco {
    #[default]
    Global,
    Local,
}

impl Espaco {
    pub fn rotulo(self) -> &'static str {
        match self {
            Self::Global => "Global",
            Self::Local => "Local",
        }
    }
}

/// Quando desenhar os colliders no viewport.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum ExibirColliders {
    Nunca,
    #[default]
    Selecionado,
    Todos,
}

impl ExibirColliders {
    pub const TODOS: [Self; 3] = [Self::Nunca, Self::Selecionado, Self::Todos];

    pub fn rotulo(self) -> &'static str {
        match self {
            Self::Nunca => "Nunca",
            Self::Selecionado => "Selecionado",
            Self::Todos => "Todos",
        }
    }
}

/// Em que pe esta a compilacao de scripts.
///
/// O editor de referencia mostra isto na barra de menu e no painel de scripts;
/// aqui e so estado, porque a Rugine ainda nao compila scripts.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum EstadoCompilacao {
    #[default]
    Ocioso,
    Compilando {
        segundos: f32,
    },
}

/// Tudo que o editor guarda entre frames.
pub struct Editor {
    pub cena: Scene,

    pub ferramenta: Ferramenta,
    pub espaco: Espaco,
    pub colliders: ExibirColliders,

    pub snap: bool,
    pub tamanho_snap: f32,
    pub grade: bool,
    pub exibir_ui: bool,
    pub exibir_estatisticas: bool,

    pub busca_hierarquia: String,
    pub compilacao: EstadoCompilacao,

    /// O arranjo dos paineis.
    ///
    /// `Option` porque o dock precisa sair do `Editor` durante o desenho: a
    /// `egui_dock` pede `&mut DockState` e `&mut TabViewer` ao mesmo tempo, e
    /// o visualizador carrega `&mut Editor`. Tirar, desenhar e devolver e o
    /// que evita emprestar o `Editor` duas vezes.
    pub dock: Option<DockState<Aba>>,

    /// Scripts listados no painel inferior.
    pub scripts: Vec<String>,

    /// Ultimo tamanho do viewport em pixels, para a barra de estatisticas.
    pub viewport_px: [u32; 2],

    /// Se a barra de titulo e desenhada por nos.
    ///
    /// Decidido no `main` pela plataforma, e lido pelos paineis para saber se
    /// precisam desenhar os botoes de janela e as alcas de redimensionar.
    pub barra_propria: bool,
}

impl Default for Editor {
    fn default() -> Self {
        Self {
            cena: cena_exemplo(),
            ferramenta: Ferramenta::default(),
            espaco: Espaco::default(),
            colliders: ExibirColliders::default(),
            snap: false,
            tamanho_snap: 0.5,
            grade: true,
            exibir_ui: true,
            exibir_estatisticas: true,
            busca_hierarquia: String::new(),
            compilacao: EstadoCompilacao::Ocioso,
            dock: Some(layout_padrao()),
            scripts: vec![
                "Girar.cpp".to_owned(),
                "Item.cpp".to_owned(),
                "Item.h".to_owned(),
                "Jogador.cpp".to_owned(),
                "Sobrevivencia.cpp".to_owned(),
            ],
            viewport_px: [0, 0],
            barra_propria: false,
        }
    }
}

/// Cena inicial, para o editor nao abrir vazio.
///
/// Uma hierarquia com profundidade real e o unico jeito de ver se a arvore da
/// Hierarquia funciona — indentacao, recolher e selecao so aparecem quando ha
/// filhos de filhos.
fn cena_exemplo() -> Scene {
    let mut cena = Scene::new();

    cena.spawn("Sol", None);

    let jogador = cena.spawn("Jogador", None);
    cena.spawn("Camera", Some(jogador));
    let modelo = cena.spawn("Modelo", Some(jogador));
    for indice in 0..7 {
        cena.spawn(format!("SobreviventeCorpo.{indice}"), Some(modelo));
    }

    let hud = cena.spawn("HUD", None);
    let painel = cena.spawn("PainelStatus", Some(hud));
    cena.spawn("RotuloVida", Some(painel));
    cena.spawn("FundoVida", Some(painel));

    cena
}
