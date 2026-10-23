//! Alvo de render offscreen.
//!
//! Num editor o viewport e um retangulo dentro da janela, nao a janela. A cena
//! e desenhada aqui, e a UI exibe esta textura como uma imagem. O efeito
//! colateral util e que isto e exatamente a infraestrutura necessaria para
//! teste de pixel headless e para capturas de tela.

/// Textura na qual a cena e desenhada.
pub struct RenderTarget {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    format: wgpu::TextureFormat,
    width: u32,
    height: u32,
}

impl RenderTarget {
    /// Menor dimensao aceita.
    ///
    /// Uma textura de lado zero e erro de validacao, e o viewport do editor
    /// passa por zero enquanto o usuario arrasta um divisor de painel.
    pub const MIN_SIZE: u32 = 1;

    /// Maior dimensao aceita.
    ///
    /// O limite real vem de `Limits::max_texture_dimension_2d` (8192 no
    /// baseline do wgpu). Arrastar um painel nao deveria conseguir pedir uma
    /// textura que o device recusa.
    pub const MAX_SIZE: u32 = 8192;

    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        width: u32,
        height: u32,
    ) -> Self {
        let width = width.clamp(Self::MIN_SIZE, Self::MAX_SIZE);
        let height = height.clamp(Self::MIN_SIZE, Self::MAX_SIZE);

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("rugine::viewport"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            // RENDER_ATTACHMENT para desenhar nela; TEXTURE_BINDING para a UI
            // conseguir amostra-la; COPY_SRC para ler de volta (screenshot e
            // teste de pixel).
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self {
            texture,
            view,
            format,
            width,
            height,
        }
    }

    /// Recria a textura se o tamanho pedido for diferente do atual.
    ///
    /// Retorna `true` quando recriou — quem exibe a textura precisa saber,
    /// porque a `TextureView` anterior deixou de valer.
    pub fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) -> bool {
        let width = width.clamp(Self::MIN_SIZE, Self::MAX_SIZE);
        let height = height.clamp(Self::MIN_SIZE, Self::MAX_SIZE);

        if self.width == width && self.height == height {
            return false;
        }

        *self = Self::new(device, self.format, width, height);
        true
    }

    pub fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub fn texture(&self) -> &wgpu::Texture {
        &self.texture
    }

    pub fn format(&self) -> wgpu::TextureFormat {
        self.format
    }

    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height.max(1) as f32
    }
}
