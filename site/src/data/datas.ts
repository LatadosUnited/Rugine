/**
 * Datas do frontmatter sao dias, sem hora: `2026-10-22` vira meia-noite UTC.
 * Formatar no fuso local (UTC-3 aqui) jogava o dia para 21. Com o fuso fixo
 * em UTC, o dia que sai e o dia que foi escrito, em qualquer maquina de build.
 */
const LONGA = new Intl.DateTimeFormat("pt-BR", {
  day: "2-digit",
  month: "long",
  year: "numeric",
  timeZone: "UTC",
});

const CURTA = new Intl.DateTimeFormat("pt-BR", { timeZone: "UTC" });

export const dataLonga = (d: Date) => LONGA.format(d);
export const dataCurta = (d: Date) => CURTA.format(d);
