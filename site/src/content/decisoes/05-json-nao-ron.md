---
titulo: "O arquivo .cena é JSON, não RON"
resumo: "Os componentes vão pelo nome, com os dados num valor genérico — e o ron::Value não representa enums."
area: nucleo
escolha: "JSON com versão"
descartado: "RON"
ordem: 50
---

RON é a escolha natural num projeto Rust, e foi a primeira tentativa. Ela não
sobreviveu ao formato do arquivo.

Cada componente é gravado pelo nome do tipo, com os dados num valor genérico,
porque é o registro de tipos que sabe interpretá-los. E o `ron::Value` **não
representa enums**: o `FormaColisor` (caixa ou esfera) não voltaria igual da ida
e volta.

As demais regras saíram de pensar em quem edita o arquivo à mão:

- Campo ausente usa o padrão.
- Componente desconhecido vira aviso; o resto da cena carrega.
- `versao` maior que a conhecida é recusada com um erro que diz exatamente isso.
- Salvar grava num `.tmp` e troca no fim: morrer no meio não deixa a cena pela
  metade.
