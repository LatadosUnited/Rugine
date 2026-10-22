//! Dono do estado de GPU e do ciclo de vida da surface.

use std::sync::Arc;

use raw_window_handle::{HasDisplayHandle, HasWindowHandle};

use crate::camera::CameraBinding;
use crate::cena_gpu::CenaGpu;
use crate::depth::DepthBuffer;
use crate::error::RenderError;
use crate::mesh::{self, Instancia, Mesh};
use crate::pipelines::Pipelines;
use crate::target::RenderTarget;

/// Contadores do ultimo frame, para a barra de estatisticas do editor.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderStats {
    pub draw_calls: u32,
    pub triangles: u32,
}

/// O que ocupa a janela num frame.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FrameContent {
    /// Só a UI. Modo editor: a cena ja foi para a textura offscreen, e a UI a
    /// exibe como imagem dentro do painel de viewport.
    Ui,
    /// A cena ocupando a janela, com a UI por cima. Modo HUD de jogo.
    SceneThenUi,
}

/// Acesso ao frame da janela, entregue ao callback de `present`.
///
/// Existe para que a camada de UI consiga desenhar na janela sem ser dona da
/// surface — ela continua sendo do `Renderer`.
pub struct SurfacePass<'a> {
    pub device: &'a wgpu::Device,
    pub queue: &'a wgpu::Queue,
    pub encoder: &'a mut wgpu::CommandEncoder,
    pub view: &'a wgpu::TextureView,
    pub size_in_pixels: [u32; 2],
}

/// Backends preferidos quando WGPU_BACKEND nao esta definido.
///
/// No Windows, DX12 em vez do padrao do wgpu (que inclui Vulkan): DX12 e a API
/// nativa da plataforma e o caminho de driver melhor mantido. Vulkan no Windows
/// e com frequencia segunda classe, sobretudo em iGPUs Intel mais antigas —
/// numa Intel HD 530 este projeto reproduz um ACCESS_VIOLATION *dentro* do
/// driver Vulkan durante a criacao do device, que nenhum tratamento de erro do
/// lado do Rust consegue capturar, porque nao e um `Err`, e um segfault.
///
/// Quem quiser Vulkan continua tendo: `WGPU_BACKEND=vulkan`.
fn default_backends() -> wgpu::Backends {
    if cfg!(target_os = "windows") {
        wgpu::Backends::DX12
    } else {
        // Linux/Android: Vulkan; macOS/iOS: Metal; web: WebGPU/WebGL.
        wgpu::Backends::PRIMARY
    }
}

/// Todo o estado de GPU da engine.
///
/// A ordem dos campos importa para o drop: a `surface` precisa morrer antes do
/// `device`, e o Rust destroi campos na ordem de declaracao.
pub struct Renderer {
    surface: wgpu::Surface<'static>,
    config: wgpu::SurfaceConfiguration,
    device: wgpu::Device,
    queue: wgpu::Queue,

    /// Guardado porque reconfigurar a surface (resize, troca de monitor)
    /// consulta as capacidades do adapter de novo.
    adapter: wgpu::Adapter,

    /// A surface avisou que esta subotima; reconfiguramos antes do proximo
    /// frame em vez de no meio deste, que ainda tem uma textura em voo.
    needs_reconfigure: bool,

    /// Onde a cena e desenhada no modo editor.
    viewport: RenderTarget,
    /// Profundidade do alvo offscreen.
    depth_viewport: DepthBuffer,
    /// Profundidade quando a cena vai direto para a janela.
    depth_janela: DepthBuffer,

    camera: CameraBinding,
    pipelines: Pipelines,
    cubo: Mesh,
    grade: Mesh,

    /// Buffer de instancias, recriado so quando a cena nao cabe mais nele.
    instancias: wgpu::Buffer,
    capacidade_instancias: usize,

    stats: RenderStats,

    pub clear_color: wgpu::Color,
}

