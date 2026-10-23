import { defineConfig } from "astro/config";

// Site estatico. `base` existe porque o destino provavel e o GitHub Pages de
// projeto, que serve tudo sob /<repo>/; com `site` e `base` definidos aqui, os
// links escritos com `caminho()` valem igual no dev e no Pages.
//
// SITE_BASE sobrepoe isso para quem serve o site na raiz de um dominio — um
// tunel, por exemplo, onde "/" e a home e "/Rugine/" nao existe:
//
//     SITE_BASE=/ npm run build
const base = process.env.SITE_BASE ?? "/Rugine";

export default defineConfig({
  site: process.env.SITE_URL ?? "https://latadosunited.github.io",
  base,
  trailingSlash: "always",
  build: { format: "directory" },
  // A arvore ja vem montada no HTML; o unico script do cliente e o de
  // navegar/arrastar. Sem islands, nao ha framework nenhum para hidratar.
  devToolbar: { enabled: false },
});
