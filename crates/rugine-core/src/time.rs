//! Relogio do frame.

use std::time::Instant;

/// Mede o tempo entre frames.
///
/// `delta` e capado: se a janela for arrastada ou o processo suspenso, o
/// primeiro frame depois da pausa chegaria com um delta enorme e faria a
/// simulacao dar um salto (ou atravessar colisores). Capar e mais barato do
/// que tratar o caso em cada sistema.
pub struct Time {
    start: Instant,
    last: Instant,
    delta: f32,
    frames: u64,
    max_delta: f32,
}

impl Time {
    /// Teto de delta em segundos, equivalente a ~5 fps.
    pub const DEFAULT_MAX_DELTA: f32 = 0.2;

    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start: now,
            last: now,
            delta: 0.0,
            frames: 0,
            max_delta: Self::DEFAULT_MAX_DELTA,
        }
    }

    /// Fecha o frame anterior. Chamado uma vez por frame pelo runner.
    pub fn tick(&mut self) {
        let now = Instant::now();
        self.delta = (now - self.last).as_secs_f32().min(self.max_delta);
        self.last = now;
        self.frames += 1;
    }

    /// Tempo do ultimo frame, em segundos.
    pub fn delta(&self) -> f32 {
        self.delta
    }

    /// Tempo desde o inicio da aplicacao, em segundos.
    pub fn elapsed(&self) -> f32 {
        self.last.duration_since(self.start).as_secs_f32()
    }

    /// Frames completos desde o inicio.
    pub fn frames(&self) -> u64 {
        self.frames
    }

    /// Frames por segundo instantaneo (1/delta), ou 0 no primeiro frame.
    pub fn fps(&self) -> f32 {
        if self.delta > 0.0 { 1.0 / self.delta } else { 0.0 }
    }
}

impl Default for Time {
    fn default() -> Self {
        Self::new()
    }
}
