# Rugine

Engine de jogos em Rust sobre [wgpu](https://wgpu.rs) — Vulkan, DX12, Metal e
WebGPU por uma única API.

**Estado: Marco 3.** Editor com painéis encaixáveis, grafo de cena, câmera
orbital e profundidade. Cada entidade é desenhada como um cubo na sua matriz de
mundo, então editar uma transform no Inspector move o objeto no viewport. Ainda
não dá para fazer um jogo: não há malhas de verdade, materiais, física nem
scripts.

## Rodar

```bash
cargo run -p editor
```

O sandbox (jogo sem editor, cena direto na janela) continua existindo:

```bash
cargo run -p sandbox
```

| Variável | Efeito |
|---|---|
| `RUST_LOG=debug` | Logs do wgpu — primeira ferramenta quando um frame sai errado |
| `RUST_LOG=info,rugine_ui=debug` | Quantas primitivas a UI gera por frame |
| `WGPU_BACKEND=vulkan\|dx12\|gl` | Força o backend, sobrepondo o padrão |
| `RUST_LOG=info,rugine_render=debug` | Quantas instâncias vão para a GPU por frame |
| `RUGINE_CAPTURA=saida.png` | Salva o viewport em PNG e encerra |
| `RUGINE_CAPTURA_FRAME=90` | Em que frame capturar (padrão 30) |
| `RUGINE_VSYNC=0` | Desliga a sincronização, para medir o custo real do frame |
| `RUGINE_EXIT_AFTER=3` | Sandbox encerra após N segundos (smoke test / CI) |

### Vsync

Ligado por padrão: sem ele a GPU desenha frames que ninguém vê. Desligar serve
para **medir** — com vsync o contador de FPS mede a taxa do monitor, não o custo
do frame. No editor, menu Ver › Vsync; em qualquer binário, `RUGINE_VSYNC=0`.

Medido no sandbox (81 cubos, Intel HD 530, build de debug): 60 fps com vsync,
300–530 fps sem. Ou seja, o vsync estava escondendo uma folga de 5× a 8×.

O controle fica cinza quando a GPU ou o driver não oferecem nenhum modo sem
sincronização para aquela janela — o `AutoNoVsync` do wgpu cairia para Fifo e a
opção não faria efeito nenhum. Melhor um controle desabilitado com o motivo no
tooltip do que um que muda de estado sem mudar o comportamento.

### Navegação no viewport

Segue o esquema do Unity:

| Gesto | Ação |
|---|---|
| Botão direito | Olhar em volta, sem sair do lugar |
| Direito + `WASD` / `QE` | Voar; `Shift` acelera 3× |
| `Alt` + botão esquerdo | Orbitar em torno do pivô |
| Botão do meio | Deslocar |
| Roda | Aproximar |
| `F` | Enquadrar a seleção |
| Botão esquerdo | Selecionar o objeto sob o cursor |
| Esquerdo sobre uma alça | Arrastar para mover ou escalar no eixo |
| `Q`/`W`/`E`/`R` | Trocar de ferramenta (fora do modo de voo) |

Com `Snap` ligado na barra do viewport, o arrasto da alça anda em degraus do
tamanho configurado.

## Ver o que foi desenhado

"Nenhum erro de validação" não é o mesmo que "apareceu na tela" — a diferença
já custou caro aqui. `RUGINE_CAPTURA` lê o viewport de volta e grava um PNG:

```bash
RUGINE_CAPTURA=/tmp/viewport.png cargo run -p editor
```

O log também reporta quantos pixels fortemente coloridos a captura tem, o que
responde "algo foi desenhado?" sem abrir a imagem — útil para bissecionar.

A leitura reordena os canais quando a surface é BGRA. Sem isso a captura mente
justamente sobre a cor, que costuma ser o que se quer verificar.

## Checar o Linux sem CI

`clippy` e `check` não linkam, então dá para validar o alvo Linux de dentro do
Windows, sem toolchain cruzada e sem esperar o GitHub:

```bash
rustup target add x86_64-unknown-linux-gnu
cargo clippy --workspace --all-targets --target x86_64-unknown-linux-gnu -- -D warnings
```

Vale a pena: o primeiro erro que isso pega foi um `mut` usado só dentro de um
bloco `#[cfg(target_os = "windows")]`. No Linux o bloco some, o `mut` fica sem
uso e `-D warnings` o transforma em erro — algo que não aparece compilando no
Windows.

O CI no GitHub Actions faz o mesmo nas duas plataformas, mas só em pull request
ou acionado à mão: compilar wgpu, naga e egui duas vezes custa minutos demais
para rodar a cada commit.

## Arquitetura

```
rugine-core     matemática e tempo, não depende de nada da engine
      ↑
rugine-scene    entidades, hierarquia, seleção, ajustes de cena
rugine-render   wgpu: surface, frames, alvo offscreen. NÃO conhece winit
      ↑
rugine-ui       egui: input, tesselação, pintura
      ↑
rugine-app      janela, loop, o trait App
      ↑
apps/editor     os painéis
apps/sandbox    jogo de exemplo, sem editor
```

As dependências são declaradas uma só vez, em `[workspace.dependencies]` na
raiz. Com vários crates, versões espalhadas pelos manifestos é o caminho mais
curto para duas versões de wgpu no mesmo binário.

### Decisões que valem saber

**`rugine-render` não depende do `winit`.** A ponte com a janela usa os traits
do `raw-window-handle`, que é o denominador comum entre winit, SDL e um canvas
de browser. Trocar o backend de janela não toca no renderer.

**A janela entra como `Arc<Window>`.** Isso dá uma `Surface` de lifetime
estático, que *mantém a janela viva* em vez de emprestá-la — e elimina a briga
de lifetime mais clássica de wgpu + winit.

**A barra de título é desenhada pela engine, não pelo sistema.** A janela sobe
sem decoração e o editor desenha título, botões e alças de redimensionar. O
caminho nativo foi descartado de propósito: pintar a barra do Windows exige
`DWMWA_CAPTION_COLOR`, que só existe no Windows 11 e não tem equivalente no
Linux. `drag_window` e `drag_resize_window` da winit funcionam em Windows, X11 e
Wayland — uma implementação para os três.

No **macOS** é diferente: lá `drag_resize_window` retorna sempre
`NotSupported`, e uma janela sem decoração ficaria *sem nenhuma forma de ser
redimensionada*. Por isso `Decoracao::recomendada()` devolve `Sistema` no macOS.
Não é preferência estética, é requisito.

**Três modos de UI, explícitos.** `UiMode::Disabled` manda a cena direto para a
janela; `Overlay` desenha a cena e põe a interface por cima (HUD); `Editor` manda
a cena para uma textura e deixa a interface dona da janela. Sem essa distinção,
a interface limparia a tela e apagaria a cena num jogo sem editor.

**A janela é vista em dois formatos ao mesmo tempo.** A cena escreve numa view
sRGB; a interface, numa view linear da *mesma* textura, via `view_formats`. A
egui já entrega cores com gama aplicada, e escrevê-las num alvo sRGB aplicaria a
correção duas vezes. Reinterpretar a textura evita escolher entre cena certa e
UI certa.

**O tamanho do viewport chega com um frame de atraso.** A UI imediata só sabe
quanto espaço o painel tem enquanto o constrói, e aí a cena do frame já foi
desenhada. Um frame ao arrastar um divisor é imperceptível; recriar a textura no
meio do frame não é opção.

**`EntityId` carrega geração.** Sem ela, remover uma entidade e criar outra
reaproveitaria o slot e um id antigo passaria a apontar para a entidade errada
em silêncio. Há teste para isso.

**O NDC está fixado em `rugine_core::math::camera`.** Mundo right-handed Y-up
(como Blender e glTF) e profundidade `0.0..1.0` (a do wgpu). Convenção errada de
NDC não dá erro de compilação, dá depth buffer invertido. Há testes.

**Padrão DX12 no Windows.** Numa Intel HD 530, o driver Vulkan dá
`ACCESS_VIOLATION` *dentro do driver* ao criar o device — um segfault, que
nenhum tratamento de erro em Rust captura. `WGPU_BACKEND=vulkan` continua
disponível.

**`Renderer::render` não retorna erro.** Perder um frame é normal (janela
redimensionando, compositor ocupado) e a resposta certa é pular o frame, não
propagar falha até o jogo.

## Armadilhas da egui 0.37

Versão recente e com mudanças grandes; quem for mexer na UI vai esbarrar nelas:

- `SidePanel`/`TopBottomPanel` **não existem mais**. É `egui::Panel::left/right/top/bottom`,
  e painéis pendem de uma `Ui` (a raiz), não do `Context`.
- `default_fonts` **não é mais feature padrão**. Sem ela a egui sobe sem fonte
  nenhuma e nenhum texto é desenhado.
- `TexturesDelta` tem `Drop` que entra em **pânico** se os deltas não forem
  consumidos. Aplicar não basta: é preciso `clear()`.
- `Context::run_pass` recebe `FnMut`, não `FnOnce` — a egui pode rodar mais de
  um passe por frame, e o código de UI precisa tolerar isso.
- `wants_keyboard_input` virou `egui_wants_keyboard_input`.

## O que falta

Os painéis existem e editam estado real, mas vários botões são forma sem função,
e isso está dito na própria interface. O que falta, em ordem de dependência:

1. **Gizmo de rotação** — mover e escalar existem; girar precisa de
   interseção com anéis, que é outra conta.
2. **Teste de pixel headless** — o `RenderTarget` já tem `COPY_SRC`, então ler
   de volta e verificar é barato. Exige GPU, então não roda nos runners do CI.
3. **Malhas de verdade** — glTF em vez de um cubo para tudo. Dá conteúdo à aba
   Projeto.
4. **Normais com escala não uniforme** — o shader usa a matriz de modelo direto,
   o que distorce a normal quando a escala não é uniforme. O certo é a
   transposta da inversa, e isso custa um uniform por instância.
5. **Iluminação e neblina** — liga os controles do painel Cena, que hoje editam
   dados que ninguém lê. A luz atual é uma direção fixa no shader.
6. **Histórico de comandos** — desfazer/refazer exige que toda mutação de cena
   passe por comandos. Decidir antes do ECS, não depois.
7. **ECS** — provavelmente `bevy_ecs`, não um próprio. O `rugine-scene` atual é
   pequeno de propósito, para ser descartável.

## Licença

MIT OR Apache-2.0
