# PROGRESSO

> Estado do projeto em disco. **Atualizar ao fim de cada passo, sempre.**
> É este arquivo que faz uma janela de contexto nova saber onde retomar.

## Onde estamos

- **Bloco atual:** F — branches e remoto
- **Último passo concluído:** Passo 58a — `exec::branch::create` + `POST /repos/{id}/branches` (criar branch, com checkout e upstream opcionais)
- **Próximo passo:** Passo 58b — a UI de criar branch: caixa com nome, ponto de partida (HEAD, commit selecionado, ref) e os dois interruptores
- **Comando para retomar:** `/blocao F` (ou `/bloco F` para o modo passo-a-passo)

## Mapa dos blocos

| Bloco | Tema | Passos | Estado |
|---|---|---|---|
| A | servidor de pé | 1–12 | **concluído** (12/12) |
| B | frontend de pé | 13–22 | **concluído** (10/10) |
| C | abrir repositório | 23–34 | **concluído** (12/12) |
| D | log (o coração) | 35–46 | **concluído** (12/12) |
| E | trabalho local | 47–56 | **concluído** (10/10) |
| F | branches e remoto | 57–72 | em andamento (1/16) |
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
| 23 | `GET /api/v1/fs/list` com `isRepo`, confinamento na home e ocultos escondidos | 2026-08-21 |
| 24 | `FolderBrowser`: coluna navegável por ↓↑ → ← enter `.`, marcando os repos | 2026-08-21 |
| 25a | `porc-git` `RepoRead`/`Git2Repo` + registry de `repo_id` + `POST/GET /api/v1/repos` | 2026-08-21 |
| 25b | Repo aberto na UI: topo, sidebar, painel de detalhe e título da aba; fim dos placeholders | 2026-08-21 |
| 26a | `porc-index` com SQLite (WAL) + `recents`; `GET /api/v1/recents`, `DELETE /recents/{id}` | 2026-08-21 |
| 26b | `Recents` na tela inicial, atalhos 1–9, "esquecer" e marca de repo sumido do disco | 2026-08-21 |
| 27a | `config.toml` (`root`, `scan_depth`, `scan_limit`) + `discover::scan` + `GET /fs/scan` | 2026-08-21 |
| 27b | `Discovered`: lista "nesta máquina" com a raiz à vista e botão de recarregar | 2026-08-21 |
| 28a | `porc-git/exec/`: regras de todo shell-out + `git init`; `POST /api/v1/repos/init` | 2026-08-21 |
| 28b | `NewRepo`: criar repositório na pasta atual (tecla `n`), com branch inicial opcional | 2026-08-21 |
| 29a | `jobs.rs` (registry, cancelamento, cleanup) + `ws.rs` multiplexado + rotas de job + job de teste | 2026-08-21 |
| 29b | `lib/ws.ts` com backoff e reassinatura, `lib/jobs.ts` ligando eventos ao cache, `JobsPanel` | 2026-08-21 |
| 30 | `parse/progress.rs`: `Splitter` por `\r`+`\n` e eventos tipados, com stderr real nos testes | 2026-08-21 |
| 31a | `exec::stream` (relógio de inatividade) + `exec/clone.rs` + `POST /api/v1/jobs/clone` | 2026-08-21 |
| 31b | `CloneRepo` (tecla `c`) com branch/depth/remote/submódulos; clone pronto já abre o repo | 2026-08-21 |
| 32 | `Jobs::shutdown()` cancela e espera a limpeza ao sair; MSRV corrigido para 1.87 | 2026-08-21 |
| 33 | Askpass por socket unix efêmero (0600) + `AskpassPrompt` na UI; prompt sobrevive a reload | 2026-08-21 |
| 34 | `exec/error.rs`: stderr → frase + ação sugerida; "ver detalhes" na UI (**Bloco C concluído**) | 2026-08-21 |
| 35 | `GET /repos/{id}/log`: revwalk paginado por cursor opaco (fronteira), 500 por página | 2026-08-21 |
| 36 | `CommitList`: TanStack Virtual + infinite query, linha slim, teclado j/k/setas/Home/End/PageUp/Down | 2026-08-22 |
| 37 | Lanes do grafo no `log()`: algoritmo incremental, cursor `v2` carrega o estado inteiro | 2026-08-22 |
| 38 | `LogGraph`: `<canvas>` sobreposto à lista, redesenhado no mesmo render do scroll | 2026-08-22 |
| 39a | `GET /repos/{id}/refs`: branches, remotas, tags e HEAD destacado, sem paginação | 2026-08-22 |
| 39b | `RefBadges` na linha do commit: `useRefs` + mapa oid→marcadores, sem estourar a linha | 2026-08-22 |
| 40a | `GET /repos/{id}/commits/{oid}`: mensagem completa, assinaturas, diffstat por arquivo | 2026-08-22 |
| 40b | `CommitDetail`: mensagem, assinaturas, pais clicáveis e lista de arquivos no painel direito | 2026-08-22 |
| 41a | `GET /repos/{id}/commits/{oid}/diff?path=`: hunks estruturados por arquivo, sob demanda | 2026-08-22 |
| 41b | `FileDiffView`: modo unificado e lado a lado, realce leve por token, avisos de binário/não-UTF8 | 2026-08-22 |
| 42 | Job de indexação (`porc-index::commits` + `walk_for_index` + `index_job`), disparado ao abrir | 2026-08-22 |
| 43a | `commits_fts` (FTS5) + `GET /repos/{id}/search?q=`: prefixo, incremental, entrada perigosa não quebra | 2026-08-22 |
| 43b | `Log` (`SearchBox` + `SearchResults`): busca com debounce, sem janela separada, lista achatada | 2026-08-22 |
| 44a | `search_commits` ganha hash prefixado (com índice, sem FTS5), `autor:`, `depois:`, `antes:` | 2026-08-22 |
| 44b | `SearchResults` seleciona sozinho quando a busca parece hash e acha exatamente um commit | 2026-08-22 |
| 45a | `exec::Pipe` (stdout genérico) + `parse::records` + job `path-filter`: `git log -z -- <path>` | 2026-08-22 |
| 45b | `list_paths` (git2, árvore de HEAD) + `GET /repos/{id}/paths?prefix=`: autocomplete por nível | 2026-08-22 |
| 45c | `PathFilter` + `FlatCommitList` extraído da busca; progresso/cancelar de graça via `JobsPanel` | 2026-08-22 |
| 46a | `path_filter::by_content` (`-S`/`-G`) + job `pickaxe` com `handle.hit()` por commit achado | 2026-08-22 |
| 46b | `PickaxeFilter`: cancela e reinicia a cada tecla, lista cresce ao vivo pela cauda `hits` (**Bloco D concluído**) | 2026-08-22 |
| 47 | `GET /repos/{id}/status`: `git status --porcelain=v2 -z` + parser + `git2::state()`, agrupado em staged/unstaged/untracked | 2026-08-22 |
| 48a | `exec::stage` (`add`/`reset` por lista de caminhos) + `POST /repos/{id}/stage` e `/unstage`, devolvendo o status atualizado | 2026-08-22 |
| 48b | `StatusPanel` + `lib/status.ts`: três grupos numa lista só, teclado (j/k/espaço/s/u/a/Esc) e otimista | 2026-08-22 |
| 49a | `RepoRead::worktree_diff` (índice↔worktree e HEAD↔índice) + `GET /repos/{id}/diff?path=&side=` | 2026-08-22 |
| 49b | `DiffView` extraído do `FileDiffView` + `StatusDetail`: o diff do lado certo no painel direito | 2026-08-22 |
| 50a | `porc-git::patch` (recorte de patch por hunk, `@@` recalculado) + `RepoRead::worktree_patch` | 2026-08-22 |
| 50b | `exec::run_with_input` + `exec::apply::cached` + `POST /repos/{id}/apply` | 2026-08-22 |
| 50c | `HunkAction` no `DiffView` + `useApplyHunks`: botão "preparar"/"desfazer" no cabeçalho de cada hunk | 2026-08-22 |
| 51a | `patch::HunkSelection` com linhas (adição não escolhida some, remoção vira contexto) + `hunks[].lines` na rota | 2026-08-22 |
| 51b | Clique em linha no modo unificado marca; o botão do hunk passa a dizer "preparar N linhas" | 2026-08-22 |
| 52a | `exec::commit` (mensagem por stdin, erros próprios, `commit.template`) + `POST /commit` e `GET /commit/template` | 2026-08-22 |
| 52b | `CommitBox` no pé do status: régua 50/72, template semeado uma vez, ctrl+enter, tecla `c` | 2026-08-22 |
| 53 | `amend`/`signoff`/`gpgSign` no `exec::commit`, na rota e em três interruptores do `CommitBox` | 2026-08-22 |
| 54 | `--fixup=<oid>` (mensagem do git, alvo validado como hash) + botão "corrigir este commit" no detalhe | 2026-08-22 |
| 55a | `exec::discard` (`checkout`, `remove_untracked` confinado, `apply --reverse`) + `POST /discard` e `/discard/hunks` | 2026-08-22 |
| 55b | `ConfirmDiscard` (nomeia o que se perde, foco no cancelar) + tecla `d` no `StatusPanel` | 2026-08-22 |
| 55c | Ação destrutiva por hunk no `DiffView`, só do lado de fora do commit | 2026-08-22 |
| 56a | `RepoRead::range_diff`/`range_file_diff` (+ `summarize_trees` extraída) e `GET /compare` e `/compare/diff` | 2026-08-22 |
| 56b | `CompareView`: dois campos de revisão com sugestão, botão de trocar os lados, lista e diff (**Bloco E concluído**) | 2026-08-22 |
| 57a | `RepoRead::remotes`/`stashes` + `GET /repos/{id}/remotes` e `/stashes` — o que faltava para a sidebar | 2026-08-22 |
| 57b | `features/sidebar/Sidebar`: branches, remotas agrupadas por remote, tags e stashes numa lista só, com filtro e teclado (**Passo 57 concluído**) | 2026-08-22 |
| 58a | `exec::branch::create` (`git branch` / `git switch --create`, `--track` de três estados) + `POST /repos/{id}/branches` | 2026-08-22 |

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

- **2026-08-22** — **Bloco E começou.** O Passo 47 tocou 8 arquivos (`exec/status.rs` e
  `parse/status_v2.rs` novos, mais `exec/mod.rs`/`parse/mod.rs` para registrar os dois,
  `model.rs`, `read.rs`, `routes/repos.rs`, `lib.rs`), acima do limite de 3-4. Não deu para
  cortar: um endpoint de status precisa do shell-out, do parser, dos tipos de domínio, da
  leitura de estado via git2 e da rota — tirar qualquer um deixaria o passo pela metade, e a
  regra 4 (compila e roda) é mais forte que a regra 2. Mesmo raciocínio do Passo 1 e dos
  Passos 19/20.
- **2026-08-22** — **`status` shell-out, `state()` (merge/rebase) via `git2`** — as duas fontes
  do `CLAUDE.md`/`BLOCO-E.md`, mas juntas numa rota só. `RepoRead::state()` cresce o trait do
  jeito que ele já vinha crescendo desde o Bloco D (`refs`, `commit_detail`…): usa
  `git2::Repository::state()`, que só olha a presença de marcadores em disco
  (`MERGE_HEAD`, `rebase-merge/`…) — barato, e por isso não vale a pena reimplementar como
  teste de disco próprio (diferente do `discover::is_repo`, que existe justamente para não
  abrir um `Repository` por entrada listada).
- **2026-08-22** — `git status --porcelain=v2 -z` **não é streaming**: roda com `exec::run`
  (não `exec::stream`), sob o `LOCAL_TIMEOUT` de 30s como o `init`. É rápido e local — mesmo
  numa worktree de milhares de arquivos, não é o tipo de comando que fica minutos calado.
- **2026-08-22** — Com `-z`, **todo** o formato porcelain v2 termina em NUL, inclusive o
  cabeçalho `# branch.*` — confirmado por `xxd` contra o stdout real do `git` deste
  repositório antes de escrever o parser, mesma disciplina do Passo 45. A única entrada com
  **dois** NULs por registro é o rename/copy (tipo `2`): o caminho antigo é o registro NUL
  seguinte inteiro, não um campo dentro do mesmo registro — por isso `parse::status_v2::parse`
  não separa por linha e sim consome um registro extra da própria iteração quando vê tipo `2`.
- **2026-08-22** — Um arquivo com `X` e `Y` diferentes de `.` (parcialmente stageado, ex.
  `MM`) vira **duas** `StatusEntry`, uma em `staged` e outra em `unstaged` — o mesmo arquivo
  nos dois grupos, igual ao que o `git status` de terminal mostra em duas seções. Verificado
  ponta a ponta: `crates/porc-git/src/lib.rs` com uma linha stageada e outra não apareceu nos
  dois grupos na resposta HTTP real.
- **2026-08-22** — Conflito (`u`, linha de unmerged) vira `StatusKind::Unmerged` dentro de
  `unstaged` — o Passo 47 pede três grupos, não quatro, e um quarto grupo "conflitos" fica
  para o Bloco G, que é quem de fato faz algo com ele (marcar ours/theirs, resolver). Por ora
  o objetivo é só não escondê-lo.
- **2026-08-22** — `WorktreeStatus.state` nasce sempre `RepoState::Clean` dentro de
  `parse::status_v2::parse` (a função só vê o stdout do `status`, que não carrega essa
  informação) e é sobrescrito pela rota depois de chamar `RepoRead::state()` — duas fontes,
  uma struct só, montada em dois passos em vez de uma terceira função que as combinasse.
- **2026-08-22** — Aceite do Passo 47 (2026-08-22): `cargo test --workspace` — 129 testes,
  tudo verde (17 novos: 2 de `exec::status`, 12 de `parse::status_v2`, 2 de `read::state`).
  Clippy e `cargo fmt --check` limpos. Handshake HTTP completo contra este próprio
  repositório: sujei a worktree de propósito (uma linha stageada e outra não no mesmo
  arquivo, um `.gitignore` renomeado com `git mv` para exercitar o tipo `2`, um arquivo novo
  não rastreado) e `GET /status` devolveu exatamente os três grupos certos — `crates/porc-
  git/src/lib.rs` nos dois (`staged`/`unstaged`), o rename com `oldPath: ".gitignore"` e
  `kind: "renamed"`, o arquivo novo em `untracked`. Estado limpo depois de desfazer tudo.

- **2026-08-22** — **Passo 48 quebrado em 48a (backend) e 48b (frontend)**: não havia rota
  nenhuma de mutação de git ainda (`stage`/`unstage` não existiam em lugar algum), então o
  passo "stage e unstage por arquivo" precisava tanto do shell-out quanto da rota antes de a
  UI ter o que chamar — juntar os dois passaria do limite de 3-4 arquivos (backend sozinho já
  usa 4: `exec/stage.rs`, `exec/mod.rs`, `routes/repos.rs`, `lib.rs`). Mesmo raciocínio do
  25a/25b, 39a/39b etc.
- **2026-08-22** — `exec::stage::add`/`reset` são só `git add -- <paths>` / `git reset --
  <paths>`, sem `-A` e sem pathspec vazio nunca: a rota rejeita `paths` vazio com 400
  (`RepoError::NoPaths`) antes de chamar o git — "selecionar tudo" é resolvido no cliente
  (que já tem a lista do último `status`), não um flag que o servidor interpreta.
- **2026-08-22** — `git reset -- <paths>` (não `git restore --staged`) para unstage: mais
  antigo, mais universalmente entendido, e funciona igual com `HEAD` unborn (testado —
  `reset_desfaz_o_add` roda num repo recém-`init`ado sem nenhum commit). `git restore` exige
  git ≥ 2.23; o piso do projeto é 2.30, então os dois serviriam, mas `reset` é o que todo
  material sobre porcelain v2 já assume.
- **2026-08-22** — `POST /stage` e `/unstage` devolvem o `WorktreeStatus` já atualizado (não
  `204`/`{"ok":true}`): a mutação e a releitura são baratas as duas (comando local, sem
  streaming), e devolver o estado novo poupa a UI de um segundo round-trip só para saber se
  o clique funcionou. `read_status` virou função compartilhada entre `status`, `stage` e
  `unstage` — as três rotas combinam as duas mesmas fontes (`exec::status` + `git2::state()`).
- **2026-08-22** — Aceite do Passo 48a (2026-08-22): `cargo test --workspace` — 132 testes,
  tudo verde (3 novos em `exec::stage`, incluindo stage em lote de dois arquivos de uma vez).
  Clippy e `cargo fmt --check` limpos. Handshake HTTP completo contra este próprio
  repositório: sujei a worktree com um arquivo novo, `POST /stage` moveu ele de `untracked`
  para `staged` na resposta, `POST /unstage` devolveu para `untracked`, e `paths: []`
  devolveu 400 sem tocar o git. Estado limpo depois (nada ficou staged, arquivo de teste
  removido).

- **2026-08-22** — O centro do app virou **duas abas** (`log` / `status`), como o `CLAUDE.md`
  manda ("centro (log ou status)"), e o estado da aba mora no `Shell` — **não** é persistido:
  quem abre o app quer o histórico, e restaurar "status" no boot seguinte esconderia o log de
  quem só passou por ali uma vez.
- **2026-08-22** — O `StatusPanel` é **uma lista achatada** com cabeçalho de grupo no meio, não
  três listas independentes: o cursor atravessa os três grupos com uma tecla só, que é o que faz
  "marcar cinco arquivos espalhados e stagear" ser um gesto em vez de três. A identidade de uma
  linha é o par `grupo:caminho`, nunca só o caminho — um arquivo parcialmente stageado (`MM`)
  aparece nos dois grupos, e são coisas diferentes (o que vai no commit vs. o que ficou de fora).
- **2026-08-22** — Teclado do status: `j`/`k`/setas movem, **espaço** marca e desce (marcar cinco
  seguidos é espaço cinco vezes), `s` prepara, `u` desfaz, `a` marca/desmarca o grupo do cursor,
  `Esc` limpa. Sem nada marcado, `s`/`u` valem para a linha do cursor — o caso comum de um arquivo
  só não paga o custo de marcar antes. O painel toma o foco ao montar: sem isso `s` não chegaria a
  lugar nenhum antes de um clique, e o aceite do passo é "só pelo teclado".
