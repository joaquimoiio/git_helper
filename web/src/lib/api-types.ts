/* Espelho dos tipos serde do `porc-server`. Arquivo único e escrito à mão: enquanto a API
 * couber numa tela, gerar isto de um schema custa mais manutenção do que resolve.
 *
 * O Rust nomeia em `snake_case` e serializa em `camelCase`
 * (`#[serde(rename_all = "camelCase")]`), então é o `camelCase` que aparece aqui. */

/** `GET /health` — rota pública, é o que a segunda instância consulta. */
export interface Health {
  name: string;
  version: string;
  pid: number;
  instanceId: string;
}

/** `POST /api/v1/session` — troca o token de boot pelos cookies. */
export interface SessionRequest {
  token: string;
}

export interface SessionResponse {
  ok: boolean;
}

/** `GET /api/v1/whoami` — provisória; vira informação de repositório no Bloco C. */
export interface WhoAmI {
  authenticated: boolean;
  pid: number;
  version: string;
}

export interface Ok {
  ok: boolean;
}

/** Estado do `HEAD`. `unborn` é o repositório recém-criado: tem branch, ainda não tem commit. */
export type Head =
  | { kind: "branch"; name: string; commit: string }
  | { kind: "detached"; commit: string }
  | { kind: "unborn"; name: string };

/**
 * `POST /api/v1/repos` e `GET /api/v1/repos/{repoId}`.
 *
 * Depois desta resposta o cliente só fala em `repoId`: nenhuma outra rota de git aceita
 * caminho. `path` está aqui para mostrar ao usuário, não para devolver ao servidor.
 */
export interface Repo {
  repoId: string;
  path: string;
  name: string;
  bare: boolean;
  detached: boolean;
  head: Head;
  /** O nome já resolvido do `head` — branch, ou hash curto em detached. */
  branch: string;
}

/**
 * `GET /api/v1/recents` — repositórios lembrados em disco, entre boots.
 *
 * Reabrir é `POST /api/v1/repos` com o `path`: não há rota que abra por id, porque o banco é
 * um arquivo que o usuário pode editar e o confinamento precisa ser revalidado.
 */
export interface RecentEntry {
  repoId: string;
  path: string;
  name: string;
  /** Epoch em milissegundos. */
  openedAt: number;
  /** `false` quando a pasta sumiu ou deixou de ser repositório. */
  available: boolean;
}

/** Uma subpasta em `GET /api/v1/fs/list`. `path` é sempre canônico — é o que volta no `?path=`. */
export interface FsEntry {
  name: string;
  path: string;
  isRepo: boolean;
  hidden: boolean;
}

/* ---------------------------------------------------------------------------- jobs */

export type JobState = "running" | "done" | "error" | "cancelled";

export interface Progress {
  phase: string;
  /** `null` é fase indeterminada — o git passa boa parte do clone sem total conhecido. */
  fraction: number | null;
  detail: string | null;
}

/**
 * Um pedido de senha em aberto.
 *
 * Mora no estado do job, e não só no evento do socket: é assim que uma aba recarregada no meio de
 * um clone por SSH reencontra o campo de senha.
 */
export interface PendingPrompt {
  promptId: string;
  /** O texto que o ssh escreveria no terminal — inclui o caminho da chave. */
  prompt: string;
}

/** O mesmo objeto em `GET /api/v1/jobs/{id}` e no evento `job.done`. */
export interface JobSnapshot {
  jobId: string;
  kind: string;
  state: JobState;
  progress: Progress | null;
  log: string[];
  /** Resultados que chegaram **durante** o job, um a um — o pickaxe usa isto. Cauda capada
   * (as mais recentes); o `result` do `job.done` traz a lista completa, sem corte. */
  hits: unknown[];
  startedAt: number;
  finishedAt: number | null;
  /** Texto legível quando `state` é `error`. Nunca stderr cru. */
  message: string | null;
  /** A próxima coisa a fazer. `null` quando não há conselho honesto a dar. */
  action: string | null;
  /** `null` é o caso normal; preenchido, o job está parado esperando uma senha. */
  pendingPrompt: PendingPrompt | null;
  result: unknown;
}

