//! Formato de vertice da engine.

/// Vertice basico: posicao e cor.
///
/// `repr(C)` e obrigatorio — sem ele o Rust pode reordenar os campos e o
/// layout enviado a GPU nao corresponderia ao declarado em `LAYOUT`.
/// `Pod`/`Zeroable` sao o que autoriza reinterpretar o slice como bytes sem
/// `unsafe` no call site.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl Vertex {
    /// Descricao do layout para o pipeline.
    ///
    /// Os `@location` no WGSL tem que casar com os indices daqui; e um contrato
    /// que o compilador de Rust nao verifica, so o wgpu em runtime.
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
    };

    pub const fn new(position: [f32; 3], color: [f32; 3]) -> Self {
        Self { position, color }
    }
}
