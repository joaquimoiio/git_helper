# BLOCO G — merge e conflitos

**Objetivo:** merge com preview e **resolução de conflito dentro da interface**, em merge
tool de 3 vias.

**Ao fim do bloco:** dá para resolver um merge conflituoso sem abrir editor externo.

---

## Detalhe técnico do bloco

**Merge é shell-out**, porque precisa disparar hooks e escrever o estado
(`MERGE_HEAD`, `MERGE_MSG`, índice em estágios) que o `git` do usuário entende — se o
app travar no meio, ele resolve no terminal e nada se perde.

**Estágios do índice** são a fonte da verdade do conflito: estágio 1 = base,
2 = ours (local), 3 = theirs (remoto). Os três painéis vêm daí, via git2, e o quarto
painel (resultado) parte do arquivo da worktree com os marcadores já escritos pelo git.

**O risco mais grave do projeto está aqui:** um merge tool de 3 vias pressupõe três
textos. Não existe base textual para binário, e `DU`/`UD`/`AA` (delete/modify, add/add)
não têm três painéis. Se a UI fingir que dá para editar esses casos, ela perde trabalho
do usuário silenciosamente. Por isso o Passo 78 é separado e explícito: esses casos ganham
um caminho "escolher o arquivo inteiro (nosso / deles)" e **nunca** entram no editor de
hunks.

**Abortar tem que ser sempre acessível** — `git merge --abort` visível em qualquer estado
de conflito, não escondido num submenu.

---

## Passos

### Passo 73 — merge fast-forward com preview
Antes de confirmar, mostrar exatamente quais commits entram e quais arquivos mudam.
**Aceite:** fazer um merge FF vendo o preview correto antes.

### Passo 74 — `--no-ff` e `--squash`
Escolha explícita dos três modos (ff / no-ff / squash) na mesma caixa, com edição da
mensagem de merge.
**Aceite:** os três modos produzem o histórico esperado.

### Passo 75 — detectar conflito
Estado de merge em andamento exposto no status, painel dedicado listando os arquivos
conflitados com o tipo de conflito de cada um.
**Aceite:** provocar um conflito e ver a lista com os tipos corretos.

### Passo 76 — merge tool de 3 vias
Quatro painéis (base | local | remoto → resultado), lidos dos estágios do índice,
sincronizados no scroll, tipografia mono, mesma linguagem visual do diff.
**Aceite:** abrir um arquivo conflitado e ver os três lados alinhados.

### Passo 77 — resolver por hunk
Aceitar local / remoto / ambos por hunk, com atalhos de teclado, edição manual do
resultado e navegação entre conflitos (n / p).
**Aceite:** resolver um arquivo com três conflitos só pelo teclado.

### Passo 78 — casos degenerados
Binário, rename/delete, `AA`, `DU`, `UD`, submódulo: caminho "escolher o arquivo inteiro"
com explicação do que aconteceu. Nunca abrir o editor de hunks nesses casos.
**Aceite:** conflito de arquivo binário e conflito de delete/modify oferecem escolha
inteira e explicam o caso.

### Passo 79 — concluir ou abortar
Marcar resolvido (`git add`), concluir o merge (com hooks) ou `--abort`, sempre visível.
**Aceite:** concluir um merge resolvido e abortar outro, voltando ao estado anterior.

---

**Fim do bloco:** atualizar `PROGRESSO.md` e dizer
*"bloco concluído, pode dar `/clear` e rodar `/bloco H`"*.