- **2026-08-22** — O otimista do stage/unstage **prevê o movimento entre grupos** (`predict` em
  `lib/status.ts`), com rollback em erro, e a resposta do servidor **substitui** o cache em vez de
  invalidá-lo — as rotas do 48a já devolvem o `WorktreeStatus` novo, e um `invalidateQueries` aqui
  gastaria um round-trip para descobrir o que o servidor acabou de dizer. A única troca de `kind`
  que ele adivinha é `untracked` → `added`; o resto herda o que estava lá.
- **2026-08-22** — `useStatus` é o **oposto** do log e das refs (`staleTime: Infinity`): usa
  `staleTime: 0` e refetch ao voltar o foco da aba. O status muda por fora — o usuário salva no
  editor dele —, e voltar para a aba é exatamente o gesto de quem fez isso. Watcher de verdade
  (fsmonitor/inotify) não é deste bloco.
- **2026-08-22** — Aceite do Passo 48b (2026-08-22): `npm run build` e `npm run lint` limpos (só o
  aviso informativo pré-existente do `react-compiler` sobre `useVirtualizer`). Sem backend novo —
  reusa `status`/`stage`/`unstage` (47/48a), já verificados ponta a ponta por HTTP.

- **2026-08-22** — **Três testes já estavam vermelhos ao entrar no Passo 49a**, e não por bug: os
  do `exec::path_filter` rodam contra o repositório do **próprio porcelain** e comparavam contra
  listas de oid escritas à mão, que o commit `207b025` invalidou. Pior, o
  `pickaxe_sem_ocorrencia` procurava um literal que passou a existir no histórico no commit que
  criou o próprio arquivo de teste. Corrigidos para comparar contra o `git log` **de verdade**
  rodado na hora (o contrato é "a mesma coisa que o git responderia", não um instante do
  histórico) e para montar a agulha inexistente com um carimbo de nanossegundos. Sem isso, todo
  aceite do Bloco E rodaria vermelho por motivo alheio ao passo.
- **2026-08-22** — `worktree_diff(side, path)` cobre os dois lados com **uma** função:
  `Unstaged` é `diff_index_to_workdir`, `Staged` é `diff_tree_to_index` contra a árvore do
  `HEAD` (`None` em unborn, e aí tudo no índice é adição). Sai na mesma `FileDiff` do
  `commit_diff` de propósito — é o que faz o visualizador do Passo 41 servir aos dois sem saber
  de onde o patch veio. A conversão `git2::Patch` → `FileDiff` virou `patch_to_file_diff`,
  compartilhada pelas duas.
- **2026-08-22** — O pathspec do `worktree_diff` vai com `disable_pathspec_match(true)`:
  comparação **literal**, não glob. Um arquivo de verdade chamado `notas[1].txt` não pode virar
  padrão só por ter colchete no nome. E `include_untracked` + `show_untracked_content`, senão o
  arquivo novo — justamente o que se quer olhar antes do primeiro `add` — não teria diff nenhum.
- **2026-08-22** — **Sem `find_similar` no `worktree_diff`**, ao contrário do `commit_diff`: com
  o diff já reduzido a um pathspec não há o outro lado do par para a detecção de rename achar.
  Um arquivo renomeado e stageado aparece como adição do caminho novo — que é exatamente o
  caminho que o `status` deu à UI, e o conteúdo mostrado é o certo.
- **2026-08-22** — Erro novo `GitError::FileUnchanged` (404), separado do `FileNotInCommit`:
  pedir o diff staged de um arquivo que só está modificado no worktree não é o mesmo que pedir
  um arquivo que um commit não tocou — ali o commit é imutável, aqui basta stagear para o mesmo
  pedido passar a valer.
- **2026-08-22** — `DiffSide` deriva `Deserialize` no `porc-git` e é o próprio tipo do parâmetro
  de query, **sem padrão**: qual lado se está olhando é a informação central da rota, e adivinhar
  mostraria o diff errado sem avisar. Um `side` desconhecido morre no serde, com 400.
- **2026-08-22** — O `DiffView` foi extraído do `FileDiffView` e **ficou onde nasceu**
  (`features/log/`), importado pelo `features/status/StatusDetail`. Mover para um `features/diff/`
  é renomeação, não decisão: vale quando aparecer o terceiro consumidor (Passo 56).
- **2026-08-22** — Com o centro em `status`, o painel direito mostra o **diff do arquivo sob o
  cursor**, não o commit selecionado no log: o detalhe é sempre o detalhe do assunto do centro. E
  o lado vem do **grupo** da linha, sem seletor próprio — a linha de `MM` aparece nos dois grupos
  justamente porque são dois diffs, e perguntar o lado de novo seria perguntar duas vezes.
- **2026-08-22** — `stage`/`unstage` **invalidam** as queries `worktree-diff` (invalidação de
  verdade, não `setQueryData` como no status): o servidor devolve o status novo, mas não os
  patches, e o diff aberto na tela é justamente de um lado que acabou de mudar.
- **2026-08-22** — Aceite do Passo 49a (2026-08-22): `cargo test --workspace` — 137 testes, tudo
  verde (4 novos de `worktree_diff`: mudança só no worktree aparece de um lado só, `add` troca o
  lado, arquivo novo vem inteiro como adição, arquivo limpo não tem diff de nenhum dos dois).
  Clippy e `cargo fmt --check` limpos. Handshake HTTP completo contra um repositório sintético em
  `~/porc-aceite-e`: `side=unstaged` trouxe o hunk certo de `a.txt` (`dois`→`DOIS` mais `quatro`),
  `side=staged` deu 404 com a frase certa antes do `add` e o hunk igual depois, `b.txt` não
  rastreado veio inteiro como adição, e `side=lixo` deu 400 no serde.
- **2026-08-22** — Aceite do Passo 49b (2026-08-22): `npm run build` e `npm run lint` limpos.
  Sem backend novo — reusa o `/diff` do 49a, já verificado ponta a ponta.

- **2026-08-22** — **Passo 50 quebrado em 50a (recorte), 50b (aplicação + rota) e 50c (UI)**: o
  caminho inteiro toca 7 arquivos. 50a é o texto do patch (`patch.rs`, `lib.rs`, `read.rs`), 50b
  é o que o executa (`exec/mod.rs`, `exec/apply.rs`, `routes/repos.rs`, `porc-server/lib.rs`).
- **2026-08-22** — O patch recortado sai do **patch cru do libgit2** (`worktree_patch`, novo no
  `RepoRead`), não de uma reconstrução a partir da `FileDiff` que a UI tem. O cabeçalho de
  verdade carrega `diff --git`, `index`, `new file mode`/`deleted file mode` e os modos — nada
  disso cabe na forma que a interface consome, e inventá-lo seria adivinhar o que o git já disse.
  `with_worktree_patch` monta o diff **uma vez** e entrega o `git2::Patch` a um fecho: é o que
  garante que a numeração de hunk que a UI mostra é a mesma que o `git apply` recebe. (Fecho e
  não valor de retorno porque `Patch` empresta o `Diff`, que empresta o `Repository`.)
- **2026-08-22** — No recorte, **a contagem do lado antigo não muda e a do lado novo muda**: o
  `git apply --cached` aplica contra o índice, que continua sendo o de antes de qualquer hunk,
  mas a linha inicial do lado **novo** de cada hunk mantido precisa do deslocamento acumulado
  **só dos mantidos**. Sem isso, stagear o terceiro de três hunks emite um patch apontando para
  uma linha que não existe do lado que ele constrói. Tem teste de unidade e teste contra o `git`
  de verdade (`o_ultimo_hunk_sozinho_tambem_aplica`).
- **2026-08-22** — `exec::run_with_input` alimenta o `stdin` do git por **pipe**, nunca por
  arquivo temporário: um patch em disco é uma janela em que outro processo pode lê-lo ou trocá-lo,
  e mais um arquivo para limpar quando o comando morre no meio. A escrita vai numa task à parte —
  escrever e esperar na mesma task se trancariam com o pipe cheio. `run` e `run_with_input`
  compartilham `run_inner`, para o `SIGTERM`-graça-`kill_on_drop` existir num lugar só.
- **2026-08-22** — `git apply --cached --whitespace=nowarn`: o patch saiu do arquivo do próprio
  usuário, então espaço em branco "errado" nele é o que ele escreveu. Recusar por causa disso
  (o que um `apply.whitespace = error` na config dele faria) transformaria a config de um
  `git apply` de terminal numa parede aqui dentro.
- **2026-08-22** — `POST /repos/{id}/apply` tem **um** campo `side` que decide as duas coisas:
  de que lado os trechos estão agora e, por consequência, para onde vão (`unstaged` → stagear;
  `staged` → desfazer, com `--reverse`). Elas nunca divergem — ninguém stagea o que já está no
  índice. `--cached` nos dois sentidos: o arquivo em disco não é tocado, e há teste para isso.
- **2026-08-22** — `GitError::NotUtf8` novo (422, não 400): recortar patch de arquivo em encoding
  legado produziria um patch que não bate mais com o conteúdo — o `git apply` recusaria, ou pior,
  aplicaria outra coisa. O pedido está bem formado; o arquivo é que não serve.
- **2026-08-22** — Aceite do Passo 50a (2026-08-22): `cargo test --workspace` — 145 testes, tudo
  verde (8 novos: 7 do `patch::parse`/`select` e 1 casando o patch cru do libgit2 com o parser e
  conferindo que os dois caminhos veem o mesmo número de hunks). Clippy e `fmt` limpos.
- **2026-08-22** — Aceite do Passo 50b (2026-08-22): `cargo test --workspace` — 150 testes, tudo
  verde (5 novos de `exec::apply`, todos contra o `git` de verdade: um hunk de três entra e os
  outros ficam de fora conferido pelo `git diff --cached`, o último hunk sozinho aplica,
  `--reverse` tira do índice, o disco do usuário não é tocado, patch que não casa falha em vez de
  aplicar torto). Clippy e `fmt` limpos. Handshake HTTP completo contra `~/porc-aceite-e` (30
  linhas, 3 hunks): `POST /apply` com `hunks:[1]` deixou `a.txt` **nos dois grupos** e o
  `git diff --cached` de verdade mostrou só `+MUDOU-B`; `side:"staged"` com `hunks:[0]` esvaziou
  o índice de novo; seleção vazia e hunk inexistente deram 400 com a frase certa.

- **2026-08-22** — A ação de hunk é um **botão no cabeçalho do próprio `@@`**, um hunk por
  clique, sem marcação múltipla e sem confirmação: nada ali destrói trabalho (o arquivo em disco
  não é tocado, `--cached`) e a volta é o mesmo botão do outro lado. `HunkAction` é opcional no
  `DiffView` — ausente no diff de commit, que é imutável e não teria para onde apontar.
- **2026-08-22** — `useApplyHunks` **não tem otimista**, ao contrário do stage por arquivo:
  prever o efeito de um hunk sobre os três grupos exigiria simular o patch no cliente, que é
  exatamente o que o `BLOCO-E.md` manda não fazer. O ida-e-volta é local e rápido.
- **2026-08-22** — `DiffSide` mudou de casa: nasceu em `lib/status.ts` e foi para
  `lib/api-types.ts`, que é onde moram os tipos que espelham o serde. `status.ts` re-exporta,
  para os componentes continuarem importando de um lugar só.
- **2026-08-22** — Aceite do Passo 50c (2026-08-22): `npm run build` e `npm run lint` limpos.
  Sem backend novo — reusa o `/apply` do 50b, já verificado ponta a ponta com o `git diff
  --cached` de verdade.

- **2026-08-22** — As **duas regras** do recorte por linha, que é onde patch parcial feito à mão
  costuma corromper trabalho silenciosamente: **adição não escolhida some** (ela ainda não existe
  do lado antigo, e não vai passar para o novo) e **remoção não escolhida vira contexto** (a linha
  continua nos dois lados, e o `git apply` precisa vê-la para casar o trecho). Trocar uma pela
  outra ou some com uma linha que devia ficar, ou faz o patch deixar de casar. As contagens dos
  dois lados do `@@` são recontadas a partir do que sobrou, não herdadas.
- **2026-08-22** — A linha é identificada pela posição **entre as linhas de mudança** do hunk (a
  primeira `+`/`-` é 0), não pela posição no corpo. Contexto não conta dos dois lados do fio. É o
  que faz a numeração do cliente e a do servidor coincidirem sempre: o texto do patch tem linhas
  que a UI não desenha (o marcador `\ No newline at end of file`), e numerar sobre "todas as
  linhas" divergiria exatamente nos arquivos que têm isso. Selecionar contexto também não
  significaria nada — contexto não muda de lado.
- **2026-08-22** — O marcador `\ No newline at end of file` **acompanha a linha dele**: fica se a
  linha foi mantida, cai se ela foi descartada. Tem teste.
- **2026-08-22** — Hunk cuja seleção não deixou mudança nenhuma (só contexto) **não é emitido**, e
  se nenhum hunk sobrar o pedido vira `EmptySelection` (400): um `git apply` de patch que não muda
  nada falha, e falhar com a frase certa é melhor que falhar com a do git.
- **2026-08-22** — `select_hunks(&[usize])` virou atalho para `select(&[HunkSelection])`. Uma
  entrada só de verdade, para não haver dois caminhos de recorte que possam divergir.
- **2026-08-22** — **Seleção por linha só no modo unificado.** No lado a lado, uma linha da
  esquerda e uma da direita dividem a mesma fileira, e um clique ali seria ambíguo entre "esta
  remoção" e "esta adição". Quem quer escolher linha troca para o unificado, que é onde cada linha
  é uma linha. O botão por hunk continua nos dois modos.
- **2026-08-22** — O `DiffView` recebe `key={path:side}` do `StatusDetail`: sem remontar, linhas
  escolhidas num arquivo sobreviveriam para o seguinte e virariam um patch em cima do arquivo
  errado.
- **2026-08-22** — Aceite do Passo 51a (2026-08-22): `cargo test --workspace` — 156 testes, tudo
  verde (6 novos do recorte por linha, incluindo o marcador de fim-sem-quebra, e 1 novo de
  `exec::apply` contra o `git` de verdade). Clippy e `fmt` limpos. Handshake HTTP completo: num
  hunk com 4 linhas de mudança (`-linha 6`, `-linha 7`, `+VIZINHA-1`, `+VIZINHA-2`), mandar
  `lines:[0,2]` deixou no `git diff --cached` **exatamente** `-linha 6` e `+VIZINHA-1`; o hunk
  inteiro (sem `lines`) continuou funcionando; linha fora da faixa deu 400.
- **2026-08-22** — Aceite do Passo 51b (2026-08-22): `npm run build` e `npm run lint` limpos. Sem
  backend novo — reusa o `/apply` do 51a.

- **2026-08-22** — `git commit -F -`: a mensagem vai por **pipe**, nunca em `argv` (que qualquer
  `ps` da máquina mostra) e nunca em arquivo temporário. Shell-out e não `git2` porque um commit
  precisa disparar `pre-commit`/`commit-msg`/`post-commit`, respeitar `commit.gpgSign`,
  `user.signingkey` e `core.hooksPath` — um commit escrito por dentro do libgit2 nasceria sem
  nada disso.
- **2026-08-22** — `--cleanup=strip`, que é o que o git aplica depois de uma sessão de editor —
  e a caixa de mensagem da interface **é** o editor. Tira espaço no fim, linhas em branco
  sobrando e as linhas de comentário que um `commit.template` traz. Consequência conhecida, a
  mesma do terminal: uma linha começando com `#` é comentário e some.
- **2026-08-22** — `Failure` ganhou o campo **`stdout`**. Não é generalidade gratuita: o
  `git commit` escreve "nothing to commit" no **stdout**, não no stderr, e sem isso a única
  informação útil da falha se perdia (o teste `indice_vazio_vira_erro_proprio` pegou isso).
- **2026-08-22** — `CommitError` tem erros próprios (`EmptyMessage` 400, `NothingToCommit`,
  `IdentityMissing` e `Refused` 409) em vez de cair no `diagnose` do Passo 34, que é um
  catálogo de erro de **rede**. E `Refused` **passa a saída do hook adiante** (3 primeiras
  linhas não vazias): a regra de "nunca stderr cru" existe contra jargão do git, e a mensagem de
  um hook é escrita pelo time do usuário — é a única coisa útil que existe sobre aquela recusa.
- **2026-08-22** — Mensagem vazia é checada **antes** de chamar o git: uma volta inteira ao
  processo para descobrir o que dá para saber daqui não se paga.
- **2026-08-22** — `commit.template` é rota própria (`GET /repos/{id}/commit/template`), não um
  campo do `status`: é config lida uma vez ao abrir a caixa, e o status é pedido a toda hora.
  Template ausente, ilegível ou apontando para o nada é `null` — nunca pode impedir alguém de
  commitar. O `~/` é expandido na mão (não há shell, e o git só o expande quando ele mesmo lê o
  arquivo).
- **2026-08-22** — A caixa é **uma área de texto só**, assunto e corpo juntos, e não dois campos:
  é a forma do `commit.template`, a forma que o git recebe e a forma que quem cola uma mensagem
  pronta espera. A régua de 50/72 **avisa e não trava** (inclusive sobre a linha em branco
  faltando depois do assunto): a convenção é forte, mas é convenção.
- **2026-08-22** — O template entra **uma vez**, e só numa caixa vazia (`useRef` de semeado):
  recarregá-lo por cima de algo que a pessoa está escrevendo apagaria o trabalho dela. E a caixa
  só é limpa **em caso de sucesso** — uma mensagem longa não pode sumir porque um hook recusou.
