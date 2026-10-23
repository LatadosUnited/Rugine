---
titulo: "O renderizador não depende do winit"
resumo: "A ponte com a janela usa os traits do raw-window-handle, o denominador comum entre winit, SDL e um canvas de browser."
area: render
escolha: "raw-window-handle como fronteira"
descartado: "rugine-render depender de winit"
ordem: 20
---

`rugine-render` cuida de surface, frames e alvo offscreen. Ele **não conhece
winit**: o que ele recebe é algo que implementa os traits do
`raw-window-handle`.

O ganho é que trocar o backend de janela não toca no renderizador. winit hoje,
SDL ou um canvas de browser depois — para o renderizador é o mesmo handle.

**A janela entra como `Arc<Window>`.** Isso dá uma `Surface` de lifetime
estático, que *mantém a janela viva* em vez de emprestá-la. É o que elimina a
briga de lifetime mais clássica de `wgpu` + `winit`, em que a surface empresta a
janela e nada mais pode movê-la depois.
