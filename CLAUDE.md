# porcelain — contexto permanente

Cliente Git **local-first**: binário único que sobe um servidor HTTP em `127.0.0.1` e
serve uma interface web. Os repositórios são os do disco do usuário, o processo roda
como o usuário, nada sai da máquina.

**Critério de sucesso:** o usuário deve conseguir passar o dia inteiro de trabalho sem
precisar do `git` no terminal.

- Nome do app: `porcelain`. Binário: `porc`.
- Alvo principal: **Linux**. Desenvolvimento no macOS. Windows é secundário.
- Referência de funcionalidade: TortoiseGit. Referência de experiência: **não** é.

---

## Regras de trabalho (as mais importantes deste arquivo)

1. **Um passo por vez.** Nunca entregar dois passos na mesma resposta. Ao terminar um
   passo: atualizar `PROGRESSO.md`, e **parar**.
2. **Um passo mexe em no máximo 3-4 arquivos.** Se passar disso, o passo é grande demais
   — quebre em dois e registre a quebra no `PROGRESSO.md`.
3. **Sempre o arquivo inteiro.** Ao alterar um arquivo existente, mostrar o conteúdo
   completo atualizado. Nunca `// resto igual`, nunca trecho solto sem caminho.
4. **Cada passo deixa o projeto compilando e rodando.** Nunca deixar o usuário com código
   quebrado esperando o passo seguinte.
5. **Comandos prontos para copiar.** Nunca "instale as dependências" — sempre a linha exata.
6. **Não pular para frente.** Ideia boa fora do escopo do passo atual vira uma linha em
   "Pendências / ideias" no `PROGRESSO.md`.
7. **Fim de bloco:** dizer explicitamente
   *"bloco concluído, pode dar `/clear` e rodar `/bloco <próximo>`"* e não continuar.
8. Se o usuário relatar um erro: corrigir só aquilo, devolver o arquivo corrigido inteiro,
   não recomeçar nem reescrever o que já está certo.
9. Se o usuário disser "próximo", "ok" ou "segue": ir ao passo seguinte sem recapitular.

## Formato obrigatório de entrega de passo

```
## Passo N — <nome curto>

**O que este passo faz:** uma frase.

**Arquivos:** lista dos arquivos criados ou alterados.

<código completo de cada arquivo, com o caminho no topo>

**Comando para rodar:**
<comandos exatos, em bloco de código, na ordem>

**Como sei que funcionou:** o que aparece na tela ou no terminal.

**Próximo passo será:** uma frase.
```

---

## Stack (decidida — não reabrir sem motivo forte)

| Camada | Escolha |
|---|---|
| Backend | Rust + `axum` + `tokio` + `tower-http` |
| Git (leitura) | `git2` (libgit2) atrás do trait `RepoRead` |
| Git (rede/estado) | shell-out para o binário `git` do sistema (≥ 2.30, exigido no boot) |
| Índice/persistência | `rusqlite` (feature `bundled`) + FTS5 |
| Frontend | React + TypeScript + Vite |
| Estilo | Tailwind v4 (CSS-first, `@theme`), tokens próprios, sem lib de componentes |
| Estado | Zustand (cliente) + TanStack Query (servidor) |
| Virtualização | TanStack Virtual |
| Grafo do log | **Canvas 2D**, nunca SVG/DOM |
| Tempo real | **um** WebSocket multiplexado por `topic` |
| Embed | `rust-embed` (`embed-web`) / proxy para Vite em dev (`dev-proxy`) |

### Fronteira git2 vs shell-out

**git2:** revwalk/log/grafo, diff (commit, index, worktree), blame, refs, tags,
stash list, reflog. Tudo que é loop apertado ou precisa de estrutura, não de texto.

**shell-out:** `status` (`--porcelain=v2 -z`, para aproveitar fsmonitor/untracked-cache),
pickaxe `-S`/`-G` (libgit2 não tem), filtro por caminho, clone/fetch/pull/push
(credential helper, SSH agent, LFS), merge/rebase/cherry-pick/revert (precisam disparar
hooks e escrever o estado que o `git` do usuário entende), rebase interativo
(`GIT_SEQUENCE_EDITOR`), worktree, sparse, LFS, hooks.

**Regras de todo shell-out, sem exceção:**
- `--no-optional-locks`
- `GIT_TERMINAL_PROMPT=0` + timeout (senão um prompt trava o job para sempre)
- saída estável: `-z` ou `--format` com separador `%x00`. **Nunca** parsear porcelana
  instável (`git status` sem `--porcelain`, `git log` default).