- **2026-08-22** — Aceite do Passo 52a (2026-08-22): `cargo test --workspace` — 161 testes, tudo
  verde (5 novos de `exec::commit`, incluindo um `pre-commit` de verdade que recusa e cuja
  mensagem chega ao erro). Clippy e `fmt` limpos. Handshake HTTP completo contra
  `~/porc-aceite-e`: commit sem nada preparado → 409 "não há nada preparado para commitar";
  mensagem vazia → 400; `stage` + `commit` devolveu o oid e um status vazio, e o
  `git log -1` de verdade mostrou assunto e corpo separados corretamente; `commit.template`
  configurado voltou com o conteúdo do arquivo.
- **2026-08-22** — Aceite do Passo 52b (2026-08-22): `npm run build` e `npm run lint` limpos.
  Sem backend novo — reusa `/commit` e `/commit/template` do 52a.

- **2026-08-22** — GPG é **flag efêmera, nunca `git config`**: `--gpg-sign`/`--no-gpg-sign` valem
  só para aquele commit. E o interruptor da UI tem **três** estados, não dois — desligá-lo volta
  para "como você configurou" (`null`, sem flag nenhuma), não para "não assinar". Quem tem
  `commit.gpgSign = true` continua assinando sem precisar lembrar de ligar nada.
- **2026-08-22** — `CommitError::SigningFailed` é checado **antes** de tudo em `classify`: o git
  reporta falha de assinatura como `gpg failed to sign the data` seguido de `failed to write
  commit object`, e sem a checagem específica isso viraria uma `Refused` genérica com o jargão
  do gpg dentro. A causa quase sempre é pinentry — o `gpg-agent` precisa perguntar a passphrase
  e não tem onde, porque não herdamos terminal e o askpass do Passo 33 é do git e do ssh, não do
  gpg. Fazer o gpg passar pelo mesmo socket é ideia para outro bloco (ver pendências).
- **2026-08-22** — Com `--amend` o botão de commitar **não** exige nada preparado: emendar só a
  mensagem é o uso mais comum de todos. E ligar o amend carrega a mensagem anterior por
  `useCommitDetail(headOid)` — com a mesma regra do template: só entra na caixa vazia ou por cima
  do template intocado, nunca por cima do que a pessoa escreveu.
- **2026-08-22** — Esse carregamento é **ajuste durante o render** (com um `carregadoDe`), não
  `useEffect`: a mensagem chega de uma consulta, então não dá para resolvê-la no evento do
  clique, e o `set-state-in-effect` do oxlint pegou a primeira versão. Mesma correção do Passo
  41b, não supressão.
- **2026-08-22** — Aceite do Passo 53 (2026-08-22): `cargo test --workspace` — 166 testes, tudo
  verde (5 novos: amend reescreve em vez de criar outro e o log continua com **um** commit,
  amend sem commit anterior tem erro próprio, signoff põe o trailer com a identidade
  configurada, `--no-gpg-sign` vence um `commit.gpgSign=true` com chave inexistente, e o
  `classify` de stderr real de falha de assinatura). Clippy, `fmt`, `npm run build` e
  `npm run lint` limpos. Handshake HTTP: signoff pôs `Signed-off-by:` de verdade; amend trocou a
  mensagem e o log ficou com **dois** commits (não três); `gpgSign:true` com chave inexistente
  deu 409 com a frase de pinentry; `gpgSign:false` passou. **Assinar de verdade não foi
  exercitado** — não há chave GPG nesta máquina; o que se verificou foi o caminho de erro e o de
  desligar. Vale um teste manual de quem tiver chave (anotado nas pendências).

- **2026-08-22** — No `--fixup`, **a mensagem é do git**: ele monta `fixup! <assunto do alvo>`
  sozinho, e é essa string exata que o `rebase --autosquash` reconhece depois. Escrever a
  mensagem à mão seria a forma mais fácil de produzir um fixup que o autosquash ignora. Por isso
  `--fixup` e `-F -` são mutuamente exclusivos aqui (o git recusa os dois juntos), o stdin não é
  alimentado nesse caminho, e a checagem de "mensagem vazia" não vale para ele. Na UI, o gesto
  inteiro é **um botão** no painel de detalhe — não há caixa de texto porque não há texto nosso.
- **2026-08-22** — O alvo do fixup é validado como **hash** (7 a 40 hexadecimais), não como
  revisão livre: o valor vem do cliente e vira parte de um argumento. Aceitar revisão daria a ele
  `HEAD@{…}`, `:/texto` e companhia — sintaxes que o git resolve e que ninguém aqui está
  preparado para explicar. A UI sempre tem o oid completo do commit selecionado. Mesmo
  raciocínio da validação de `name` no `init`.
- **2026-08-22** — Aceite do Passo 54 (2026-08-22): `cargo test --workspace` — 168 testes, tudo
  verde (2 novos: o fixup monta a mensagem certa **e um `rebase --autosquash` de verdade a funde
  de volta no alvo**, deixando um commit só; e cinco alvos inválidos — `HEAD`, `:/assunto`,
  `../etc`, `zzzz`, vazio — recusados antes de chamar o git). Clippy, `fmt`, `npm run build` e
  `npm run lint` limpos. Handshake HTTP: `fixup` de um commit real produziu
  `fixup! adiciona c (mensagem emendada)`; alvo `HEAD` deu 400.

- **2026-08-22** — Descarte são **três comandos**, porque são três coisas: rastreado volta com
  `git checkout -- <paths>` (ao **índice**, não ao `HEAD` — o que estava preparado continua
  preparado, mesma semântica do terminal); não rastreado é **apagado** com `remove_file`, porque
  o git não tem versão nenhuma dele para restaurar; e trecho é `git apply --reverse` **sem**
  `--cached`. `remove_file` e não `git clean`: o `clean` opera por pathspec e tem vizinhos
  (`-x`, `-d`, `-f`) cuja diferença entre "apaga o que você pediu" e "apaga a pasta inteira" é
  uma letra.
- **2026-08-22** — Cada caminho de `remove_untracked` é canonicalizado e conferido contra a raiz
  do repositório, e diretório é recusado: um `..` no meio ou um symlink apontando para fora não
  podem virar um `remove_file` em outro lugar do disco, e "descartar uma pasta" seria uma
  remoção recursiva pedida por um clique. Tem teste com um arquivo de fora que continua lá.
- **2026-08-22** — **Quem decide entre restaurar e apagar é o servidor**, pelo `status`: o
  cliente só nomeia caminhos. Deixá-lo escolher o comando seria deixá-lo pedir a remoção de um
  arquivo rastreado por engano.
- **2026-08-22** — A tecla `d` **não descarta: abre a confirmação**. A tecla é o começo do
  gesto, nunca o fim. A confirmação **nomeia o que será perdido**, um caminho por linha, em vez
  de perguntar "tem certeza?", e o foco nasce no **cancelar** — um Enter distraído não pode ser
  o gesto que apaga. Nada de otimista aqui: mostrar o resultado antes de o git confirmar seria
  dizer que algo foi perdido antes de saber se foi.
- **2026-08-22** — O botão de descarte por hunk só aparece do lado **unstaged**. Do lado do
  índice existe "desfazer", que não perde nada — oferecer descarte ali seria oferecer perda onde
  há uma saída sem perda nenhuma. No `DiffView` ele é uma prop separada (`destructive`), com cor
  de remoção e por último na barra: quem a passa é obrigado a pensar duas vezes antes de passá-la.
- **2026-08-22** — Aceite dos Passos 55a/b/c (2026-08-22): `cargo test --workspace` — 173 testes,
  tudo verde (5 novos de `exec::discard`: checkout volta ao índice, checkout **preserva o
  preparado**, não rastreado é apagado, caminho com `..` não apaga nada fora, e reverter um hunk
  desfaz só ele no disco). Clippy, `fmt`, `npm run build` e `npm run lint` limpos. Handshake HTTP
  contra `~/porc-aceite-e`: descartar o hunk 0 de um arquivo com dois hunks tirou `MUDOU-A` do
  **disco** e deixou `MUDOU-C`; descartar `a.txt` + `lixo.txt` deixou o status limpo e apagou o
  não rastreado; lista vazia deu 400.

- **2026-08-22** — `summarize_trees` foi **extraída** do `commit_detail`: o diffstat entre duas
  árvores é literalmente a mesma conta no detalhe de um commit (pai ↔ commit) e na comparação
  arbitrária (duas árvores quaisquer). Em dois lugares seriam dois lugares onde a contagem por
  arquivo pode divergir do agregado.
- **2026-08-22** — Os lados da comparação são **revisões**, resolvidas por `revparse_single` —
  a mesma resolução do terminal, então `HEAD~2`, `origin/main`, uma tag ou um hash colado
  funcionam. Isso é seguro aqui de um jeito que **não** seria num shell-out: nada disto vira
  `argv`, é chamada de biblioteca dentro do próprio repositório do usuário. O que ela não
  resolve vira `InvalidCommit` → 400.
- **2026-08-22** — `RangeDiff` é um tipo próprio, não `CommitDetail` reaproveitado: ali não há
  autor, committer nem mensagem, porque não há um commit sendo mostrado. E `from`/`to` voltam
  **resolvidos em oid** mesmo quando o pedido veio por nome — é assim que fica claro o que foi
  comparado de verdade.
- **2026-08-22** — Na UI os dois lados são **campos de texto com sugestão** (`datalist` das
  refs), não seletores fechados: o valor aceito é qualquer revisão que o git entenda, e uma
  lista fechada barraria todas as outras sem ganhar nada. Botão `⇄` para trocar os lados, que é
  o gesto seguinte mais comum. E o `DiffView` ali vai **sem ação nenhuma** nos hunks: comparar é
  leitura, não há para onde mover um trecho entre dois pontos do histórico.
- **2026-08-22** — Aceite do Passo 56a (2026-08-22): `cargo test --workspace` — 177 testes, tudo
  verde (4 novos: resolução por nome de revisão devolvendo oid completo dos dois lados; comparar
  um commit deste repositório com o pai dando **exatamente** o mesmo diffstat que o
  `commit_detail` dele; o diff de um arquivo dentro da comparação com o mesmo número de linhas
  do diff pelo commit; e três revisões inexistentes viradas em 400). Clippy e `fmt` limpos —
  dois avisos reais corrigidos de verdade (um `&mut` desnecessário e itens depois do módulo de
  teste, resíduo da extração). Handshake HTTP: comparar duas branches deu `2 arquivos, +2 -0`,
  batendo com o `git diff --shortstat` de verdade; o diff de `c.txt` veio com o hunk certo;
  revisão inexistente deu 400.
- **2026-08-22** — Aceite do Passo 56b (2026-08-22): `npm run build` e `npm run lint` limpos.
  Sem backend novo — reusa `/compare` e `/compare/diff` do 56a.

- **2026-08-22** — **Bloco F começou. Passo 57 quebrado em 57a (backend) e 57b (sidebar)**: o
  `GET /refs` do Passo 39a já entrega branches, remotas, tags e o `HEAD` destacado, mas a sidebar
  do 57 pede duas coisas que não existiam em lugar nenhum — os **remotes** configurados e a pilha
  de **stash**. Backend sozinho já usa 4 arquivos (`model.rs`, `read.rs`, `routes/repos.rs`,
  `porc-server/lib.rs`), o teto da regra 2. Mesmo raciocínio de 25a/25b, 39a/39b, 48a/48b.
- **2026-08-22** — **`refs` continua sendo uma rota só, e `remotes`/`stashes` são outras duas.**
  Não é fragmentação: `refs` são pontas do histórico (mudam a cada fetch, e o log as consome para
  marcar linha), remote é **configuração** (só muda quando o usuário a muda, Passo 62) e stash é
  uma **pilha** que não marca linha nenhuma do log. Três ritmos de invalidação diferentes; juntá-las
  numa rota "sidebar" faria a UI rebuscar as três sempre que uma mudasse.
- **2026-08-22** — A sidebar agrupa remota por remote usando a **lista de remotes do git**, não
  partindo `origin/main` na primeira barra. Partir acerta quase sempre, mas o git aceita remote com
  barra no nome — e quem sabe onde o nome do remote termina é o próprio git. É também o que o
  Passo 62 (gerenciar remotes) e o 70 (escolher para onde empurrar) vão consumir.
- **2026-08-22** — `StashEntry` **não** é uma `RefMarker`: `refs/stash` é uma ref só, e a pilha
  inteira vive no **reflog** dela. Marcar o log com "stash" seria marcar um commit que nem está no
  histórico. A `message` sai como o git a escreveu (`On main: assunto`), sem reescrita nossa — é a
  mesma linha do `git stash list`, e quem stashou reconhece.
- **2026-08-22** — `Repository::stash_foreach` exige `&mut Repository` mesmo só lendo (é o reflog
  de `refs/stash` que ele percorre, e o libgit2 marca a operação inteira como mutável) — daí o
  `let mut repo` dentro de `stashes()`, que é o único método de leitura assim.
- **2026-08-22** — Em **git2 0.21** as APIs de string ficaram falíveis: `StringArray::iter()`
  entrega `Result<Option<&str>, Error>`, `Remote::url()` entrega `Result<&str, Error>` (e `Ok("")`
  quando não há URL nenhuma) e `Remote::pushurl()` entrega `Result<Option<&str>, Error>`. Nome ou
  URL não-UTF-8 é **descartado**, não remendado com `lossy`: um nome remendado não seria aceito de
  volta por comando nenhum, e uma URL remendada é pior que nenhuma. `pushUrl` só vem preenchida
  quando existe `remote.<nome>.pushurl` própria — sem ela a UI não repete a mesma linha duas vezes.
- **2026-08-22** — Aceite do Passo 57a (2026-08-22): `cargo test --workspace` — 179 testes, tudo
  verde (2 novos: o `origin` deste próprio checkout com URL de fetch e sem `pushurl`; e uma pilha
  de dois stashes num repositório sintético, conferindo que o último a entrar é o `index: 0`, que
  os oids diferem e que o arquivo em disco voltou ao conteúdo commitado). Clippy e
  `cargo fmt --check` limpos. Handshake HTTP completo contra um repositório sintético em
  `~/porc-aceite-f` (dois remotes, um deles com `pushurl` própria; duas branches; uma tag; dois
  stashes): `/remotes` devolveu `backup` com `fetchUrl` e `pushUrl` diferentes e `origin` com
  `pushUrl: null`; `/stashes` devolveu os dois na mesma ordem do `git stash list` de verdade
  (`segundo` em 0, `primeiro` em 1); `repo_id` inexistente deu 404 e sem cookie de sessão deu 401.
  Repositório de teste apagado e a entrada dele removida dos recentes ao final.

- **2026-08-22** — A sidebar é **uma lista achatada** com cabeçalho de grupo no meio, a mesma
  forma do `StatusPanel` e pelo mesmo motivo: o cursor atravessa branches, remotas, tags e
  stashes com uma tecla só, e o filtro corta a lista inteira sem obrigar ninguém a escolher em
  qual seção procurar. O filtro casa contra o **nome inteiro** (`origin/main`) e a linha mostra
  o nome **curto** (`main`), porque o nome do remote já está no cabeçalho do grupo — repeti-lo
  em cada linha gastaria metade da largura da sidebar com a mesma palavra.
- **2026-08-22** — O agrupamento das remotas usa o remote de **nome mais longo** que prefixa a
  ref (`remoteOf`), não o primeiro que casa: com `origin` e `sub/backup` configurados,
  `sub/backup/main` é do segundo, e um `startsWith` ingênuo o jogaria em `origin` se a ordem
  ajudasse. Remota cujo prefixo não bate com remote nenhum (o remote foi removido e as
  `refs/remotes/<nome>/…` ficaram) vai para um grupo **"remotas sem remote"** em vez de sumir —
  esconder seria mentir sobre o que existe no disco. Todo remote configurado ganha cabeçalho
  mesmo com zero remotas buscadas: é ali que se vê que o remote existe, e é de onde o Passo 62
  vai gerenciá-los.
- **2026-08-22** — Ordenação por `localeCompare(…, { numeric: true })`, não alfabética pura:
  numa lista de tags `v2` tem que vir antes de `v10`.
- **2026-08-22** — Enter numa ref **leva ao commit dela no log** (`useCommitSelection` + o
  centro volta para a aba `log`), e nada mais. Trocar de branch é o Passo 59 — dar checkout a um
  Enter aqui antes de existir o aviso de worktree suja (Passo 60) seria a forma mais fácil de
  alguém perder trabalho com um toque. Stash também navega: é um commit de verdade, o detalhe
  abre, mas nenhuma linha do log acende, porque ele não está no histórico.
- **2026-08-22** — Teclado: `j`/`k`/setas movem, `Enter` revela, `/` salta para o filtro, `Esc`
  limpa; no campo de filtro, `↓` entra na lista e `Enter` revela a primeira ponta que sobrou —
  "digitar três letras e chegar lá" sem passar pela lista no meio. A sidebar **não** rouba o
  foco ao montar (diferente do `StatusPanel`): quem abre o app está olhando o centro.
- **2026-08-22** — `<RefTree key={repo.repoId}>`: trocar de repositório remonta a árvore e zera
  filtro e cursor. Sem isso, o filtro do repositório anterior esconderia as refs do novo.
- **2026-08-22** — `useRemotes`/`useStashes` nasceram com `staleTime: Infinity`, como
  `useRefs` — as três só mudam por ação nossa (Passos 60, 62, 63) ou por fora, e é o WebSocket
  que vai invalidá-las. Três chaves de cache diferentes de propósito, pelo mesmo motivo que são
  três rotas (57a): ritmos de invalidação diferentes.
- **2026-08-22** — Este projeto **não tem prettier configurado**; rodar `npx prettier --write`
  reformatou o arquivo novo para 80 colunas, contra as ~100 do resto do código. Desfeito na mão.
  Formatação de TS/TSX aqui é manual, conferida por `npm run lint` (oxlint) — não rodar prettier.
