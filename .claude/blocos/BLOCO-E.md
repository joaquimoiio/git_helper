# BLOCO E — trabalho local

**Objetivo:** o ciclo status → stage → commit, com granularidade de arquivo, hunk e linha.

**Ao fim do bloco:** dá para preparar e fechar um commit sem tocar no terminal.

---

## Detalhe técnico do bloco

**`status` é shell-out**, `git status --porcelain=v2 -z --branch --untracked-files=all`.
Motivo: libgit2 não usa fsmonitor nem untracked-cache; em worktree grande a diferença é
de segundos para dezenas de ms, e "segundos por refresh" mataria a premissa do dia
inteiro sem terminal. O formato v2 é estável e documentado — parser em
`porc-git/parse/status_v2.rs`, com testes.

**Diff é git2**, porque precisamos de hunks estruturados, não de texto para reparsear.

**Stage por hunk e por linha:** construir um patch parcial e aplicar com
`git apply --cached` (e `--reverse` para unstage). É o caminho que preserva exatamente a
semântica do git; reimplementar a aplicação de patch é fonte garantida de corrupção
silenciosa.

**Discard sempre confirma.** É a operação que destrói trabalho sem reflog para socorrer.
Confirmação explícita, sempre, mesmo em hunk pequeno.

**Encoding e CRLF:** arquivos latin-1, binários ou de CRLF misto têm que cair num modo
"não editável" com aviso, nunca serializar bytes inválidos.

---

## Passos

### Passo 47 — status
Shell-out porcelain-v2 + parser + rota, agrupado em staged / unstaged / untracked, com
estado de merge/rebase em andamento já exposto (usado no bloco G).
**Aceite:** sujar a worktree e ver os três grupos corretos na UI.

### Passo 48 — stage e unstage por arquivo
Ações por linha e em lote, com atalhos de teclado e refresh otimista.
**Aceite:** stage/unstage de vários arquivos só pelo teclado.

### Passo 49 — diff do working tree e do index
Selecionar um arquivo mostra o diff do lado certo (unstaged vs staged), reaproveitando o
visualizador do Passo 41.
**Aceite:** editar um arquivo e ver o diff correto nos dois lados.

### Passo 50 — stage e unstage por hunk
Seleção de hunk no visualizador + `git apply --cached` (`--reverse` para desfazer).
**Aceite:** stagear um hunk de um arquivo com três hunks e o status refletir parcial.

### Passo 51 — stage e unstage por linha
Seleção de linhas dentro do hunk, montando o patch parcial correspondente (cuidando dos
contadores do cabeçalho `@@`).
**Aceite:** stagear duas linhas de um hunk e conferir com `git diff --cached`.

### Passo 52 — commit
Caixa de mensagem (assunto + corpo, com régua de 50/72), validação, template de mensagem
do `.gitconfig`, atalho para commitar.
**Aceite:** fazer um commit pela UI e vê-lo no topo do log.

### Passo 53 — amend, signoff e GPG
`--amend` (carregando a mensagem anterior), `--signoff`, e assinatura GPG respeitando a
config do usuário — com erro legível se a chave pedir pinentry.
**Aceite:** emendar o último commit e assinar um commit.

### Passo 54 — `--fixup`
Criar commit `--fixup=<oid>` a partir de um commit selecionado no log.
**Aceite:** gerar um fixup e ver a mensagem `fixup! …` correta.

### Passo 55 — discard seletivo
Descartar arquivo inteiro ou hunk, sempre com confirmação explícita nomeando o que será
perdido.
**Aceite:** descartar um hunk e ver o arquivo voltar parcialmente.

### Passo 56 — diff arbitrário
Comparar dois commits quaisquer e duas branches quaisquer, com seletor de ambos os lados.
**Aceite:** comparar duas branches e ver o diff acumulado.

---

**Fim do bloco:** atualizar `PROGRESSO.md` e dizer
*"bloco concluído, pode dar `/clear` e rodar `/bloco F`"*.
