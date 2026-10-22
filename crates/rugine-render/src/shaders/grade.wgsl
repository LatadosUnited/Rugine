// Grade do chao: linhas com cor por vertice, sem iluminacao.

struct Camera {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) cor: vec3<f32>,
}

@vertex
fn vs_main(
    @location(0) position: vec3<f32>,
    @location(1) cor: vec3<f32>,
) -> VertexOutput {
    var saida: VertexOutput;
    // A grade vive em coordenadas de mundo, sem matriz de modelo.
    saida.clip_position = camera.view_proj * vec4<f32>(position, 1.0);
    saida.cor = cor;
    return saida;
}

@fragment
fn fs_main(entrada: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(entrada.cor, 1.0);
}
