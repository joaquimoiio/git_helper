# BLOCO C — abrir repositório

**Objetivo:** chegar num repositório — abrindo do disco, criando com `init`, ou clonando
com progresso real e cancelamento. Aqui nasce a infra de **jobs**, que os blocos F e H
reutilizam inteira.

**Ao fim do bloco:** dá para clonar um repo grande vendo objetos/deltas/throughput ao
vivo, cancelar no meio e não sobrar pasta parcial.

---

## Detalhe técnico do bloco

**Navegador de pastas pelo backend.** O navegador não escolhe diretórios, então o seletor
é uma rota. `GET /api/v1/fs/list?path=` devolve subdiretórios + flag `is_repo`.
Canonicaliza com `fs::canonicalize` (resolve symlinks) e verifica prefixo contra a raiz
configurada. `..`, symlink apontando para fora e path não-canônico → 403.

**`repo_id`.** Hash opaco do path canônico. Nenhuma rota de git aceita path de repo vindo
do cliente — é isso que mata path traversal na origem. O registry em memória só é
preenchido por ação explícita do usuário.

**Infra de jobs (o núcleo deste bloco):**
- Registry `JobId → { CancellationToken, Child, cleanup: Vec<Action> }`.
- `POST /api/v1/jobs/<tipo>` → `202 { job_id }`. `DELETE /api/v1/jobs/:id` cancela.
- `GET /api/v1/jobs/:id` devolve o último estado — é assim que a UI reconecta sem perder
  progresso se a aba recarregar.
- Progresso vai pelo WS: `job.progress | job.log | job.done | job.error`.
- Processo em **process group próprio** (`setsid`). Cancelar mata o grupo, senão
  `git-remote-https` e `ssh` ficam órfãos.
- **Cleanup registrado antes de começar.** No clone: remover o destino **se e somente se
  fomos nós que criamos o diretório**. Nunca apagar pasta preexistente.

**Parser de `--progress`.** O git usa **`\r`** para atualizar a linha, não `\n` — ler
stderr em chunks e separar por CR. Eventos tipados: `CountingObjects`,
`Compressing{pct}`, `Receiving{objs, total, bytes, bytes_per_s}`, `ResolvingDeltas{pct}`.

**Askpass.** Passphrase SSH nunca toca o disco. `GIT_ASKPASS` aponta para o próprio
binário em modo `porc askpass`, que busca o valor num socket unix efêmero (0600). Nada em
`argv`, que é visível em `ps`. E todo shell-out leva `GIT_TERMINAL_PROMPT=0` + timeout,
senão um prompt inesperado trava o job para sempre.

**Erros legíveis.** Nunca despejar stderr cru. Mapear pelo menos: host desconhecido,
host key não confiável, chave rejeitada / permission denied, repositório inexistente
(404), sem permissão (403), rede indisponível, destino já existe e não está vazio.

---

## Passos

### Passo 23 — `GET /api/v1/fs/list`
Rota listando subdiretórios de um caminho, com `is_repo`, canonicalização e confinamento
à raiz configurável. Ocultos escondidos por padrão.
**Aceite:** `curl` lista o home; `?path=../../etc` dá 403.

### Passo 24 — navegador de pastas na UI
Componente de árvore/coluna navegável 100% por teclado (setas, Enter, Backspace),
mostrando quais pastas já são repositórios.
**Aceite:** navegar até um repo do disco só com o teclado.

### Passo 25 — abrir repositório
`POST /api/v1/repos` com o path escolhido: valida, registra, devolve `repo_id`, HEAD,
branch atual e se está detached/bare. Sidebar mostra o repo aberto.
**Aceite:** abrir um repo real e ver o nome da branch atual na UI.

### Passo 26 — lista de recentes
`porc-index` cria o SQLite e persiste os repositórios abertos com timestamp. Tela inicial
lista os recentes com atalho numérico.
**Aceite:** fechar o servidor, subir de novo, o repo aparece em recentes.

### Passo 27 — detecção automática de repos
Raiz configurável em `~/.config/porcelain/config.toml`; varredura com profundidade
limitada listando os repos encontrados.
**Aceite:** configurar a raiz e ver a lista dos repos daquela pasta.

### Passo 28 — `git init`
Criar repositório numa pasta existente (com escolha do nome da branch inicial),
abrindo-o em seguida.
**Aceite:** criar um repo novo e vê-lo aberto com branch `main` e zero commits.

### Passo 29 — infra de jobs + WS
Registry de jobs, `job_id`, WebSocket multiplexado (com guard de Origin no upgrade),
cliente reconectando e recuperando estado via `GET /jobs/:id`. Um job de teste que conta
até 10 prova o canal.
**Aceite:** iniciar o job de teste, ver o progresso na UI, recarregar a aba e o progresso
continuar de onde estava.

### Passo 30 — parser de `--progress`
`porc-git/parse/progress.rs` transformando o stderr do git (separado por `\r`) em eventos
tipados, com testes unitários sobre saída real capturada.
**Aceite:** `cargo test` passa com amostras de stderr de clone real.

### Passo 31 — clone com progresso
`POST /api/v1/jobs/clone` com URL, destino, nome da pasta, branch, `--depth`,
`--recurse-submodules` e nome do remote. Progresso real: objetos recebidos, deltas
resolvidos, throughput.
**Aceite:** clonar um repo público e ver as barras se moverem de verdade.

### Passo 32 — cancelar clone
Botão de cancelar → `DELETE /api/v1/jobs/:id` → kill do process group + remoção da pasta
parcial **que nós criamos**.
**Aceite:** cancelar no meio de um clone grande; o processo morre e a pasta some. Cancelar
um clone para dentro de pasta preexistente **não** apaga a pasta.

### Passo 33 — askpass (passphrase SSH)
Modo `porc askpass` + socket unix efêmero + prompt na UI. `GIT_TERMINAL_PROMPT=0` e
timeout em todos os shell-outs.
**Aceite:** clonar via SSH com chave protegida por passphrase, digitando na interface.

### Passo 34 — erros de clone legíveis
Mapeamento de stderr para mensagens humanas + ação sugerida, com o stderr cru disponível
atrás de "ver detalhes" (nunca como mensagem principal).
**Aceite:** clonar de host inexistente, de repo inexistente e com chave errada → três
mensagens distintas e compreensíveis.

---

**Fim do bloco:** atualizar `PROGRESSO.md` e dizer
*"bloco concluído, pode dar `/clear` e rodar `/bloco D`"*.
