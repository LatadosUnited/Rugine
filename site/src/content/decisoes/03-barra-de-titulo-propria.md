---
titulo: "A barra de título é desenhada pela engine"
resumo: "Pintar a barra nativa exigiria uma API que só existe no Windows 11 e não tem equivalente no Linux. Fora do macOS, onde desenhar a nossa deixaria a janela sem como redimensionar."
area: editor
escolha: "Janela sem decoração, barra desenhada"
descartado: "Barra do sistema, pintada por API nativa"
ordem: 30
---

A janela sobe sem decoração e o editor desenha título, botões e alças de
redimensionar.

O caminho nativo foi descartado de propósito: pintar a barra do Windows no tema
do editor exige `DWMWA_CAPTION_COLOR`, que só existe no Windows 11 e não tem
equivalente no Linux. Já `drag_window` e `drag_resize_window` da winit funcionam
em Windows, X11 e Wayland — uma implementação para os três.

**No macOS é diferente, e por requisito, não por gosto.** Lá
`drag_resize_window` retorna sempre `NotSupported`: uma janela sem decoração
ficaria *sem nenhuma forma de ser redimensionada*. Por isso
`Decoracao::recomendada()` devolve `Sistema` no macOS.