- process group próprio (`setsid`/`process_group(0)`) — cancelar mata o grupo inteiro,
  senão `git-remote-https` e `ssh` ficam órfãos.
- override de config sempre efêmero via `git -c chave=valor`.
  **Nunca** `git config --global` sem o usuário pedir explicitamente na UI.

---

## Estrutura de pastas

```
git_helper/
  Cargo.toml                     workspace
  crates/
    porc-cli/                    flags, lockfile, abrir navegador, modos helper
                                 (askpass, sequence-editor)
    porc-server/                 axum: router, middleware/, routes/, ws/, embed
    porc-git/                    domínio: read/ (git2), exec/ (shell-out), model/, parse/
    porc-index/                  SQLite: schema, commits, search (FTS5), recents, config
  web/
    src/styles/tokens.css        FONTE ÚNICA de cor, espaço, raio, tipo, duração
    src/app/  src/features/  src/lib/
    dist/                        gerado
  assets/fonts/                  Inter + JetBrains Mono, woff2 subsetados
  packaging/                     porcelain.desktop, ícones, install.sh
  .claude/blocos/                especificação de cada bloco
  CLAUDE.md  PROGRESSO.md
```

**Fronteiras de crate (invioláveis):**
- `porc-git` não conhece HTTP nem axum.
- `porc-server` não faz `use git2`.
- `porc-cli` não conhece git.

**Build:**
- `dev-proxy` (default em debug): Rust faz proxy de `/` para `http://127.0.0.1:5173`.
  Dev roda `npm run dev` + `cargo run`, com HMR e sem recompilar Rust por causa de CSS.
- `embed-web` (default em release): `build.rs` roda `npm ci && npm run build` e embute
  `web/dist`. `cargo build --release` produz **um arquivo**, fontes inclusas, offline.

---

## Segurança (não negociável)

Ordem das camadas no `Router`: `trace → origin_guard → rate_limit → auth → csrf → rotas`.
Middleware em `crates/porc-server/src/middleware/`.

1. **Bind** literal em `127.0.0.1`. Nunca `0.0.0.0`, nunca `[::]`, sem flag para mudar.
2. **Guard de `Host`/`Origin`** (`middleware/origin.rs`) — defesa real contra DNS
   rebinding. Bind em localhost não protege: o browser da vítima entrega o request. O que
   protege é o header `Host` trazer o domínio do atacante. 403 se `Host` não for
   `127.0.0.1:<porta>` ou `localhost:<porta>`; `Origin`, se presente, tem que bater;
   `Origin: null` é rejeitado. **Aplicar também no upgrade do WebSocket** (WS não sofre
   CORS e escapa de `SameSite`).
3. **Token de sessão** (`middleware/auth.rs`) — 256 bits de `getrandom` por boot, nunca
   persistido. Entregue em `?t=` na abertura do browser; `POST /api/v1/session` troca por
   cookie `HttpOnly; SameSite=Strict; Path=/` e redireciona **limpando a query** (fora do
   histórico e do `Referer`). Comparação em tempo constante (`subtle`).
4. **CSRF** (`middleware/csrf.rs`) — double-submit: cookie `csrf` legível por JS + header
   `X-CSRF-Token` obrigatório em todo método ≠ GET/HEAD.
5. **Confinamento** — repositório é sempre `repo_id` (hash opaco do path canônico).
   Nenhuma rota aceita path de repo vindo do cliente. `fs::canonicalize` + verificação de
   prefixo; `..`, symlink para fora e path não-canônico são 403.
6. **Rate limit** (`tower_governor`) em busca por conteúdo, `fs/list` e criação de jobs,
   mais teto global de jobs concorrentes por repo.
7. **Segredos** — passphrase SSH nunca toca o disco. Vive em memória e chega ao `git` via
   `GIT_ASKPASS` apontando para `porc askpass`, que busca o valor num socket unix
   efêmero (0600). Nada em `argv` (visível em `ps`).
