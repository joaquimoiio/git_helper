# BLOCO F — branches e remoto

**Objetivo:** trocar de branch, criar branch e sincronizar com o remote — com push
completo, não a metade que a maioria dos clientes entrega.

**Ao fim do bloco:** o dia de trabalho normal já roda inteiro na interface.

---

## Detalhe técnico do bloco

**Tudo que fala com a rede é shell-out + job** (reaproveitando a infra do Bloco C:
progresso, cancelamento, process group, askpass, erros legíveis). libgit2 reimplementa
mal credential helper, `libsecret`, SSH agent e LFS.

**Switch/checkout com busca fuzzy** incluindo remotas: escolher `origin/feature-x` cria a
local com tracking automático. Se houver mudanças não commitadas, aviso **claro** com
opção de stash automático antes — e o stash criado fica identificado para poder voltar.

**Push completo, sem atalhos:** branch atual; branch nova com `-u`; tags (uma ou todas);
deletar branch remota; escolher o remote quando houver vários; `--force-with-lease`
disponível; `--force` puro **só** atrás de confirmação explícita que diz o que pode ser
perdido.

**Aviso pré-push.** Antes de empurrar, comparar com o remote conhecido e avisar se ele
está à frente — o objetivo é que "rejected, non-fast-forward" nunca apareça sem contexto.

**Auto-fetch em background** configurável (intervalo em `config.toml`), alimentando o
indicador de "X à frente / Y atrás" por branch. Nunca dispara operação que muda a
worktree.

---

## Passos

### Passo 57 — listar refs
Sidebar com branches locais, remotas (agrupadas por remote), tags e stashes, com busca e
navegação por teclado.
**Aceite:** a sidebar reflete `git branch -a` do repo aberto.

### Passo 58 — criar branch
A partir de HEAD, de commit selecionado no log ou de qualquer ref, com opções de já fazer
checkout e setar upstream.
**Aceite:** criar branch a partir de um commit antigo selecionado no log.

### Passo 59 — switch/checkout com fuzzy
Busca fuzzy sobre locais e remotas; escolher remota cria a local com tracking.
**Aceite:** trocar de branch digitando três letras; escolher uma remota cria o tracking.

### Passo 60 — aviso de mudanças pendentes
Detectar worktree suja no switch, listar o que está pendente e oferecer stash automático
antes (identificado para retorno).
**Aceite:** tentar trocar com arquivo modificado → aviso; aceitar o stash → troca limpa.

### Passo 61 — renomear e deletar branch local
Com proteção contra deletar a branch atual e aviso se não estiver merged (com `-D` atrás
de confirmação).
**Aceite:** renomear uma branch e deletar outra não mergeada com confirmação.

### Passo 62 — gerenciar remotes
Adicionar, renomear, remover, editar URL. Múltiplos remotes na sidebar.
**Aceite:** adicionar um segundo remote e vê-lo agrupado corretamente.

### Passo 63 — fetch
De um remote ou de todos, com `--prune`, progresso em tempo real e cancelamento (mesmo
mecanismo do clone).
**Aceite:** fetch com progresso; cancelar no meio não deixa processo órfão.

### Passo 64 — indicador à frente / atrás
Contagem por branch (`rev-list --left-right --count`), exibida na sidebar, atualizada
após fetch.
**Aceite:** após fetch, a branch mostra "2 atrás" corretamente.

### Passo 65 — pull
Modo merge e modo rebase, explícitos na UI (nunca implícito pela config), com o resultado
levando ao painel de conflito se houver.
**Aceite:** pull nos dois modos funcionando.

### Passo 66 — push da branch atual
Com progresso e cancelamento.
**Aceite:** push simples com barra de progresso real.

### Passo 67 — push de branch nova com `-u`
Detectar ausência de upstream e oferecer `-u` já selecionado.
**Aceite:** push de branch recém-criada seta o upstream numa ação só.

### Passo 68 — aviso pré-push
Checar se o remote está à frente antes de empurrar e avisar com as opções reais
(pull --rebase, force-with-lease, cancelar).
**Aceite:** com o remote à frente, o aviso aparece **antes** de tentar.

### Passo 69 — force-with-lease e force
`--force-with-lease` como opção normal; `--force` puro só depois de confirmação explícita
que nomeia o que pode ser perdido.
**Aceite:** force-with-lease funciona; force puro exige confirmação escrita.

### Passo 70 — deletar branch remota e escolher remote
Deleção remota com confirmação, e seletor de remote quando houver mais de um.
**Aceite:** deletar uma branch remota de teste e empurrar para o segundo remote.

### Passo 71 — tags
Criar lightweight e anotada (com mensagem e assinatura opcional), deletar local e remota,
push de uma ou de todas.
**Aceite:** criar uma tag anotada e empurrá-la.

### Passo 72 — auto-fetch em background
Intervalo configurável, com indicador discreto de última sincronização e possibilidade de
desligar.
**Aceite:** com o auto-fetch ligado, o contador "atrás" atualiza sozinho.

---

**Fim do bloco:** atualizar `PROGRESSO.md` e dizer
*"bloco concluído, pode dar `/clear` e rodar `/bloco G`"*.
