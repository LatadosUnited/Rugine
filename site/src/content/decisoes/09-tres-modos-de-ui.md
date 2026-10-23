---
titulo: "Três modos de UI, explícitos"
resumo: "Sem a distinção entre Disabled, Overlay e Editor, a interface limparia a tela e apagaria a cena num jogo sem editor."
area: editor
escolha: "UiMode::Disabled | Overlay | Editor"
descartado: "Um caminho só, com a UI sempre dona da janela"
ordem: 90
---

Quem desenha na janela muda conforme o programa:

| Modo | Quem é dono da janela |
|---|---|
| `Disabled` | A cena vai direto para a janela |
| `Overlay` | A cena desenha, e a interface vai por cima (HUD) |
| `Editor` | A cena vai para uma textura; a interface é dona da janela |

Tornar isso explícito evita o erro silencioso: num jogo sem editor, uma
interface que sempre limpa a tela apagaria a cena desenhada antes dela.

Duas consequências práticas: o modo `Editor` é o que exige o viewport em
textura, e é dele que vem o atraso de um frame no tamanho do viewport — a UI
imediata só sabe quanto espaço o painel tem enquanto o constrói, e aí a cena do
frame já foi desenhada.