export interface Accepted {
  jobId: string;
}

/** O que chega pelo WebSocket. Um socket só, multiplexado — o `type` diz o assunto. */
export type ServerMessage =
  | { type: "ready" }
  | { type: "pong" }
  /** Ficamos para trás e perdemos eventos: recarregar o estado por HTTP é a saída. */
  | { type: "resync" }
  | { type: "job.progress"; jobId: string; progress: Progress }
  | { type: "job.log"; jobId: string; line: string }
  /** Um resultado achado durante o job — o pickaxe filtrando conforme os oids chegam. */
  | { type: "job.hit"; jobId: string; hit: unknown }
  | { type: "job.done"; job: JobSnapshot }
  | { type: "job.error"; jobId: string; message: string }
  /** O git ou o ssh está pedindo uma senha; o job está parado até alguém responder. */
  | { type: "job.askpass"; jobId: string; promptId: string; prompt: string }
  /** O pedido expirou ou o job morreu: tirar o prompt da tela. */
  | { type: "job.askpassClosed"; jobId: string; promptId: string };

/** `POST /api/v1/jobs/{jobId}/askpass`. O segredo vai daqui direto para o socket do helper. */
export interface AskpassAnswer {
  promptId: string;
  secret: string;
}

/**
 * `POST /api/v1/jobs/clone` — começa o clone e responde `202` com o `jobId`.
 *
 * A validação inteira acontece antes do 202: quem recebeu o id sabe que a URL é aceitável e que a
 * pasta de destino serve. O que pode dar errado depois é rede e credencial, e isso vira evento.
 */
export interface CloneRequest {
  url: string;
  /** Pasta existente onde o clone cai. */
  path: string;
  /** Nome da pasta a criar. Ausente, sai da URL. */
  folder?: string;
  branch?: string;
  depth?: number;
  recurseSubmodules?: boolean;
  /** Nome do remote. Ausente é `origin`. */
  remote?: string;
}

/** `POST /api/v1/repos/init` — cria o repositório e já devolve ele aberto. */
export interface InitRequest {
  /** Pasta existente onde o repositório nasce. */
  path: string;
  /** Subpasta a criar. Ausente inicializa a própria `path`. */
  name?: string;
  /** Ausente deixa o `init.defaultBranch` do usuário decidir. */
  branch?: string;
}

/** Um repositório achado pela varredura de `GET /api/v1/fs/scan`. */
export interface ScanEntry {
  name: string;
  path: string;
  /** Caminho relativo à raiz — é o que distingue dois repos de mesmo nome. */
  relative: string;
  depth: number;
}

/**
 * `GET /api/v1/fs/scan` — repositórios debaixo da raiz configurada.
 *
 * Sem parâmetros: raiz e profundidade saem do `config.toml`. Deixar o cliente escolher onde e
 * quão fundo varrer seria dar a ele um `find` na máquina do usuário.
 */
export interface Scan {
  root: string;
  depth: number;
  /** `true` quando a varredura bateu no teto: a lista está incompleta. */
  truncated: boolean;
  repos: ScanEntry[];
}

/** `GET /api/v1/fs/list` — o seletor de pastas mora no servidor; o navegador não tem um. */
export interface FsListing {
  path: string;
  /** `null` na raiz do confinamento: não há para onde subir. */
  parent: string | null;
  root: string;
  entries: FsEntry[];
}

/* ---------------------------------------------------------------------------- log */

