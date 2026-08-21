# PROGRESSO

> Estado do projeto em disco. **Atualizar ao fim de cada passo, sempre.**
> É este arquivo que faz uma janela de contexto nova saber onde retomar.

## Onde estamos

- **Bloco atual:** B — frontend de pé
- **Último passo concluído:** Passo 22 — título de aba e favicon (**Bloco B concluído**)
- **Próximo passo:** Passo 23 — primeiro passo do Bloco C
- **Comando para retomar:** `/blocao C` (ou `/bloco C` para o modo passo-a-passo)

## Mapa dos blocos

| Bloco | Tema | Passos | Estado |
|---|---|---|---|
| A | servidor de pé | 1–12 | **concluído** (12/12) |
| B | frontend de pé | 13–22 | **concluído** (10/10) |
| C | abrir repositório | 23–34 | em andamento (0/12) |
| D | log (o coração) | 35–46 | pendente |
| E | trabalho local | 47–56 | pendente |
| F | branches e remoto | 57–72 | pendente |
| G | merge e conflitos | 73–79 | pendente |
| H | resto | 80–96 | pendente |
| I | empacotar | 97–103 | pendente |

## Passos concluídos

| # | Passo | Data |
|---|---|---|
| 0 | Andaime em `.claude/` (CLAUDE.md, PROGRESSO.md, blocos A–I, comandos) | 2026-08-21 |
| 1 | Workspace Cargo com os 4 membros (`porc-cli`, `porc-server`, `porc-git`, `porc-index`) | 2026-08-21 |
| 2 | `porc --version` imprime `porcelain 0.1.0`; `porc-server::serve()` fixa a fronteira | 2026-08-21 |
| 3 | axum em `127.0.0.1:7867` com `GET /health` → `{name,version,pid}`; tracing ligado | 2026-08-21 |
| 4 | `bind()` separado de `serve()`; porta ocupada cai para `:0`; URL real no stdout | 2026-08-21 |
| 5 | Token de 256 bits por boot; `POST /api/v1/session` troca por cookie `porc_sess` | 2026-08-21 |
| 6 | `middleware/auth.rs`: tudo exige sessão menos `/health` e o handshake; `whoami` | 2026-08-21 |
| 7 | `middleware/origin.rs`: `Host`/`Origin` exigem loopback + porta real; 3 testes | 2026-08-21 |
| 8 | `middleware/csrf.rs`: double-submit `porc_csrf` + `X-CSRF-Token`; `POST /api/v1/ping` | 2026-08-21 |
| 9 | `porc-cli/browser.rs`: `open`/`xdg-open`/`cmd start`; falha só vira warning | 2026-08-21 |
| 10 | Lockfile `porc.lock` + `porc_server::probe()`; 2ª execução só abre aba e sai 0 | 2026-08-21 |
| 11 | `clap`: `--port`, `--no-browser`, `--repo` (canonicalizado), `--enable-terminal` | 2026-08-21 |
| 12 | Shutdown gracioso por `POST /api/v1/shutdown` e `Ctrl+C`; página `/` provisória | 2026-08-21 |
| 13 | `web/` com Vite + React + TS, porta 5173 fixa (`strictPort`) | 2026-08-21 |
| 14 | Tailwind v4 + `web/src/styles/tokens.css` (12 neutros, 5 tipos, 4px, 120/180ms) | 2026-08-21 |
| 15 | Inter e JetBrains Mono variáveis (latin/latin-ext) em `assets/fonts/`, zero CDN | 2026-08-21 |
| 16 | Tema claro/escuro por `light-dark()` + `data-theme`, store Zustand persistido | 2026-08-21 |
| 17 | Shell de três painéis (sidebar/centro/detalhe), borda de 1px, densidade slim | 2026-08-21 |
| 18 | `Splitter` com pointer capture + store `porc.layout` persistido; ctrl+b / ctrl+d | 2026-08-21 |
| 19 | Feature `dev-proxy`: fallback do router encaminha para o Vite; HMR direto na 5173 | 2026-08-21 |
| 20 | Feature `embed-web`: `build.rs` roda o Vite, `rust-embed` serve com ETag/cache; CSP | 2026-08-21 |
| 21 | `lib/api.ts` + `api-types.ts`, TanStack Query no root, handshake `?t=` com cortina de erro | 2026-08-21 |
| 22 | Título `<repo> · <branch> — porcelain` e favicon próprio, monocromático nos dois temas | 2026-08-21 |

