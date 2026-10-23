/**
 * As crates e o que cada uma pode conhecer. `camada` e a altura no diagrama:
 * uma crate so depende de camadas abaixo da sua, e isso e o que mantem o
 * renderizador ignorante de winit e o core ignorante de todo o resto.
 */
export interface Caixa {
  id: string;
  nome: string;
  camada: number;
  /** Posicao dentro da camada, da esquerda para a direita. */
  coluna: number;
  papel: string;
  /** Crates da propria engine de que depende. */
  usa: string[];
  /** Dependencias externas que definem a crate. */
  externas: string[];
  /** O que ela deliberadamente NAO conhece. */
  naoConhece?: string;
  app?: boolean;
}

export const CAIXAS: Caixa[] = [
  {
    id: "core",
    nome: "rugine-core",
    camada: 0,
    coluna: 1,
    papel: "Matemática, tempo e raios. A convenção de NDC mora aqui, com testes.",
    usa: [],
    externas: ["glam"],
    naoConhece: "Nada da engine. É o chão.",
  },
  {
    id: "render",
    nome: "rugine-render",
    camada: 1,
    coluna: 0,
    papel: "wgpu: surface, frames, alvo offscreen, instâncias, profundidade, captura.",
    usa: ["core"],
    externas: ["wgpu", "bytemuck", "raw-window-handle"],
    naoConhece: "winit. A ponte com a janela é raw-window-handle.",
  },
  {
    id: "scene",
    nome: "rugine-scene",
    camada: 1,
    coluna: 2,
    papel: "Entidades, hierarquia, componentes, registro de tipos, arquivo .cena e ajustes de cena.",
    usa: ["core"],
    externas: ["serde", "serde_json", "glam"],
    naoConhece: "wgpu e egui. A cena não sabe como é desenhada.",
  },
  {
    id: "ui",
    nome: "rugine-ui",
    camada: 2,
    coluna: 0,
    papel: "egui: input, tesselação e pintura, mais o tema escuro e os ícones.",
    usa: ["render"],
    externas: ["egui", "egui-wgpu", "egui-winit"],
  },
  {
    id: "app",
    nome: "rugine-app",
    camada: 3,
    coluna: 1,
    papel: "Janela, loop de eventos e o trait App. Decide os três modos de UI.",
    usa: ["core", "render", "ui"],
    externas: ["winit", "pollster"],
  },
  {
    id: "editor",
    nome: "apps/editor",
    camada: 4,
    coluna: 0,
    papel: "Os painéis, os gizmos, o picking, o Inspector e o salvar/abrir.",
    usa: ["app", "scene", "render", "core"],
    externas: ["egui_dock", "rfd", "png"],
    app: true,
  },
  {
    id: "sandbox",
    nome: "apps/sandbox",
    camada: 4,
    coluna: 2,
    papel: "Jogo de exemplo: cena direto na janela, sem editor. Smoke test da engine.",
    usa: ["app"],
    externas: [],
    app: true,
  },
];

export const POR_CAIXA = Object.fromEntries(CAIXAS.map((c) => [c.id, c]));

for (const c of CAIXAS) {
  for (const u of c.usa) {
    const alvo = POR_CAIXA[u];
    if (!alvo) throw new Error(`${c.id} usa ${u}, que nao existe`);
    if (alvo.camada >= c.camada) {
      throw new Error(`${c.id} usa ${u}, que nao esta numa camada abaixo`);
    }
  }
}
