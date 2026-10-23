---
titulo: "Fundação: janela, grafo de cena e editor encaixável"
marco: 1
data: 2026-10-22
resumo: "O workspace, a primeira janela com wgpu, o grafo de cena e os painéis do editor. No fim do dia já havia uma interface de verdade — e nada para desenhar nela."
commits: ["a8c6a02", "d3a0edc", "88cadb4", "761337c"]
tecnologias: ["workspace", "janela", "grafo", "ui", "paineis", "barra", "viewport"]
---

O primeiro marco não é um jogo nem um renderizador: é o esqueleto que permite
todos os outros. Cinco crates (`core`, `render`, `scene`, `ui`, `app`), duas
aplicações (`editor` e `sandbox`) e um lugar só para declarar versões de
dependência.

## O que ficou de pé

- **Janela e triângulo.** `winit` abre a janela, `wgpu` desenha. No Windows o
  padrão virou DX12 na primeira hora de trabalho: na Intel HD 530 desta máquina
  o driver Vulkan dá `ACCESS_VIOLATION` *dentro do driver* ao criar o device, e
  um segfault no driver nenhum `Result` de Rust captura.
- **Grafo de cena.** Entidades numa arena, com pai e filhos, e a matriz de mundo
  composta pela cadeia de pais.
- **Interface do editor.** Menu, Hierarquia, Inspector, Cena, Projeto e Scripts
  em `egui`, com painéis que viram aba um do outro.
- **Viewport em textura.** A cena passou a ser desenhada numa textura própria,
  o que transformou o viewport num painel como os outros.

## O que custou tempo

**A UI subiu branca.** Seguir o tema do sistema parecia correto e deixava o
editor claro numa máquina de tema claro. O tema escuro é forçado.

**Ícones viravam quadrados.** Glifos de fonte não estavam lá; os ícones passaram
a ser desenhados com primitivas.

**O CI no push era insustentável.** Compilar `wgpu`, `naga` e `egui` em duas
plataformas levava minutos demais para rodar a cada commit. O CI ficou só em
pull request, e a checagem rápida do Linux virou um comando local:

```bash
rustup target add x86_64-unknown-linux-gnu
cargo clippy --workspace --all-targets --target x86_64-unknown-linux-gnu -- -D warnings
```

O primeiro erro que isso achou foi um `mut` usado só dentro de um bloco
`#[cfg(target_os = "windows")]`: no Linux o bloco desaparece, o `mut` fica sem
uso, e `-D warnings` transforma o aviso em erro. Compilando no Windows isso
nunca apareceria.