## Decisões tomadas que valem para o futuro

- **2026-08-21** — Nome do app: `porcelain`, binário `porc`.
- **2026-08-21** — Depende do `git` do sistema (≥ 2.30), verificado no boot com erro
  legível. É o que torna credential helper, hooks, LFS, rebase interativo e worktree
  viáveis.
- **2026-08-21** — Terminal PTY fica no escopo do v1 mas **desligado por padrão**
  (`--enable-terminal`). É a única superfície onde falha de autenticação vira RCE.
- **2026-08-21** — Índice de busca em SQLite (`rusqlite` bundled + FTS5), descartável.
  Índice só em memória inviabilizaria o boot < 1s.
- **2026-08-21** — **Paths de commits não são indexados.** Filtro por caminho é
  `git log -- <path>` em streaming cancelável. Indexar path em monorepo de 100k commits
  gera dezenas de milhões de linhas e a manutenção cresce mais rápido que o benefício.
- **2026-08-21** — `status` sai do libgit2 e vira shell-out `--porcelain=v2 -z`: libgit2
  não usa fsmonitor nem untracked-cache, e a diferença em worktree grande é de segundos
  para dezenas de ms.
- **2026-08-21** — Grafo do log em **Canvas 2D**, não SVG/DOM. Nós e arestas em DOM não
  chegam perto de 500ms nem com virtualização.
- **2026-08-21** — **Um** WebSocket multiplexado por `topic` (job, repo.changed, search,
  term), não vários. Um ponto só para autenticar, validar Origin e reconectar.
- **2026-08-21** — Toda leitura de git atrás do trait `RepoRead`, para permitir trocar
  libgit2 por `gix` depois sem tocar em rota nem UI.
- **2026-08-21** — Tailwind **v4** (CSS-first, `@theme`), porque casa com o requisito de
  tokens num arquivo único do qual tudo deriva.
- **2026-08-21** — Binário se chama `porc` via `[[bin]]` explícito no `porc-cli`; o nome do
  crate continua `porc-cli`. `rust-version = "1.75"` fixado no workspace como piso do
  toolchain.
- **2026-08-21** — `Cargo.lock` **é versionado** (workspace com binário publicável), por
  isso não entra no `.gitignore`.
- **2026-08-21** — O Passo 1 tocou 9 arquivos em vez dos 3-4 do limite: são o `Cargo.toml`
  do workspace mais `Cargo.toml` + `lib.rs`/`main.rs` de cada membro, todos triviais.
  Quebrar em dois passos deixaria o workspace sem compilar no meio, o que viola a regra 4
  (mais forte). Andaime inicial é a exceção; a partir do Passo 2 vale o limite normal.
- **2026-08-21** — Existe `/blocao <X>`, que roda o bloco inteiro num laço (escreve em
  disco, roda o aceite de cada passo, atualiza `PROGRESSO.md` a cada passo, para se o
  aceite falhar duas vezes). Ele suspende as regras 1 e 7 do `CLAUDE.md` e **não** cola
  o conteúdo dos arquivos na resposta — no Claude Code o código vai para o disco, não
  para o chat. `/bloco <X>` continua existindo para o modo passo-a-passo.
- **2026-08-21** — `bind()` e `serve()` são separados (`porc_server::bind → Bound::serve`).
  É o que permite ao CLI conhecer a porta real, gravar o lockfile e abrir o navegador antes
  de entregar a thread ao axum.
