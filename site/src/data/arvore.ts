import { AREAS, ORDEM_AREAS, type AreaId } from "./areas";
import { ERAS, eraDoNivel } from "./eras";
import { FILA, TECNOLOGIAS, type Tecnologia } from "./tecnologias";

export type Estado = "pesquisado" | "disponivel" | "bloqueado";

export const ROTULO_ESTADO: Record<Estado, string> = {
  pesquisado: "Pesquisado",
  disponivel: "Disponível",
  bloqueado: "Bloqueado",
};

export const POR_ID: Record<string, Tecnologia> = Object.fromEntries(
  TECNOLOGIAS.map((t) => [t.id, t]),
);

for (const t of TECNOLOGIAS) {
  for (const r of t.req) {
    if (!POR_ID[r]) throw new Error(`${t.id} requer ${r}, que nao existe`);
    if (POR_ID[r]!.nivel >= t.nivel) {
      // Uma aresta que nao desce quebra a leitura de cima para baixo: o que
      // depende de algo tem de ficar embaixo dele, sempre.
      throw new Error(`${t.id} (nivel ${t.nivel}) requer ${r}, que nao esta acima`);
    }
  }
}

export function estado(t: Tecnologia): Estado {
  if (t.pronto) return "pesquisado";
  return t.req.every((r) => POR_ID[r]!.pronto) ? "disponivel" : "bloqueado";
}

/** quem depende de cada tecnologia */
export const LIBERADOS: Record<string, string[]> = {};
for (const t of TECNOLOGIAS) {
  for (const r of t.req) (LIBERADOS[r] ||= []).push(t.id);
}

/** Todos os pre-requisitos, diretos e indiretos. */
export function ancestrais(id: string, acc = new Set<string>()): Set<string> {
  for (const r of POR_ID[id]!.req) {
    if (!acc.has(r)) {
      acc.add(r);
      ancestrais(r, acc);
    }
  }
  return acc;
}

// ---------------------------------------------------------------- geometria
//
// A arvore corre de cima para baixo: o nivel (profundidade na corrente de
// pre-requisitos) e a linha, e a area e a coluna. Era rotacionada ao contrario
// antes, e numa tela de celular isso significava rolar 2700 px de lado para ver
// o fim; na vertical a prancha fica mais alta que larga, e rolar para baixo e o
// gesto que todo mundo ja faz.

export const CARTAO_L = 164;
export const CARTAO_A = 62;
/** Duas pistas por area: nenhum nivel tem mais de duas da mesma area. */
export const PISTAS_POR_AREA = 2;
export const PISTA_L = 178;
export const NIVEL_A = 104;
/** Calha da esquerda, com o nome da era. */
export const CALHA = 108;
/** Faixa do topo, com o nome de cada area. */
export const TOPO = 46;

export const NIVEIS = Math.max(...TECNOLOGIAS.map((t) => t.nivel)) + 1;
export const COLUNAS = ORDEM_AREAS.length * PISTAS_POR_AREA;
export const LARGURA = CALHA + COLUNAS * PISTA_L;
export const ALTURA = TOPO + NIVEIS * NIVEL_A + 12;

/**
 * A pista de cada tecnologia: dentro do nivel, quem e da mesma area entra na
 * ordem da `trilha`. O `trilha` nao e a pista: ele so desempata, para que o
 * dado possa ganhar um item sem renumerar o resto.
 */
const PISTA = new Map<string, number>();
for (let nivel = 0; nivel < NIVEIS; nivel++) {
  ORDEM_AREAS.forEach((area, iArea) => {
    const noNivel = TECNOLOGIAS.filter((t) => t.nivel === nivel && t.area === area).sort(
      (a, b) => a.trilha - b.trilha,
    );
    if (noNivel.length > PISTAS_POR_AREA) {
      throw new Error(
        `nivel ${nivel} tem ${noNivel.length} de ${area}, e so cabem ${PISTAS_POR_AREA}`,
      );
    }
    noNivel.forEach((t, i) => PISTA.set(t.id, iArea * PISTAS_POR_AREA + i));
  });
}

export interface Ponto {
  x: number;
  y: number;
}

