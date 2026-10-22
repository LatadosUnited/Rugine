//! Os pipelines de desenho do mundo.
//!
//! Dois, porque topologia e propriedade de pipeline: a grade e lista de
//! linhas e as malhas sao lista de triangulos, e nao ha como alternar entre
//! as duas sem trocar de pipeline.

use crate::depth::DepthBuffer;
use crate::mesh::{Instancia, Vertex, VertexLinha};

pub struct Pipelines {
    pub malha: wgpu::RenderPipeline,
    pub grade: wgpu::RenderPipeline,
}

impl Pipelines {
    pub fn new(
        device: &wgpu::Device,
        formato: wgpu::TextureFormat,
        camera_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        // Os dois pipelines compartilham o layout: ambos leem so a camera no
        // grupo 0.
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("rugine::layout_mundo"),
            // `Option` por slot no wgpu 30: um `None` deixaria um indice de
            // grupo vago entre dois usados.
            bind_group_layouts: &[Some(camera_layout)],
            // Antigo `push_constant_ranges`. Zero: nao usamos immediates.
            immediate_size: 0,
        });

        let shader_malha = device.create_shader_module(wgpu::include_wgsl!("shaders/malha.wgsl"));
        let shader_grade = device.create_shader_module(wgpu::include_wgsl!("shaders/grade.wgsl"));

        let malha = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rugine::pipeline_malha"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader_malha,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                // Dois buffers: o da malha, por vertice, e o das instancias.
                buffers: &[Some(Vertex::LAYOUT), Some(Instancia::LAYOUT)],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_malha,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: formato,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                // Culling das faces de tras: os cubos sao fechados, entao
                // metade dos triangulos nunca e visivel.
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(DepthBuffer::state()),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let grade = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("rugine::pipeline_grade"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader_grade,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &[Some(VertexLinha::LAYOUT)],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader_grade,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: formato,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::LineList,
                // Linhas nao tem frente nem verso.
                cull_mode: None,
                ..Default::default()
            },
            // A grade participa do teste de profundidade — e o que faz um
            // cubo apoiado no chao esconder as linhas atras dele — mas nao
            // escreve nele, para nao brigar com geometria coplanar.
            depth_stencil: Some(wgpu::DepthStencilState {
                depth_write_enabled: Some(false),
                ..DepthBuffer::state()
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        Self { malha, grade }
    }
}
