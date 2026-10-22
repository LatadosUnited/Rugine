//! Uniform de camera.
//!
//! Uma matriz view-projection num uniform buffer, no bind group 0. Todo
//! pipeline que desenha no mundo usa este mesmo grupo — e a razao de ele ser
//! o grupo 0: o que muda com menos frequencia fica nos grupos baixos, para
//! nao ser reenviado a cada troca de material.

use rugine_core::math::Mat4;

/// O que vai para a GPU.
///
/// `repr(C)` e obrigatorio: o layout tem que casar com o `struct` do WGSL, e
/// sem ele o Rust pode reordenar os campos.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

/// Buffer, layout e bind group da camera.
pub struct CameraBinding {
    buffer: wgpu::Buffer,
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
}

impl CameraBinding {
    pub fn new(device: &wgpu::Device) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("rugine::camera"),
            size: size_of::<CameraUniform>() as wgpu::BufferAddress,
            // COPY_DST porque a matriz e reescrita a cada frame via
            // `Queue::write_buffer`, sem recriar o buffer.
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("rugine::camera_layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                // Só o estagio de vertice le a camera; declarar o fragmento
                // tambem custaria validacao a toa.
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("rugine::camera_bind_group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: buffer.as_entire_binding(),
            }],
        });

        Self {
            buffer,
            layout,
            bind_group,
        }
    }

    /// Envia a matriz para a GPU.
    ///
    /// `to_cols_array_2d` e nao `to_cols_array`: a `glam` e o WGSL concordam
    /// em matrizes por coluna, entao a conversao e direta — mas o array
    /// aninhado deixa explicito que sao quatro colunas, nao dezesseis floats
    /// soltos em ordem a adivinhar.
    pub fn atualizar(&self, queue: &wgpu::Queue, view_proj: Mat4) {
        let uniform = CameraUniform {
            view_proj: view_proj.to_cols_array_2d(),
        };
        queue.write_buffer(&self.buffer, 0, bytemuck::bytes_of(&uniform));
    }

    pub fn layout(&self) -> &wgpu::BindGroupLayout {
        &self.layout
    }

    pub fn bind_group(&self) -> &wgpu::BindGroup {
        &self.bind_group
    }
}
