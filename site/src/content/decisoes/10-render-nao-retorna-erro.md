---
titulo: "Renderer::render não retorna erro"
resumo: "Perder um frame é normal. A resposta certa é pular o frame, não propagar falha até o jogo."
area: render
escolha: "Pular o frame perdido"
descartado: "Result a cada frame"
ordem: 100
---

Uma janela sendo redimensionada, um compositor ocupado, uma surface
temporariamente fora de data: adquirir a textura do frame falha de vez em
quando, e isso não é defeito.

Se `render` devolvesse `Result`, todo chamador — inclusive o código de jogo —
teria de decidir o que fazer com um erro cuja única resposta sensata é "tenta no
próximo frame". Na prática, viraria um `.unwrap()` ou um `let _ =` em cada ponto
de uso.

Por isso `Renderer::render` não retorna erro: ele pula o frame e registra o
motivo no log. Os erros que **importam** — criar o device, compilar um shader,
abrir um arquivo — continuam sendo `Result`, porque aí não há "próximo frame"
que resolva.
