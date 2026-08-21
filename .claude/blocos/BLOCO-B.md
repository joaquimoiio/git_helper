# BLOCO B — frontend de pé

**Objetivo:** a casca visual do app, com os design tokens definidos antes de qualquer
componente, e o build de release produzindo **um arquivo só**.

**Ao fim do bloco:** `cargo build --release` gera um binário único que serve a interface
completa (layout de três painéis, tema escuro/claro, fontes locais) sem pasta de assets
ao lado e sem internet.

---

## Detalhe técnico do bloco

**Tokens antes de componente.** `web/src/styles/tokens.css` é a fonte única. Nenhum
componente escreve cor, espaço, raio, tamanho de fonte ou duração crus — sempre
`var(--…)`. Tailwind v4 lê os tokens via `@theme`, então a utility class e o token são a
mesma coisa. Se um valor não existe como token, o passo é criar o token, não hardcodar.

**Escalas mínimas:**
- Tipo: **exatamente 5 tamanhos** no app inteiro.
- Cor: escala neutra de ~12 degraus + semânticas dessaturadas (add, del, conflito, erro).
- Espaço: escala de 4px.
- Duração: 120ms e 180ms. Só isso.

**Tema.** Escuro é o default; claro é espelho exato — os mesmos nomes de token com
valores invertidos, nunca um segundo conjunto de nomes. Componente nenhum sabe qual tema
está ativo.

**Dev vs release.** Em dev o Rust faz proxy de `/` para o Vite (`:5173`) — sem isso o
ciclo de frontend fica insuportável, porque cada mudança de CSS exigiria recompilar Rust.
Em release, `build.rs` roda o build do Vite e `rust-embed` embute `web/dist`.

**Fontes.** Inter + JetBrains Mono em `woff2`, subsetadas, servidas do próprio projeto.
Zero CDN — o app tem que funcionar offline e a CSP é `default-src 'self'`.

---

## Passos

### Passo 13 — Vite + React + TS isolado
Criar `web/` com Vite (template `react-ts`), rodando em `:5173` sem qualquer relação com
o Rust ainda.
**Aceite:** `npm run dev` em `web/` abre a página padrão.

### Passo 14 — Tailwind v4 e `tokens.css`
Instalar Tailwind v4 + `@tailwindcss/vite`. Criar `web/src/styles/tokens.css` com as
escalas: neutros, semânticas dessaturadas, espaço 4px, raio, 5 tamanhos de tipo, 120/180ms.
Uma página de amostra provando que os tokens estão aplicados.
**Aceite:** a página mostra a escala de cinzas e os 5 tamanhos de tipo.

### Passo 15 — fontes locais
Inter e JetBrains Mono em `assets/fonts/` (woff2), declaradas com `@font-face` e ligadas
aos tokens de família. Nenhum `<link>` para fonts.googleapis.
**Aceite:** DevTools → Network mostra as fontes vindo do próprio servidor; modo offline
mantém a tipografia.

### Passo 16 — tema escuro e claro
Tokens redefinidos para claro como espelho exato, alternados por `data-theme` no root,
com preferência do sistema como default e escolha persistida.
**Aceite:** alternar o tema muda tudo sem nenhum componente saber do tema.

### Passo 17 — shell de layout
Três painéis: sidebar / centro / detalhe, separados por borda de 1px de baixo contraste.
Sem sombra, sem gradiente. Conteúdo de placeholder.
**Aceite:** a página mostra os três painéis na densidade certa.

### Passo 18 — painéis redimensionáveis e persistidos
Arrastar as divisórias, colapsar sidebar e detalhe por atalho, larguras salvas em
localStorage via store Zustand de layout.
**Aceite:** redimensionar, recarregar a página, larguras preservadas.

### Passo 19 — feature `dev-proxy`
`porc-server` com a feature `dev-proxy` (default em debug) fazendo proxy de `/` para
`http://127.0.0.1:5173`, respeitando as camadas de segurança.
**Aceite:** `npm run dev` + `cargo run` → a UI aparece na porta do Rust, com HMR vivo.

### Passo 20 — feature `embed-web`
`build.rs` rodando `npm ci && npm run build` quando `embed-web` está ligada, com
`cargo:rerun-if-changed=web/src`. `rust-embed` servindo `web/dist` com MIME e cache
corretos. CSP `default-src 'self'` aplicada.
**Aceite:** `cargo build --release` → um binário; movê-lo para outra pasta e rodar
continua servindo a UI inteira.

### Passo 21 — cliente de API e handshake
`web/src/lib/api.ts` tipado, TanStack Query no root, e o boot lendo `?t=` da URL,
chamando `POST /api/v1/session`, guardando o CSRF e limpando a query do histórico.
Estado de erro legível se o token faltar.
**Aceite:** abrir a URL com token → a UI carrega e a barra de endereço fica sem o `?t=`;
abrir sem token → mensagem clara, não tela branca.

### Passo 22 — título de aba e favicon
Título `<repo> · <branch> — porcelain` (por enquanto placeholder) e favicon
monocromático desenhado no projeto, também embutido.
**Aceite:** a aba mostra título e ícone próprios.

---

**Fim do bloco:** atualizar `PROGRESSO.md` e dizer
*"bloco concluído, pode dar `/clear` e rodar `/bloco C`"*.