/**
 * Uma linha do log, já com a coluna do grafo (Passo 37: lanes calculadas no servidor).
 *
 * `parentLanes[i]` é a coluna para onde a aresta de `parents[i]` desce. `null` é a fronteira
 * de um clone raso — o pai existe na mensagem do commit, mas não localmente, e a linha não
 * continua para lugar nenhum. Diferente disso é um pai cujo commit ainda não chegou nesta
 * página: a coluna já é conhecida (`parentLanes[i]` não é `null`), só o commit em si que o
 * cliente ainda não tem — e é o próprio cliente quem decide isso, comparando o oid do pai
 * contra o que já carregou (Passo 38).
 */
export interface Commit {
  /** Hash completo. A UI corta os 7 primeiros para exibir. */
  oid: string;
  /** Em ordem: o primeiro pai é a linha principal. Mais de um significa merge. */
  parents: string[];
  author: string;
  email: string;
  /** Data do autor, em segundos desde a época. */
  time: number;
  /** Fuso do autor em minutos. */
  offset: number;
  /** Primeira linha da mensagem. */
  summary: string;
  /** Coluna do grafo desta linha. */
  lane: number;
  parentLanes: (number | null)[];
}

/** `GET /api/v1/repos/{repoId}/log` — uma página do log, a partir do `HEAD` ou do cursor. */
export interface LogPage {
  commits: Commit[];
  /** `null` na última página. Opaco para o cliente: volta como veio. */
  nextCursor: string | null;
}

/**
 * Nome, e-mail e quando — de quem escreveu o commit ou de quem o aplicou. Mesma forma para os
 * dois porque o git as guarda da mesma forma; só o significado muda.
 */
export interface Signature {
  name: string;
  email: string;
  /** Em segundos desde a época. */
  time: number;
  /** Em minutos, para a UI mostrar a hora local de quem assinou. */
  offset: number;
}

export type FileChangeKind = "added" | "modified" | "deleted" | "renamed" | "copied" | "typechange";

/** Um arquivo tocado pelo commit, com contagem de linhas — não o patch em si (Passo 41). */
export interface FileChange {
  /** Caminho atual; em `deleted`, o caminho que o arquivo tinha antes de sumir. */
  path: string;
  /** Só presente em `renamed`/`copied`, e só quando difere de `path`. */
  oldPath: string | null;
  kind: FileChangeKind;
  insertions: number;
  deletions: number;
  /** `true` quando o git marcou o arquivo como binário — contagem de linhas não tem sentido. */
  binary: boolean;
}

/**
 * `GET /api/v1/repos/{repoId}/commits/{oid}` — o que o painel de detalhe mostra ao selecionar
 * uma linha do log. O diff é contra o primeiro pai só (vazio, na raiz) — igual ao `git show`
 * padrão num merge.
 */
export interface CommitDetail {
  oid: string;
  parents: string[];
  author: Signature;
  committer: Signature;
  /** Mensagem completa: resumo (primeira linha) e corpo. */
  message: string;
  insertions: number;
  deletions: number;
  files: FileChange[];
}

/**
 * `GET /api/v1/repos/{repoId}/compare?from=&to=` — a diferença acumulada entre dois pontos
 * quaisquer do histórico (Passo 56).
 *
 * Não é `CommitDetail` com outro nome: não há autor, committer nem mensagem, porque não há um
 * commit sendo mostrado. `from` e `to` voltam **resolvidos** em oid, mesmo quando o pedido foi
 * por nome de branch — é assim que fica claro o que foi comparado de verdade.
 */
export interface RangeDiff {
  from: string;
  to: string;
  insertions: number;
  deletions: number;
  files: FileChange[];
}

export type DiffLineKind = "context" | "addition" | "deletion";

/** Uma linha dentro de um hunk. Sem o `\n` final — apresentação decide como quebrar. */
export interface DiffLine {
  kind: DiffLineKind;
  /** Número no arquivo antigo. `null` em `addition` — a linha não existia lá. */
  oldLineno: number | null;
  /** Número no arquivo novo. `null` em `deletion` — a linha não existe mais aqui. */
  newLineno: number | null;
  content: string;
}

