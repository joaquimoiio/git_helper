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
