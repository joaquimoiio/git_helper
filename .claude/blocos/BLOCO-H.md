# BLOCO H — o resto

**Objetivo:** tudo que falta para não precisar do terminal — stash, cherry-pick, revert,
blame, reflog, rebase interativo, command palette, watcher, terminal embutido.

**Ao fim do bloco:** o v1 está funcionalmente completo.

---

## Detalhe técnico do bloco

**Watcher (Passos 87-88).** `notify` observando **`.git/` apenas**, nunca a worktree
recursivamente — em monorepo isso estoura `inotify max_user_watches` no Linux e satura
FSEvents no macOS. Debounce de 250ms, coalescendo num bitset
`REFS | INDEX | HEAD | STASH | CONFIG` derivado dos paths tocados. Emite
`repo.changed{kinds}` no WS; o cliente invalida só as queries daquele domínio.

Dois erros que a implementação ingênua comete:
1. **Eco.** Nossa operação → evento → invalidação → refetch → nova operação, em loop.
   Jobs marcam uma janela de supressão. Isso é obrigatório, não otimização.
2. **`.git/index` é reescrito inteiro** a cada operação e um `fetch` dispara centenas de
   eventos em `refs/` e no packed-refs. Sem coalescing a UI entra em refetch contínuo.

**Rebase interativo (Passos 94-95).** Shell-out com `GIT_SEQUENCE_EDITOR` apontando para
o próprio binário em modo helper, que recebe o todo-list já montado pela UI. Detalhe
crítico: o processo pode parar no meio e deixar `.git/rebase-merge/` — e o usuário pode
fechar a aba nesse estado. A UI tem que **detectar rebase em andamento no boot** e
oferecer continue/abort, nunca assumir que só existe rebase iniciado por ela.

**Atalhos (Passo 90).** Ctrl+K abre a palette. Evitar o que o navegador captura:
Ctrl+W, Ctrl+T, Ctrl+N, Ctrl+L, Ctrl+Shift+{N,T,W,Q}, Ctrl+{1..9}. Preferir teclas
únicas em contexto (j/k, g/G, /, n/p) e Ctrl+K como porta de entrada de tudo.

**Terminal PTY (Passo 96).** Opt-in via `--enable-terminal`, cwd fixo no repo, aviso na
UI. É a única superfície onde uma falha nas camadas de Origin/auth vira execução
arbitrária — por isso é o último passo funcional, depois de tudo testado.

**Log de comandos git (Passo 86).** Todo shell-out registra argv (com segredos
redigidos), cwd, duração, exit code e as primeiras linhas de stderr, visível numa aba da
UI. É a ferramenta de depuração do próprio app e o que dá confiança de que o app não faz
nada pelas costas.

---

## Passos

### Passo 80 — stash
Salvar com mensagem (incluindo/excluindo untracked), listar, aplicar, pop, drop, criar
branch a partir de um stash.
**Aceite:** salvar dois stashes, aplicar um e dropar o outro.

### Passo 81 — cherry-pick
Um ou vários commits selecionados no log, em ordem, com conflito caindo no painel do
bloco G.
**Aceite:** cherry-pick de três commits selecionados de uma vez.

### Passo 82 — revert
De um ou mais commits, com opção de `--no-commit`.
**Aceite:** reverter um commit e ver o commit de revert no log.

### Passo 83 — blame
Por arquivo, com "ir para o commit" e "ver versão anterior desta linha" (blame recursivo
no pai).
**Aceite:** blame de um arquivo e salto para o commit de uma linha.

### Passo 84 — reflog
Lista com ação, ref, mensagem e data, acessível por atalho, com salto para o commit.
**Aceite:** achar no reflog um commit "perdido" após um reset.

### Passo 85 — comparar duas branches
Commits à frente/atrás em duas colunas + diff acumulado.
**Aceite:** comparar `main` com uma feature e ver as duas listas.

### Passo 86 — log de comandos git
Registro de todo shell-out (argv redigido, cwd, duração, exit code, stderr) visível na UI.
**Aceite:** fazer um fetch e ver o comando exato que o servidor rodou.

### Passo 87 — watcher de `.git`
`notify` + debounce 250ms + coalescing em bitset + supressão de eco por janela de job.
**Aceite:** commitar pelo terminal e a UI perceber sem refresh manual, sem entrar em loop.

### Passo 88 — invalidação seletiva
`repo.changed{kinds}` → invalidar só as queries do domínio afetado no TanStack Query.
**Aceite:** mudar só uma ref não refaz o status inteiro.

### Passo 89 — command palette
Ctrl+K com todas as ações e todas as branches, busca fuzzy, execução por Enter.
**Aceite:** trocar de branch e abrir o reflog só pela palette.

### Passo 90 — mapa de atalhos configurável
Tabela de atalhos em `config.toml`, tela de referência (`?`) e validação contra os
capturados pelo navegador.
**Aceite:** remapear um atalho e vê-lo valer após reload.

### Passo 91 — múltiplos repos abertos
Troca rápida entre repos abertos, com estado de layout e seleção por repo.
**Aceite:** abrir três repos e alternar sem perder posição no log.

### Passo 92 — copiar e abrir na forja
Copiar hash / nome de branch; abrir commit, branch ou arquivo no GitHub/GitLab a partir
da URL do remote.
**Aceite:** abrir um commit no GitHub direto do log.

### Passo 93 — worktrees
Listar, criar e remover worktrees (shell-out), com abertura direta da worktree criada.
**Aceite:** criar uma worktree e abri-la como repo.

### Passo 94 — rebase interativo: a UI
Todo-list arrastável com pick/reword/edit/squash/fixup/drop, preview do resultado, sem
executar nada ainda.
**Aceite:** montar e reordenar um todo-list de 5 commits.

### Passo 95 — rebase interativo: execução
`GIT_SEQUENCE_EDITOR` apontando para o helper; pausa em reword/edit, continue, skip,
abort, e **detecção de rebase em andamento no boot**.
**Aceite:** squashar dois commits, parar num reword, concluir; e reabrir o app no meio de
um rebase e ver o estado corretamente.

### Passo 96 — terminal embutido
PTY via WebSocket, atrás de `--enable-terminal`, cwd no repo, com aviso de segurança.
**Aceite:** com a flag, o terminal roda `git status`; sem a flag, a rota não existe.

---

**Fim do bloco:** atualizar `PROGRESSO.md` e dizer
*"bloco concluído, pode dar `/clear` e rodar `/bloco I`"*.
