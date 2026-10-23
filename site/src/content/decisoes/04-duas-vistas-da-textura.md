---
titulo: "A janela é vista em dois formatos ao mesmo tempo"
resumo: "A cena escreve numa view sRGB; a interface, numa view linear da mesma textura. Sem isso, era escolher entre cena certa e UI certa."
area: render
escolha: "view_formats: sRGB para a cena, linear para a UI"
descartado: "Um formato só para as duas"
ordem: 40
---

A `egui` já entrega cores com gama aplicada. Escrevê-las num alvo sRGB aplicaria
a correção **duas vezes**, e a interface sairia clara demais. Mandar a cena para
um alvo linear erraria do outro lado.

A saída é reinterpretar a mesma textura: via `view_formats`, a cena escreve numa
view sRGB e a interface numa view linear da *mesma* memória. Nenhuma das duas
precisa compensar a outra, e não há passe de cópia no meio.
