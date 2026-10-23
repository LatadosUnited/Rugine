/**
 * Links sempre a partir da raiz do site. O `base` existe porque o destino e um
 * GitHub Pages de projeto, que serve tudo sob /Rugine/; escrever "/arvore/" na
 * mao funcionaria no dev e quebraria no Pages.
 */
const BASE = import.meta.env.BASE_URL.replace(/\/+$/, "");

export function caminho(destino = ""): string {
  const limpo = destino.replace(/^\/+/, "");
  return limpo === "" ? `${BASE}/` : `${BASE}/${limpo}`;
}

/** Dois caminhos sao a mesma pagina mesmo que um tenha barra no fim. */
export function mesmaPagina(a: string, b: string): boolean {
  const n = (s: string) => s.replace(/\/+$/, "") || "/";
  return n(a) === n(b);
}
