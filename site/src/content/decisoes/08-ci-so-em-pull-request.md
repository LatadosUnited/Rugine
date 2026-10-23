---
titulo: "CI só em pull request"
resumo: "Compilar wgpu, naga e egui em duas plataformas custa minutos demais para rodar a cada commit. A checagem do Linux ficou local."
area: projeto
escolha: "CI em pull_request e workflow_dispatch"
descartado: "CI a cada push"
ordem: 80
---

A primeira execução do CI mostrou o custo: `wgpu`, `naga` e `egui` compilados em
Linux e Windows, sem cache quente, levam minutos — e a cada commit isso não se
paga.

O CI roda em `pull_request` e à mão (`workflow_dispatch`), e é onde se busca a
confirmação de verdade antes de integrar algo.

O caminho barato do dia a dia é local, e não precisa de toolchain cruzada porque
`clippy` e `check` não linkam:

```bash
rustup target add x86_64-unknown-linux-gnu
cargo clippy --workspace --all-targets --target x86_64-unknown-linux-gnu -- -D warnings
```

Duas escolhas dentro do workflow seguem a mesma lógica: **sem `fail-fast`**,
porque o Windows passando não diz nada sobre o Linux, e `--all-targets` em vez
de um `cargo build` separado, que seria uma segunda compilação inteira do
workspace.
