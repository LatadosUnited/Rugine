//! Estado do editor: o que os paineis leem e escrevem.

use egui_dock::DockState;

use crate::camera::CameraEditor;
use crate::gizmo::Eixo;
use rugine_app::prelude::{Quat, Vec3};
use rugine_scene::{EntityId, Scene};

use crate::docking::{Aba, layout_padrao};

/// Ferramenta de manipulacao ativa na barra do viewport.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub enum Ferramenta {
    Selecionar,
    /// Padrao: e a ferramenta mais usada, e com o gizmo visivel desde a
    /// abertura o editor nao exige descobrir um atalho antes de manipular.
    #[default]
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

    /// Camera do viewport.
    pub camera: CameraEditor,

    /// Alca do gizmo sob o mouse, para destaque.
    pub alca_sob_mouse: Option<Eixo>,

    /// Arrasto de alca em andamento.
    pub arrasto: Option<Arrasto>,

    /// Se o modo de voo (botao direito segurado) esta ativo.
    ///
    /// Guardado entre frames porque o gesto comeca e termina em frames
    /// diferentes, e precisa sobreviver ao ponteiro sair do viewport.
    pub voando: bool,

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
            camera: CameraEditor::default(),
            alca_sob_mouse: None,
            arrasto: None,
            voando: false,
            viewport_px: [0, 0],
            barra_propria: false,
        }
    }
}

/// Um arrasto de alca em andamento.
///
/// Guarda o estado do inicio do gesto, nao o do quadro anterior: aplicar
/// sempre o deslocamento total em relacao ao agarre evita o acumulo de
/// arredondamento que faria o objeto escorregar durante o arrasto.
pub struct Arrasto {
    pub eixo: Eixo,
    /// Posicao ao longo do eixo onde o arrasto comecou. So para mover e
    /// escalar.
    pub agarre: f32,
    pub translacao_inicial: Vec3,
    pub escala_inicial: Vec3,
    pub rotacao_inicial: Quat,
    /// Quanto ja se girou, acumulado e sem saltar ao cruzar meia volta.
    ///
    /// `atan2` devolve o angulo em `-pi..pi`, entao passar por 180 graus
    /// daria um salto de uma volta inteira. Guardar o acumulado e comparar
    /// com o anterior e o que permite girar varias voltas seguidas.
    pub angulo_acumulado: f32,
    pub angulo_anterior: f32,
}

/// Cena inicial, para o editor nao abrir vazio.
///
/// Uma hierarquia com profundidade real e o unico jeito de ver se a arvore da
/// Hierarquia funciona — indentacao, recolher e selecao so aparecem quando ha
/// filhos de filhos.
///
/// As transforms importam tanto quanto os nomes: com tudo na origem, os cubos
/// se empilham no mesmo ponto e a cena parece ter um objeto so. Espalhados, e
/// possivel ver o depth buffer separando o que esta na frente, e arrastar um
/// pai no Inspector move os filhos junto — que e o que prova que a matriz de
/// mundo compoe.
fn cena_exemplo() -> Scene {
    let mut cena = Scene::new();

    let sol = cena.spawn("Sol", None);
    posicionar(&mut cena, sol, Vec3::new(-3.5, 4.0, -3.0), Vec3::splat(0.6));

    // O "sobrevivente": um boneco de caixas, montado como filho de `Modelo`,
    // que por sua vez e filho de `Jogador`. Dois niveis de profundidade de
    // proposito, para exercitar a composicao de transforms.
    let jogador = cena.spawn("Jogador", None);
    posicionar(&mut cena, jogador, Vec3::new(0.0, 0.0, 0.0), Vec3::ONE);

    let camera = cena.spawn("Camera", Some(jogador));
    posicionar(
        &mut cena,
        camera,
        Vec3::new(0.0, 1.8, 0.6),
        Vec3::splat(0.3),
    );

    let modelo = cena.spawn("Modelo", Some(jogador));
    posicionar(&mut cena, modelo, Vec3::ZERO, Vec3::ONE);

    // (posicao, escala) de cada parte, relativas ao `Modelo`.
    let partes = [
        (Vec3::new(0.0, 1.55, 0.0), Vec3::new(0.45, 0.45, 0.45)), // cabeca
        (Vec3::new(0.0, 0.95, 0.0), Vec3::new(0.70, 0.80, 0.40)), // tronco
        (Vec3::new(-0.50, 0.95, 0.0), Vec3::new(0.22, 0.70, 0.22)), // braco esq
        (Vec3::new(0.50, 0.95, 0.0), Vec3::new(0.22, 0.70, 0.22)), // braco dir
        (Vec3::new(-0.20, 0.30, 0.0), Vec3::new(0.25, 0.65, 0.25)), // perna esq
        (Vec3::new(0.20, 0.30, 0.0), Vec3::new(0.25, 0.65, 0.25)), // perna dir
        (Vec3::new(0.0, 1.30, 0.0), Vec3::new(0.30, 0.20, 0.30)), // pescoco
    ];

    for (indice, (posicao, escala)) in partes.into_iter().enumerate() {
        let parte = cena.spawn(format!("SobreviventeCorpo.{indice}"), Some(modelo));
        posicionar(&mut cena, parte, posicao, escala);
    }

    let hud = cena.spawn("HUD", None);
    posicionar(&mut cena, hud, Vec3::new(4.0, 1.0, 2.5), Vec3::ONE);

    let painel = cena.spawn("PainelStatus", Some(hud));
    posicionar(&mut cena, painel, Vec3::ZERO, Vec3::new(1.6, 0.9, 0.1));

    let rotulo = cena.spawn("RotuloVida", Some(painel));
    posicionar(
        &mut cena,
        rotulo,
        Vec3::new(-0.25, 0.2, 0.6),
        Vec3::splat(0.3),
    );

    let fundo = cena.spawn("FundoVida", Some(painel));
    posicionar(
        &mut cena,
        fundo,
        Vec3::new(0.0, -0.2, 0.6),
        Vec3::new(0.8, 0.15, 0.2),
    );

    // Comeca com algo selecionado: o editor abre ja mostrando o gizmo, em
    // vez de exigir um clique para descobrir que ele existe.
    cena.select(Some(jogador));

    cena
}

/// Atalho para montar a cena de exemplo sem repetir o acesso ao `Transform`.
fn posicionar(cena: &mut Scene, id: EntityId, posicao: Vec3, escala: Vec3) {
    if let Some(entidade) = cena.get_mut(id) {
        entidade.transform.translation = posicao;
        entidade.transform.scale = escala;
    }
}
