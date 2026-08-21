# BLOCO D — o log (o coração)

**Objetivo:** o log com grafo e a busca. É o bloco que define se o projeto presta.

**Ao fim do bloco:** repo de 100k commits abre instantaneamente, rola sem engasgo, e a
busca filtra o grafo ao vivo — inclusive busca por conteúdo (pickaxe).

---

## Detalhe técnico do bloco

**Paginação.** `git2::Revwalk` com `TOPOLOGICAL | TIME`. Página = 500 commits. Junto com
a página, o servidor devolve um **snapshot do estado das lanes** no ponto de corte. O
cursor é `base64(último_oid + snapshot)`, opaco. A página seguinte continua o desenho sem
recalcular desde o início — é isso que faz o custo não crescer com o scroll.

**Os 500ms nunca são 100k commits renderizados.** A primeira página desenha em <50ms, o
resto é virtualizado, e a barra de rolagem usa o total de um `rev-list --count` que roda
em background. O que precisa ser rápido é o **scroll**.

**Grafo em Canvas.** Layout de lanes calculado no Rust e enviado pronto (colunas +
arestas). O React renderiza **um** `<canvas>` sobreposto à lista virtualizada,
redesenhado no scroll. A lista de texto continua DOM (selecionável, acessível); só o
grafo é pintado. Linhas finas, **sem arco-íris** — diferenciação por posição e peso.

**Arestas longas.** Um merge cujo segundo pai está 40k commits abaixo cria aresta que
atravessa dezenas de páginas. Carregar como linha real destrói a paginação. Arestas que
saem do intervalo carregado viram **stub com marcador**, materializando só se o destino
entrar na viewport.

**Duas classes de busca, deliberadamente diferentes:**

| Classe | O quê | Como | Meta |
|---|---|---|---|
| Indexável | mensagem, corpo, autor, e-mail, data, hash | SQLite + FTS5 | < 20ms |
| Não-indexável | conteúdo (`-S`/`-G`), caminho | shell-out em streaming, cancelável | primeiros hits < 150ms |

Para conteúdo/caminho o critério **não** é resultado completo em 150ms — é primeiros hits
em 150ms com o resto fluindo por WS. É isso que produz a sensação de instantâneo.

**Índice.** Job de background ao abrir o repo: revwalk completo → tabela `commits`
(oid, parents, author, email, ts, summary) + FTS5 sobre `summary|body|author`. ~2-4s em
100k commits, com o log **já usável** durante — a indexação nunca bloqueia o boot; a
busca por mensagem fica indisponível por alguns segundos, com indicador na UI.
Manutenção: reindexar só o delta desde as refs conhecidas. Após `gc`/rebase, oids órfãos
são tolerados na leitura e limpos por um prune de alcançabilidade. Deletar o `.db` e
reconstruir é sempre saída válida.

---

## Passos

### Passo 35 — `GET /log` paginado
Revwalk com cursor opaco, sem grafo ainda: oid, autor, e-mail, timestamp, summary,
parents. `limit` default 500.
**Aceite:** `curl` traz 500 commits e o cursor; passar o cursor traz os 500 seguintes.

### Passo 36 — lista virtualizada
TanStack Virtual + infinite query. Linha slim: hash curto (mono), autor, data relativa,
mensagem. Navegação por teclado (j/k, setas, Home/End, PageUp/Down).
**Aceite:** rolar 100k commits sem engasgo; medir o tempo até a primeira pintura.

### Passo 37 — lanes do grafo no servidor
Algoritmo incremental de lanes O(n·lanes), com snapshot no cursor. Arestas fora do
intervalo viram stub. Testes com histórico sintético cheio de merges.
**Aceite:** duas páginas consecutivas encaixam sem descontinuidade visual nos dados.

### Passo 38 — grafo em canvas
Canvas sobreposto à lista virtualizada, redesenhado no scroll, linhas finas monocromáticas.
**Aceite:** o grafo acompanha o scroll sem defasagem em repo com muitos merges.

### Passo 39 — refs na linha do commit
Branches locais/remotas, tags e HEAD marcados na própria linha, sem estourar a densidade.
**Aceite:** o commit de HEAD e os de ponta de branch aparecem marcados.

### Passo 40 — detalhe do commit
Painel direito: mensagem completa, autor/committer, datas, parents clicáveis, stats e
lista de arquivos.
**Aceite:** selecionar um commit no log preenche o detalhe.

### Passo 41 — diff do commit
Diff via git2 com hunks estruturados, realce leve, modo unificado e lado a lado.
Detecção de binário e de conteúdo não-UTF8 → modo "ilegível" com aviso, nunca crash nem
JSON quebrado.
**Aceite:** ver o diff de um commit de texto e de um que troca um PNG.

### Passo 42 — job de indexação
Ao abrir o repo, indexar em background com progresso na UI, sem bloquear o log.
**Aceite:** abrir repo grande, usar o log imediatamente, ver o indicador terminar.

### Passo 43 — busca por mensagem e autor
FTS5, incremental a cada tecla (com debounce curto), filtrando o grafo ao vivo — sem
janela separada.
**Aceite:** digitar filtra em < 20ms medidos no servidor.

### Passo 44 — busca por hash e por data
Prefixo de hash (curto ou completo) com índice, e intervalo de datas. Sintaxe unificada
na mesma caixa (`autor:`, `depois:`, `antes:`).
**Aceite:** colar um hash curto salta direto para o commit.

### Passo 45 — filtro por caminho
`git log -- <path>` em streaming, cancelável, com autocomplete de caminho.
**Aceite:** filtrar por um arquivo e ver só os commits que o tocam.

### Passo 46 — busca por conteúdo (pickaxe)
`git log -S` e `-G` em streaming por WS, cancelado e reiniciado a cada tecla, com os
oids filtrando o grafo conforme chegam e indicador de "buscando".
**Aceite:** buscar uma string que existe no histórico e ver os primeiros resultados
aparecerem quase de imediato num repo grande.

---

**Fim do bloco:** atualizar `PROGRESSO.md` e dizer
*"bloco concluído, pode dar `/clear` e rodar `/bloco E`"*.
