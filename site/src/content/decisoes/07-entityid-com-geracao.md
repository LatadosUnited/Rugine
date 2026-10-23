---
titulo: "EntityId carrega geração"
resumo: "Sem geração, remover uma entidade e criar outra reaproveita o slot — e um id antigo passa a apontar para a entidade errada, em silêncio."
area: nucleo
escolha: "índice + geração, com teste"
descartado: "índice puro na arena"
ordem: 70
---

As entidades vivem numa arena, e um id é um índice nela. Remover a entidade 7 e
criar outra reaproveitaria o slot 7: qualquer id antigo guardado por aí — numa
seleção, num painel, num comando de desfazer — passaria a resolver para a
entidade nova.

Esse é o tipo de bug que não dá erro: dá o objeto errado.

`EntityId` guarda índice **e** geração. O slot reusado tem geração maior, então
o id antigo não resolve para nada. Há teste para isso.

Duas consequências do mesmo desenho: remover uma entidade leva a descendência
junto, e a seleção se limpa quando o alvo morre.
