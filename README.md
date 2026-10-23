# Rugine

Engine de jogos em Rust sobre wgpu, com editor próprio.

| Pasta | Conteúdo |
|---|---|
| [`engine/`](engine/) | A engine e o editor: workspace Cargo, crates e aplicações. Como rodar e as decisões de projeto estão no [README da engine](engine/README.md). |
| [`site/`](site/) | O site do projeto, em Astro: árvore de pesquisa, arquitetura, diário de desenvolvimento e decisões técnicas. |

## Rodar a engine

```bash
cd engine
cargo run -p editor
```

## Rodar o site

```bash
cd site
npm install
npm run dev
```

O site é estático e sai de `npm run build` em `site/dist/`. Ele é gerado a
partir dos dados em `site/src/data/`: a árvore de pesquisa é montada no build, e
o build **falha** quando um pré-requisito aponta para algo que não existe ou que
não está acima na árvore — é o que impede o mapa de mentir sobre o estado do
projeto.

As páginas de texto ficam em Markdown, em `site/src/content/diario/` e
`site/src/content/decisoes/`.