impl Renderer {
    /// Inicializa a GPU para uma janela.
    ///
    /// `window` entra como `Arc` de proposito: a `SurfaceTarget` do wgpu aceita
    /// qualquer `Arc<W>` com os traits de handle, e o resultado e uma surface
    /// de lifetime estatico que *mantem a janela viva* em vez de empresta-la.
    /// Com uma referencia emprestada, a surface herdaria o lifetime da janela e
    /// o runner nao conseguiria guardar as duas na mesma struct.
    pub async fn new<W>(window: Arc<W>, width: u32, height: u32) -> Result<Self, RenderError>
    where
        W: HasWindowHandle + HasDisplayHandle + Send + Sync + 'static,
    {
        // `..._from_env()` deixa WGPU_BACKEND=vulkan|dx12|gl funcionar, o que e
        // a primeira coisa que se quer ao investigar um bug de driver.
        //
        // Sem display handle: no Windows (Vulkan/DX12) e no macOS (Metal) ele
        // nao e usado. No Linux sobre GLES/Wayland ele passa a ser necessario
        // para apresentar — nesse ponto, o caminho e receber a
        // OwnedDisplayHandle do winit e usar `new_with_display_handle`.
        let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle_from_env();

        // Só sobrescrevemos quando o usuario nao pediu backend explicito, para
        // que WGPU_BACKEND continue tendo a palavra final.
        if wgpu::Backends::from_env().is_none() {
            descriptor.backends = default_backends();
        }

        let instance = wgpu::Instance::new(descriptor);

        let surface = instance.create_surface(window)?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::from_env()
                    .unwrap_or(wgpu::PowerPreference::HighPerformance),
                // Amarrar o adapter a surface: em maquinas com GPU integrada e
                // dedicada, nem todo adapter consegue apresentar nesta janela.
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await?;

        let info = adapter.get_info();
        log::info!(
            "GPU: {} | backend {:?} | tipo {:?} | driver {}",
            info.name,
            info.backend,
            info.device_type,
            info.driver_info,
        );

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: Some("rugine::device"),
                // Pedir o minimo e deliberado: cada feature ou limite acima do
                // baseline corta GPUs da lista de suportadas. Subimos isto por
                // necessidade comprovada, nunca por precaucao.
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::default(),
                ..Default::default()
            })
            .await?;

        // `get_default_config` escolhe formato, alpha mode e color space a
        // partir do que o par adapter/surface suporta de fato. Preencher a
        // SurfaceConfiguration a mao e como campos novos (o wgpu 30 adicionou
        // `color_space`) quebram o build a cada versao.
        let mut config = surface
            .get_default_config(&adapter, width.max(1), height.max(1))
            .ok_or(RenderError::SurfaceUnsupported)?;

        // AutoVsync evita queimar GPU desenhando frames que ninguem ve. Um modo
        // sem vsync vira opcao de configuracao quando houver profiling.
        config.present_mode = wgpu::PresentMode::AutoVsync;

        // Permite criar uma view linear da mesma textura da janela. A cena
        // quer o alvo sRGB (a conversao sai de graca no hardware), mas a egui
        // ja entrega cores com gama aplicada: escrever o que ela produz num
        // alvo sRGB aplicaria a correcao duas vezes e a interface sairia
        // lavada. Reinterpretar a mesma textura em dois formatos resolve sem
        // precisar escolher entre cena certa e UI certa.
        let formato_linear = config.format.remove_srgb_suffix();
        if formato_linear != config.format {
            config.view_formats = vec![formato_linear];
        }

        surface.configure(&device, &config);

        log::info!(
            "surface: {}x{} | formato {:?} | present {:?}",
            config.width,
            config.height,
            config.format,
            config.present_mode,
        );

        // O alvo offscreen usa o mesmo formato da surface, para que um unico
        // pipeline sirva aos dois caminhos (jogo e editor).
        let viewport = RenderTarget::new(&device, config.format, config.width, config.height);
        let depth_viewport = DepthBuffer::new(&device, config.width, config.height);
        let depth_janela = DepthBuffer::new(&device, config.width, config.height);

        let camera = CameraBinding::new(&device);
        let pipelines = Pipelines::new(&device, config.format, camera.layout());
        let cubo = mesh::cubo(&device);
        let grade = mesh::grade(&device, 20, 1.0);

        let capacidade_instancias = 256;
        let instancias = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rugine::instancias"),
            size: (capacidade_instancias * size_of::<Instancia>()) as wgpu::BufferAddress,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(Self {
            surface,
            config,
            device,
            queue,
            adapter,
            needs_reconfigure: false,
            viewport,
            depth_viewport,
            depth_janela,
            camera,
            pipelines,
            cubo,
            grade,
            instancias,
            capacidade_instancias,
            stats: RenderStats::default(),
            clear_color: wgpu::Color {
                r: 0.02,
                g: 0.02,
                b: 0.04,
                a: 1.0,
            },
        })
    }

    /// Acompanha a janela quando ela muda de tamanho.
    ///
    /// Retorna `true` se a surface foi de fato reconfigurada. O winit emite
    /// `Resized` tambem em situacoes que nao mudam o tamanho (troca de escala,
    /// restauracao da janela), e reconfigurar a surface a cada um desses
    /// eventos descarta a swapchain sem motivo.
    pub fn resize(&mut self, width: u32, height: u32) -> bool {
        // Janela minimizada chega como 0x0, e uma surface de area zero e um
        // erro de validacao no wgpu. Ignorar e manter a config anterior: o
        // proximo resize valido reconfigura.
        if width == 0 || height == 0 {
            return false;
        }
        if self.config.width == width && self.config.height == height {
            return false;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
        // O depth tem que ter exatamente o tamanho do alvo de cor, senao o
        // wgpu recusa o render pass.
        self.depth_janela.resize(&self.device, width, height);
        self.needs_reconfigure = false;
        true
    }

    /// Redimensiona o alvo offscreen do viewport.
    ///
    /// Retorna `true` se a textura foi recriada — nesse caso quem a exibe
    /// precisa registrar a nova `TextureView`, porque a anterior morreu.
    pub fn resize_viewport(&mut self, width: u32, height: u32) -> bool {
        let mudou = self.viewport.resize(&self.device, width, height);
        if mudou {
            self.depth_viewport.resize(&self.device, width, height);
        }
        mudou
    }

    pub fn viewport(&self) -> &RenderTarget {
        &self.viewport
    }

    /// Largura/altura atuais da surface, em pixels fisicos.
    pub fn size(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Proporcao da surface, pronta para matriz de projecao.
    pub fn aspect_ratio(&self) -> f32 {
        self.config.width as f32 / self.config.height.max(1) as f32
    }

    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    pub fn adapter(&self) -> &wgpu::Adapter {
        &self.adapter
    }

    /// Formato da surface. E o formato contra o qual os pipelines da *cena*
    /// sao compilados.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Formato linear da janela, para pipelines de interface.
    ///
    /// Igual a `surface_format` quando a surface ja e linear.
    pub fn ui_format(&self) -> wgpu::TextureFormat {
        self.config.format.remove_srgb_suffix()
    }

    /// Contadores do ultimo frame desenhado.
    pub fn stats(&self) -> RenderStats {
        self.stats
    }

    /// Envia a cena do frame para a GPU.
    ///
    /// Chamado antes de desenhar: escreve a matriz de camera e as instancias
    /// nos buffers. Separado do desenho porque os dois caminhos (janela e
    /// alvo offscreen) partilham o mesmo envio.
    pub fn enviar_cena(&mut self, cena: &CenaGpu) {
        self.camera.atualizar(&self.queue, cena.view_proj);

        if cena.instancias.len() > self.capacidade_instancias {
            // Cresce em dobro em vez de ajustar exato: redimensionar a cada
            // entidade criada faria uma realocacao por clique no editor.
            let nova = (cena.instancias.len() * 2).max(self.capacidade_instancias * 2);
            self.instancias = self.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("rugine::instancias"),
                size: (nova * size_of::<Instancia>()) as wgpu::BufferAddress,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            self.capacidade_instancias = nova;
            log::debug!("buffer de instancias cresceu para {nova}");
        }

        if !cena.instancias.is_empty() {
            self.queue
                .write_buffer(&self.instancias, 0, bytemuck::cast_slice(&cena.instancias));
        }

        log::debug!(
            "cena: {} instancias, grade={}",
            cena.instancias.len(),
            cena.grade
        );

        // 12 triangulos por cubo; a grade e linha e nao conta como triangulo.
        self.stats = RenderStats {
            draw_calls: u32::from(cena.grade) + u32::from(!cena.instancias.is_empty()),
            triangles: (cena.instancias.len() * 12) as u32,
        };
    }

    /// Desenha a cena numa `TextureView`, com o depth correspondente.
    ///
    /// E o unico lugar que monta o render pass do mundo; os dois caminhos
    /// (direto na janela, ou no alvo offscreen) passam por aqui.
    fn draw_scene_into(
        &self,
        encoder: &mut wgpu::CommandEncoder,
        view: &wgpu::TextureView,
        depth: &DepthBuffer,
        cena: &CenaGpu,
    ) {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("rugine::scene_pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(self.clear_color),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: Some(depth.attachment()),
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });

        pass.set_bind_group(0, self.camera.bind_group(), &[]);

        if cena.grade {
            pass.set_pipeline(&self.pipelines.grade);
            pass.set_vertex_buffer(0, self.grade.vertices().slice(..));
            pass.set_index_buffer(
                self.grade.buffer_indices().slice(..),
                wgpu::IndexFormat::Uint16,
            );
            pass.draw_indexed(0..self.grade.indices(), 0, 0..1);
        }

        if !cena.instancias.is_empty() {
            pass.set_pipeline(&self.pipelines.malha);
            pass.set_vertex_buffer(0, self.cubo.vertices().slice(..));
            pass.set_vertex_buffer(1, self.instancias.slice(..));
            pass.set_index_buffer(
                self.cubo.buffer_indices().slice(..),
                wgpu::IndexFormat::Uint16,
            );
            // Um draw para todas as entidades.
            pass.draw_indexed(0..self.cubo.indices(), 0, 0..cena.instancias.len() as u32);
        }
    }

    /// Desenha a cena no alvo offscreen do viewport (caminho do editor).
    pub fn draw_viewport(&mut self, cena: &CenaGpu) {
        self.enviar_cena(cena);

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("rugine::viewport_encoder"),
            });

        self.draw_scene_into(
            &mut encoder,
            self.viewport.view(),
            &self.depth_viewport,
            cena,
        );

        self.queue.submit(std::iter::once(encoder.finish()));
    }

    /// Adquire o frame da janela e entrega ao callback.
    ///
    /// O callback devolve command buffers que precisam ser submetidos *antes*
    /// do encoder principal — e o caso dos buffers de upload da egui.
    ///
    /// Nao retorna erro: perder um frame e normal (janela redimensionando,
    /// monitor trocando, compositor ocupado) e a resposta certa e quase sempre
    /// pular o frame, nao propagar falha ate o jogo.
    pub fn present<F>(&mut self, content: FrameContent, cena: &CenaGpu, draw: F)
    where
        F: FnOnce(SurfacePass<'_>) -> Vec<wgpu::CommandBuffer>,
    {
        let Some(frame) = self.acquire_frame() else {
            return;
        };

        // Duas views da mesma textura: a cena escreve na sRGB, a UI na linear.
        let view_cena = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let view_ui = frame.texture.create_view(&wgpu::TextureViewDescriptor {
            label: Some("rugine::frame_ui"),
            format: Some(self.ui_format()),
            ..Default::default()
        });

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("rugine::frame"),
            });

        // Desenhada aqui dentro, e nao pelo callback: o callback nao pode
        // pegar `&mut self`, ja que `present` o tomou.
        if content == FrameContent::SceneThenUi {
            self.draw_scene_into(&mut encoder, &view_cena, &self.depth_janela, cena);
        }

        let extra = draw(SurfacePass {
            device: &self.device,
            queue: &self.queue,
            encoder: &mut encoder,
            view: &view_ui,
            size_in_pixels: [self.config.width, self.config.height],
        });

        self.queue
            .submit(extra.into_iter().chain(std::iter::once(encoder.finish())));
        self.queue.present(frame);
    }

    /// Desenha a cena direto na janela (caminho do jogo, sem editor).
    pub fn render(&mut self, cena: &CenaGpu) {
        self.enviar_cena(cena);

        let Some(frame) = self.acquire_frame() else {
            return;
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("rugine::frame"),
            });

        self.draw_scene_into(&mut encoder, &view, &self.depth_janela, cena);

        self.queue.submit(std::iter::once(encoder.finish()));
        self.queue.present(frame);
    }

    /// Tenta pegar a textura do proximo frame, tratando os estados da surface.
    ///
    /// `None` significa "pule este frame"; nao e erro.
    fn acquire_frame(&mut self) -> Option<wgpu::SurfaceTexture> {
        if self.needs_reconfigure {
            self.surface.configure(&self.device, &self.config);
            self.needs_reconfigure = false;
        }

        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(frame) => Some(frame),

            // Usavel, mas a surface mudou de propriedades. Desenhamos neste
            // frame e reconfiguramos antes do proximo.
            wgpu::CurrentSurfaceTexture::Suboptimal(frame) => {
                self.needs_reconfigure = true;
                Some(frame)
            }

            // Recuperaveis reconfigurando: tentamos de novo no proximo frame.
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                None
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                log::warn!("surface perdida; reconfigurando");
                self.surface.configure(&self.device, &self.config);
                None
            }

            // Transitorios: so pular o frame.
            wgpu::CurrentSurfaceTexture::Timeout => {
                log::trace!("timeout ao adquirir o frame; pulando");
                None
            }
            wgpu::CurrentSurfaceTexture::Occluded => None,

            // Erro nosso, de validacao. Nao da para recuperar desenhando.
            wgpu::CurrentSurfaceTexture::Validation => {
                log::error!("erro de validacao ao adquirir o frame da surface");
                None
            }
        }
    }
}