/** Um trecho contíguo de mudança, com um pouco de contexto ao redor. */
export interface DiffHunk {
  /** A linha `@@ -a,b +c,d @@ …` que o git gera. */
  header: string;
  lines: DiffLine[];
}

/**
 * `GET /api/v1/repos/{repoId}/commits/{oid}/diff?path=…` — o diff de **um** arquivo, sob
 * demanda. Três formas, não hunks vazios com uma flag: `binary`/`notUtf8` não têm o que um
 * hunk mostraria.
 */
export type FileDiff =
  | { kind: "text"; hunks: DiffHunk[] }
  /** O git marcou o arquivo como binário. */
  | { kind: "binary" }
  /** Texto de verdade, mas alguma linha não é UTF-8 válido — encoding legado, não binário. */
  | { kind: "notUtf8" };

/**
 * `GET /api/v1/repos/{repoId}/search?q=…` — um resultado de busca por mensagem/autor
 * (FTS5, Passo 43). Linha completa, não só `oid`: a busca cobre o histórico indexado
 * inteiro, que pode ir muito além do que o log já carregou por rolagem.
 */
export interface SearchHit {
  oid: string;
  author: string;
  email: string;
  /** Em segundos desde a época. */
  time: number;
  summary: string;
}

export type RefKind = "branch" | "remote" | "tag" | "head";

/**
 * `GET /api/v1/repos/{repoId}/refs` — toda ponta do repositório, para marcar linhas do log.
 *
 * Não pagina: o número de branches, remotas e tags não cresce com o histórico. O cliente
 * cruza `commit` (oid) contra os commits que o log já tem — o servidor não sabe quais.
 */
export interface RefMarker {
  /** Nome curto: `main`, `origin/main`, `v1.0`. Nunca o `refs/heads/…` inteiro. */
  name: string;
  kind: RefKind;
  commit: string;
  /** `true` quando é a ponta para a qual o `HEAD` aponta agora. */
  isHead: boolean;
}

/**
 * `GET /api/v1/repos/{repoId}/remotes` — os remotes configurados, o que o `git remote -v` lista.
 *
 * A sidebar precisa deles para **agrupar** as remotas: o nome curto de uma remota é
 * `origin/main`, e quem sabe onde o nome do remote termina é o próprio git (ele aceita remote
 * com barra no nome).
 */
export interface Remote {
  name: string;
  /** `null` num remote sem URL de fetch — raro, mas o git aceita a configuração. */
  fetchUrl: string | null;
  /** Só presente quando existe `remote.<nome>.pushurl` própria. */
  pushUrl: string | null;
}

/** `GET /api/v1/repos/{repoId}/stashes` — a pilha de stash, do topo para o fundo. */
export interface StashEntry {
  /** Posição na pilha: 0 é o topo, o `stash@{0}` do terminal. */
  index: number;
  oid: string;
  /** Como o git a escreveu (`WIP on main: 1234567 assunto`), sem reescrita nossa. */
  message: string;
}

/* -------------------------------------------------------------------------- status */

/**
 * Qual dos dois diffs do trabalho local — o que o terminal escreve como `git diff` e
 * `git diff --cached`. Parâmetro de `GET /repos/{repoId}/diff` e campo de `apply`.
 */
export type DiffSide = "unstaged" | "staged";

/**
 * O que aconteceu com um caminho no working tree.
 *
 * `unmerged` é conflito (linha `u` do porcelain v2). Não abre um quarto grupo: vem dentro de
 * `unstaged`, visível, e o Bloco G decide o que fazer com ele.
 */
export type StatusKind =
  | "added"
  | "modified"
  | "deleted"
  | "renamed"
  | "copied"
  | "typechange"
  | "unmerged"
  | "untracked";

/** Um caminho dentro de um dos três grupos do status. */
export interface StatusEntry {
  path: string;
  /** Só presente em `renamed`/`copied`. */
  oldPath: string | null;
  kind: StatusKind;
}