- **2026-08-22** — Aceite do Passo 57b (2026-08-22): `npm run build` e `npm run lint` limpos (só
  o aviso pré-existente do `react-compiler`). `cargo build --release` embutiu o bundle novo
  (contém "filtrar refs", "remotas sem remote", "refs do repositório" e "nada com esse nome",
  confirmado por grep no JS servido). Contra **este** repositório por HTTP, `/refs` deu `main`
  (`isHead`) e `origin/main` e `/remotes` deu `origin` — exatamente o `git branch -a`. E contra
  um repositório sintético em `~/porc-aceite-57b` desenhado para os casos difíceis (branches
  `main`/`feature/x`/`feature/y`, tags `v2` e `v10`, remotes `origin` **e `sub/backup`** — com
  barra no nome —, uma `refs/remotes/sumido/antiga` órfã e dois stashes), o `build`/`filter` do
  componente foi rodado sobre o JSON **real** das três rotas (via `jiti`, harness temporário
  fora do projeto): `sub/backup/main` caiu em `sub/backup` e não em `origin`, `sumido/antiga`
  caiu em "remotas sem remote", `v2` veio antes de `v10`, o filtro `main` acertou os quatro
  grupos e `sub/backup` afunilou para um. Repositório de teste apagado, entrada removida dos
  recentes e servidor derrubado ao final.

- **2026-08-22** — **Passo 58 quebrado em 58a (backend) e 58b (UI)**: o backend sozinho já usa os
  4 arquivos do teto (`exec/branch.rs`, `exec/mod.rs`, `routes/repos.rs`, `porc-server/lib.rs`).
  Mesmo raciocínio de 25a/25b, 48a/48b, 57a/57b.
- **2026-08-22** — Criar branch é **shell-out**, não `git2::Repository::branch`: escrever a ref
  pelo libgit2 funcionaria, mas o passo pede checkout e upstream opcionais no mesmo gesto — e aí
  o git precisa escrever índice e worktree, disparar `post-checkout` e gravar
  `branch.<nome>.remote`/`.merge` respeitando o `branch.autoSetupMerge` do usuário.
- **2026-08-22** — Com checkout é `git switch --create`, **não** `git checkout -b`: o `checkout` é
  dois comandos num só (trocar de branch e restaurar arquivo), e é a metade que restaura arquivo
  que destrói trabalho sem aviso. Exige git ≥ 2.23; o piso do projeto é 2.30.
- **2026-08-22** — **A ordem das flags do `switch` importa**: `--create` pede um valor, então
  `git switch --create --track <nome>` faria o git tomar `--track` como o nome da branch. O
  tracking vai **antes** do `--create`, que fica colado no nome. No `git branch` não há esse
  risco, mas a montagem é a mesma para não haver duas ordens no mesmo arquivo.
- **2026-08-22** — `track: Option<bool>` tem **três** estados, igual ao `gpg_sign` do Passo 53:
  ausente respeita o `branch.autoSetupMerge` (que já liga o tracking sozinho ao partir de uma
  remota — o que o Passo 59 vai querer), `Some` força por flag, nunca por `git config`.
- **2026-08-22** — O ponto de partida é **oid ou nome de ref**, nunca revisão livre: `HEAD~2`,
  `:/assunto` e `main@{1}` são recusados com 400 antes de chegar ao git. É o mesmo raciocínio da
  validação do alvo do `--fixup` (Passo 54) — a UI sempre tem o oid completo do commit
  selecionado ou o nome da ref clicada —, e é também o que impede um valor começando com `-` de
  virar flag na linha de comando. Nome de branch reusa o `validate_branch_name` do `init`, para
  haver uma régua só no projeto inteiro.
- **2026-08-22** — `tip_of` faz `rev-parse --verify refs/heads/<nome>`, com o caminho inteiro e
  não o nome curto: um arquivo chamado `nova` na worktree tornaria `git rev-parse nova` ambíguo.
- **2026-08-22** — A rota devolve `{name, oid, repo}` — o `Repo` **depois**, porque com checkout o
  `HEAD` mudou e é dali que a barra de topo e o título da aba se atualizam sem um segundo
  request. A lista de refs **não** vem junto: é outra rota, com outro ritmo de invalidação
  (decisão do 57a), e o cliente a invalida.
- **2026-08-22** — Ponto de partida que não resolve é **400**, não 404: mesma régua que o oid de
  commit que não existe neste repositório já tinha (`GitError::InvalidCommit`). Branch que já
  existe é 409, worktree que seria sobrescrita é 409 (`WouldOverwrite`, com frase própria — o
  fluxo de stash automático é o Passo 60).
- **2026-08-22** — Aceite do Passo 58a (2026-08-22): `cargo test --workspace` — **186 testes**,
  tudo verde (7 novos de `exec::branch`: criar do `HEAD` sem mover o `HEAD`, criar de um commit
  antigo, `switch --create` deixando o `HEAD` na nova e o arquivo do último commit fora da
  worktree, `--track` gravando `branch.<nome>.merge`, nome repetido, partida inexistente e a
  tabela de nomes/partidas recusados antes do git). Clippy e `cargo fmt --check` limpos.
  Handshake HTTP completo contra um repositório sintético em `~/porc-aceite-58a` (três commits):
  criar em `ed32736` (o commit mais antigo) devolveu a branch apontando para ele; `checkout:true`
  devolveu o `repo` já com `head.name: "trabalhando"`; `track:true` sobre `main` gravou
  `refs/heads/main` no `branch.com-upstream.merge`; nome repetido deu 409, partida inexistente e
  `HEAD~2` e nome `-f` deram 400; sem cookie 401 e `repo_id` desconhecido 404. O `git branch -vv`
  **de verdade** confirmou as quatro branches, o upstream e o `HEAD`. Repositório apagado,
  entrada removida dos recentes e servidor derrubado ao final.

- **2026-08-21** — O confinamento do `fs/list` é em três camadas, nesta ordem: componente
  `..` no pedido → 403 sem tocar o disco; `fs::canonicalize` (404 se não existe); prefixo
  contra a raiz canônica → 403. **Cada entrada listada também é canonicalizada e conferida**:
  symlink dentro da raiz apontando para fora não aparece, em vez de aparecer e dar 403 ao ser
  clicado. A resposta devolve sempre o caminho canônico, nunca o que o cliente mandou.
- **2026-08-21** — Raiz padrão do navegador de pastas é a **home** do usuário
  (`routes::fs::default_root`), guardada canonicalizada em `AppState.fs_root`. O Passo 27
  troca por raízes de `config.toml`.
- **2026-08-21** — `is_repo` vive no `porc-git` (`discover.rs`) e é só teste de disco: `.git`
  existindo (diretório em worktree normal, **arquivo** em worktree linkada e submódulo) ou os
  três marcadores de bare (`HEAD` + `objects/` + `refs/`). Abrir um `git2::Repository` por
  entrada listada custaria caro e não diria mais nada.

- **2026-08-21** — No `FolderBrowser`, **Enter numa pasta que já é repositório confirma; nas
  outras, entra**. É a diferença entre "escolher" e "navegar" numa tecla só, e evita um
  segundo atalho para a ação principal. `→` sempre entra, `←`/Backspace sobem, `.` alterna
  ocultos. Subir de nível restaura o cursor na pasta de onde se veio (`cameFrom`).

- **2026-08-21** — O Passo 25 foi **quebrado em 25a (backend) e 25b (UI)**: junto passaria de
  9 arquivos. A quebra respeita a regra 4 porque 25a compila e serve sozinho — a UI só não
  usava a rota ainda.
- **2026-08-21** — `repo_id` = **sha256 do caminho canônico, 16 bytes em hex**. Derivado do
  caminho e não de um contador ou do segredo do boot: reabrir o mesmo repositório tem que dar o
  mesmo id, senão cache do cliente e lista de recentes apontariam para ids mortos a cada boot.
  Nova dependência: `sha2`.
- **2026-08-21** — `POST /api/v1/repos` é a **única** rota de git que aceita caminho, e usa o
  mesmo `routes::fs::resolve` do `fs/list` (por isso ele é `pub(crate)`). Depois dela tudo é
  `repo_id`; `GET /api/v1/repos/{id}` não sabe converter caminho nenhum.
- **2026-08-21** — `Git2Repo::open` usa `Repository::open`, **não** `discover`: `discover`
  subiria a árvore e uma pasta qualquer dentro de um repo passaria a "ser" o repo. Num app cujo
  confinamento é por caminho isso é surpresa demais, e o navegador já marca quais pastas são
  repositórios.
- **2026-08-21** — `Head` tem **três** estados (`branch`, `detached`, `unborn`), não dois mais
  um erro: `unborn` é o repo recém-criado, com branch e sem commit. Tratá-lo como erro
  obrigaria a UI inteira a lidar com "repo aberto mas quebrado" logo depois de um `git init`.
- **2026-08-21** — `git2` entra com `default-features = false` (sem `https`, `ssh`,
  `ssh_key_from_memory`). Rede é shell-out para o `git` do sistema — ligar TLS e libssh2 aqui
  só somaria dependência de build para código que nunca será chamado.
- **2026-08-21** — Em `git2` 0.21 `Reference::shorthand` e `symbolic_target` devolvem
  `Result`/`Result<Option<_>>`, não `Option`. Ref com nome não-UTF-8 cai em `detached` (mostra o
  hash) em vez de derrubar a abertura do repositório.
- **2026-08-21** — O libgit2 devolve worktree e gitdir com barra no fim; `path.components()
  .collect()` a remove. É o que faz o caminho de `RepoInfo` bater byte a byte com o
  `canonicalize` do servidor — que é de onde sai o `repo_id`.
- **2026-08-21** — Parâmetro de rota no axum 0.8 é `{repo_id}`, não `:repo_id`.
- **2026-08-21** — Zustand guarda **só o `repoId`** (a seleção, que é UI); quem o repositório é
  vem do TanStack Query. E não é persistido: o registry do servidor morre com o processo, então
  um id em `localStorage` apontaria para nada no boot seguinte.

- **2026-08-21** — `Index::open_or_memory` **nunca falha**: índice que não abre cai para
  `:memory:` com um `warn`. Um cache descartável não pode impedir o usuário de usar o git dele;
  o que se perde é a memória entre boots. SQLite em WAL + `synchronous = NORMAL` (perder as
  últimas escritas num crash custa reindexação, não dado do usuário).
- **2026-08-21** — Não existe feature `fts5` no `rusqlite` 0.40: FTS5 já vem compilado no
  SQLite do `bundled` e é SQL, não API do crate. O teste `fts5_esta_disponivel` fixa isso agora
  para o Bloco H não descobrir tarde.
- **2026-08-21** — `recents` guarda **caminho**, não `repo_id`: o id é derivado do caminho,
  então o caminho é o dado primário. `opened_at` em **milissegundos** (segundos empatariam entre
  dois repos abertos no mesmo segundo, e a ordem é a única informação da lista).
  `touch_recent_at` existe para o teste ser determinístico.
- **2026-08-21** — Recentes são recurso próprio (`/api/v1/recents`), não `/repos/recent`: são
  caminhos lembrados em disco, enquanto `/repos` são repositórios abertos neste boot. E **não
  há rota que abra um recente por id** — a UI manda o caminho para `POST /api/v1/repos`, que
  revalida o confinamento. O banco é um arquivo que o usuário pode editar.
- **2026-08-21** — Recente cuja pasta sumiu continua na lista, desabilitado (`available:
  false`). Sumir sozinho esconderia do usuário que ele moveu a pasta.

- **2026-08-21** — Config em `config.toml` com chaves **`snake_case`** (`scan_depth`), ao
  contrário do `camelCase` da API: quem escreve o arquivo é uma pessoa num editor. Chaves:
  `root` (com `~` expandido), `scan_depth` (padrão 3), `scan_limit` (padrão 200). Config
  **nunca derruba o boot** — ausente, ilegível, TOML inválido ou raiz inexistente viram `warn` e
  os padrões.
- **2026-08-21** — No macOS o `directories` põe config e dados **no mesmo lugar**
  (`~/Library/Application Support/porcelain/`), não em `~/.config`. O `~/.config/porcelain/` do
  `CLAUDE.md` é o caminho do **Linux**.
- **2026-08-21** — `AppState.fs_root` virou `AppState.settings` (`config::Settings`), que
  carrega raiz canônica + limites da varredura.
- **2026-08-21** — `discover::scan` tem três regras que seguram o custo: **não desce dentro de
  um repositório encontrado** (a varredura procura projetos, e descer num monorepo custaria
  minutos), **não segue symlink** (`DirEntry::file_type`, que não resolve — assim o confinamento
  não depende de checagem, e não há ciclo) e **pula ocultos**. Fila em vez de recursão.
  `scan_depth` conta níveis **abaixo** da raiz: 1 são os filhos diretos.
- **2026-08-21** — `GET /api/v1/fs/scan` **não aceita parâmetro nenhum**: raiz e profundidade
  vêm da config. Deixar o cliente escolher onde e quão fundo varrer seria dar a ele um `find` na
  máquina do usuário. Devolve `truncated` quando bateu no teto, para a UI não deixar o usuário
  procurando um repositório que ficou de fora.

- **2026-08-21** — Nasceu `porc-git/exec/`, e **ninguém monta um `Command` na mão**: todo
  shell-out sai de `exec::command`, que já traz `--no-optional-locks`, `GIT_TERMINAL_PROMPT=0`,
  `LC_ALL=C` (stderr estável para o mapeamento de erro do Passo 34), grupo de processos próprio
  e `stdin` fechado. `-C <path>` em vez de `current_dir`, que é global ao processo.
- **2026-08-21** — O timeout usa `select!` com o futuro **preso** (`tokio::pin!`), não
  `tokio::time::timeout`: envolver em `timeout` derrubaria o futuro na hora e o `kill_on_drop`
  mandaria `SIGKILL` sem período de graça. O caminho certo é `SIGTERM` no grupo → 2s de graça
  (o git remove `index.lock` e temporários) → `kill_on_drop` como último recurso.
- **2026-08-21** — `LOCAL_TIMEOUT` de 30s vale só para comando local. Comando de rede não pode
  ter teto total (um clone grande demora mais); lá o relógio é de **inatividade**, contra o
  progresso — Passo 31.
- **2026-08-21** — `git init` é shell-out e não `git2::Repository::init`, para respeitar
  `init.templateDir`, `init.defaultBranch` e os hooks de template do usuário. Branch inicial
  vazia = usa o `init.defaultBranch` dele; preencher "main" seria escolher por quem já escolheu.
