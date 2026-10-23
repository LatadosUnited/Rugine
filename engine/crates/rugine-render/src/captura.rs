//! Leitura do viewport de volta para a CPU.
//!
//! Serve para duas coisas: capturar a tela do que a engine desenhou e,
//! principalmente, verificar o render sem depender de alguem olhar. "Nenhum
//! erro de validacao" nao e o mesmo que "apareceu na tela", e a diferenca
//! entre os dois ja custou caro neste projeto.

/// Pixels lidos do viewport.
pub struct Captura {
    pub largura: u32,
    pub altura: u32,
    /// RGBA de 8 bits por canal, sem padding entre linhas.
    pub rgba: Vec<u8>,
}

impl Captura {
    /// Cor de um pixel.
    pub fn pixel(&self, x: u32, y: u32) -> [u8; 4] {
        let indice = ((y * self.largura + x) * 4) as usize;
        [
            self.rgba[indice],
            self.rgba[indice + 1],
            self.rgba[indice + 2],
            self.rgba[indice + 3],
        ]
    }

    /// Quantos pixels satisfazem um predicado.
    ///
    /// E o que transforma "parece certo" em numero: contar pixels de uma cor
    /// esperada responde se algo foi desenhado, sem olhar.
    pub fn contar(&self, predicado: impl Fn([u8; 4]) -> bool) -> usize {
        self.rgba
            .chunks_exact(4)
            .filter(|p| predicado([p[0], p[1], p[2], p[3]]))
            .count()
    }
}

/// Alinhamento exigido pelo wgpu para bytes por linha numa copia.
const ALINHAMENTO: u32 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;

/// Le a textura de volta para a CPU.
///
/// Bloqueia ate a GPU terminar: e diagnostico, nao caminho de frame.
pub(crate) fn ler_textura(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    textura: &wgpu::Texture,
    largura: u32,
    altura: u32,
) -> Option<Captura> {
    // A surface costuma ser BGRA, nao RGBA. Sem reordenar, vermelho e azul
    // saem trocados e a captura mente justamente sobre a cor — que e o que
    // se quer verificar nela.
    let trocar_rb = matches!(
        textura.format(),
        wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb
    );

    // Cada linha do buffer de destino tem que comecar num multiplo de 256
    // bytes, entao quase sempre sobra padding no fim da linha — que e
    // removido ao montar o `Vec` final.
    let bytes_por_pixel = 4;
    let bytes_crus = largura * bytes_por_pixel;
    let bytes_por_linha = bytes_crus.div_ceil(ALINHAMENTO) * ALINHAMENTO;

    let destino = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("rugine::captura"),
        size: (bytes_por_linha * altura) as wgpu::BufferAddress,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("rugine::captura_encoder"),
    });

    encoder.copy_texture_to_buffer(
        wgpu::TexelCopyTextureInfo {
            texture: textura,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        wgpu::TexelCopyBufferInfo {
            buffer: &destino,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_por_linha),
                rows_per_image: Some(altura),
            },
        },
        wgpu::Extent3d {
            width: largura,
            height: altura,
            depth_or_array_layers: 1,
        },
    );

    queue.submit(std::iter::once(encoder.finish()));

    // O mapeamento e assincrono por causa do WebGPU. No desktop, esperar a
    // fila e suficiente para o callback ter rodado.
    let (envio, recebimento) = std::sync::mpsc::channel();
    destino.slice(..).map_async(wgpu::MapMode::Read, move |r| {
        let _ = envio.send(r);
    });

    if device.poll(wgpu::PollType::wait_indefinitely()).is_err() {
        log::error!("captura: falha ao esperar a GPU");
        return None;
    }

    match recebimento.recv() {
        Ok(Ok(())) => {}
        outro => {
            log::error!("captura: mapeamento falhou ({outro:?})");
            return None;
        }
    }

    // `get_mapped_range` passou a devolver Result no wgpu 30.
    let Ok(dados) = destino.slice(..).get_mapped_range() else {
        log::error!("captura: nao foi possivel ler o intervalo mapeado");
        return None;
    };

    let mut rgba = Vec::with_capacity((largura * altura * bytes_por_pixel) as usize);
    for linha in 0..altura {
        let inicio = (linha * bytes_por_linha) as usize;
        let fatia = &dados[inicio..inicio + bytes_crus as usize];

        if trocar_rb {
            for pixel in fatia.chunks_exact(4) {
                rgba.extend_from_slice(&[pixel[2], pixel[1], pixel[0], pixel[3]]);
            }
        } else {
            rgba.extend_from_slice(fatia);
        }
    }

    drop(dados);
    destino.unmap();

    Some(Captura {
        largura,
        altura,
        rgba,
    })
}
