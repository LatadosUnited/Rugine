---
titulo: "Mundo 3D: câmera, profundidade e uma instância por entidade"
marco: 2
data: 2026-10-22
resumo: "O desenho deixou de ser um triângulo e passou a ser a cena: profundidade, uma instância por entidade e navegação no esquema do Unity. O vsync saiu do caminho para a medição começar a valer."
commits: ["4b8401d", "6faef61", "9c1d3b0"]
tecnologias: ["profundidade", "instancias", "navegacao", "vsync"]
---

Com a interface pronta, o que faltava era ter o que olhar dentro dela.

## O que ficou de pé

- **Câmera e profundidade.** Câmera por uniform, depth buffer `Depth32Float`,
  mundo right-handed com Y para cima (como Blender e glTF) e profundidade
  `0.0..1.0` (a do `wgpu`). Convenção errada de NDC não dá erro de compilação:
  dá depth buffer invertido. Por isso ela está fixada num lugar só,
  `rugine_core::math::camera`, com testes.
- **Instâncias.** Uma instância por entidade numa única chamada de desenho, com
  a matriz de modelo indo como quatro `vec4`. É o que faz editar a transform no
  Inspector mover o objeto no viewport.
- **Navegação estilo Unity.** Botão direito olha em volta, `WASD` voa, `Alt`
  orbita, o botão do meio desloca e `F` enquadra a seleção. Olhar move o pivô;
  orbitar move a câmera em torno dele.
- **Vsync alternável.** Ligado por padrão, desligável em execução.

## O que o vsync escondia

Com vsync, o contador de FPS mede a taxa do monitor, não o custo do frame. No
sandbox (81 cubos, Intel HD 530, build de debug) a medição foi:

| Vsync | FPS |
|---|---|
| Ligado | 60 |
| Desligado | 300–530 |

Ou seja, havia uma folga de 5× a 8× que o número de 60 fps não mostrava. O
controle fica cinza quando o driver não oferece nenhum modo sem sincronização
para aquela janela — melhor um controle desabilitado com o motivo no tooltip do
que um que troca de estado sem trocar de comportamento.