- **2026-08-21** — No `init`, `name` é validado como **componente** de caminho (sem `/`, `\`,
  `\0`, `.`, `..`) e o resultado é **reconferido** pelo `resolve` depois de criado. Confinamento
  que se checa só na entrada é confinamento que se perde na próxima refatoração.
  `create_dir`, não `create_dir_all`.
- **2026-08-21** — `POST /api/v1/repos/init` devolve o repositório **já aberto**: um `init` que
  respondesse só "ok" obrigaria a UI a adivinhar o caminho canônico do que acabou de criar.
  Rota estática convive com `/repos/{repo_id}` — o matchit do axum dá prioridade ao literal.

- **2026-08-21** — **O estado do job vive no servidor, não na aba.** `GET /api/v1/jobs/{id}`
  devolve o estado completo (progresso, log recente, resultado), e é o mesmo objeto do evento
  `job.done`. O WebSocket é o caminho rápido, **não** a fonte da verdade — é isso que faz
  recarregar a aba no meio de um clone não perder nada.
- **2026-08-21** — `cancel(id)` **não** marca `cancelled` na hora: só dispara o token e volta.
  Quem finaliza é a task, depois de parar o processo e rodar a limpeza. Marcar cedo mentiria
  enquanto o `git` ainda estivesse vivo — e é nessa janela que a pasta parcial ainda existe.
- **2026-08-21** — Limpeza é registrada **antes** de começar (`Cleanup::RemoveCreatedDir`, que
  por construção só recebe pasta criada por nós) e roda **fora do lock** do registry:
  `remove_dir_all` numa árvore grande travaria a UI inteira. Job que termina em `done` tem a
  lista de limpeza esvaziada, para ninguém rodá-la depois por engano.
- **2026-08-21** — Os métodos finais do `JobHandle` (`done`, `fail`, `cancelled`) **consomem** a
  alça: o tipo garante que um job termina uma vez só, em vez de um `if` esquecido no meio da task.
- **2026-08-21** — Cliente lento no WebSocket recebe **`resync`**, não histórico: o servidor não
  guarda o que passou, o cliente reconsulta por HTTP. `MAX_RUNNING = 8` jobs simultâneos;
  `LOG_TAIL = 200` linhas por job.
- **2026-08-21** — O socket do cliente é um **objeto de módulo** com backoff próprio
  (250ms→5s), não um `useEffect`: reconectar é o caso normal (sleep, restart, aba em segundo
  plano). Assinaturas são do *cliente*, não da conexão, e são reenviadas no `onopen` — e toda
  conexão nova invalida a lista de jobs, porque entre a queda e a volta pode ter acontecido de tudo.
- **2026-08-21** — `useJobEvents()` é montado **uma vez**, na raiz: dois pontos de escuta
  aplicariam o mesmo evento duas vezes e uma linha de log apareceria em dobro.

- **2026-08-21** — O `--progress` do git separa as atualizações com **`\r`**, não `\n`: quem
  lesse esse stderr com `lines()` ficaria sem nenhum evento até o fim de cada fase e então
  receberia uma "linha" de quinze mil caracteres. `Splitter` corta em `\r` **e** `\n` e guarda o
  pedaço incompleto entre chunks (leitura de socket não respeita fronteira de linha).
- **2026-08-21** — `Enumerating` **não tem fração**: o git conta objetos sem saber quantos serão.
  `fraction()` devolve `None` ali, e a UI tem que saber desenhar barra indeterminada — é
  justamente a parte mais lenta de um clone grande, e inventar um número seria mentir nela.
- **2026-08-21** — O throughput vem do próprio git (`14.69 MiB/s`), não é recalculado: uma conta
  nossa daria um número diferente do que o usuário vê no terminal dele.
- **2026-08-21** — As amostras dos testes do parser são stderr **real**, capturado de
  `git clone --progress` de `rust-lang/log` e `tokio-rs/tokio`, com os `\r` e os espaços de
  preenchimento como o git os escreveu.

- **2026-08-21** — Comando de rede usa `exec::stream`, cujo relógio é de **inatividade**
  (`IDLE_TIMEOUT = 120s`), não de duração. Um clone grande demora; um clone travado fica mudo, e
  só o segundo merece morrer.
- **2026-08-21** — `clone::prepare` roda **antes** de `run` justamente para o chamador saber o
  destino e o `creates_target` antes de o git começar. Sem isso não dá para registrar a limpeza
  antes, e cancelar viraria adivinhação sobre o que pode ser apagado. `git clone` cria a pasta
  sozinho: `creates_target` é só "ela não existia".
- **2026-08-21** — `-c protocol.ext.allow=never` no clone. O transporte `ext::` do git **executa
  um comando arbitrário** (`git clone "ext::sh -c ..."`); o `--` impede a URL de virar flag, mas
  `ext::` é uma URL *válida* que roda shell. Recusado também no `prepare`, com mensagem própria.
- **2026-08-21** — `--depth` vem acompanhado de `--no-single-branch`: sozinho, `--depth` traz só
  a branch pedida, e o usuário descobriria isso ao procurar as outras.
- **2026-08-21** — O clone que termina **abre o repositório** e devolve o `Repo` no `result` do
  job; a UI semeia o cache e seleciona. Se o clone terminar mas o repositório não abrir, o job
  falha e a pasta **não** é apagada — o usuário fica com algo para investigar.
- **2026-08-21** — Cancelamento que chega entre o EOF do stderr e o `wait` faz o git morrer por
  sinal, com status de falha. `exec::stream` consulta o token antes de chamar isso de erro: quem
  apertou cancelar não pode ver uma mensagem de falha.

- **2026-08-21** — `SIGTERM` (e não `SIGKILL`) no cancelamento se pagou de forma verificável: ao
  cancelar um clone para dentro de uma pasta **preexistente e vazia**, o próprio git limpa o que
  havia escrito e a pasta fica vazia de novo. Com `SIGKILL` sobraria um `.git` pela metade que
  não podemos apagar (a pasta é do usuário).
- **2026-08-21** — Encerrar o app **cancela os jobs em andamento e espera a limpeza**
  (`Jobs::shutdown()`, teto de 6s), dentro do futuro de graceful shutdown do axum. Sem isso,
  fechar o porcelain no meio de um clone deixaria a pasta parcial para trás: a task morre com o
  runtime e a limpeza registrada nunca rodaria.
- **2026-08-21** — `rust-version` do workspace corrigido de **1.75 para 1.87**. Não é escolha
  nossa: é o maior `rust-version` da árvore de dependências (`wasip2` 1.87, `hashbrown`/`clap`
  1.85). O 1.75 era uma promessa que o `cargo build` de quem tem 1.80 desmentiria na hora.

- **2026-08-21** — O modo askpass **não é um subcomando**. O contrato de `GIT_ASKPASS` é o
  programa inteiro: o git roda `$GIT_ASKPASS "<prompt>"` e lê a primeira linha do stdout. Um
  `porc askpass …` fazia o `clap` recusar o prompt como "subcomando desconhecido" — erro real,
  visto no primeiro teste. Quem denuncia o modo é `PORC_ASKPASS_SOCKET` no ambiente, checado
  **antes** do `clap` e antes do log.
- **2026-08-21** — O segredo trafega por um socket unix num diretório efêmero **`0700`, com o
  socket `0600`**. O `chmod` do diretório vem antes de o socket existir: entre criar e restringir
  há uma janela, e no `/tmp` compartilhado essa janela é a diferença entre privado e público. Vão
  no ambiente o **caminho** do socket e o caminho do helper; o segredo, nunca.
- **2026-08-21** — Ambiente do helper: `GIT_ASKPASS` (credencial HTTPS), `SSH_ASKPASS`
  (passphrase da chave), `SSH_ASKPASS_REQUIRE=force` (sem isso o ssh ≥ 8.4 só chama o askpass
  quando não enxerga tty, e podemos ter herdado um) e `DISPLAY=:0` para o ssh antigo.
- **2026-08-21** — Ninguém responder faz o servidor **fechar o socket sem escrever nada**: o
  helper sai com erro e o ssh desiste da chave, em vez de tentar com passphrase vazia e queimar
  uma tentativa.
- **2026-08-21** — O pedido de senha em aberto vive **no snapshot do job** (`pendingPrompt`), não
  num store do cliente. É o que faz uma aba recarregada no meio de um clone por SSH reencontrar
  o campo de senha, em vez de olhar uma barra parada até o pedido expirar (180s).
- **2026-08-21** — Askpass é `#[cfg(unix)]`. No Windows não há socket unix em tokio e o git usa o
  Credential Manager; lá o clone com chave protegida ainda não passa pela interface.

- **2026-08-21** — A **ordem** das checagens em `exec::error::diagnose` é o que faz o mapeamento
  funcionar: o git empilha mensagens, e `Permission denied (publickey)` vem seguido do genérico
  `Could not read from remote repository`. Casar o genérico primeiro jogaria fora a única
  informação útil. Específico sempre antes.
- **2026-08-21** — O que não reconhecemos **não ganha conselho inventado**: `action` fica `None`.
  Chutar mandaria o usuário para o lugar errado, e o stderr cru está a um clique.
- **2026-08-21** — Host key que **mudou** não recebe conselho de "confie assim mesmo": é
  exatamente o que um ataque de intermediário produz. A ação diz isso e manda confirmar com quem
  administra o servidor. Um teste guarda essa frase.
- **2026-08-21** — `ServerMessage::JobDone` carrega `Box<JobSnapshot>`: a enum é **clonada para
  cada assinante** do canal de eventos, e o snapshot é três vezes maior que qualquer outra
  variante. No JSON não muda nada.

- **2026-08-21** — O cursor do log é a **fronteira do revwalk**: os commits já descobertos e
  ainda não emitidos no ponto de corte (pais dos emitidos + tips que a página não alcançou).
  Retomar dali custa o mesmo que começar, e é o que faz o preço da página não crescer com o
  scroll. Quem garante que isso é correto é o `Sort::TOPOLOGICAL`: um commit só sai depois de
  todos os filhos dele, então nenhum commit já emitido pode reaparecer como ancestral de quem
  ficou na fronteira. Sem topológico, a paginação por fronteira duplicaria commits.
- **2026-08-21** — O cursor é `v1.<oid>.<oid>…` (hex separado por ponto), **não base64** como
  dizia o BLOCO-D: são caracteres seguros em query string e não vale somar uma dependência
  antes de o cursor carregar bytes que não sejam oid. O prefixo de versão existe justamente
  para o Passo 37 trocar o formato (com o estado das lanes) sem que um cursor velho passe por
  novo. `encode_cursor`/`decode_cursor` moram no `model.rs`; para o cliente é opaco.
- **2026-08-21** — Pai que não existe no odb (clone raso, enxerto) é **filtrado** da fronteira:
  empurrá-lo na página seguinte faria o revwalk falhar inteiro. A borda do histórico raso é o
  fim da paginação, e vem como `nextCursor: null`.
- **2026-08-21** — O log anda a partir do **`HEAD`**, não de todos os refs. Refs como pontas
  extras do revwalk é assunto do Bloco F (sidebar de branches); enquanto isso, uma ponta só é
  o que a UI tem como mostrar.
- **2026-08-21** — `Commit` leva o **oid completo**, sem `short`: abreviar com garantia de
  unicidade custa uma consulta ao odb por commit (500 por página) e é decisão de apresentação.
  A UI corta os 7 primeiros. E a data é a do **autor** (é a que o `git log` mostra), enquanto
  a ordenação é a do committer (`Sort::TIME`) — depois de um rebase as duas divergem.
- **2026-08-21** — Cursor que não decodifica, ou que aponta para objeto de outro repositório,
  é `GitError::InvalidCursor` → **400**, não 500: é pedido malformado, não falha de leitura.
  O `warn` de "falha lendo repositório" passou a casar só com `GitError::Read`.

- **2026-08-22** — `useLog` (`web/src/lib/log.ts`) é um `useInfiniteQuery` que achata as páginas
  num array só de `Commit`, memoizado. `staleTime: Infinity`: nada por baixo dos pés muda o log
  enquanto a aba está aberta — quem vai invalidar isso é o WebSocket (`repo.changed`) no Bloco E,
  não `refetchOnWindowFocus`.
- **2026-08-22** — `CommitList` usa altura de linha **fixa** (22px, `estimateSize` sem medição),
  não `measureElement`: a linha é sempre uma única linha de texto truncado, então medir custaria
  um layout por linha para um valor que nunca varia. É o que permite ao `TanStack Virtual` pular
  direto para qualquer índice sem esperar o DOM.
- **2026-08-22** — O `useEffect` que busca a próxima página olha o **último item virtualizado**
  (`items.at(-1)`), não um sentinela de rolagem à parte: o virtualizador já sabe exatamente quais
  índices estão visíveis, e um segundo observador (IntersectionObserver) duplicaria essa
  informação. `move()` (teclado) também pode disparar a busca — `Home`/`End`/`PageDown` andam
  mais rápido que a rolagem por mouse e não podem esperar o próximo scroll para pedir mais.
- **2026-08-22** — Teclas de navegação são **j/k** além de setas (convenção vim, que o público do
  TortoiseGit-mas-teclado espera), com `PageUp`/`PageDown` medindo a viewport real
  (`clientHeight / ROW_HEIGHT`) em vez de um salto fixo de 10 como no `FolderBrowser` — linhas de
  log são bem mais numerosas por tela que entradas de pasta.

- **2026-08-22** — O cursor virou **`v2`**: não guarda mais só a fronteira do revwalk (os oids
  ainda por emitir), guarda o **estado inteiro das lanes** — cada coluna, livre ou esperando um
  oid, incluindo as livres **no meio** do vetor. É a posição de uma lane livre que decide onde a
  próxima página aloca uma lane nova; sem isso, duas páginas do mesmo grafo poderiam empacotar as
  colunas de jeitos diferentes dos dois lados da emenda. Lanes livres **no final** são cortadas
  do cursor (não mudam decisão nenhuma, só o tamanho). Um `v1` velho não decodifica como `v2` —
  é exatamente para isso que o prefixo de versão existe.
- **2026-08-22** — O algoritmo de lanes não usa fila/pilha separada: ele *é* o `lanes: Vec<Option<Oid>>`
  mutado durante o próprio loop do revwalk, sem estrutura auxiliar. Cada commit emitido é
  casado contra a(s) lane(s) que o esperavam (`Sort::TOPOLOGICAL` garante que todo commit
  emitido foi esperado por pelo menos uma); múltiplas lanes esperando o mesmo oid — o caso
  normal de dois branches reconvergindo numa base comum — colapsam na de menor índice e as
  outras se libertam. Pai que não existe no odb (fronteira de clone raso) não ganha lane: a
  linha termina ali de verdade, diferente de um pai que só ainda não chegou nesta *página*
  (isso o cliente decide sozinho comparando oid, Passo 38).
- **2026-08-22** — `Commit.parentLanes` é `(number | null)[]`, não `number[]`: `null` é
  especificamente a fronteira de clone raso. Não confundir com "o commit desse pai ainda não
  carregou" — a coluna já é conhecida (não é `null`) mesmo antes de o commit chegar; é assim que
  o Passo 38 consegue desenhar um stub na coluna certa sem esperar a página que o materializa.
- **2026-08-22** — Testes do algoritmo usam um repositório sintético **fork + merge** construído
  direto com `git2` (commits/blob/tree via API, sem `git init` externo nem `tempfile`): é o menor
  histórico que obriga tanto a convergência (duas lanes esperando o mesmo pai) quanto o
  nascimento de lane nova (segundo pai de um merge). O teste de paginação compara, commit a
  commit, `(oid, lane, parentLanes)` obtidos de uma vez só contra os mesmos obtidos com
  `limit=1` — a emenda de página mais extrema possível.

- **2026-08-22** — `LogGraph` não tem laço de animação próprio: ele lê `items` e
  `virtualizer.scrollOffset` no mesmo render que reposiciona as linhas de texto em
  `CommitList` (o `useVirtualizer` já dispara esse render a cada passo de rolagem) e redesenha
  num `useEffect`. Dois relógios (um `requestAnimationFrame` à parte para o canvas, outro para
  o DOM) é como grafo e lista ficam visivelmente defasados um do outro durante a rolagem.
- **2026-08-22** — A cor do traço/nó vem de um `<span>` de tamanho zero com a classe do token
  (`text-n-7`/`text-n-10`) lido via `getComputedStyle` uma vez por redesenho, não hardcoded:
  `canvas` não entende `var()` nem `light-dark()` diretamente, e é assim que o grafo respeita o
  tema sem duplicar a paleta em JS.
- **2026-08-22** — Aresta cujo pai ainda não carregou (página seguinte) vira um coto curto de
  tamanho fixo com uma marca — nunca uma linha até uma posição calculada, porque o cliente não
  sabe a que distância o destino está antes de a página chegar. `rowOf` (oid → índice) é
  recalculado só quando `commits` muda, e é essa mesma memoização, recalculada depois que a
  página seguinte chega, que faz a aresta "materializar" sozinha — sem código dedicado para
  completar arestas pendentes.
- **2026-08-22** — A largura da faixa do grafo (`gutterWidth`) é a maior lane já vista entre
  `commit.lane` **e** `commit.parentLanes`, não só a primeira: uma lane pode já ter sido
  alocada como destino de um merge antes de qualquer commit carregado ocupá-la de fato.
  Recalculada por `useMemo` sobre `commits` — uma vez por página nova, nunca por quadro de
  rolagem, no mesmo espírito do índice oid→linha do `LogGraph`.
- **2026-08-22** — Arestas são retas (sem curva em S), inclusive as que mudam de lane entre
  linhas adjacentes. O `canvas` recorta sozinho o que sai da própria altura, então uma aresta
  para um pai carregado mas fora da tela não precisa de recorte manual — só o caso "pai ainda
  não carregado" (sem posição nenhuma para mirar) precisa do coto. Curva suave é polimento
  visual, não corretude; fica para depois se incomodar na prática.

- **2026-08-22** — O Passo 39 foi quebrado em **39a (backend) e 39b (frontend)**, no mesmo
  espírito do 25a/25b, 26a/26b etc.: `RefMarker`/`RefKind` + `refs()` + a rota sozinhos já
  compilam, testam e respondem — a UI só ainda não os usa.
- **2026-08-22** — `refs()` **não pagina** e não faz parte de `LogPage`: o número de branches,
  remotas e tags de um repositório não cresce com o histórico (é da ordem de dezenas, no
  máximo centenas), diferente dos commits. Repeti-las a cada página do log seria banda
  desperdiçada; o cliente busca uma vez e cruza pelo `commit` (oid) contra o que o log já tem.
- **2026-08-22** — Só refs **diretas** (`reference.kind() == Direct`) viram marcador. Uma ref
  **simbólica** como `refs/remotes/origin/HEAD` (o ponteiro para a branch padrão do remoto) é
  descartada: a branch real para a qual ela aponta (`origin/main`) já aparece na varredura como
  ref direta, e listar as duas seria a mesma ponta duas vezes com nomes diferentes.
- **2026-08-22** — Tag vira marcador via `reference.peel_to_commit()`, que resolve tanto uma
  tag leve (a ref já é o commit) quanto uma anotada (a ref aponta para um objeto tag, que por
  sua vez aponta para o commit) com o mesmo código. Tag para blob ou árvore — rara, mas válida
  em git — não tem commit para marcar e é descartada, não vira erro.
- **2026-08-22** — `HEAD` destacado ganha um `RefKind::Head` **à parte**, marcador sintético
  com `name: "HEAD"`, em vez de tentar encaixar no `RefKind::Branch`: não há branch nenhuma
  para nomear, e fingir uma abriria uma exceção no contrato ("toda `Branch` tem uma ref real
  em `refs/heads/`") que o resto do código passaria a assumir.

- **2026-08-22** — `useRefs` mora em `lib/repo.ts`, não em `lib/log.ts`: refs são propriedade
  do **repositório** (existem independente de qualquer página do log carregada), a lista de
  commits é que as consulta para decorar linhas — a mesma razão pela qual a rota não pagina.
  `staleTime: Infinity` como o log, pelo mesmo motivo: quem vai invalidar isto é o WebSocket do
  Bloco E, não um refetch por foco de janela.
- **2026-08-22** — Diferenciação de `RefBadges` é só **traço e peso**, nunca cor: borda sólida
  para branch/remota, tracejada para tag; a ponta do `HEAD` (`isHead`) preenchida
  (`bg-n-6`) e em negrito, as outras só com borda. É a mesma regra do grafo — "sem
  arco-íris" vale para toda marca no log, não só para as lanes.
- **2026-08-22** — A lista de badges de uma linha tem `max-w-48 overflow-hidden`: um commit
  com muitas tags trunca a lista de marcadores em vez de espremer a mensagem, que é a
  informação principal da linha. Não existe "ver mais" ainda — se aparecer na prática, é
  ideia para o Bloco F, que já vai ter uma tela própria de refs.

- **2026-08-22** — O Passo 40 também foi quebrado em **40a (backend) e 40b (frontend)**, no
  mesmo padrão do 25a/25b, 39a/39b etc.
- **2026-08-22** — `commit_detail` faz o diff contra o **primeiro pai só** (contra a árvore
  vazia, na raiz) — a mesma simplificação que `git show` usa por padrão num merge, sem `-m`.
  Diff combinado dos dois lados de um merge é decisão de apresentação para revisitar se fizer
  falta na prática; o Passo 41 (hunks de verdade) herda a mesma simplificação.
- **2026-08-22** — `Diff::stats()` do git2 só dá o agregado do diff inteiro — não existe API
  pronta para inserções/remoções **por arquivo**. A contagem por arquivo sai de
  `Diff::foreach` com um `line` callback, somando `+`/`-` por caminho num `HashMap`; os dois
  testes contra commits reais deste próprio repositório (`2cea64a`, raiz, 15 arquivos; `0b61c6f`,
  50 arquivos) comparam esse agregado calculado à mão com o número que `git show --stat`
  mostra, e com a soma dos arquivos individuais — os três batem.
- **2026-08-22** — O oid do commit é validado e tratado como pedido malformado — **400**, não
  404 — igual ao cursor do log: `GitError::InvalidCommit` cobre tanto hex inválido quanto oid
  bem formado que não existe neste repositório. Mesmo raciocínio do Passo 35 para o cursor,
  reaplicado aqui.

- **2026-08-22** — Seleção de commit (`useCommitSelection`, `lib/commit.ts`) é uma store à
  parte de `useRepoSelection`, não um campo a mais nela: clicar num **pai** no painel de
  detalhe troca o assunto do painel sem mexer no cursor da lista — os dois só precisam
  coincidir quando é o cursor do log que muda, nunca o contrário. Se fossem a mesma store,
  qualquer navegação por pai teria que também mover (ou fingir que não move) a linha
  destacada na lista.
- **2026-08-22** — O `useEffect` que sincroniza `CommitList` → `useCommitSelection` depende só
  de `[selected, commits, selectCommit]` — deliberadamente **não** depende de nada do painel
  de detalhe. É essa assimetria que faz "clicar num pai" funcionar: o efeito só dispara quando
  o cursor do log muda, e navegar pelos pais no painel não mexe nele.
- **2026-08-22** — O painel de detalhe (`Detail` em `Shell.tsx`) parou de mostrar os metadados
  do repositório (caminho, `head`, `repoId`) assim que há um commit para mostrar — a lista de
  metadados era só um placeholder até o log existir. Repositório `unborn` (sem commit nenhum)
  ainda cai numa mensagem própria, porque não há o que selecionar.
- **2026-08-22** — Datas de assinatura são formatadas deslocando o `Date` pelo `offset` do
  autor/committer e exibindo como se fosse UTC (`timeZone: "UTC"` no `toLocaleString`), com o
  deslocamento mostrado à parte (`UTC±hh:mm`) — é o único jeito de o `Intl` mostrar o fuso de
  quem assinou sem duplicar por cima do fuso do navegador de quem está lendo.

- **2026-08-22** — O Passo 41 também foi quebrado em **41a (backend) e 41b (frontend)**.
- **2026-08-22** — `GET .../diff` pede **um arquivo por vez** (`?path=`), não o commit inteiro:
  um commit pode tocar centenas de arquivos, e mandar todos os hunks de uma vez na mesma
  resposta seria o oposto de "sob demanda" — a UI só busca o diff do arquivo que o usuário
  abriu. `path` é sempre o lado **novo** do delta (o mesmo valor que `FileChange.path` do
  Passo 40 já devolveu), nunca o `oldPath` de um rename.
- **2026-08-22** — **Descoberta que quase furou o teste do arquivo binário**: a flag `BINARY`
  de um `DiffDelta` **não** vem preenchida por `diff_tree_to_tree` sozinho — o libgit2 só
  inspeciona o conteúdo (a heurística de `\0` nos primeiros bytes) ao montar o `Patch` de
  verdade. Testar `delta.flags()` **antes** de criar o `Patch` sempre dava `0x0`, até para o
  `.woff2` mais óbvio; o que sai marcado é o `patch.delta()` **depois** do `Patch::from_diff`.
  Confirmado com um binário ad hoc antes de mexer no código de produção. Sem esse teste, a
  detecção de binário simplesmente não funcionaria nunca — silenciosamente, sem erro nenhum.
- **2026-08-22** — `FileDiff` é enum de **três** formas (`Text`/`Binary`/`NotUtf8`), não hunks
  vazios mais uma flag: um `Vec` vazio por "binário" obrigaria a UI a adivinhar por que não
  tem hunk nenhum. Mesmo raciocínio do `Head` de três estados do Passo 25.
  `NotUtf8` derruba o arquivo **inteiro**, não linha a linha: uma linha em UTF-8 e a seguinte
  em mojibake (de `from_utf8_lossy`) seria pior que os dois avisos claros e separados.
- **2026-08-22** — `FileNotInCommit` (caminho que este commit não tocou) é **404**, diferente
  de `InvalidCommit`/`InvalidCursor` (**400**): o pedido está bem formado, é o recurso que não
  existe — a mesma distinção que `RepoError::Unknown` já fazia para um `repo_id` desconhecido.

- **2026-08-22** — Fechar o arquivo aberto ao trocar de commit é ajuste de estado **durante a
  renderização** (comparar `oid` contra um `oidOfOpenFile` guardado e resetar os dois juntos se
  divergirem), não um `useEffect`. O oxlint (`react(set-state-in-effect)`) sinalizou o efeito
  original: `setState` síncrono dentro de `useEffect` é sempre um render a mais que o padrão
  "ajustar estado durante o render" evita.
- **2026-08-22** — Lado a lado pareia remoção com adição **por índice dentro da corrida**
  (todas as remoções de um trecho contra todas as adições daquele mesmo trecho, na ordem),
  não por semelhança de conteúdo (LCS). É a simplificação deliberada de um viewer leve: o lado
  mais curto de uma corrida desigual fica com célula em branco, sem tentar adivinhar qual
  linha "parece" com qual.
- **2026-08-22** — Painel de detalhe é estreito demais para lista de arquivos e diff ao mesmo
  tempo, então abrir um arquivo **substitui** a lista (com um "← caminho" para voltar) em vez
  de expandir inline — mesma lógica de espaço que já vale para a lista de branches na sidebar.
- **2026-08-22** — Diferente do resto do Passo 40/41, `useFileDiff` **não** tenta usar
  `FileChange.binary` (já conhecido do diffstat) para pular a requisição de um arquivo binário:
  o endpoint de diff já responde `{kind:"binary"}` de qualquer forma, e checar dos dois lados
  seria duas fontes de verdade para a mesma pergunta. Uma requisição a mais por arquivo binário
  é troco barato pela simplicidade.

- **2026-08-22** — **Passo 42 não precisou de metade frontend**: o `JobsPanel` do Bloco C já é
  genérico por `job.kind` (mostra fase, detalhe, barra indeterminada, "ver detalhes", cancelar
  — sem `match` nenhum sobre o tipo do job), então um job `"index"` aparece e termina sozinho,
  sem tocar em nenhum arquivo de `web/`. Confirmado batendo o job real por HTTP: aberto o
  repositório, `GET /jobs` já trazia `kind:"index"` progredindo e depois `done` com
  `result:{commits:N}`, exatamente o formato que o `JobsPanel` já sabe desenhar.
  Diferente dos passos 25, 26, 28, 31, 39, 40, 41 (todos com metade de UI de verdade), este
  ficou só no backend — dividir em "a/b" teria criado um "b" vazio.
- **2026-08-22** — `index_job::maybe_spawn` desiste **sem criar job nenhum** em dois casos:
  a) o repositório já está indexado até o `HEAD` atual; b) `Jobs::create` bate no teto de jobs
  concorrentes (`MAX_RUNNING = 8`). No caso b) desiste em silêncio (`tracing::debug!`, não
  `warn!`) — indexação não é essencial o bastante para brigar por vaga com um clone de
  verdade, e o próximo repositório aberto tenta de novo.
