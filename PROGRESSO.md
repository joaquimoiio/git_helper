# PROGRESSO

> Estado do projeto em disco. **Atualizar ao fim de cada passo, sempre.**
> É este arquivo que faz uma janela de contexto nova saber onde retomar.

## Onde estamos

- **Bloco atual:** — (nenhum iniciado)
- **Último passo concluído:** Passo 0 — andaime em `.claude/` criado
- **Próximo passo:** Passo 1 — instalar toolchain Rust e criar o workspace vazio que compila
- **Comando para retomar:** `/bloco A`

## Mapa dos blocos

| Bloco | Tema | Passos | Estado |
|---|---|---|---|
| A | servidor de pé | 1–12 | pendente |
| B | frontend de pé | 13–22 | pendente |
| C | abrir repositório | 23–34 | pendente |
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
- **2026-08-21** — `.claude/commands/` em vez de `.claude/skills/`: skills carregam por
  relevância implícita; aqui queremos invocação explícita e determinística, garantindo
  que a janela leia exatamente `CLAUDE.md` + `PROGRESSO.md` + **um** `BLOCO-X.md`.

## Pendências / ideias anotadas

- (vazio)

## Ambiente verificado

- macOS arm64 (Darwin 25.5.0) — máquina de desenvolvimento
- Node v22.23.1, npm 10.9.8 — ok
- git 2.54.0 — ok
- **Rust: não instalado** — Passo 1 começa por instalar o toolchain
