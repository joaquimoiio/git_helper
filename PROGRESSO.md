# PROGRESSO

> Estado do projeto em disco. **Atualizar ao fim de cada passo, sempre.**
> É este arquivo que faz uma janela de contexto nova saber onde retomar.

## Onde estamos

- **Bloco atual:** D — log (o coração)
- **Último passo concluído:** Passo 35 — `GET /api/v1/repos/{id}/log` paginado por cursor
- **Próximo passo:** Passo 36 — lista virtualizada do log (TanStack Virtual + infinite query)
- **Comando para retomar:** `/blocao D` (ou `/bloco D` para o modo passo-a-passo)

## Mapa dos blocos

| Bloco | Tema | Passos | Estado |
|---|---|---|---|
| A | servidor de pé | 1–12 | **concluído** (12/12) |
| B | frontend de pé | 13–22 | **concluído** (10/10) |
| C | abrir repositório | 23–34 | **concluído** (12/12) |
| D | log (o coração) | 35–46 | em andamento (1/12) |
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

## Pendências / ideias anotadas

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
- Os aceites **visuais** dos Passos 17, 18, 22 e 24 (três painéis na densidade certa,
  arrastar e recarregar, aba com título e ícone, navegar por teclado até um repo) foram
  verificados por build limpo, lint limpo e inspeção do código, **não** num navegador de
  verdade. Vale o usuário abrir uma vez.
- Abrir uma pasta **dentro** de um repositório não abre o repositório (é `open`, não
  `discover`). Se isso incomodar na prática, o lugar de resolver é o `POST /api/v1/repos`,
  reconferindo o resultado do `discover` contra a raiz do confinamento.
- A sidebar ainda lista uma branch só (a atual). Refs de verdade pedem rota própria — Bloco F.
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
- `Git2Repo::log` abre um `Repository` por chamada (é o padrão de todo o `read.rs`). Com
  página de 500 isso é ruído; o pool de handles com semáforo previsto no `CLAUDE.md` só se
  paga quando houver várias leituras por interação (diff + refs + log na mesma tela).

## Ambiente verificado

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
