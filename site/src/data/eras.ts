/**
 * Cada era cobre um intervalo de niveis. Nivel e profundidade na arvore: uma
 * corrente de pre-requisitos dentro da mesma era gasta um nivel por elo.
 */
export interface Era {
  n: string;
  nome: string;
  data: string;
  de: number;
  ate: number;
  /** Era que ainda nao aconteceu: nada dentro dela esta pesquisado. */
  futura?: boolean;
  resumo: string;
}

export const ERAS: Era[] = [
  {
    n: "Era I",
    nome: "Fundação",
    data: "Marco 1 · 22 out",
    de: 0,
    ate: 1,
    resumo: "Workspace, janela, grafo de cena e a interface do editor.",
  },
  {
    n: "Era II",
    nome: "Mundo 3D",
    data: "Marco 2 · 22 out",
    de: 2,
    ate: 3,
    resumo: "Câmera, profundidade, uma instância por entidade e navegação.",
  },
  {
    n: "Era III",
    nome: "Manipulação",
    data: "Marco 3 · 22 out",
    de: 4,
    ate: 6,
    resumo: "Picking, gizmos de mover, escalar e rotacionar.",
  },
  {
    n: "Era IV",
    nome: "Componentes",
    data: "Marco 4 · 22–23 out",
    de: 7,
    ate: 9,
    resumo: "Componentes, registro de tipos e cenas salvas em arquivo.",
  },
  {
    n: "Era V",
    nome: "Próximo",
    data: "a pesquisar",
    de: 10,
    ate: 11,
    futura: true,
    resumo: "Desfazer, luz, iluminação de verdade e malhas de glTF.",
  },
  {
    n: "Era VI",
    nome: "Horizonte",
    data: "depende do resto",
    de: 12,
    ate: 13,
    futura: true,
    resumo: "Modo Play, física, scripts, sombras e pós-processamento.",
  },
];

export function eraDoNivel(nivel: number): Era {
  const e = ERAS.find((era) => nivel >= era.de && nivel <= era.ate);
  if (!e) throw new Error(`nivel ${nivel} fora de toda era`);
  return e;
}