- **2026-08-22** — A indexação é **substituição total**, não delta real: `replace_commits`
  apaga e reinsere tudo numa transação só, e o "pular se já indexado" olha só se
  `indexed_tip == HEAD` atual — **não** o delta desde a última indexação que o `BLOCO-D.md`
  descreve. É a simplificação deliberada da v1 (documentada, não escondida): correta sempre
  (reindexar tudo nunca deixa o índice errado), só não é o mínimo de trabalho possível depois
  de um único commit novo. Vira delta de verdade — parando o `walk_for_index` no primeiro oid
  já conhecido — se o custo incomodar em repositórios muito grandes reabertos com frequência.
- **2026-08-22** — Todos os commits ficam em memória (`Vec<CommitRow>`) até o fim da varredura,
  em vez de gravar em lotes: `replace_commits` já é uma transação atômica por design (índice
  velho intacto é sempre melhor que índice pela metade se o job morrer no meio), e gravar aos
  pedaços quebraria essa garantia sem necessidade — 100k commits cabem em poucas dezenas de MB.
- **2026-08-22** — Cancelamento é checado **a cada commit** dentro do fecho passado a
  `walk_for_index` (`handle.is_cancelled()`), não só entre lotes: é barato (um load atômico) e
  é o que faz cancelar um índice de 100k commits responder na hora, não no próximo lote de
  2000.

- **2026-08-22** — `commits_fts` é uma tabela FTS5 **à parte** de `commits`, não `content=`
  externo com gatilhos: como `replace_commits` já apaga e reinsere tudo a cada reindexação
  (Passo 42), sincronizar as duas manualmente na mesma transação é mais simples que manter
  gatilhos de INSERT/UPDATE/DELETE que essa estratégia de substituição total nunca dispararia
  de qualquer jeito (não há UPDATE linha a linha, só apagar tudo e inserir tudo de novo).
- **2026-08-22** — Toda palavra da busca vira **frase entre aspas** (`"palavra"`) antes de
  virar `MATCH`, com `"` interna dobrada (`""`) para escapar — é o que impede `AND`, `OR`,
  `NOT`, `-prefixo` ou uma aspa solta digitados pelo usuário de virarem sintaxe de operador do
  FTS5 e derrubarem a consulta. Só a **última** palavra ganha o sufixo `*` de prefixo: é o que
  faz a letra recém-digitada já filtrar sem esperar a palavra terminar (a busca por mensagem e
  autor tem que ser útil a cada tecla, não só em palavras completas) — teste dedicado cobre a
  entrada perigosa (`AND`, `-erro`, aspa solta) não falhando.
- **2026-08-22** — `q` vazia devolve lista vazia **sem consultar o SQLite**: no vocabulário
  desta busca, caixa limpa é "sem filtro" (mostra o log inteiro), não "filtra por nada" (que
  daria zero resultados se fosse tratado como consulta FTS5 de verdade — `MATCH ''` é erro de
  sintaxe, não resultado vazio).
- **2026-08-22** — O teste do orçamento de performance do Passo 43 usa **50 mil** linhas
  sintéticas (maior que qualquer repositório real medido neste projeto até agora) e cobra
  menos de 100ms — mais folgado que os 20ms do `BLOCO-D.md` de propósito, porque é `cargo test`
  em debug, não o binário de release medindo de verdade; a intenção é pegar uma regressão
  grosseira (ex.: um `LIKE` no lugar do `MATCH`), não validar a meta exata.
- **2026-08-22** — `search_commits` devolve as linhas prontas (`author`, `email`, `time`,
  `summary`), não só `oid`: o cliente pode mostrar um resultado de busca que **não está** numa
  página do log já carregada (busca cobre o histórico inteiro indexado; a paginação do log só
  cobre o que foi rolado até agora), então a resposta não pode depender de o commit já estar
  na lista do cliente.

- **2026-08-22** — Resultado de busca é uma **lista achatada** (`SearchResults`), sem grafo:
  a busca cobre o histórico indexado inteiro, não só o que o log paginou até agora, e o
  conjunto de resultados não é contíguo no grafo original — recalcular lanes para um
  subconjunto arbitrário é problema bem maior que "busca por mensagem e autor" pede. Clicar
  num resultado ainda preenche o painel de detalhe normalmente (`useCommitSelection` já era
  independente da lista do log desde o Passo 40).
- **2026-08-22** — Debounce mora em `lib/search.ts` (`useDebouncedValue`, genérico) e não no
  componente: `useSearch` já recebe uma query estável, e a rota nunca vê um request por tecla
  numa digitação normal — 150ms, curto o bastante para parecer ao vivo.
- **2026-08-22** — Busca vazia (ou só espaço) é tratada **duas vezes**, de propósito: o
  `Log` nem monta `SearchResults` (mostra `CommitList` normal), e `useSearch` também não
  consulta o servidor se isso um dia mudar (`enabled: trimmed.length > 0`). Sem `content=`
  externo nem gatilho FTS5, redundância barata é melhor que um dos dois lados divergir.

- **2026-08-22** — O Passo 44 também foi quebrado em **44a (backend) e 44b (frontend)**. E,
  diferente de todos os `a` anteriores neste bloco, o 44a **não tocou a rota**: `search_commits`
  manteve a mesma assinatura, então `routes::repos::search` (Passo 43a) já passa a suportar
  hash/`autor:`/`depois:`/`antes:` sem uma linha mudar — a sintaxe unificada é decisão de
  parsing dentro do `porc-index`, não de contrato HTTP.
- **2026-08-22** — Hash colado **pula o FTS5 inteiro**: um único token hex de 4-40 caracteres
  vai direto a `WHERE repo_id=? AND oid LIKE 'prefixo%'` contra a tabela `commits` normal — "com
  índice" no `BLOCO-D.md` quer dizer a chave primária `(repo_id, oid)`, não busca textual. Um
  hash não é texto para relevância nenhuma; é um valor exato (até onde foi digitado).
  Consequência aceita: uma palavra que por acaso só usa `a-f0-9` (`"cafe"`, `"deed"`, 4+
  letras) também cai nesse caminho — a mesma ambiguidade que o próprio `git` tem com hashes
  curtos, documentada e não meia-solução.
- **2026-08-22** — Sem texto livre nenhum (só `autor:`/`depois:`/`antes:`, ou nada), a consulta
  vai pela tabela `commits` normal (`ORDER BY ts DESC`), não pelo FTS5: relevância (`rank`) só
  faz sentido quando há o que combinar por texto. Com texto livre presente, os filtros
  estruturados viram condições extras na mesma consulta FTS5 (`AND author LIKE ? AND ts...`) —
  é a mesma tabela `commits_fts` do Passo 43, só com mais `WHERE`.
- **2026-08-22** — `AAAA-MM-DD` vira segundos-desde-a-época por conta própria
  (`days_from_civil`, o algoritmo de Howard Hinnant para dia-civil→dias-desde-1970), sem
  dependência de data nenhuma: é uma multiplicação e algumas divisões inteiras, e trazer um
  crate de calendário para só isso seria peso para nada. `depois:` é inclusive, `antes:` é
  **exclusive** — "antes do dia", não "até o dia" (`antes:2024-06-20` não inclui commits feitos
  naquele dia).
- **2026-08-22** — Token com prefixo reconhecido mas inválido (`autor:` vazio, `depois:` com
  data que não parseia) é **ignorado silenciosamente**, não erro: `autor:` sozinho é um estado
  passageiro normal no meio da digitação (o usuário ainda vai completar o nome), e derrubar a
  busca inteira por causa disso seria pior que só não filtrar por aquele critério ainda.

- **2026-08-22** — "Salta direto para o commit" é `useCommitSelection.select(oid)`, não uma
  navegação de tela: o painel de detalhe já é independente da lista do log desde o Passo 40,
  então preencher a seleção já é o suficiente para o commit aparecer — sem precisar sair da
  busca, fechar a caixa ou trocar de view. O critério para disparar é estrito **de propósito**
  (`HASH_LIKE.test(query) && results.data.length === 1`): mais de um resultado é ambíguo
  (prefixo curto demais) e não dispara nada sozinho — o usuário escolhe clicando.
  `HASH_LIKE` no cliente espelha `is_hash_like` do `porc-index` (mesma regra, 4-40 hex), mas
  são só regex isoladas — não vale compartilhar código entre um binário Rust e um bundle JS
  por uma linha de regex.

- **2026-08-22** — O Passo 45 foi quebrado em **três**, não dois: **45a** (job de filtro por
  caminho), **45b** (autocomplete de caminho) e **45c** (frontend) — os dois primeiros são
  recursos de backend genuinamente independentes (um filtra o log, o outro lista arquivos da
  árvore), e juntá-los no mesmo "a" passaria do limite de arquivos por passo sem necessidade.
- **2026-08-22** — Filtro por caminho é **shell-out puro, sem SQLite e sem `porc-index`** —
  decisão já fechada no `CLAUDE.md` ("paths de commits não são indexados"). `exec::stream`
  (Passo 31, construído para o progresso do clone) só lia stderr; ganhou um parâmetro `Pipe`
  (`Stdout`/`Stderr`) para este passo poder ler o **stdout** do `git log`, que é onde o
  resultado de verdade sai — o clone continua lendo stderr, só passou a dizer isso
  explicitamente (`Pipe::Stderr`) em vez de implícito no nome da função.
- **2026-08-22** — `git log -z --format=%H%x00%an%x00%ae%x00%at%x00%s -- <path>` end-to-end
  testado byte a byte (`xxd`) antes de escrever `RecordSplitter`: com `-z`, o git termina
  **todo** campo do formato em NUL, inclusive o último de cada commit — não sobra `\n` nenhum
  para confundir o parser. Sem essa checagem manual, a suposição errada mais provável seria
  achar que só `-z` já bastava (sem `%x00` no format) ou vice-versa.
- **2026-08-22** — `RecordSplitter` corta em bytes crus (`Vec<u8>`), não em `String` como o
  `Splitter` do `--progress` (Passo 30): frontier de campo pode cair no meio de um caractere
  multi-byte UTF-8, e só decodificar (`from_utf8_lossy`) depois de já ter um campo **completo**
  evita partir um caractere ao meio entre dois `push()`.
- **2026-08-22** — O job devolve o resultado **inteiro** em `job.done` (`{"commits": [...]}`),
  não em pedaços pelo WebSocket — "streaming" aqui é entre o `git` e o servidor (stdout lido
  aos poucos, sem carregar o histórico inteiro na memória do processo do git de uma vez), não
  entre servidor e cliente. Entrega incremental de verdade para o cliente é do Passo 46
  (pickaxe), cujo aceite exige "primeiros resultados quase de imediato" — o aceite deste passo
  ("ver só os commits que tocam o arquivo") não pede isso, e implementá-lo aqui seria
  antecipar trabalho que o passo seguinte já vai precisar fazer de qualquer forma.
- **2026-08-22** — `path-filter` reaproveita `porc_index::commits::SearchHit` como formato do
  resultado (mesmos campos: oid/author/email/time/summary), mesmo sem tocar o índice —
  `porc-server` já depende de `porc-index` por outro caminho (a busca), e criar uma quarta
  struct quase idêntica só para isto teria sido duplicação sem motivo. O nome "SearchHit" é um
  pouco estranho aqui (isto não é FTS5), mas é exatamente a mesma forma de dado.

