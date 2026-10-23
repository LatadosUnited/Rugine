---
titulo: "Componentes e arquivo .cena"
marco: 4
data: 2026-10-23
resumo: "A entidade deixou de ser só nome e transform: ganhou uma lista de componentes, um registro de tipos e um arquivo em disco. É o marco que abre a porta para desfazer, duplicar e luz."
commits: ["75ef89d", "7572dde"]
tecnologias: ["componentes", "colisor", "registro", "arquivo", "inspector", "contorno"]
---

## O que ficou de pé

- **Componentes.** Cada entidade guarda uma lista de `Box<dyn Componente>`, na
  ordem em que o Inspector os mostra. A lista é privada: toda mudança passa por
  métodos da `Scene`, que é onde o desfazer vai entrar.
- **Malha e Colisor.** A Malha ainda é sempre o cubo, com cor. O Colisor é caixa
  ou esfera no espaço local, com centro deslocável — e só é desenhado: ainda
  nada colide.
- **Registro de tipos.** Nome → criar, salvar e carregar. O mesmo catálogo
  alimenta o menu "Adicionar componente" e o arquivo.
- **Arquivo `.cena`.** JSON legível com `versao`, hierarquia, transforms,
  componentes e configurações de cena.
- **Inspector de componentes.** Um bloco recolhível por componente, com remover,
  e as linhas caindo em duas quando o painel afina. Há teste para o painel a
  partir de 230 px, e ele roda sem GPU.

## Por que o arquivo é JSON

RON seria a escolha óbvia num projeto Rust, e foi a primeira. O problema é que
cada componente vai pelo nome, com os dados num valor genérico — e o
`ron::Value` não representa enums. O `FormaColisor` não sobreviveria à ida e
volta.

As outras regras do formato saíram de pensar em quem edita o arquivo à mão e em
quem abre um arquivo antigo:

- Campo ausente usa o padrão, então um arquivo escrito à mão só precisa do
  essencial.
- Componente desconhecido ou com dados inválidos vira aviso: o resto da cena
  carrega.
- `versao` maior que a conhecida é recusada com um erro que diz isso, em vez de
  um erro confuso de campo inesperado.
- Salvar grava num `.tmp` ao lado e troca no fim. Se o processo morrer no meio,
  a cena anterior continua inteira.

## O que isto libera

O registro é pré-requisito de três coisas da próxima era: **desfazer** (recriar
um componente removido), **duplicar entidade** (copiar componentes sem código
por tipo) e o **componente Luz**, que entra no arquivo sem uma linha nova de
serialização.