export function posicao(t: Tecnologia): Ponto {
  const pista = PISTA.get(t.id)!;
  return {
    x: CALHA + pista * PISTA_L + (PISTA_L - CARTAO_L) / 2,
    y: TOPO + t.nivel * NIVEL_A + (NIVEL_A - CARTAO_A) / 2,
  };
}

/** Onde a aresta sai do pai e entra no filho: por baixo e por cima. */
export function saida(t: Tecnologia): Ponto {
  const p = posicao(t);
  return { x: p.x + CARTAO_L / 2, y: p.y + CARTAO_A };
}
export function entrada(t: Tecnologia): Ponto {
  const p = posicao(t);
  return { x: p.x + CARTAO_L / 2, y: p.y };
}

export interface Aresta {
  de: string;
  para: string;
  d: string;
  estado: Estado;
}

export const ARESTAS: Aresta[] = TECNOLOGIAS.flatMap((t) =>
  t.req.map((r) => {
    const a = saida(POR_ID[r]!);
    const b = entrada(t);
    // Bezier vertical: as tangentes saem para baixo e chegam por cima, o que
    // deixa a direcao da dependencia legivel mesmo quando a aresta atravessa
    // meia prancha.
    const meio = (b.y - a.y) * 0.5;
    return {
      de: r,
      para: t.id,
      d: `M${a.x} ${a.y} C${a.x} ${a.y + meio}, ${b.x} ${b.y - meio}, ${b.x} ${b.y}`,
      estado: estado(t),
    };
  }),
);

export const FAIXAS_ERA = ERAS.map((era) => ({
  ...era,
  y: TOPO + era.de * NIVEL_A,
  altura: (era.ate - era.de + 1) * NIVEL_A,
}));

export const COLUNAS_AREA = ORDEM_AREAS.map((area, i) => ({
  id: area,
  ...AREAS[area],
  x: CALHA + i * PISTAS_POR_AREA * PISTA_L,
  largura: PISTAS_POR_AREA * PISTA_L,
}));

// ------------------------------------------------------------------ resumos

export const CONTAGENS: Record<Estado, number> = { pesquisado: 0, disponivel: 0, bloqueado: 0 };
for (const t of TECNOLOGIAS) CONTAGENS[estado(t)]++;

export const TOTAL = TECNOLOGIAS.length;
export const PERCENTUAL = (100 * CONTAGENS.pesquisado) / TOTAL;

export function porArea(area: AreaId) {
  return TECNOLOGIAS.filter((t) => t.area === area);
}

/** O que o cliente precisa para montar o painel, sem reenviar o arquivo todo. */
export interface DadosCliente {
  id: string;
  nome: string;
  area: AreaId;
  areaNome: string;
  cor: string;
  estado: Estado;
  nivel: number;
  era: string;
  desc: string;
  efeitos: string[];
  onde: string[];
  pronto?: string;
  req: string[];
  libera: string[];
  fila: number;
  x: number;
  y: number;
}

export const DADOS_CLIENTE: DadosCliente[] = TECNOLOGIAS.map((t) => {
  const era = eraDoNivel(t.nivel);
  const p = posicao(t);
  return {
    id: t.id,
    nome: t.nome,
    area: t.area,
    areaNome: AREAS[t.area].nome,
    cor: AREAS[t.area].cor,
    estado: estado(t),
    nivel: t.nivel,
    era: `${era.n}, ${era.nome}`,
    desc: t.desc,
    efeitos: t.efeitos,
    onde: t.onde ?? [],
    pronto: t.pronto,
    req: t.req,
    libera: LIBERADOS[t.id] ?? [],
    fila: FILA.indexOf(t.id),
    x: p.x,
    y: p.y,
  };
});

/** O primeiro da fila que ja da para pesquisar: por onde a prancha abre. */
export const FOCO =
  FILA.find((id) => estado(POR_ID[id]!) === "disponivel") ?? TECNOLOGIAS[0]!.id;

export { FILA, TECNOLOGIAS, AREAS, ORDEM_AREAS, ERAS };
export type { Tecnologia, AreaId };