- **2026-08-21** — Sessão e CSRF são **dois segredos independentes** do mesmo tipo
  (`session::Secret`), nunca o mesmo valor. O de CSRF vai num cookie legível por JS; vazá-lo
  não dá sessão a ninguém.
- **2026-08-21** — `/health` expõe `instanceId` (16 hex de um segredo dedicado), não um
  prefixo do token de sessão. O "token_hint" do BLOCO-A virou isso: rota pública não vaza
  nem um bit do segredo de sessão.
- **2026-08-21** — Quem confirma a instância viva é a resposta de `/health` (pid +
  instanceId batendo com o lockfile), não o arquivo nem uma checagem de PID. Arquivo mente,
  processo vivo não.
- **2026-08-21** — O probe HTTP do lockfile mora no `porc-server` (`probe(port)`), escrito
  na mão sobre `TcpStream`. Quem define o formato de `/health` é o servidor, e um cliente
  HTTP completo (reqwest + TLS) num app 100% loopback não se paga.
- **2026-08-21** — `clap` com `name = "porcelain"` e `bin_name = "porc"`: `--version`
  imprime o produto (`porcelain 0.1.0`), o `Usage` mostra o executável.
- **2026-08-21** — `~/.cargo/bin` não está no PATH de shells não-interativos desta máquina;
  scripts precisam de `export PATH="$HOME/.cargo/bin:$PATH"`.
- **2026-08-21** — `.claude/commands/` em vez de `.claude/skills/`: skills carregam por
  relevância implícita; aqui queremos invocação explícita e determinística, garantindo
  que a janela leia exatamente `CLAUDE.md` + `PROGRESSO.md` + **um** `BLOCO-X.md`.

- **2026-08-21** — A escala neutra é nomeada por **profundidade** (`--n-0` = fundo,
  `--n-11` = texto de maior contraste), não por claridade, e cada token é escrito uma vez
  com `light-dark(claro, escuro)`. O tema claro não é um segundo conjunto de nomes: é o
  mesmo token com o valor espelhado. Trocar `color-scheme` troca a interface inteira.
- **2026-08-21** — `prefers-color-scheme` nunca responde "sem preferência" (o valor saiu da
  spec), então quem nunca escolheu no sistema vê o claro. Escuro continua sendo o default
  de *projeto* — a paleta foi desenhada para ele —, mas não é imposto por CSS.
- **2026-08-21** — `--spacing: 4px` no `@theme` faz **toda** utility de espaçamento do
  Tailwind derivar do token (`p-2` = `calc(var(--spacing) * 2)`). Não existe escala de
  espaço paralela.
- **2026-08-21** — Fontes: subsets `latin` e `latin-ext` das versões **variáveis** de Inter
  e JetBrains Mono (um arquivo por subset cobre todos os pesos), em `assets/fonts/` fora de
  `web/`, alcançadas por alias `@fonts` do Vite + `server.fs.allow`. OFL, licença junto.
  `font-display: block` porque o arquivo vem do mesmo processo que serviu o HTML.
- **2026-08-21** — A fronteira de autenticação virou o prefixo **`/api/`**: a casca do app
  (HTML, JS, CSS, fontes) é pública. Não é conveniência — o handshake acontece dentro dela,
  e exigir sessão no bundle seria um ciclo. Não há o que proteger ali; o que a UI mostra vem
  de `/api/`.
- **2026-08-21** — `dev-proxy` e `embed-web` são **as duas** features default, porque o
  Cargo não sabe variar feature por perfil. Quem desempata é `debug_assertions` no `cfg`, e
  um `compile_error!` cobre a combinação em que sobraria nenhum frontend para servir.
- **2026-08-21** — O HMR **não** passa pelo proxy: `hmr.clientPort: 5173` faz o cliente do
  Vite abrir o WebSocket direto na porta dele. Proxiar upgrade de WebSocket significaria
  manter código de produção que só o dev usa.
