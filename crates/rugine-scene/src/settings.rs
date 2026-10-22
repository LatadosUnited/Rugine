//! Ajustes globais da cena — o que o painel "Cena" do editor edita.
//!
//! Hoje isto e so dado: nada aqui chega ao renderer ainda, porque a Rugine nao
//! tem iluminacao nem neblina. Os campos existem para que o painel edite estado
//! real da cena em vez de widgets soltos, e para que ligar cada um ao renderer
//! seja uma mudanca local quando o pipeline de iluminacao existir.

use rugine_core::math::Vec3;

/// Como o fundo da cena e preenchido.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum SkyKind {
    /// Cor chapada.
    #[default]
    Color,
    /// Gradiente procedural horizonte/zenite.
    Gradient,
    /// Cubemap / HDRI.
    Skybox,
}

impl SkyKind {
    pub const ALL: [Self; 3] = [Self::Color, Self::Gradient, Self::Skybox];

    pub fn label(self) -> &'static str {
        match self {
            Self::Color => "Cor",
            Self::Gradient => "Gradiente",
            Self::Skybox => "Skybox",
        }
    }
}

/// Cor RGB linear, 0.0..1.0 por canal.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl Color {
    pub const fn new(r: f32, g: f32, b: f32) -> Self {
        Self { r, g, b }
    }

    /// A partir de 8 bits por canal, que e como o editor mostra.
    pub fn from_u8(r: u8, g: u8, b: u8) -> Self {
        Self::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
    }

    pub fn to_u8(self) -> [u8; 3] {
        [
            (self.r.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.g.clamp(0.0, 1.0) * 255.0).round() as u8,
            (self.b.clamp(0.0, 1.0) * 255.0).round() as u8,
        ]
    }

    pub fn to_vec3(self) -> Vec3 {
        Vec3::new(self.r, self.g, self.b)
    }
}

/// Ceu e luz vinda dele.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Sky {
    pub kind: SkyKind,
    pub color: Color,
    /// Multiplicador do que o ceu emite visualmente.
    pub brightness: f32,
    /// Quanto o ceu contribui para a iluminacao da cena.
    pub light: f32,
}

impl Default for Sky {
    fn default() -> Self {
        Self {
            kind: SkyKind::Color,
            color: Color::from_u8(117, 110, 99),
            brightness: 1.0,
            light: 1.0,
        }
    }
}

/// Neblina e luz ambiente.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Ambient {
    /// Densidade da neblina. Valores pequenos: isto e exponencial, nao linear.
    pub fog_density: f32,
    pub light: Color,
    /// Distancia em metros a partir da qual a neblina satura.
    pub fog_distance: f32,
}

impl Default for Ambient {
    fn default() -> Self {
        Self {
            fog_density: 0.028,
            light: Color::from_u8(43, 43, 48),
            fog_distance: 60.0,
        }
    }
}

/// Pos-processamento.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PostProcessing {
    pub bloom: bool,
    pub bloom_intensity: f32,
    pub exposure: f32,
    pub vignette: f32,
}

impl Default for PostProcessing {
    fn default() -> Self {
        Self {
            bloom: true,
            bloom_intensity: 0.35,
            exposure: 1.0,
            vignette: 0.2,
        }
    }
}

/// Tudo que o painel "Cena" edita.
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct SceneSettings {
    pub sky: Sky,
    pub ambient: Ambient,
    pub post: PostProcessing,
}
