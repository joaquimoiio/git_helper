import { resolve } from "node:path";

import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// As fontes moram fora de `web/`, em `assets/fonts/`, porque não são código do frontend —
// são material do produto, versionado junto do resto. O alias faz o `url()` do CSS
// encontrá-las e o `fs.allow` autoriza o dev server a servi-las de fora da raiz.
const repoRoot = resolve(import.meta.dirname, "..");

export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: { "@fonts": resolve(repoRoot, "assets/fonts") },
  },
  server: {
    // A porta é fixa e `strictPort` é obrigatório: o proxy do Rust em modo `dev-proxy`
    // aponta para 5173 literal. Cair para 5174 em silêncio deixaria o Rust servindo nada.
    host: "127.0.0.1",
    port: 5173,
    strictPort: true,
    fs: { allow: [repoRoot] },
    // Em dev o navegador abre a porta do `porc`, não esta. O proxy do Rust encaminha HTTP;
    // o WebSocket do HMR o cliente abre direto aqui. Proxiar upgrade de WebSocket no
    // servidor significaria manter código de produção que só o HMR usa.
    hmr: { protocol: "ws", host: "127.0.0.1", port: 5173, clientPort: 5173 },
  },
});