- **2026-08-22** — Autocomplete de caminho é **git2**, não shell-out: diferente do filtro por
  caminho em si (Passo 45a, que varre histórico e por isso é `git log` streaming), isto só lê a
  árvore de `HEAD` — estrutura, não texto, exatamente a fronteira que o `CLAUDE.md` já traçava
  entre os dois mundos.
- **2026-08-22** — `list_paths` resolve o **diretório** do prefixo (`root_tree.get_path`) e
  itera só aquele nível (`tree.iter()`, não recursivo), em vez de andar a árvore inteira e
  descartar o que não bate: um monorepo com dezenas de milhares de arquivos tornaria uma
  varredura completa por tecla digitada perceptível, enquanto listar um diretório é O(entradas
  daquela pasta). Prefixo que resolve para um arquivo, ou para um caminho que não existe, é
  "nada para completar" (`Ok(vec![])`), não erro — os dois são estados normais de alguém ainda
  digitando.
- **2026-08-22** — Diretório sai com `/` no final (`"crates/"`), arquivo sem: é o mesmo sinal
  que um `ls` colorido dá, e é o que permite à UI decidir se o Enter/Tab deve continuar
  completando ali dentro ou fechar a busca com aquele caminho.

- **2026-08-22** — `FlatCommitList` foi **extraído** de `SearchResults` neste passo: a mesma
  linha clicável (hash/mensagem/autor/data relativa) que a busca por mensagem já desenhava
  serve sem alteração nenhuma para o filtro por caminho — os dois produzem exatamente
  `SearchHit[]`. Extrair só quando a segunda cópia ia mesmo acontecer (não antes, no Passo 43)
  é a mesma disciplina que "não repita até doer" pede.
- **2026-08-22** — `PathFilter` **não reimplementa** progresso nem cancelar: o job
  `path-filter` já aparece e termina sozinho no `JobsPanel` genérico (a mesma descoberta do
  Passo 42 para o job de indexação), então a única peça nova de UI de fato é a caixa com
  autocomplete e a lista de resultado — lidos do **mesmo** cache de jobs que o `JobsPanel` usa
  (`usePathFilterJob` só faz `useJobs().data?.find(...)`), sem um caminho de estado paralelo.
- **2026-08-22** — Mensagem e caminho são **dois botões na mesma faixa**, não uma segunda
  linha acima da caixa: o painel de detalhe já é estreito (Passo 41 já bateu nesse limite), e
  uma faixa a mais só para alternar de modo custaria altura que a lista de commits precisa
  mais. `Tab` no campo de caminho completa para a primeira sugestão — se ela for uma pasta
  (termina em `/`), o autocomplete do nível de dentro já dispara sozinho, sem código dedicado
  a "descer um nível".

- **2026-08-22** — `path_filter.rs` virou o lar dos **dois** filtros: `by_path` (Passo 45) e
  `by_content` (este passo) compartilham o mesmo `run` interno (`git log -z --format=…` comum)
  — só o argumento entre o formato e o `--` muda (`-- <path>` contra `-S<valor>`/`-G<valor>`).
  Exatamente o reaproveitamento que o próprio comentário do módulo já previa desde o Passo 45,
  sem precisar adivinhar a forma exata até este passo chegar de verdade.
- **2026-08-22** — `-S<valor>`/`-G<valor>` vai **grudado num token só** (a sintaxe curta do
  git), nunca `-S` e o valor como dois argumentos separados: um valor que começa com `-` ou
  parece outra flag nunca é lido como tal, porque só existe *depois* do prefixo dentro do
  mesmo item de `argv` — mesma defesa que o `--` já dava ao `by_path`, só que sem precisar do
  `--` nenhum aqui (pickaxe não tem pathspec).
- **2026-08-22** — **A entrega incremental de verdade nasce aqui**, não no Passo 45: `JobSnapshot`
  ganhou `hits: Vec<Value>` (cauda capada em `LOG_TAIL`, igual ao `log`) e `ServerMessage`
  ganhou `job.hit` — um evento por commit achado, publicado por `handle.hit()` de dentro do
  próprio fecho de streaming do `git`, não acumulado para o fim. O `result` final do `done`
  ainda carrega a lista **completa** (sem corte), então quem só olha o fim (reconectou tarde,
  por exemplo) não perde nada — o corte só vale para a cauda ao vivo.
- **2026-08-22** — Medido contra um repositório sintético de 6000 commits com conteúdo
  alternado (`needle-marker-0`/`needle-marker-1` a cada commit, pensado para o `-S` mudar de
  contagem quase toda vez): o job de pickaxe terminou em menos de 100ms, os `hits` da cauda já
  vieram capados nos 200 mais recentes no primeiro poll. Rápido demais para observar
  crescimento incremental por HTTP polling (o que é uma boa notícia de performance, não uma
  falha de teste) — a garantia de que cada commit vira um evento **assim que é encontrado**,
  não só no fim, está no teste direto de `jobs.rs` (`hit_publica_e_acumula_no_snapshot`), que
  não depende de cronometrar operação nenhuma de git.
- **2026-08-22** — `PathFilterStartError` virou **`RepoJobStartError`**: o mesmo par de erros
  (`RepoError`/`JobsError`) que `path-filter` já usava serve `pickaxe` sem alteração — um
  rename honesto para o que o tipo sempre foi, não uma generalização especulativa.

- **2026-08-22** — "Cancelado e reiniciado a cada tecla" é uma **ref**, não outro `useState`:
  o `jobId` da busca em andamento (`runningJobId`) precisa do valor de **antes** do efeito
  rodar para cancelar o certo, e só depois passa a apontar para o novo — um `useState`
  criaria uma corrida entre ler o valor antigo e gravar o novo dentro do mesmo efeito.
- **2026-08-22** — Query vazia (`trimmedValue === ""`) é tratada **fora** do efeito
  (`activeJobId = trimmedValue === "" ? null : jobId`, calculado durante a renderização), não
  com um `setJobId(null)` dentro dele: o oxlint (`react(set-state-in-effect)`) pegou a versão
  antiga — `setState` síncrono logo na entrada de um efeito quase sempre quer dizer que o
  valor era derivável sem estado nenhum, e aqui era exatamente esse o caso.
- **2026-08-22** — `startRef`/`cancelRef` guardam a mutação mais recente para o efeito chamar
  sem precisar dela na lista de dependências (`start`/`cancel` são objetos novos a cada
  render; colocá-los nas dependências reiniciaria a busca a cada render, não só a cada tecla
  ou troca de modo). A atualização das refs mora num `useEffect` **próprio, sem lista de
  dependências** (roda a cada render) — nunca escrita direto no corpo do componente: o oxlint
  (`react(refs)`) sinalizou que gravar `ref.current` durante a renderização é o que o React
  quer evitar, mesmo sendo um padrão comum em React "cru".
- **2026-08-22** — Enquanto o job está `running`, a lista mostra `job.hits` (a cauda ao vivo,
  capada em 200); assim que chega a `done`, passa a mostrar `job.result.commits` (a lista
  completa, sem corte) — é a troca de fonte que faz a tela nunca "perder" resultados que
  chegaram além da cauda, mesmo tendo mostrado só os últimos 200 enquanto a busca ainda corria.
- **2026-08-22** — **Bloco D concluído.** As três buscas do log (mensagem/autor via FTS5,
  caminho e conteúdo via shell-out streaming) compartilham a mesma casca de UI (`Log.tsx`, um
  seletor de três modos numa faixa só) e o mesmo componente de lista achatada
  (`FlatCommitList`), mas têm mecanismos de fundo completamente diferentes — índice SQLite
  para uma, jobs canceláveis com `git log` streaming para as outras duas — exatamente a
  divisão de fronteiras que o `CLAUDE.md` já desenhava antes de o bloco começar.

## Pendências / ideias anotadas

- Os aceites **visuais** do Bloco E (painel de status navegado só pelo teclado, seleção de linha
  no diff, confirmação de descarte, caixa de mensagem com a régua, a aba de comparação) foram
  verificados por build limpo, lint limpo, inspeção do código e pelo contrato HTTP completo
  batido com o binário de release — **não** por clicar de verdade num navegador. Vale o usuário
  abrir uma vez e fazer o ciclo inteiro à mão.
- O `StatusPanel` não tem rolagem automática para o cursor (`scrollIntoView`): numa lista de
  status maior que a tela, `j`/`k` movem o destaque para fora da vista. O `CommitList` resolve
  isso pelo virtualizador; aqui a lista não é virtualizada.
- Não há atalho de teclado para trocar de aba no centro (log/status/comparar). Vira comando na
  command palette (Ctrl+K) quando ela existir.
- O descarte por **linha** existe no servidor (o mesmo `HunkPick` com `lines`), mas a UI só
  oferece o hunk inteiro no botão destrutivo quando não há linhas marcadas — marcar linhas e
  clicar em "descartar" já funciona, só não está anunciado em lugar nenhum.
- `git checkout -- <paths>` no discard volta ao **índice**, não ao `HEAD`. É a semântica certa e
  a do terminal, mas a confirmação não diz isso — quem tem algo preparado do mesmo arquivo pode
  esperar voltar ao último commit.

- **Assinar um commit GPG de verdade não foi exercitado** (Passo 53): não há chave nesta
  máquina. Verificados só o caminho de erro (frase de pinentry) e o de desligar a assinatura.
- O `gpg` não passa pelo askpass do Passo 33 (`GIT_ASKPASS`/`SSH_ASKPASS` são do git e do ssh).
  Uma chave com passphrase e sem `gpg-agent` já destravado sempre vai falhar com a frase do
  `SigningFailed`. Ligar o `gpg` ao mesmo socket exigiria um `pinentry-program` próprio — dá
  para fazer, mas é assunto de outro bloco.
- Commit feito pela interface **não atualiza o índice de busca** (FTS5): quem reconstrói é o job
  de indexação, que só nasce ao abrir o repositório. Um commit novo só aparece na busca no boot
  seguinte.

- `POST /api/v1/ping` e `GET /api/v1/whoami` continuam provisórias (existem para exercitar
  CSRF e auth). `ping` sai no Bloco E. O `whoami` **não** virou informação de repositório como
  estava previsto: quem faz isso é `GET /api/v1/repos/{id}`, e o `whoami` sobrou. Pode sair
  junto com o `ping`.
- Falta a camada de `rate_limit` (`tower_governor`) prevista na ordem do router. Já existem
  três rotas caras esperando por ela: `fs/list`, `fs/scan` (varre diretório de verdade) e
  `repos/init`.
- Falta a **checagem de `git >= 2.30` no boot**, que é decisão fechada do `CLAUDE.md` e não é
  passo de nenhum bloco. Não pode morar no `porc-cli` (ele não conhece git); o lugar é o
  `bind()` do `porc-server` chamando um `porc_git::exec::version()`.
- Os aceites **visuais** dos Passos 17, 18, 22, 24, 36, 38, 39b, 40b, 41b, 43b, 44b e 45c (três
  painéis na densidade certa, arrastar e recarregar, aba com título e ícone, navegar por
  teclado até um repo, rolar o log sem engasgo, o grafo acompanhando a rolagem, os marcadores
  de ref na linha certa, o painel de detalhe preenchendo ao selecionar, o diff de um arquivo de
  texto e de um binário, a busca filtrando ao digitar, um hash colado saltando direto, o filtro
  por caminho com autocomplete) foram verificados por build limpo, lint limpo, inspeção do
  código e pelo contrato HTTP completo batido contra repositórios sintéticos e este próprio
  repositório (3000 commits em linha reta para a paginação do Passo 36; um fork+merge de
  verdade para `lane`/`parentLanes` no Passo 38; `main`/`isHead` no Passo 39a/b; o commit raiz
  deste projeto batendo com `git show --stat` no Passo 40a/b; `web/src/app/Shell.tsx` e uma
  fonte `.woff2` deste próprio histórico no Passo 41a/b; a busca por "passo" achando o commit
  certo no Passo 43a/b; o hash `9db77b8` achando exatamente um commit no Passo 44a/b;
  `PROGRESSO.md` trazendo os 3 commits certos e `crates/porc-` completando as 4 crates no
  Passo 45a/b/c), **não** por clicar de verdade num navegador. Vale o usuário abrir uma vez,
  medir o tempo até a primeira pintura e navegar pelo log com as refs, o detalhe, o diff, a
  busca e o filtro por caminho num repositório de verdade, como o `BLOCO-D.md` pede.
- Abrir uma pasta **dentro** de um repositório não abre o repositório (é `open`, não
  `discover`). Se isso incomodar na prática, o lugar de resolver é o `POST /api/v1/repos`,
  reconferindo o resultado do `discover` contra a raiz do confinamento.
- A sidebar mostra as refs mas ainda não **age** sobre elas: criar (58), trocar (59), renomear e
  deletar (61) chegam como menu de contexto/atalho nos passos seguintes do Bloco F.
- A `ThemeSwitch` mora na barra de topo por falta de lugar melhor. Quando houver command
  palette (Ctrl+K), o tema vira comando e a barra fica só com repo e branch.
- O aceite do Passo 33 foi feito pelo caminho de **credencial HTTPS** (`GIT_ASKPASS`), que é o
  mesmo socket, o mesmo evento e a mesma rota: o git pediu usuário e senha, a interface respondeu
  e o servidor remoto confirmou ter recebido (`Access denied`). O ramo **específico de SSH**
  (`SSH_ASKPASS` + `SSH_ASKPASS_REQUIRE=force`) está ligado mas **não foi exercitado** — exigiria
  criar uma chave com passphrase e um servidor SSH na máquina do usuário. Vale um teste manual.
- Matar o `porc` com sinal que ele não trata (SIGTERM, SIGKILL) deixa o diretório do socket de
  askpass em `$TMPDIR/porcelain-askpass-*`. O `Drop` e o `Jobs::shutdown()` cobrem a saída
  normal; uma faxina de diretórios órfãos no boot resolveria o resto.
- O botão "job de teste" na barra de status é provisório: existe para exercitar o canal sem
  depender de rede. Sai quando o clone (Passo 31) der um job de verdade para a mesma infra.
- O `teto global de jobs por repo` do BLOCO-C está implementado como teto **global**
  (`MAX_RUNNING = 8`), não por repositório. Vira por-repo quando houver mais de um repo aberto
  ao mesmo tempo (Bloco H).
- A CSP de release é `default-src 'self'`, e o WebSocket é da mesma origem. CSP 3 diz que
  `'self'` cobre `ws://` do mesmo host, e os navegadores atuais fazem isso — mas o aceite foi
  por cliente próprio, não por navegador. Vale conferir uma vez com o DevTools aberto.
- Em dev a CSP é frouxa (`'unsafe-inline'` + `ws://127.0.0.1:5173`) porque o Vite injeta
  script inline e abre o WebSocket do HMR. O binário de release nunca usa essa versão.
- O log não tem **total de commits** nem posição: a barra de rolagem virtual do Passo 36 vai
  precisar de um `rev-list --count` em background, como diz o BLOCO-D. Não entrou no Passo 35
  porque a rota de página não depende dele.
- Falta o teto de commits por request na camada de `rate_limit` que ainda não existe. Hoje o
  que segura o log é só o clamp de `limit` em 2000 por página.
