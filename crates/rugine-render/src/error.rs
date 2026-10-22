//! Erros de inicializacao da GPU.
//!
//! Erros *dentro* do loop de render (frame perdido, surface desatualizada) nao
//! aparecem aqui: eles sao recuperaveis e tratados em `Renderer::render`.
//! Estes sao os que impedem a engine de comecar.

#[derive(Debug, thiserror::Error)]
pub enum RenderError {
    #[error("falha ao criar a surface da janela: {0}")]
    CreateSurface(#[from] wgpu::CreateSurfaceError),

    #[error("nenhuma GPU compativel encontrada: {0}")]
    RequestAdapter(#[from] wgpu::RequestAdapterError),

    #[error("falha ao abrir o device da GPU: {0}")]
    RequestDevice(#[from] wgpu::RequestDeviceError),

    #[error("a GPU encontrada nao consegue apresentar nesta janela")]
    SurfaceUnsupported,
}
