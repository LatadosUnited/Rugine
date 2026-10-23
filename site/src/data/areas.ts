/** As tres frentes de trabalho. A cor identifica a area em todo o site. */
export const AREAS = {
  nucleo: { nome: "Núcleo", cor: "#cf7f4f", resumo: "Dados da cena, componentes, arquivo" },
  render: { nome: "Renderização", cor: "#5fa7c2", resumo: "wgpu: frames, câmera, instâncias" },
  editor: { nome: "Editor", cor: "#a688cf", resumo: "Painéis, seleção, gizmos, ferramentas" },
} as const;

export type AreaId = keyof typeof AREAS;

export const ORDEM_AREAS: AreaId[] = ["nucleo", "render", "editor"];

/** Um glifo por area, no lugar das ilustracoes que um jogo teria. */
export const GLIFOS: Record<AreaId, string> = {
  nucleo:
    '<path d="M12 2.5l8 4.6v9.8l-8 4.6-8-4.6V7.1z" fill="none" stroke="currentColor" stroke-width="1.6"/><circle cx="12" cy="12" r="3.2" fill="currentColor"/>',
  render:
    '<path d="M12 3L21 19H3z" fill="none" stroke="currentColor" stroke-width="1.6"/><path d="M12 9.5l3.6 6.5H8.4z" fill="currentColor"/>',
  editor:
    '<circle cx="12" cy="12" r="8.5" fill="none" stroke="currentColor" stroke-width="1.6"/><path d="M12 5.5l2.2 6.5L12 18.5 9.8 12z" fill="currentColor"/>',
};
