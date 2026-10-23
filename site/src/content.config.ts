import { defineCollection } from "astro:content";
import { glob } from "astro/loaders";
// Direto do zod: o reexport `z` de `astro:content` esta depreciado na Astro 7.
import * as z from "zod";

/**
 * Duas colecoes de texto. O `diario` conta o que aconteceu, em ordem; as
 * `decisoes` explicam uma escolha cada, e nao mudam de lugar quando o proximo
 * marco sai. Separa-las evita o diario virar o lugar onde se procura "por que
 * JSON e nao RON".
 */
const diario = defineCollection({
  loader: glob({ base: "./src/content/diario", pattern: "**/*.md" }),
  schema: z.object({
    titulo: z.string(),
    marco: z.number(),
    data: z.coerce.date(),
    resumo: z.string(),
    commits: z.array(z.string()).default([]),
    /** ids de tecnologias que este marco trouxe, para ligar na arvore. */
    tecnologias: z.array(z.string()).default([]),
  }),
});

const decisoes = defineCollection({
  loader: glob({ base: "./src/content/decisoes", pattern: "**/*.md" }),
  schema: z.object({
    titulo: z.string(),
    resumo: z.string(),
    area: z.enum(["nucleo", "render", "editor", "projeto"]),
    /** O que foi escolhido, em duas ou tres palavras. */
    escolha: z.string(),
    /** O que foi recusado, para a decisao ter os dois lados. */
    descartado: z.string(),
    ordem: z.number().default(50),
  }),
});

export const collections = { diario, decisoes };
