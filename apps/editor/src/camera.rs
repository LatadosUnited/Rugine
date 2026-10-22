//! Camera orbital do viewport.
//!
//! Orbital, e nao primeira pessoa: num editor a operacao mais comum e olhar um
//! objeto de varios angulos, e orbitar em torno de um alvo faz isso num gesto
//! so. Voo livre e util para cenas grandes e entra depois, como modo
//! alternativo.

use rugine_app::prelude::{Mat4, Vec3};
use rugine_core::math::camera;

pub struct CameraOrbital {
    /// Ponto em torno do qual a camera gira.
    pub alvo: Vec3,
    /// Angulo horizontal, em radianos.
    pub yaw: f32,
    /// Angulo vertical, em radianos.
    pub pitch: f32,
    /// Distancia ate o alvo.
    pub distancia: f32,

    pub fov_graus: f32,
    pub perto: f32,
    pub longe: f32,
}

impl Default for CameraOrbital {
    fn default() -> Self {
        Self {
            alvo: Vec3::ZERO,
            yaw: -0.6,
            pitch: 0.45,
            distancia: 12.0,
            fov_graus: 55.0,
            perto: 0.1,
            longe: 1000.0,
        }
    }
}

impl CameraOrbital {
    /// Limite do pitch, um pouco antes do polo.
    ///
    /// Exatamente em 90 graus a direcao da camera fica paralela ao vetor "para
    /// cima" e a matriz de view degenera — a cena some ou vira de cabeca para
    /// baixo. Parar antes e mais simples do que tratar o caso.
    const PITCH_MAX: f32 = 1.5533; // ~89 graus

    pub fn posicao(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        self.alvo
            + Vec3::new(
                self.distancia * cos_pitch * sin_yaw,
                self.distancia * sin_pitch,
                self.distancia * cos_pitch * cos_yaw,
            )
    }

    /// Matriz view-projection pronta para o uniform.
    pub fn view_proj(&self, proporcao: f32) -> Mat4 {
        let view = camera::look_at(self.posicao(), self.alvo, Vec3::Y);
        let proj = camera::perspective(
            self.fov_graus.to_radians(),
            proporcao.max(0.01),
            self.perto,
            self.longe,
        );
        proj * view
    }

    /// Gira em torno do alvo. `delta` em pixels de mouse.
    pub fn orbitar(&mut self, delta_x: f32, delta_y: f32) {
        const SENSIBILIDADE: f32 = 0.008;

        self.yaw -= delta_x * SENSIBILIDADE;
        self.pitch =
            (self.pitch + delta_y * SENSIBILIDADE).clamp(-Self::PITCH_MAX, Self::PITCH_MAX);
    }

    /// Desloca o alvo no plano da tela. `delta` em pixels de mouse.
    pub fn deslocar(&mut self, delta_x: f32, delta_y: f32) {
        let frente = (self.alvo - self.posicao()).normalize_or_zero();
        let direita = frente.cross(Vec3::Y).normalize_or_zero();
        let cima = direita.cross(frente).normalize_or_zero();

        // A velocidade acompanha a distancia: de longe, um pixel precisa
        // cobrir mais mundo, senao arrastar a cena vira um exercicio de
        // paciencia.
        let velocidade = self.distancia * 0.0015;

        self.alvo += (-direita * delta_x + cima * delta_y) * velocidade;
    }

    /// Aproxima ou afasta. `passos` e o scroll do mouse.
    pub fn aproximar(&mut self, passos: f32) {
        // Multiplicativo, nao aditivo: de longe cada passo cobre muito, de
        // perto cobre pouco. Um passo fixo tornaria a aproximacao final
        // lenta demais e a inicial grosseira.
        self.distancia = (self.distancia * (1.0 - passos * 0.1)).clamp(0.5, 500.0);
    }

    /// Enquadra um ponto, mantendo o angulo atual.
    pub fn enquadrar(&mut self, ponto: Vec3) {
        self.alvo = ponto;
        self.distancia = self.distancia.clamp(2.0, 25.0);
    }
}