- **[ATUALIZADA 2026-08-22, achado no aceite do Bloco D — ver "Ambiente verificado"]**
  `Git2Repo::log` abre um `Repository` por chamada. A nota original ("com página de 500 isso é
  ruído") **só valia para repositórios pequenos** — medido contra 100k commits, é o oposto de
  ruído: é ~900ms na primeira página, quase 20× o orçamento de 50ms do `CLAUDE.md`. A causa
  não é o `Repository::open` em si (< 1ms) nem o `find_commit`/`odb.exists` por commit (< 1ms
  para os 500 juntos) — é o **primeiro `next()` do revwalk com `Sort::TOPOLOGICAL`**, sozinho,
  que veio a ~900ms num diagnóstico isolado (linha a linha, com `Instant::now()` entre cada
  etapa). O libgit2 pré-processa a alcançabilidade do histórico **inteiro** antes de devolver o
  primeiro commit em ordem topológica — o custo é do tamanho do repositório, não da página, e
  isso é verdade mesmo sem merge nenhum (testado numa cadeia linear de 100k commits). Duas
  saídas possíveis, nenhuma tentada ainda: (a) o pool de handles com `Repository` de vida mais
  longa que o `CLAUDE.md` já previa, se libgit2 reaproveitar o pré-processamento entre chamadas
  do mesmo handle (a **segunda** chamada no mesmo processo, mesmo `Repository`, caiu para
  ~140ms no diagnóstico — não é grátis, mas é bem menos ruim); (b) gerar um arquivo
  `commit-graph` (`git commit-graph write --reachable`, o mesmo mecanismo que acelera `git log`
  no próprio terminal) ao indexar (Passo 42 já faz uma varredura completa do histórico de
  qualquer forma) e conferir se o libgit2 o usa para topológico rápido. Isto bloqueia a meta
  "rola sem engasgo" do Bloco D em repositórios grandes de verdade — é a prioridade de
  performance mais alta pendente ao entrar no Bloco E.
- Arestas do `LogGraph` são retas, sem a curva em S que a maioria dos clientes gráficos usa
  quando uma lane nasce ou morre entre duas linhas adjacentes. Funciona e é legível; é
  polimento visual para revisitar se incomodar na prática, não bloqueia o passo.
- `gutterWidth` não tem teto: um repositório com dezenas de branches abertas ao mesmo tempo
  empurraria a faixa do grafo para uma largura grande, comendo espaço da lista de texto. Não é
  o caso comum, e truncar exigiria decidir o que fazer com as lanes que ficassem de fora — vale
  esperar aparecer de verdade antes de inventar a resposta.
- O grafo não desenha nada para a fronteira de clone raso (`parentLane: null`): a linha
  simplesmente para. Um marcador "histórico raso termina aqui" é ideia de polimento, não do
  escopo do Passo 38.

## Ambiente verificado

- **Aceite do Bloco E inteiro (2026-08-22)**, rodado uma vez ao fim com o **binário de release**
  (`cargo build --release`: 39s, 6,8 MB, frontend embutido) contra um repositório sintético em
  `~/porc-aceite-bloco-e` (um arquivo de 40 linhas com três mudanças espalhadas e um arquivo
  novo). O ciclo inteiro do bloco, ponta a ponta por HTTP, sem tocar no `git` do terminal em
  nenhum passo: (1) `status` trouxe os três grupos certos; (2) o diff do working tree veio com
  **3 hunks**; (3) stagear só o hunk do meio deixou no índice exatamente `+MEIO MUDADO`;
  (4) stagear **duas linhas** do primeiro hunk acrescentou exatamente `+TOPO MUDADO`;
  (5) `stage` do arquivo novo pelo caminho; (6) **descartar** o hunk que sobrou tirou
  `FIM MUDADO` do disco; (7) `commit` com `signoff` respondeu com o oid e um status
  completamente vazio; (8) o commit apareceu no topo do `log`; (9) `compare HEAD~1..HEAD` deu
  `+3 -2` nos dois arquivos; (10) o `git log` e o `git status` **de verdade** confirmaram a
  mensagem, o trailer `Signed-off-by:` e a worktree limpa. **O critério do bloco — "dá para
  preparar e fechar um commit sem tocar no terminal" — está cumprido.** Repositórios de teste
  apagados ao final.
- `cargo test --workspace` ao fim do Bloco E: **177 testes**, tudo verde. Clippy
  (`--workspace --all-targets --all-features`) e `cargo fmt --check` limpos; `npm run build` e
  `npm run lint` limpos (resta só o aviso informativo pré-existente do `react-compiler` sobre o
  `useVirtualizer`).

- macOS arm64 (Darwin 25.5.0) — máquina de desenvolvimento
- Node v22.23.1, npm 10.9.8 — ok
- git 2.54.0 — ok
- Rust 1.98.0 / cargo 1.98.0 — ok. **`~/.cargo/bin` não está no PATH de shell
  não-interativo**: comandos automatizados precisam de `export PATH="$HOME/.cargo/bin:$PATH"`
  antes do `cargo`. No terminal interativo do usuário funciona normal.
- **MSRV real do projeto: 1.87** (imposto pelas dependências, não por nós).
- Aceite do Passo 35 (2026-08-21): repositório sintético de **621 commits com merge**, criado
  e apagado em `~/Library/Caches/porcelain-aceite-log`. Primeira página = 500 commits +
  cursor em **35ms** (build de debug); segunda página = os 121 restantes, `nextCursor: null`,
  **zero sobreposição**. Em `~/Git/Barbearia` (122 commits, 7 merges) 18 páginas de 7 deram
  exatamente a mesma sequência de uma página de 500, e o conjunto bateu com
  `git rev-list HEAD`. Cursor lixo → 400, repo desconhecido → 404, sem sessão → 401.
- `cargo build --release` (com `npm ci`/`npm run build` do `build.rs` e o frontend embutido):
  **1m07s**, binário único de **6,1 MB**. Verificado em 2026-08-21.
- No macOS, config **e** dados ficam em `~/Library/Application Support/porcelain/`
  (`config.toml`, `porcelain.db`, `porc.lock`). No Linux é `~/.config/porcelain/` e
  `~/.local/share/porcelain/`.
- Aceite do Passo 40b (2026-08-22): `npm run build` e `npm run lint` limpos. `cargo build
  --release` embutiu o bundle novo (contém `formatSignatureDate`/o texto "selecione um commit
  no log", confirmado por grep no JS servido). Sem passo de backend novo — reusa o
  `/commits/{oid}` do 40a, já verificado ponta a ponta com números reais deste repositório.
- Aceite do Passo 44b (2026-08-22): `npm run build` e `npm run lint` limpos. `cargo build
  --release` embutiu o bundle novo (contém "colar um hash", confirmado por grep no JS
  servido). Sem passo de backend novo — reusa o `search` do 44a, já verificado ponta a ponta
  com hash real deste repositório (`9db77b8` → um resultado só).
- **Aceite do Bloco D inteiro (2026-08-22)**, rodado uma vez ao fim, contra um repositório
  sintético de **100.000 commits em cadeia linear** (`git fast-import`, criado em segundos):
  boot do binário de release até servir: **18ms** (meta < 1s, ok). Abrir o repositório: 16ms.
  Indexação de busca (Passo 42) dos 100k commits: **< 50ms**, já `done` no primeiro poll (meta
  2-4s, folgadíssimo). Busca por mensagem (FTS5): 72ms medidos por HTTP, incluindo overhead de
  processo do `curl` (meta < 20ms — o teste isolado em Rust já tinha medido < 100ms para 50k
  linhas sem esse overhead; aceitável). **Primeira página do log: ~600-900ms, repetível em
  requisições sucessivas — falha a meta de < 50ms em quase 20×.** Diagnosticado até a causa
  exata (ver a pendência atualizada acima, "`Git2Repo::log` abre um `Repository` por chamada"):
  não é `Repository::open`, não é `find_commit`/`odb.exists` por commit — é o **primeiro
  `next()` do revwalk com `Sort::TOPOLOGICAL`**, que sozinho consumiu ~900ms num diagnóstico
  isolado, porque o libgit2 pré-processa a alcançabilidade do histórico inteiro antes de
  devolver o primeiro commit em ordem topológica. "Rola sem engasgo" **não está cumprido** em
  repositórios grandes de verdade — a rolagem paga esse custo a cada página, não só na
  primeira. A busca por conteúdo (pickaxe) não foi medida neste aceite final por tempo, mas já
  tinha sido verificada ponta a ponta no Passo 46a contra 6000 commits com resultado correto e
  rápido. Repositório de teste apagado ao final; nenhum arquivo de diagnóstico ficou no
  projeto.
- Aceite do Passo 46b (2026-08-22): `npm run build` e `npm run lint` limpos (dois avisos reais
  do oxlint corrigidos de verdade, não suprimidos — ver decisões acima). `cargo build
  --release` embutiu o bundle novo (contém "expressão regular"/"procurando os primeiros
  resultados", confirmado por grep no JS servido). `cargo test --workspace` continua em
  116/116 (nenhum arquivo Rust mudou neste passo). Sem passo de backend novo — reusa `pickaxe`
  (46a), já verificado ponta a ponta com `-S`/`-G` batendo exatamente com o `git log` de
  verdade e a cauda `hits` capada em 200.
- Aceite do Passo 46a (2026-08-22): `cargo test -p porc-git` — 73 testes, incluindo os 5 novos
  de `by_content` contra este próprio repositório (`-S"porcelain"` acha os 3 commits certos,
  `-G"fn open\("` acha os 2 commits certos — os dois batendo com `git log -S`/`-G` de verdade
  rodado na mão; sem ocorrência não falha, só vem vazio; cancelamento pré-armado devolve
  `Cancelled`). `cargo test -p porc-server` — 25 testes, incluindo os 2 novos de `hit()`
  (publica e acumula; corta a cauda em `LOG_TAIL` como o `log`). `cargo test --workspace` —
  116 testes, tudo verde. Clippy e `cargo fmt --check` limpos. Handshake HTTP completo: um
  repositório sintético de 6000 commits com conteúdo alternado deu 5999 acertos para
  `-S"needle-marker-0"`, terminando `done` com o `result` completo e a cauda `hits` capada em
  200; contra este próprio repositório, `-G"fn open\("` e uma busca sem ocorrência bateram
  exatamente com o `git log` de verdade. Repositório de teste apagado ao final.
- Aceite do Passo 45c (2026-08-22): `npm run build` e `npm run lint` limpos. `cargo build
  --release` embutiu o bundle novo (contém "caminho do arquivo"/"nenhum commit tocou",
  confirmado por grep no JS servido). `cargo test --workspace` continua em 110/110 (nenhum
  arquivo Rust mudou neste passo). Sem passo de backend novo — reusa `path-filter` (45a) e
  `paths` (45b), já verificados ponta a ponta com o job terminando `done` e a árvore deste
  repositório respondendo os caminhos certos.
- Aceite do Passo 45b (2026-08-22): `cargo test -p porc-git` — 69 testes, incluindo os 4 novos
  de `list_paths` contra este próprio repositório (raiz marca pasta com `/` e arquivo sem;
  `crates/porc-` acha as 4 crates do workspace, `crates/porc-g` afunila para uma só; `limit`
  respeitado; pasta inexistente e "dentro" de um arquivo não falham, só vêm vazios).
  `cargo test --workspace` — 110 testes, tudo verde. Clippy e `cargo fmt --check` limpos.
  Handshake HTTP completo contra este repositório: raiz trouxe as pastas/arquivos certos com a
  marcação certa; `prefix=crates/porc-` trouxe as 4 crates; prefixo inexistente veio vazio.
- Aceite do Passo 45a (2026-08-22): `cargo test -p porc-git` — 65 testes, incluindo os 4 novos
  de `path_filter` (`PROGRESSO.md` neste repositório traz exatamente os 3 commits que o
  tocaram, na ordem certa; caminho que nunca existiu não falha, só vem vazio; cancelamento
  pré-armado devolve `ExecError::Cancelled`; campos do commit raiz batem) e os 4 novos de
  `parse::records` (vários commits num chunk só; campo partido no meio esperando o resto; data
  ilegível vira zero em vez de derrubar; NUL faltando no fim não solta commit incompleto).
  `cargo test --workspace` — 106 testes, tudo verde. Clippy e `cargo fmt --check` limpos.
  Handshake HTTP completo contra este próprio repositório: `POST /jobs/path-filter` com
  `path: "PROGRESSO.md"` terminou `done` com os 3 commits certos (mesmos oids, mesma ordem de
  `git log --oneline -- PROGRESSO.md`); caminho inexistente terminou `done` com lista vazia;
  `DELETE /jobs/{id}` respondeu 202 (o job já tinha terminado antes de a race chegar, repo
  pequeno demais para observar o estado `cancelled` ao vivo — a cobertura de cancelamento de
  verdade é o teste do `porc-git`).
- Aceite do Passo 44a (2026-08-22): `cargo test -p porc-index` — 17 testes, incluindo hash
  curto/completo/maiúsculo/ambíguo saltando pelo prefixo de oid; `autor:` combinado com texto
  livre e sozinho (caminho sem FTS5); `depois:`/`antes:` filtrando por intervalo; token com
  prefixo reconhecido mas inválido sendo ignorado sem erro; e `days_from_civil` batendo com
  datas conhecidas (`1970-01-01`→0, `2000-03-01`→951868800, ano bissexto). `cargo test
  --workspace` — 98 testes, tudo verde. Clippy e `cargo fmt --check` limpos. Handshake HTTP
  completo contra este próprio repositório: `q=9db77b8` achou exatamente o commit "passo 35";
  `autor:joaquimoiio` trouxe os 3 commits do autor; `depois:2026-08-20 antes:2026-08-22`
  trouxe os mesmos 3 (todos feitos nesse intervalo); `depois:2030-01-01` (futuro) veio vazio.
- Aceite do Passo 43b (2026-08-22): `npm run build` e `npm run lint` limpos. `cargo build
  --release` embutiu o bundle novo (contém "buscar por mensagem ou autor"/"nenhum commit
  encontrado", confirmado por grep no JS servido). Sem passo de backend novo — reusa o
  `/search` do 43a, já verificado ponta a ponta contra este próprio repositório.
- Aceite do Passo 43a (2026-08-22): `cargo test -p porc-index` — 13 testes, incluindo busca
  por mensagem, por autor, por prefixo incremental, entrada perigosa (`AND`/`-erro`/aspa) sem
  quebrar, busca vazia sem tocar o banco, e 50k linhas sintéticas em menos de 100ms.
  `cargo test --workspace` — 94 testes, tudo verde. Clippy e `cargo fmt --check` limpos.
  Handshake HTTP completo contra este próprio repositório: `search?q=passo` achou o commit
  "progresso: passo 35…" de verdade; `q=` vazia devolveu `[]`; `q=" OR 1=1` (sintaxe FTS5
  perigosa) devolveu 200 sem derrubar a rota.
- Aceite do Passo 42 (2026-08-22): `cargo test -p porc-git` (57, incluindo
  `walk_for_index_visita_todo_commit_alcancavel_do_head` — contagem conferida contra um
  revwalk do próprio git2, não hardcoded — e `walk_for_index_para_cedo_quando_on_commit_devolve_false`)
  e `cargo test -p porc-index` (8, incluindo os três novos de `commits::replace_commits`).
  `cargo test --workspace` — 89 testes, tudo verde. Clippy e `cargo fmt --check` limpos.
  Ponta a ponta: repositório sintético de **4000 commits** (`git fast-import`) aberto via
  handshake HTTP completo — `GET /log` respondeu em ~54ms (o log não esperou a indexação);
  `GET /jobs` mostrou o job `kind:"index"` progredindo e terminando `done` com
  `result:{commits:4000}`; a tabela `commits` do `porcelain.db` ficou com exatamente 4000
  linhas e `indexed_tip` batendo com o `git rev-parse HEAD` do repositório; reabrir o mesmo
  repositório **não** criou um segundo job (índice já em dia). Repositório e linhas de teste
  apagados ao final.
- Aceite do Passo 41b (2026-08-22): `npm run build` e `npm run lint` limpos (o
  `set-state-in-effect` do oxlint pegou o efeito de fechar arquivo ao trocar de commit — corrigido
  para ajuste durante o render, ver decisão acima). `cargo build --release` embutiu o bundle novo
  (contém "lado a lado"/"arquivo binário", confirmado por grep no JS servido). Sem passo de
  backend novo — reusa o `/diff` do 41a, já verificado ponta a ponta com números reais.
- Aceite do Passo 41a (2026-08-22): `cargo test -p porc-git` — 55 testes, incluindo os três
  novos de `commit_diff` contra este próprio repositório (`web/src/app/Shell.tsx` no commit
  `9db77b8`: 4 hunks, 146 inserções e 79 remoções batendo com `git diff-tree --numstat`, todo
  `oldLineno`/`newLineno` coerente com o `kind` da linha; a fonte `.woff2` do Passo 15 →
  `Binary`; caminho que o commit raiz não tocou → `FileNotInCommit`). `cargo test --workspace`
  — 84 testes, tudo verde. Clippy e `cargo fmt --check` limpos. Handshake HTTP completo contra
  este repositório confirmou os três casos ponta a ponta, com os mesmos números.
- Aceite do Passo 40a (2026-08-22): `cargo test -p porc-git` — 52 testes, incluindo os três
  novos de `commit_detail` (oid inválido → 400; commit raiz deste repositório com 15 arquivos
  `Added`/1385 inserções batendo com `git show --stat`; commit normal com 50 arquivos/5763
  inserções/9 remoções, soma por arquivo == agregado). `cargo test --workspace` — 81 testes,
  tudo verde. Clippy e `cargo fmt --check` limpos. Handshake HTTP completo contra este próprio
  repositório confirmou o JSON do commit raiz ponta a ponta, e um oid inválido devolveu 400.
- Aceite do Passo 39b (2026-08-22): `npm run build` e `npm run lint` limpos. `cargo build
  --release` embutiu o bundle novo (contém `isHead`/`border-dashed`, confirmado por grep no JS
  servido). Sem passo de backend novo — reusa o `/refs` do 39a, já verificado ponta a ponta.
- Aceite do Passo 39a (2026-08-22): `cargo test -p porc-git` — 49 testes, incluindo
  `refs_do_projeto_marca_a_branch_atual` (contra este repositório de verdade: `main` com
  `isHead: true`, `origin/main` presente) e `tag_e_head_destacado_ganham_marcador` (repositório
  sintético com tag leve e `HEAD` destacado). `cargo test --workspace` — 78 testes, tudo verde.
  `cargo clippy --workspace --all-targets --all-features` e `cargo fmt --check` limpos. Handshake
  HTTP completo contra este próprio repositório confirmou o JSON: `main`/`isHead:true`,
  `origin/main`/`isHead:false`.
- Aceite do Passo 38 (2026-08-22): `npm run build` e `npm run lint` limpos. `cargo build
  --release` embutiu o bundle novo (contém `getVirtualItems`/`devicePixelRatio`, confirmado por
  grep no JS servido). Repositório sintético com fork+merge criado com o `git` do sistema
  (`root`, `a`, `b` na main; `f1` na `feature`; `merge --no-ff`; `c`), aberto via o handshake
  completo: o JSON de `/log` trouxe `merge` com `parentLanes: [0, 1]`, `f1` na lane 1, e `a`
  (onde `b` e `f1` convergem) de volta na lane 0 — o mesmo shape que os testes do Passo 37 já
  garantiam, agora confirmado ponta a ponta pela rota HTTP de verdade. Repositório de teste
  apagado ao final.
- Aceite do Passo 37 (2026-08-22): `cargo test -p porc-git` — 47 testes, incluindo os dois novos
  do algoritmo de lanes (`lanes_convergem_no_fork_e_nascem_no_merge` e
  `duas_paginas_encaixam_sem_descontinuidade_nas_lanes`). `cargo test --workspace` — 75 testes,
  tudo verde. `cargo clippy --workspace --all-targets --all-features` e `cargo fmt --check`
  limpos. `cargo build` (dev) compila.
- Aceite do Passo 36 (2026-08-22): `npm run build` e `npm run lint` limpos (`oxlint` só aponta um
  aviso informativo do `react-compiler` sobre `useVirtualizer` retornar funções não memoizáveis —
  esperado da lib, sem regra configurada para isso). `cargo build --release` embutiu o bundle
  novo; repositório sintético de **3000 commits** criado via `git fast-import` em `~/` (dentro do
  confinamento — `/tmp` cai fora da raiz padrão, que é a home). Handshake → abrir → paginar bateu
  6 páginas de 500 até `nextCursor: null`, e o JS servido pelo `rust-embed` contém o código do
  `CommitList`. Repositório de teste apagado ao final.