8. **Headers** — CSP `default-src 'self'` (tudo é local, então é viável de verdade),
   `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, CORS desabilitado.
9. **Terminal PTY** — opt-in via `--enable-terminal`, cwd fixo no repo. É a única
   superfície onde falha nas camadas 2-4 vira RCE. Por isso é o penúltimo passo do projeto.

---

## Design (ler antes de escrever qualquer componente)

**Conceito:** monocromático, denso, editorial. O conteúdo é a interface.

- **Tokens primeiro.** `web/src/styles/tokens.css` é a fonte única de cor, espaçamento,
  raio, tipografia e duração. Nenhum componente define valor cru — sempre `var(--…)`.
- **Paleta:** preto, branco, cinzas neutros. Escuro default, claro como espelho exato.
  Cor só em semântica (adição, remoção, conflito, erro) e **dessaturada**, nunca neon.
- **Superfícies:** hierarquia por valor, não por sombra. Sem gradiente decorativo, sem
  glassmorphism. Separação por borda de 1px de baixo contraste.
- **Tipografia:** uma sans de UI (Inter) + uma mono real (JetBrains Mono) para hash, diff
  e path. **Máximo 5 tamanhos no app inteiro.** Fontes servidas pelo binário, zero CDN.
- **Densidade:** slim. Muita informação por tela sem parecer apertado.
- **Ícones:** stroke fino, monocromáticos, tamanho único.
- **Grafo:** linhas finas, **sem arco-íris** — diferenciação por posição e peso.
- **Movimento:** 120–180ms, só em mudança de estado real.
- **Layout:** sidebar (branches, remotes, tags, stashes) → centro (log ou status) →
  detalhe/diff. Redimensionável, colapsável, persistido.
- Roda em aba de navegador: **título da aba e favicon** refletem repo e branch atuais.
- Atalhos: teclado é primário. Ctrl+K abre a command palette. Evitar o que o navegador
  captura (Ctrl+W, Ctrl+T, Ctrl+N, Ctrl+L, Ctrl+Shift+*).

---

## Convenções de código

**Rust**
- Edition 2021, `rustfmt` default. `clippy` limpo antes de fechar um passo.
- Erros: `thiserror` nos crates de domínio, `anyhow` só na borda (`porc-cli`).
  Erro que chega ao usuário é **legível** — nunca despejar `stderr` cru na tela.
- Todo acesso a `git2` dentro de `spawn_blocking` (é bloqueante e não é `Sync`).
  Pool de handles por repo com semáforo. O event loop do tokio nunca vê libgit2.
- Nomes de rota e tipos serde em `snake_case` no Rust, serializados em `camelCase`
  (`#[serde(rename_all = "camelCase")]`).
- `tracing` para log, nunca `println!` fora do `porc-cli`.

**TypeScript**
- Sem `any`. Tipos da API espelham os tipos serde, num único `web/src/lib/api-types.ts`.
- Componentes funcionais, sem classes. Nada de barrel files gigantes.
- TanStack Query para tudo que vem do servidor; Zustand só para estado de UI
  (layout, seleção, tema). Nunca duplicar estado de servidor no Zustand.
- Nenhum valor de cor/espaço/tipo hardcoded: sempre token.

**Geral**
- Comentário explica *por quê*, não *o quê*.
- Mensagens de commit em português, imperativo, minúsculas.

---

## Orçamento de performance (metas duras)

| Métrica | Alvo |
|---|---|
| Boot do binário até servir | < 1s |
| Primeira página do log (500 commits) | < 50ms |
| Log de 100k commits utilizável | < 500ms |
| Busca indexada (mensagem/autor/hash) | < 20ms |
| Busca por conteúdo — primeiros hits | < 150ms |
| Indexação de 100k commits (background) | 2-4s, com o log já usável |

A indexação **nunca** bloqueia o boot. O log serve a primeira página direto do revwalk
enquanto o índice constrói em background, com indicador na UI.

---

## Decisões já tomadas (não reabrir)

- **Depende do `git` do sistema** (≥ 2.30). Verificado no boot com erro legível.
- **Terminal PTY é opt-in** (`--enable-terminal`), desligado por padrão.
- **Índice em SQLite** descartável — deletar o `.db` e reconstruir é sempre saída válida.
- **Paths de commits NÃO são indexados.** Filtro por caminho é `git log -- <path>` em
  streaming, igual à busca por conteúdo. Indexar path em monorepo custa dezenas de
  milhões de linhas e a manutenção cresce mais rápido que o benefício.
- **`gix` (gitoxide) é o plano B** do hot path de leitura. Por isso todo acesso de leitura
  fica atrás do trait `RepoRead` — trocar o backend depois não deve tocar rota nem UI.
- Config do app em `~/.config/porcelain/config.toml`; dados em
  `~/.local/share/porcelain/` (equivalentes no macOS via `directories`).
- **Respeitar `GIT_*` e o `.gitconfig` existente.** Nunca sobrescrever config global.