- **2026-08-21** — CSP é `default-src 'self'` no release e **frouxa em dev**
  (`'unsafe-inline'` + `connect-src ws://127.0.0.1:5173`), porque o Vite injeta script
  inline e reescreve estilo. O `cfg` garante que o binário do usuário nunca use a frouxa.
- **2026-08-21** — `build.rs` roda `npm ci` só quando falta `node_modules` (ele apaga e
  reinstala tudo; pagar isso a cada `cargo build --release` seriam minutos por build) e
  `npm run build` sempre que o embed for usado.
- **2026-08-21** — `rust-embed` 8: o `#[folder]` é relativo ao `Cargo.toml` do crate
  (`../../web/dist`) — usar `$CARGO_MANIFEST_DIR` exigiria a feature
  `interpolate-folder-path`. O trait que dá `Assets::get` chama-se `Embed`; `RustEmbed` é
  só o derive.
- **2026-08-21** — Arquivo com hash (`/assets/…`) que não existe devolve **404**, não o
  `index.html`: cair na SPA ali faria o navegador tentar executar HTML como JavaScript.
  Cache é `immutable` para o que tem hash no nome e `no-cache` + ETag para o `index.html`.
- **2026-08-21** — O template do Vite liga `erasableSyntaxOnly`: parameter properties
  (`constructor(readonly x: T)`) não compilam. Campo declarado e atribuído no corpo.
- **2026-08-21** — Atalhos de painel: `Ctrl/Cmd+B` sidebar, `Ctrl/Cmd+D` detalhe (com
  `preventDefault`, que Ctrl+D é "favoritar"). Fora da lista que o navegador rouba.
- **2026-08-21** — Título da aba é `<repo> · <branch> — porcelain`: o repositório primeiro
  porque é o que sobrevive ao truncamento da aba.
- **2026-08-21** — Os Passos 19 e 20 tocaram 6 arquivos cada, acima do limite de 3-4. Em
  ambos o excedente era inseparável (feature + dep + módulo + router; e a camada de auth,
  que sem a mudança deixaria o bundle em 401). Regra 4 é mais forte que a regra 2.
- **2026-08-21** — `page.rs` foi removido: quem serve `/` agora é o `fallback` (Vite em
  debug, `rust-embed` em release).

## Pendências / ideias anotadas

- `POST /api/v1/ping` e `GET /api/v1/whoami` são provisórias (existem para exercitar CSRF
  e auth). `ping` sai no Bloco E; `whoami` vira informação de repositório no Bloco C.
- Falta a camada de `rate_limit` (`tower_governor`) prevista na ordem do router: entra
  quando existir rota cara (busca, `fs/list`, jobs).
- Os aceites **visuais** dos Passos 17, 18 e 22 (três painéis na densidade certa, arrastar
  e recarregar, aba com título e ícone) foram verificados por build limpo e inspeção do
  HTML/CSS gerado, **não** num navegador de verdade. Vale o usuário abrir uma vez.
- `REPO`/`BRANCH` em `Shell.tsx` e a lista de commits são placeholder; saem no Bloco C.
- A `ThemeSwitch` mora na barra de topo por falta de lugar melhor. Quando houver command
  palette (Ctrl+K), o tema vira comando e a barra fica só com repo e branch.
- Falta o guard de origem no upgrade do WebSocket — não há WS ainda, mas a camada global
  já cobre por estar antes de tudo.
- Em dev a CSP é frouxa (`'unsafe-inline'` + `ws://127.0.0.1:5173`) porque o Vite injeta
  script inline e abre o WebSocket do HMR. O binário de release nunca usa essa versão.

## Ambiente verificado

- macOS arm64 (Darwin 25.5.0) — máquina de desenvolvimento
- Node v22.23.1, npm 10.9.8 — ok
- git 2.54.0 — ok
- Rust 1.98.0 / cargo 1.98.0 — ok. **`~/.cargo/bin` não está no PATH de shell
  não-interativo**: comandos automatizados precisam de `export PATH="$HOME/.cargo/bin:$PATH"`
  antes do `cargo`. No terminal interativo do usuário funciona normal.
