//! O triangulo do Marco 1.
//!
//! Isto e andaime: existe para provar que instance -> adapter -> device ->
//! surface -> pipeline -> frame esta ligado de ponta a ponta. O Marco 2
//! substitui por um sistema de meshes e passes de verdade.

use wgpu::util::DeviceExt;

use crate::vertex::Vertex;

/// Triangulo em espaco de clip, uma cor primaria por vertice, para que a
/// interpolacao do rasterizador fique visivel (se as tres cores aparecerem
/// misturadas, o estagio de vertex esta passando dados corretamente).
const VERTICES: &[Vertex] = &[
    Vertex::new([0.0, 0.6, 0.0], [1.0, 0.1, 0.2]),
    Vertex::new([-0.6, -0.4, 0.0], [0.1, 1.0, 0.3]),
    Vertex::new([0.6, -0.4, 0.0], [0.2, 0.3, 1.0]),
];

pub(crate) struct HelloTriangle {
    pipeline: wgpu::RenderPipeline,
    vertices: wgpu::Buffer,
    count: u32,
}

impl HelloTriangle {
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        // include_wgsl! embute o shader no binario em tempo de compilacao, e o
        // include_str! por tras dele faz o cargo reconstruir quando o .wgsl
        // muda. Carregar de disco em runtime (para hot-reload) e assunto de um
        // marco posterior.
        let shader = device.create_shader_module(wgpu::include_wgsl!("hello_triangle.wgsl"));

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rugine::hello_triangle"),
            // Sem bind groups ainda, entao deixamos o wgpu derivar o layout.
            // A partir do momento em que houver uniforms, isto passa a ser um
            // PipelineLayout explicito.
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(Vertex::LAYOUT)],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                // O formato tem que ser o da surface: um pipeline e compilado
                // contra um formato de alvo especifico.
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                // Back-face culling exige consistencia na ordem dos vertices;
                // os nossos estao em counter-clockwise.
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let vertices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("rugine::hello_triangle::vertices"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        Self {
            pipeline,
            vertices,
            count: VERTICES.len() as u32,
        }
    }

    pub(crate) fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        pass.set_pipeline(&self.pipeline);
        pass.set_vertex_buffer(0, self.vertices.slice(..));
        pass.draw(0..self.count, 0..1);
    }
}