/** O cabeçalho `# branch.*` do porcelain v2 — quem é o `HEAD` e como ele está frente ao upstream. */
export interface BranchStatus {
  /** `null` em repositório sem commit nenhum. */
  oid: string | null;
  /** `null` com `HEAD` destacado — o oid já diz tudo nesse caso. */
  head: string | null;
  detached: boolean;
  /** Ausente quando a branch não tem upstream configurado. */
  upstream: string | null;
  ahead: number;
  behind: number;
}

/** Operação em andamento que ainda não terminou. `clean` é o caso normal. */
export type RepoState = "clean" | "merge" | "rebase" | "cherryPick" | "revert" | "bisect";

/** `GET /api/v1/repos/{repoId}/status` — e também o que `stage`/`unstage` devolvem já atualizado. */
export interface WorktreeStatus {
  branch: BranchStatus;
  state: RepoState;
  staged: StatusEntry[];
  unstaged: StatusEntry[];
  untracked: StatusEntry[];
}

/**
 * `POST /api/v1/repos/{repoId}/stage` e `/unstage`.
 *
 * A lista é sempre explícita: não existe "tudo" implícito por lista vazia (o servidor recusa
 * com 400). Selecionar tudo é o cliente mandando os caminhos que o último `status` trouxe.
 */
export interface StagePathsRequest {
  paths: string[];
}

/**
 * `POST /api/v1/repos/{repoId}/apply` — move trechos de um lado para o outro.
 *
 * `side` diz **onde os trechos estão agora**, e com isso para onde eles vão: de `unstaged` é
 * preparar, de `staged` é desfazer. Os índices são os do `GET /diff` do mesmo arquivo e do mesmo
 * lado — é a mesma numeração porque é o mesmo diff.
 */
export interface ApplyHunksRequest {
  path: string;
  side: DiffSide;
  hunks: HunkPick[];
}

/** `POST /api/v1/repos/{repoId}/commit`. Assunto e corpo já juntos — quem limpa é o git. */
export interface CommitRequest {
  message: string;
  /** Reescreve o commit anterior em vez de criar um novo. */
  amend?: boolean;
  /** Acrescenta o trailer `Signed-off-by:` com a identidade configurada. */
  signoff?: boolean;
  /**
   * Ausente respeita o `commit.gpgSign` do usuário — que é o certo por padrão. Presente força,
   * só para este commit e por flag: o app nunca escreve na config dele.
   */
  gpgSign?: boolean;
  /**
   * Oid do commit que este corrige. Presente, **a mensagem é do git** (`fixup! <assunto>`) e o
   * campo `message` é ignorado — é essa mensagem exata que o `rebase --autosquash` reconhece.
   */
  fixup?: string;
}

export interface CommitDone {
  /** Oid completo do commit recém-criado. */
  oid: string;
  /** O status logo depois, que quase sempre é bem mais vazio do que antes. */
  status: WorktreeStatus;
}

/** `GET /api/v1/repos/{repoId}/commit/template` — o `commit.template` do usuário, se houver. */
export interface CommitTemplate {
  /** `null` quando não há `commit.template` configurado — o caso comum. */
  template: string | null;
}

/**
 * `POST /api/v1/repos/{repoId}/discard/hunks` — desfaz trechos **no arquivo em disco**.
 *
 * Sempre do lado `unstaged`: descartar é jogar fora o que não está no índice. O que está no
 * índice se desfaz com `unstage`, sem perder nada.
 */
export interface DiscardHunksRequest {
  path: string;
  hunks: HunkPick[];
}

/** Um trecho escolhido: o hunk inteiro, ou linhas dentro dele. */
export interface HunkPick {
  hunk: number;
  /**
   * Ausente é o hunk inteiro. Presente são as **linhas de mudança** escolhidas, numeradas só
   * entre as de mudança do hunk — a primeira adição ou remoção é 0. Contexto não entra na
   * conta, dos dois lados do fio: contexto não muda de lado.
   */
  lines?: number[];
}
