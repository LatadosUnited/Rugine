//! Sandbox: a aplicacao que exercita a engine sem editor.
//!
//! Serve de teste de fumaca e de exemplo de como um jogo consome a Rugine. Se
//! algo aqui ficar complicado de escrever, e sinal de que a API esta errada.
//!
//! Ao contrario do editor, roda em `UiMode::Disabled`: a cena ocupa a janela
//! inteira e nao ha egui no caminho. E o modo de um jogo publicado.

use rugine_app::prelude::*;

/// Numero de cubos na grade de demonstracao, por lado.
const LADO: i32 = 4;

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
        let (largura, altura) = ctx.renderer.size();
        log::info!("sandbox iniciado em {largura}x{altura}");
    }

    fn update(&mut self, ctx: &mut Context<'_>) {
        let t = ctx.time.elapsed();

        // Camera orbitando sozinha: sem interacao, e o que prova que a matriz
        // de view-projection chega a GPU a cada frame.
        let raio = 11.0;
        let olho = Vec3::new(
            raio * (t * 0.25).sin(),
            5.0 + (t * 0.4).sin() * 1.5,
            raio * (t * 0.25).cos(),
        );
        let view = camera::look_at(olho, Vec3::ZERO, Vec3::Y);
        let proj = camera::perspective(60f32.to_radians(), ctx.renderer.aspect_ratio(), 0.1, 500.0);

        // Uma grade de cubos pulando em fases diferentes. Eles se cruzam
        // visualmente, que e justamente o que exige depth buffer: sem ele, o
        // ultimo desenhado cobriria os outros.
        let mut instancias = Vec::new();
        for x in -LADO..=LADO {
            for z in -LADO..=LADO {
                let fase = (x + z) as f32 * 0.5;
                let altura = (t * 1.6 + fase).sin() * 0.8;

                let modelo =
                    Mat4::from_translation(Vec3::new(x as f32 * 1.6, altura, z as f32 * 1.6));

                // Cor variando no plano, para as faces nao virarem uma massa
                // cinza unica.
                let u = (x + LADO) as f32 / (LADO * 2) as f32;
                let v = (z + LADO) as f32 / (LADO * 2) as f32;
                instancias.push(Instancia::new(
                    modelo,
                    [0.3 + u * 0.6, 0.4, 0.3 + v * 0.6, 1.0],
                ));
            }
        }

        ctx.definir_cena(CenaGpu {
            view_proj: proj * view,
            instancias,
            grade: true,
        });

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
            let stats = ctx.renderer.stats();
            log::info!(
                "{:>6.1} fps | frame {} | {} draws | {} tris",
                ctx.time.fps(),
                ctx.time.frames(),
                stats.draw_calls,
                stats.triangles,
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
            // `RUGINE_VSYNC=0` sobrepoe isto sem recompilar.
            vsync: true,
            ..Default::default()
        },
        Sandbox {
            next_report: 0.0,
            exit_after,
        },
    )
}
