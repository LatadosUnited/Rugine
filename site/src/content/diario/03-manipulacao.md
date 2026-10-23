---
titulo: "Manipulação: picking, gizmos e o dia em que nada aparecia"
marco: 3
data: 2026-10-22
resumo: "Selecionar, mover, escalar e rotacionar. O marco mais difícil até aqui não foi a matemática dos gizmos: foi descobrir por que eles não apareciam, sem um único erro de validação."
commits: ["bd54357", "1a56d5e", "6f6115f", "a9c97cb", "2ec23f7"]
tecnologias: ["picking", "gizmos", "rotacao", "arrasto", "sobreposicao", "captura", "contorno"]
---

## O que ficou de pé

- **Picking.** O clique vira um raio contra a caixa de cada entidade, na CPU,
  sem passe extra na GPU. O raio é levado ao espaço local da entidade: origem
  com `w = 1`, direção com `w = 0`.
- **Gizmos de mover e escalar.** Hastes por eixo, agarradas pelo mesmo teste de
  raio do picking, com tamanho constante na tela — perto ou longe, a alça tem o
  mesmo número de pixels.
- **Gizmo de rotação.** Anéis por eixo, modos Global e Local, ângulo acumulado
  (várias voltas sem salto em 180°) e snap de 15°.
- **Camada sobreposta.** Um pipeline sem teste de profundidade para o que fica
  por cima de tudo.

## Os gizmos invisíveis

Nenhum erro de validação, nenhum painel vermelho, e nada na tela. "Sem erro" não
é o mesmo que "apareceu" — e a diferença custou o dia.

A ferramenta que resolveu foi a **captura do viewport**: ler a textura de volta
da GPU e gravar um PNG, com a contagem de pixels fortemente coloridos no log.

```bash
RUGINE_CAPTURA=/tmp/viewport.png cargo run -p editor
```

Isso responde "algo foi desenhado?" sem abrir a imagem, o que torna possível
bissecionar. Duas causas apareceram:

1. **O buffer de instâncias.** No DX12 desta máquina, instâncias lidas de um
   deslocamento no meio do buffer não renderizavam nada — sem erro, sem aviso. A
   camada sobreposta ganhou buffer próprio.
2. **O viewport em 163 px esticado.** A captura mostrou a cena desenhada numa
   resolução errada e ampliada. O tamanho do painel só é conhecido enquanto a UI
   é construída, e aí o frame da cena já foi desenhado: o tamanho chega com um
   frame de atraso, de propósito.

A captura também reordena os canais quando a surface é BGRA. Sem isso ela
mentiria justamente sobre a cor, que é o que normalmente se quer conferir.

## O arrasto que piscava

Arrastar uma alça fazia o objeto tremer. O deslocamento estava sendo medido
contra uma origem que andava junto com o objeto: a cada frame a referência era
a nova posição, então o erro realimentava. A correção foi congelar o gizmo
durante o arrasto e medir sempre contra o estado de quando o arrasto começou.

No mesmo passo, o cubo do centro do gizmo passou a escalar os três eixos juntos,
multiplicando a escala em vez de somar — o que mantém a proporção.
