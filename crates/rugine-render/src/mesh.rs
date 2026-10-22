//! Malhas e instancias.

use rugine_core::math::Mat4;
use wgpu::util::DeviceExt as _;

/// Vertice de malha solida.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
}

impl Vertex {
    /// Os `@location` no WGSL tem que casar com estes indices; e um contrato
    /// que o Rust nao verifica, so o wgpu em runtime.
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
    };

    const fn new(position: [f32; 3], normal: [f32; 3]) -> Self {
        Self { position, normal }
    }
}

/// Vertice de linha, para a grade.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VertexLinha {
    pub position: [f32; 3],
    pub color: [f32; 3],
}

impl VertexLinha {
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3],
    };
}

/// Uma copia de uma malha, com sua transformacao e cor.
///
/// Instanciar em vez de um draw por entidade: a cena do editor tem dezenas de
/// entidades, e cada draw custa mais em CPU do que os poucos triangulos de um
/// cubo custam em GPU.
#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instancia {
    modelo: [[f32; 4]; 4],
    cor: [f32; 4],
}

impl Instancia {
    pub fn new(modelo: Mat4, cor: [f32; 4]) -> Self {
        Self {
            modelo: modelo.to_cols_array_2d(),
            cor,
        }
    }

    /// Uma `mat4x4` nao cabe num atributo de vertice: ela entra como quatro
    /// `vec4` em locations consecutivas, e o shader remonta. Por isso as
    /// locations 2 a 5 — 0 e 1 sao do vertice.
    pub const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        // `Instance`: avanca uma vez por instancia, nao por vertice.
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &wgpu::vertex_attr_array![
            2 => Float32x4,
            3 => Float32x4,
            4 => Float32x4,
            5 => Float32x4,
            6 => Float32x4
        ],
    };
}

/// Malha indexada na GPU.
pub struct Mesh {
    vertices: wgpu::Buffer,
    indices: wgpu::Buffer,
    count: u32,
}

impl Mesh {
    pub fn indices(&self) -> u32 {
        self.count
    }

    pub fn vertices(&self) -> &wgpu::Buffer {
        &self.vertices
    }

    pub fn buffer_indices(&self) -> &wgpu::Buffer {
        &self.indices
    }
}

/// Cubo unitario centrado na origem, com normais por face.
///
/// Vertices nao sao compartilhados entre faces de proposito: cada face tem a
/// propria normal, e compartilhar daria normais medias e um cubo com cara de
/// esfera mal feita.
pub fn cubo(device: &wgpu::Device) -> Mesh {
    const H: f32 = 0.5;

    // (normal, quatro cantos em ordem anti-horaria vista de fora)
    let faces: [([f32; 3], [[f32; 3]; 4]); 6] = [
        // frente (+Z)
        (
            [0.0, 0.0, 1.0],
            [[-H, -H, H], [H, -H, H], [H, H, H], [-H, H, H]],
        ),
        // tras (-Z)
        (
            [0.0, 0.0, -1.0],
            [[H, -H, -H], [-H, -H, -H], [-H, H, -H], [H, H, -H]],
        ),
        // direita (+X)
        (
            [1.0, 0.0, 0.0],
            [[H, -H, H], [H, -H, -H], [H, H, -H], [H, H, H]],
        ),
        // esquerda (-X)
        (
            [-1.0, 0.0, 0.0],
            [[-H, -H, -H], [-H, -H, H], [-H, H, H], [-H, H, -H]],
        ),
        // topo (+Y)
        (
            [0.0, 1.0, 0.0],
            [[-H, H, H], [H, H, H], [H, H, -H], [-H, H, -H]],
        ),
        // base (-Y)
        (
            [0.0, -1.0, 0.0],
            [[-H, -H, -H], [H, -H, -H], [H, -H, H], [-H, -H, H]],
        ),
    ];

    let mut vertices = Vec::with_capacity(24);
    let mut indices: Vec<u16> = Vec::with_capacity(36);

    for (normal, cantos) in faces {
        let base = vertices.len() as u16;
        for canto in cantos {
            vertices.push(Vertex::new(canto, normal));
        }
        // Dois triangulos por face, mantendo a ordem anti-horaria.
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    malha(device, "cubo", &vertices, &indices)
}

/// Grade no plano XZ, como lista de linhas.
///
/// Os eixos X e Z ganham cor propria: sem uma referencia de orientacao, uma
/// grade cinza uniforme nao diz para que lado a cena aponta.
pub fn grade(device: &wgpu::Device, meia_extensao: i32, passo: f32) -> Mesh {
    let cinza = [0.26, 0.26, 0.30];
    let vermelho = [0.45, 0.22, 0.24];
    let azul = [0.22, 0.28, 0.48];

    let limite = meia_extensao as f32 * passo;
    let mut vertices = Vec::new();
    let mut indices: Vec<u16> = Vec::new();

    let mut linha = |a: [f32; 3], b: [f32; 3], cor: [f32; 3], vs: &mut Vec<VertexLinha>| {
        let base = vs.len() as u16;
        vs.push(VertexLinha {
            position: a,
            color: cor,
        });
        vs.push(VertexLinha {
            position: b,
            color: cor,
        });
        indices.push(base);
        indices.push(base + 1);
    };

    for i in -meia_extensao..=meia_extensao {
        let d = i as f32 * passo;

        let cor_z = if i == 0 { vermelho } else { cinza };
        linha([-limite, 0.0, d], [limite, 0.0, d], cor_z, &mut vertices);

        let cor_x = if i == 0 { azul } else { cinza };
        linha([d, 0.0, -limite], [d, 0.0, limite], cor_x, &mut vertices);
    }

    let vertices_gpu = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("rugine::grade::vertices"),
        contents: bytemuck::cast_slice(&vertices),
        usage: wgpu::BufferUsages::VERTEX,
    });

    let indices_gpu = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("rugine::grade::indices"),
        contents: bytemuck::cast_slice(&indices),
        usage: wgpu::BufferUsages::INDEX,
    });

    Mesh {
        vertices: vertices_gpu,
        indices: indices_gpu,
        count: indices.len() as u32,
    }
}

fn malha(device: &wgpu::Device, nome: &str, vertices: &[Vertex], indices: &[u16]) -> Mesh {
    Mesh {
        vertices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("rugine::{nome}::vertices")),
            contents: bytemuck::cast_slice(vertices),
            usage: wgpu::BufferUsages::VERTEX,
        }),
        indices: device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(&format!("rugine::{nome}::indices")),
            contents: bytemuck::cast_slice(indices),
            usage: wgpu::BufferUsages::INDEX,
        }),
        count: indices.len() as u32,
    }
}
