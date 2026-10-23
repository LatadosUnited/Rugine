//! Camera do viewport, no esquema de navegacao do Unity.
//!
//! O ponto que separa esse esquema do da maioria dos outros editores: o botao
//! direito **nao orbita**. Ele olha em volta a partir de onde a camera esta,
//! com WASD voando — a camera nao gira em torno de nada, ela vira a cabeca.
//! Orbitar e Alt + botao esquerdo, em torno de um pivo.
//!
//! Sao dois modos de verdade, nao o mesmo gesto com teclas diferentes, e e por
//! isso que a camera guarda um pivo *e* uma distancia em vez de so uma
//! posicao: orbitar mantem o pivo e move a camera; olhar mantem a camera e
//! move o pivo.

use rugine_app::prelude::{Mat4, Vec3};
use rugine_core::math::camera;

pub struct CameraEditor {
    /// Ponto em torno do qual Alt + esquerdo orbita, e para onde a camera
    /// olha. Fica sempre a `distancia` a frente da camera.
    pivo: Vec3,
    /// Angulo horizontal, em radianos.
    yaw: f32,
    /// Angulo vertical, em radianos.
    pitch: f32,
    /// Distancia da camera ate o pivo.
    distancia: f32,

    pub fov_graus: f32,
    pub perto: f32,
    pub longe: f32,

    /// Velocidade de voo em unidades por segundo.
    pub velocidade_voo: f32,
}

impl Default for CameraEditor {
    fn default() -> Self {
        Self {
            pivo: Vec3::new(0.0, 1.0, 0.0),
            yaw: -0.6,
            pitch: 0.45,
            distancia: 10.0,
            fov_graus: 55.0,
            perto: 0.1,
            longe: 1000.0,
            velocidade_voo: 6.0,
        }
    }
}

impl CameraEditor {
    /// Limite do pitch, um pouco antes do polo.
    ///
    /// Exatamente em 90 graus a direcao da camera fica paralela ao vetor
    /// "para cima" e a matriz de view degenera — a cena some ou vira de
    /// cabeca para baixo. Parar antes e mais simples do que tratar o caso.
    const PITCH_MAX: f32 = 1.5533; // ~89 graus

    const SENSIBILIDADE: f32 = 0.006;

    /// Direcao para onde a camera olha.
    pub fn frente(&self) -> Vec3 {
        let (sin_pitch, cos_pitch) = self.pitch.sin_cos();
        let (sin_yaw, cos_yaw) = self.yaw.sin_cos();

        // Pitch positivo olha para baixo porque a camera fica acima do pivo:
        // o sinal aqui e o que faz arrastar o mouse para cima subir a camera.
        Vec3::new(-cos_pitch * sin_yaw, -sin_pitch, -cos_pitch * cos_yaw)
    }

    pub fn direita(&self) -> Vec3 {
        self.frente().cross(Vec3::Y).normalize_or_zero()
    }

    pub fn cima(&self) -> Vec3 {
        self.direita().cross(self.frente()).normalize_or_zero()
    }

    pub fn posicao(&self) -> Vec3 {
        self.pivo - self.frente() * self.distancia
    }

    /// Matriz view-projection pronta para o uniform.
    pub fn view_proj(&self, proporcao: f32) -> Mat4 {
        let view = camera::look_at(self.posicao(), self.pivo, Vec3::Y);
        let proj = camera::perspective(
            self.fov_graus.to_radians(),
            proporcao.max(0.01),
            self.perto,
            self.longe,
        );
        proj * view
    }

    /// Olhar em volta sem sair do lugar — o botao direito do Unity.
    ///
    /// A camera fica parada e o pivo e reposicionado a frente dela. E o que
    /// diferencia isto de orbitar: aqui quem se move e o alvo, nao a camera.
    pub fn olhar(&mut self, delta_x: f32, delta_y: f32) {
        let posicao = self.posicao();
        self.girar(delta_x, delta_y);
        self.pivo = posicao + self.frente() * self.distancia;
    }

    /// Orbitar em torno do pivo — Alt + botao esquerdo do Unity.
    ///
    /// O pivo fica parado e a camera descreve um arco em volta dele.
    pub fn orbitar(&mut self, delta_x: f32, delta_y: f32) {
        self.girar(delta_x, delta_y);
    }

    fn girar(&mut self, delta_x: f32, delta_y: f32) {
        self.yaw -= delta_x * Self::SENSIBILIDADE;
        self.pitch =
            (self.pitch + delta_y * Self::SENSIBILIDADE).clamp(-Self::PITCH_MAX, Self::PITCH_MAX);
    }

    /// Deslocar no plano da tela — botao do meio do Unity.
    pub fn deslocar(&mut self, delta_x: f32, delta_y: f32) {
        // A velocidade acompanha a distancia: de longe, um pixel precisa
        // cobrir mais mundo, senao arrastar a cena vira exercicio de
        // paciencia.
        let velocidade = self.distancia * 0.0015;
        self.pivo += (-self.direita() * delta_x + self.cima() * delta_y) * velocidade;
    }

    /// Voar com WASD enquanto o botao direito esta pressionado.
    ///
    /// `direcao` e em espaco de camera: X direita, Y cima, Z frente.
    /// O deslocamento e por segundo, nao por frame — senao a camera anda mais
    /// rapido em maquinas com mais fps.
    pub fn voar(&mut self, direcao: Vec3, delta_tempo: f32, rapido: bool) {
        if direcao == Vec3::ZERO {
            return;
        }

        let mundo = (self.direita() * direcao.x + Vec3::Y * direcao.y + self.frente() * direcao.z)
            .normalize_or_zero();

        // Shift multiplica por tres, como no Unity.
        let velocidade = self.velocidade_voo * if rapido { 3.0 } else { 1.0 };
        self.pivo += mundo * velocidade * delta_tempo;
    }

    /// Aproxima ou afasta. `passos` e o scroll do mouse.
    pub fn aproximar(&mut self, passos: f32) {
        // Multiplicativo, nao aditivo: de longe cada passo cobre muito, de
        // perto cobre pouco. Um passo fixo tornaria a aproximacao final lenta
        // demais e a inicial grosseira.
        self.distancia = (self.distancia * (1.0 - passos * 0.1)).clamp(0.5, 500.0);
    }

    /// Enquadra um ponto mantendo o angulo atual — o `F` do Unity.
    pub fn enquadrar(&mut self, ponto: Vec3) {
        self.pivo = ponto;
        self.distancia = self.distancia.clamp(2.0, 20.0);
    }
}
