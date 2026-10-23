# Site da Rugine

Site estático em [Astro](https://astro.build) 7 com TypeScript. Nenhum framework
de UI: as páginas saem prontas do build, e o único JavaScript que vai para o
navegador é o que move a árvore de pesquisa.

```bash
npm install
npm run dev      # http://localhost:4321/Rugine/
npm run build    # sai em dist/
npm run check    # tipos das páginas e dos dados
```

## Onde fica cada coisa

| Caminho | Conteúdo |
|---|---|
| `src/data/tecnologias.ts` | As peças da árvore: área, nível, pré-requisitos, efeitos, commit e arquivos. É aqui que se mexe para a árvore mudar. |
| `src/data/arvore.ts` | Estado derivado e geometria: quem está pesquisado, quem libera quem, e onde cada cartão cai na prancha. |
| `src/data/crates.ts` | As crates e as camadas, que viram o diagrama da página Arquitetura. |
| `src/content/diario/` | Um Markdown por marco. |
| `src/content/decisoes/` | Um Markdown por decisão técnica. |
| `src/components/Arvore.astro` | A prancha: HTML montado no build e o script de mover, aproximar, filtrar e procurar. |

## O build é a validação

`src/data/arvore.ts` falha o build quando:

- um pré-requisito aponta para um id que não existe;
- um pré-requisito **não está acima** na árvore, o que quebraria a leitura de
  cima para baixo;
- um nível acumula mais peças de uma área do que cabem nas pistas.

É de propósito: um mapa que mente sobre o estado do projeto é pior que nenhum
mapa, e esses três erros são os que passariam despercebidos numa revisão.

## A prancha corre para baixo

A primeira versão corria para o lado, e no celular isso significava arrastar
quase 2 800 px para chegar ao fim. Agora o nível (profundidade na corrente de
pré-requisitos) é a **linha** e a área é a **coluna**: a prancha fica mais alta
que larga, e rolar para baixo é o gesto que todo mundo já faz.

Mover: arrastar com o mouse, roda para rolar, `Ctrl` + roda (ou pinça no
trackpad) para aproximar, setas quando a prancha tem o foco, e os botões de era
para saltar. `Ajustar` faz a prancha caber na largura.

## Publicar

`astro.config.mjs` fixa `site` e `base` para um GitHub Pages de projeto
(`/Rugine/`). Por isso todo link interno passa por `caminho()`, em
`src/data/caminho.ts` — um `href="/arvore/"` escrito à mão funcionaria no dev e
quebraria no Pages.

O deploy é o workflow [`pages.yml`](../.github/workflows/pages.yml): todo push no
`master` que mexe em `site/` builda e publica em
<https://latadosunited.github.io/Rugine/>. O build é também a validação da
árvore, então um dado quebrado derruba o deploy em vez de ir para o ar.

Para servir o site fora do Pages, na raiz de um domínio (um túnel, por exemplo),
builde com `SITE_BASE=/`. No Git Bash, use `MSYS_NO_PATHCONV=1 SITE_BASE=/ npm
run build`: sem isso o `/` vira `C:/Program Files/Git/` e todos os links saem
quebrados.
