---
titulo: "Componentes com armazenamento próprio, não bevy_ecs"
resumo: "O que o editor precisa é acesso dinâmico pelo nome, e isso o bevy_ecs só daria com bevy_reflect em cima."
area: nucleo
escolha: "Lista de Box<dyn Componente> por entidade"
descartado: "bevy_ecs"
ordem: 60
---

Cada entidade guarda uma lista de `Box<dyn Componente>`, na ordem em que o
Inspector os mostra.

A pergunta que decidiu isso não é "o que é mais rápido para o runtime", é "o que
o editor precisa fazer": listar os componentes de uma entidade, adicionar e
remover **pelo nome**, e desenhar a UI de cada um. Tudo acesso dinâmico — que o
`bevy_ecs` só ofereceria com `bevy_reflect` por cima, somando duas dependências
grandes para resolver um problema de editor.

A decisão é reversível de propósito: a lista é privada, e toda mudança passa por
métodos da `Scene`. Se o runtime precisar de milhares de entidades por quadro, o
armazenamento muda por baixo dessa API — os componentes não.

Essa mesma privacidade é o funil onde o **desfazer** entra. Transform, nome e
visibilidade ainda são campos públicos, e precisam passar por ele também.
