//! Buffer de profundidade.
//!
//! Sem ele, o que e desenhado por ultimo cobre o que veio antes, independente
//! de estar na frente ou atras. Com uma geometria so isso nao aparece; com
//! duas que se cruzam, aparece imediatamente.

/// Textura de profundidade acompanhando um alvo de cor.
pub struct DepthBuffer {
    view: wgpu::TextureView,
    width: u32,
    height: u32,
}

impl DepthBuffer {
    /// `Depth32Float` e suportado em todo lugar no baseline do wgpu, inclusive
    /// WebGPU. `Depth24Plus` economizaria memoria, mas a precisao extra e de
    /// graca aqui e evita z-fighting em cenas grandes.
    pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;

    pub fn new(device: &wgpu::Device, width: u32, height: u32) -> Self {
        let width = width.max(1);
        let height = height.max(1);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rugine::depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: Self::FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        Self {
            view: texture.create_view(&wgpu::TextureViewDescriptor::default()),
            width,
            height,
        }
    }

    /// Recria a textura se o tamanho mudou. Retorna `true` se recriou.
    ///
    /// O depth precisa ter exatamente as mesmas dimensoes do alvo de cor; se
    /// ficarem fora de sincronia, o wgpu recusa o render pass.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) -> bool {
        let width = width.max(1);
        let height = height.max(1);

        if self.width == width && self.height == height {
            return false;
        }

        *self = Self::new(device, width, height);
        true
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Como anexar este buffer a um render pass.
    ///
    /// `Clear(1.0)` porque 1.0 e o fundo do alcance de profundidade do wgpu
    /// (`0.0..1.0`, com 0.0 colado na camera): limpar para 1.0 e dizer "tudo
    /// esta infinitamente longe" antes de desenhar.
    pub fn attachment(&self) -> wgpu::RenderPassDepthStencilAttachment<'_> {
        wgpu::RenderPassDepthStencilAttachment {
            view: &self.view,
            depth_ops: Some(wgpu::Operations {
                load: wgpu::LoadOp::Clear(1.0),
                store: wgpu::StoreOp::Store,
            }),
            stencil_ops: None,
        }
    }

    /// Estado de profundidade para um pipeline que escreve no depth.
    pub fn state() -> wgpu::DepthStencilState {
        wgpu::DepthStencilState {
            format: Self::FORMAT,
            // `Option` no wgpu 30: `None` significaria um alvo somente leitura.
            depth_write_enabled: Some(true),
            // `Less` combina com profundidade 0.0 perto: o fragmento mais
            // proximo tem o menor valor e vence.
            depth_compare: Some(wgpu::CompareFunction::Less),
            stencil: wgpu::StencilState::default(),
            bias: wgpu::DepthBiasState::default(),
        }
    }
}
