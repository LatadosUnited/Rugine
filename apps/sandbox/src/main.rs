//! Sandbox: a aplicacao que exercita a engine durante o desenvolvimento.
//!
//! Serve de teste de fumaca e de exemplo de como um jogo consome a Rugine.
//! Se algo aqui ficar complicado de escrever, e sinal de que a API da engine
//! esta errada.

use rugine_app::prelude::*;

/// O "jogo": anima a cor de fundo e reporta o framerate.
///
/// A cor oscilar prova que o `Time` esta avancando e que estamos realmente
/// redesenhando, e nao olhando um frame congelado.
struct Sandbox {
    /// Quando reportar o fps da proxima vez, em segundos de execucao.
    next_report: f32,
    /// Encerra sozinho apos N segundos, de `RUGINE_EXIT_AFTER`.
    ///
    /// Existe para smoke test: permite verificar num terminal (ou em CI) que a
    /// engine inicializa a GPU e produz frames, sem precisar de alguem para
    /// fechar a janela.
    exit_after: Option<f32>,
}

impl App for Sandbox {
    fn init(&mut self, ctx: &mut Context<'_>) {
        let (width, height) = ctx.renderer.size();
        log::info!("sandbox iniciado em {width}x{height}");
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        let t = ctx.time.elapsed();

        // Senoides defasadas, remapeadas de [-1, 1] para [0, 1] e escurecidas,
        // para que o triangulo continue legivel sobre o fundo.
        ctx.renderer.clear_color = wgpu::Color {
            r: (t * 0.7).sin() as f64 * 0.5 + 0.5,
            g: (t * 0.9 + 2.0).sin() as f64 * 0.5 + 0.5,
            b: (t * 1.1 + 4.0).sin() as f64 * 0.5 + 0.5,
            a: 1.0,
        };
        ctx.renderer.clear_color.r *= 0.15;
        ctx.renderer.clear_color.g *= 0.15;
        ctx.renderer.clear_color.b *= 0.2;

        if let Some(limite) = self.exit_after
            && t >= limite
        {
            log::info!(
                "RUGINE_EXIT_AFTER={limite}s atingido apos {} frames; encerrando",
                ctx.time.frames()
            );
            ctx.exit();
        }

        if t >= self.next_report {
            self.next_report = t + 1.0;
            log::info!(
                "{:>6.1} fps | frame {} | delta {:.2} ms",
                ctx.time.fps(),
                ctx.time.frames(),
                ctx.time.delta() * 1000.0,
            );
        }
    }

    fn resized(&mut self, width: u32, height: u32) {
        log::info!("redimensionado para {width}x{height}");
    }
}

fn main() -> anyhow::Result<()> {
    // `info` por padrao, mas RUST_LOG continua mandando: as mensagens do wgpu
    // em `debug` sao a primeira ferramenta quando um frame sai errado.
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let exit_after = std::env::var("RUGINE_EXIT_AFTER")
        .ok()
        .and_then(|valor| valor.parse::<f32>().ok());

    run(
        EngineConfig {
            title: "Rugine — sandbox".to_owned(),
            ..Default::default()
        },
        Sandbox {
            next_report: 0.0,
            exit_after,
        },
    )
}
