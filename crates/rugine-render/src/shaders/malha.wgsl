// Malha solida instanciada, com iluminacao difusa de uma luz direcional fixa.

struct Camera {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: Camera;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

// A matriz de modelo chega como quatro vec4: um atributo de vertice nao
// comporta uma mat4x4, entao ela e remontada aqui.
struct InstanceInput {
    @location(2) modelo_0: vec4<f32>,
    @location(3) modelo_1: vec4<f32>,
    @location(4) modelo_2: vec4<f32>,
    @location(5) modelo_3: vec4<f32>,
    @location(6) cor: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) cor: vec4<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn vs_main(vertice: VertexInput, instancia: InstanceInput) -> VertexOutput {
    let modelo = mat4x4<f32>(
        instancia.modelo_0,
        instancia.modelo_1,
        instancia.modelo_2,
        instancia.modelo_3,
    );

    var saida: VertexOutput;
    saida.clip_position = camera.view_proj * modelo * vec4<f32>(vertice.position, 1.0);

    // w = 0.0 descarta a translacao, que nao se aplica a direcoes.
    //
    // Isto esta correto para rotacao e escala uniforme. Com escala nao
    // uniforme a normal sai torta, e o certo seria a transposta da inversa —
    // que custa um uniform a mais por instancia. Fica para quando houver
    // material de verdade.
    saida.normal = normalize((modelo * vec4<f32>(vertice.normal, 0.0)).xyz);
    saida.cor = instancia.cor;
    return saida;
}

@fragment
fn fs_main(entrada: VertexOutput) -> @location(0) vec4<f32> {
    let direcao_luz = normalize(vec3<f32>(0.4, 0.9, 0.35));
    let difusa = max(dot(normalize(entrada.normal), direcao_luz), 0.0);

    // Ambiente alto de proposito: sem ele as faces viradas para longe da luz
    // ficam pretas e a silhueta some contra o fundo escuro do viewport.
    let ambiente = 0.38;
    let luz = ambiente + difusa * 0.72;

    return vec4<f32>(entrada.cor.rgb * luz, entrada.cor.a);
}
