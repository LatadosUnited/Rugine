---
titulo: "DX12 como backend padrão no Windows"
resumo: "O driver Vulkan da Intel HD 530 desta máquina segfaulta dentro de si mesmo ao criar o device. Nenhum tratamento de erro em Rust pega isso."
area: render
escolha: "DX12 por padrão no Windows"
descartado: "Deixar o wgpu escolher"
ordem: 10
---

O `wgpu` escolhe o backend sozinho, e no Windows ele costuma preferir Vulkan.
Nesta máquina — Intel HD 530 — o driver Vulkan dá `ACCESS_VIOLATION` **dentro do
driver** ao criar o device.

Isso não é um `Err` que se trata: é um segfault no código do driver, fora do
alcance de qualquer `Result` ou `catch_unwind`. O processo morre sem mensagem
útil, e de um jeito que parece bug da engine.

**A decisão:** no Windows o padrão é DX12. Em todas as plataformas,
`WGPU_BACKEND=vulkan|dx12|gl` continua sobrepondo o padrão, porque a escolha é
de hardware, não de projeto — numa máquina com driver Vulkan sadio, trocar é uma
variável de ambiente.
