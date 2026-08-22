//! `POST /api/v1/repos` — abrir um repositório — e as leituras por `repo_id`.
//!
//! O `POST` é a **única** rota de git que aceita caminho vindo do cliente, e por isso passa
//! pelo mesmo `resolve` do `fs/list`. Depois dele tudo é `repo_id`: `GET /api/v1/repos/{id}`
//! não sabe converter caminho nenhum, só consultar o registry.

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use porc_git::{
    exec::branch::{BranchError, CreateOptions},
    exec::commit::{CommitError, CommitOptions},
    exec::init::{InitError, InitOptions},
    exec::ExecError,
    model::{
        CommitDetail, FileDiff, Head, LogPage, RefMarker, Remote, RepoInfo, StashEntry,
        WorktreeStatus,
    },
    read::{DiffSide, Git2Repo, GitError, LogQuery, RepoRead},
};
use porc_index::commits::SearchHit;
use serde::{Deserialize, Serialize};

use crate::{routes::fs::FsError, AppState};

/// Tamanho de página do log. 500 é o número do bloco: cabe no orçamento de 50ms da primeira
/// pintura e ainda cobre bem mais do que uma tela.
const LOG_LIMIT_DEFAULT: usize = 500;

/// Teto do que o cliente pode pedir de uma vez. Não é arbitrário: acima disso a página deixa
/// de ser "a próxima tela" e vira um jeito de fazer o servidor varrer o histórico inteiro num
/// request só.
const LOG_LIMIT_MAX: usize = 2000;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenRequest {
    /// Absoluto dentro da raiz, ou relativo a ela — o mesmo contrato do `fs/list`.
    pub path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repo {
    pub repo_id: String,
    #[serde(flatten)]
    pub info: RepoInfo,
}

#[derive(Debug, thiserror::Error)]
pub enum RepoError {
    #[error(transparent)]
    Fs(#[from] FsError),
    #[error("{0} não é um repositório git")]
    NotARepository(String),
    #[error("repositório não está aberto")]
    Unknown,
    #[error(transparent)]
    Git(#[from] GitError),
    #[error(transparent)]
    Init(#[from] InitError),
    #[error(transparent)]
    Index(#[from] porc_index::IndexError),
    /// `git status` shell-out falhou — diferente de `Git`, que é sempre `git2`.
    #[error(transparent)]
    Status(#[from] ExecError),
    #[error("a leitura do repositório não terminou")]
    Join,
    #[error("nenhum caminho selecionado")]
    NoPaths,
    /// O recorte do patch não deu certo — seleção vazia, hunk que não existe, patch que não
    /// dá para interpretar. É sempre pedido malformado, nunca falha do servidor.
    #[error(transparent)]
    Patch(#[from] porc_git::patch::PatchError),
    #[error(transparent)]
    Commit(#[from] porc_git::exec::commit::CommitError),
    #[error(transparent)]
    Branch(#[from] BranchError),
}

impl IntoResponse for RepoError {
    fn into_response(self) -> Response {
        // Delegado inteiro: o `fs` já decide entre 403, 404 e 400, e repetir a tabela aqui só
        // criaria duas versões da mesma regra.
        if let RepoError::Fs(err) = self {
            return err.into_response();
        }

        let status = match self {
            RepoError::NotARepository(_) => StatusCode::BAD_REQUEST,
            RepoError::Unknown => StatusCode::NOT_FOUND,
            // Cursor que não decodifica, ou que aponta para objeto de outro repositório, é
            // pedido malformado — não falha da leitura. Mesmo raciocínio para um oid de commit
            // que não decodifica ou não existe neste repositório.
            RepoError::Git(GitError::InvalidCursor | GitError::InvalidCommit) => {
                StatusCode::BAD_REQUEST
            }
            // Diferente do oid: o `path` do diff é bem formado, só não é um dos arquivos que
            // este commit tocou — o recurso pedido é que não existe.
            RepoError::Git(GitError::FileNotInCommit | GitError::FileUnchanged) => {
                StatusCode::NOT_FOUND
            }
            // `AlreadyARepository` é 409: o pedido está bem formado, o mundo é que já está do
            // jeito que ele queria criar.
            RepoError::Init(InitError::AlreadyARepository(_)) => StatusCode::CONFLICT,
            RepoError::Init(InitError::Exec(_)) => StatusCode::INTERNAL_SERVER_ERROR,
            RepoError::Init(_) => StatusCode::BAD_REQUEST,
            RepoError::NoPaths | RepoError::Patch(_) => StatusCode::BAD_REQUEST,
            // Mensagem vazia é pedido malformado; o resto (nada preparado, identidade faltando,
            // hook recusando) é o mundo dizendo não a um pedido bem formado — 409, não 500.
            RepoError::Commit(CommitError::EmptyMessage | CommitError::InvalidFixupTarget) => {
                StatusCode::BAD_REQUEST
            }
            RepoError::Commit(CommitError::Exec(_)) => StatusCode::INTERNAL_SERVER_ERROR,
            RepoError::Commit(_) => StatusCode::CONFLICT,
            // Nome e ponto de partida são validados antes de chamar o git: recusa ali é pedido
            // malformado. Ponto de partida que não resolve também é 400, e não 404 — mesma régua
            // do oid de commit que não existe neste repositório, logo acima.
            RepoError::Branch(
                BranchError::InvalidName(_)
                | BranchError::InvalidStart(_)
                | BranchError::UnknownStart(_),
            ) => StatusCode::BAD_REQUEST,
            RepoError::Branch(BranchError::Exec(_)) => StatusCode::INTERNAL_SERVER_ERROR,
            // Branch que já existe, worktree que seria sobrescrita, hook que recusou: o pedido
            // está bem formado, o mundo é que disse não.
            RepoError::Branch(_) => StatusCode::CONFLICT,
            // Recortar um patch de arquivo em encoding legado seria produzir um patch que não
            // bate mais com o conteúdo. O pedido está bem formado; o arquivo é que não serve.
            RepoError::Git(GitError::NotUtf8) => StatusCode::UNPROCESSABLE_ENTITY,
            RepoError::Git(_) | RepoError::Join | RepoError::Index(_) | RepoError::Status(_) => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
            RepoError::Fs(_) => unreachable!("tratado acima"),
        };

        // O detalhe do libgit2 vai para o log, não para a tela: a mensagem que o usuário lê é
        // a do `thiserror`, em português.
        if let RepoError::Git(GitError::Read(err)) = &self {
            tracing::warn!(%err, "falha lendo repositório");
        }

        (status, self.to_string()).into_response()
    }
}

/// O oid que o `HEAD` aponta agora, se houver algum — é contra isto que o job de indexação
/// decide se há trabalho novo. `unborn` não tem commit nenhum, então não tem o que indexar.
fn head_oid(head: &Head) -> Option<String> {
    match head {
        Head::Branch { commit, .. } | Head::Detached { commit } => Some(commit.clone()),
        Head::Unborn { .. } => None,
    }
}

/// Abre o repositório e devolve quem ele é. Idempotente: abrir duas vezes devolve o mesmo id.
pub async fn open(
    State(state): State<AppState>,
    Json(body): Json<OpenRequest>,
) -> Result<Json<Repo>, RepoError> {
    let settings = state.settings.clone();
    let registry = state.repos.clone();
    let index = state.index.clone();

    // Todo o corpo é bloqueante: `canonicalize`, os `stat` do `is_repo`, o libgit2 e o SQLite.
    let (repo, path) = tokio::task::spawn_blocking(move || {
        let path = crate::routes::fs::resolve(&settings.root, Some(&body.path))?;
        let repo = register_opened(&registry, &index, &path)?;
        Ok::<_, RepoError>((repo, path))
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao abrir repositório");
        RepoError::Join
    })??;

    tracing::info!(repo = %repo.info.path, branch = %repo.info.branch, "repositório aberto");

    // Fogo e esquece: quem abriu não espera a indexação, e o job decide sozinho se há
    // trabalho novo (repositório já em dia é o caminho comum de reabrir).
    crate::index_job::maybe_spawn(
        &state,
        repo.repo_id.clone(),
        path,
        head_oid(&repo.info.head),
    );

    Ok(Json(repo))
}

/// Valida, lê, registra e anota nos recentes. Bloqueante — só dentro de `spawn_blocking`.
///
/// Existe porque abrir e criar terminam do mesmo jeito: a diferença entre os dois está só em
/// como o caminho chegou até aqui.
pub(crate) fn register_opened(
    registry: &crate::repos::Registry,
    index: &porc_index::Index,
    path: &std::path::Path,
) -> Result<Repo, RepoError> {
    // Checado antes de chamar o libgit2 para a mensagem ser a certa: "não é um repositório" é o
    // erro do usuário que escolheu a pasta errada, e é barato de detectar.
    if !porc_git::discover::is_repo(path) {
        return Err(RepoError::NotARepository(path.display().to_string()));
    }

    let info = Git2Repo::open(path)?.info()?;
    let repo_id = registry.register(path);

    // O índice é descartável: falhar em anotar o recente não pode impedir o usuário de abrir o
    // repositório que ele acabou de escolher. Vira log e segue.
    if let Err(err) = index.touch_recent(&repo_id, path, &info.name) {
        tracing::warn!(%err, "não consegui registrar o repositório nos recentes");
    }

    Ok(Repo { repo_id, info })
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitRequest {
    /// Pasta **existente** onde o repositório nasce. Mesmo contrato de caminho do `fs/list`.
    pub path: String,
    /// Subpasta a criar. Ausente inicializa a própria `path`.
    pub name: Option<String>,
    /// Ausente deixa o `init.defaultBranch` do usuário decidir.
    pub branch: Option<String>,
}

/// `POST /api/v1/repos/init` — cria o repositório e já o abre.
///
/// Abrir em seguida não é conveniência: um `init` que devolvesse só "ok" deixaria a UI ter que
/// adivinhar o caminho canônico do que acabou de ser criado para poder pedir o `repo_id`.
pub async fn init(
    State(state): State<AppState>,
    Json(body): Json<InitRequest>,
) -> Result<Json<Repo>, RepoError> {
    let settings = state.settings.clone();
    let requested = body.path.clone();

    let parent = tokio::task::spawn_blocking(move || {
        crate::routes::fs::resolve(&settings.root, Some(&requested))
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao resolver a pasta do init");
        RepoError::Join
    })??;

    let created = porc_git::exec::init::init(InitOptions {
        parent,
        name: body.name,
        branch: body.branch,
    })
    .await?;

    let settings = state.settings.clone();
    let registry = state.repos.clone();
    let index = state.index.clone();

    let (repo, path) = tokio::task::spawn_blocking(move || {
        // Reconferido depois de criado: o caminho passou por `create_dir` e `canonicalize`, e
        // confinamento que se checa só na entrada é confinamento que se perde numa refatoração.
        let path = crate::routes::fs::resolve(&settings.root, Some(&created.to_string_lossy()))?;

        let repo = register_opened(&registry, &index, &path)?;
        Ok::<_, RepoError>((repo, path))
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao abrir o repositório criado");
        RepoError::Join
    })??;

    tracing::info!(repo = %repo.info.path, branch = %repo.info.branch, "repositório criado");

    // Quase sempre `unborn` (repositório recém-criado sem commit), então o job nem chega a
    // nascer — mas o caminho é o mesmo do `open`, para o dia em que isso deixar de ser verdade.
    crate::index_job::maybe_spawn(
        &state,
        repo.repo_id.clone(),
        path,
        head_oid(&repo.info.head),
    );

    Ok(Json(repo))
}

/// Releitura por id — é o que a UI chama depois de um recarregamento de aba.
pub async fn get(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
) -> Result<Json<Repo>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let info = tokio::task::spawn_blocking(move || Git2Repo::open(&path)?.info())
        .await
        .map_err(|err| {
            tracing::error!(%err, "spawn_blocking falhou ao ler repositório");
            RepoError::Join
        })??;

    Ok(Json(Repo { repo_id, info }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogParams {
    /// Ausente é `LOG_LIMIT_DEFAULT`. Clampado, nunca recusado: pedir 10 mil não é ataque
    /// nem erro do usuário, é só um número que o servidor não vai atender inteiro.
    pub limit: Option<usize>,
    /// O `nextCursor` da página anterior, de volta como veio. A rota não o interpreta.
    pub cursor: Option<String>,
}

/// `GET /api/v1/repos/{repo_id}/log` — uma página do log, a partir do `HEAD` ou do cursor.
pub async fn log(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Query(params): Query<LogParams>,
) -> Result<Json<LogPage>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let query = LogQuery {
        limit: params
            .limit
            .unwrap_or(LOG_LIMIT_DEFAULT)
            .clamp(1, LOG_LIMIT_MAX),
        cursor: params.cursor,
    };

    // Revwalk é libgit2: bloqueante e fora do event loop, sem exceção.
    let page = tokio::task::spawn_blocking(move || Git2Repo::open(&path)?.log(&query))
        .await
        .map_err(|err| {
            tracing::error!(%err, "spawn_blocking falhou ao ler o log");
            RepoError::Join
        })??;

    Ok(Json(page))
}

/// `GET /api/v1/repos/{repo_id}/refs` — toda ponta do repositório, para marcar linhas do log.
///
/// Não pagina: o número de branches, remotas e tags não cresce com o histórico, então não há
/// fronteira nenhuma para cortar em página — diferente do `log`.
pub async fn refs(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
) -> Result<Json<Vec<RefMarker>>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let markers = tokio::task::spawn_blocking(move || Git2Repo::open(&path)?.refs())
        .await
        .map_err(|err| {
            tracing::error!(%err, "spawn_blocking falhou ao ler as refs");
            RepoError::Join
        })??;

    Ok(Json(markers))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateBranchRequest {
    /// Nome da branch nova, sem `refs/heads/`.
    pub name: String,
    /// Oid ou nome de ref de onde ela nasce. Ausente é o `HEAD`. Não é revisão livre: o cliente
    /// sempre tem o oid completo do commit selecionado ou o nome da ref que ele clicou.
    #[serde(default)]
    pub start: Option<String>,
    /// Deixar o `HEAD` na branch nova. Ausente é `false` — criar não é trocar.
    #[serde(default)]
    pub checkout: bool,
    /// Ausente respeita o `branch.autoSetupMerge` do usuário; presente força só para esta
    /// criação, por flag, nunca escrevendo na config global dele.
    #[serde(default)]
    pub track: Option<bool>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchCreated {
    pub name: String,
    /// Oid completo para o qual a branch nasceu apontando.
    pub oid: String,
    /// O repositório **depois**: com checkout o `HEAD` mudou, e é daqui que a barra de topo e o
    /// título da aba se atualizam sem um segundo request. A lista de refs não vem junto de
    /// propósito — é outra rota, com outro ritmo de invalidação (Passo 57a).
    pub repo: Repo,
}

/// `POST /api/v1/repos/{repo_id}/branches` — cria branch a partir do `HEAD`, de um commit ou de
/// qualquer ref, com checkout e upstream opcionais (Passo 58).
pub async fn branch_create(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Json(body): Json<CreateBranchRequest>,
) -> Result<Json<BranchCreated>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let oid = porc_git::exec::branch::create(
        &path,
        &CreateOptions {
            name: body.name.clone(),
            start: body.start,
            checkout: body.checkout,
            track: body.track,
        },
    )
    .await?;

    tracing::info!(
        repo = %path.display(),
        branch = %body.name,
        %oid,
        checkout = body.checkout,
        "branch criada"
    );

    let info = tokio::task::spawn_blocking(move || Git2Repo::open(&path)?.info())
        .await
        .map_err(|err| {
            tracing::error!(%err, "spawn_blocking falhou ao reler o repositório");
            RepoError::Join
        })??;

    Ok(Json(BranchCreated {
        name: body.name,
        oid,
        repo: Repo { repo_id, info },
    }))
}

/// `GET /api/v1/repos/{repo_id}/remotes` — os remotes configurados (Passo 57).
///
/// Rota à parte do `refs` porque é outra coisa: `refs` são pontas do histórico, remote é
/// configuração. E são consumidas em ritmos diferentes — a lista de remotes só muda quando o
/// usuário a muda (Passo 62), as refs mudam a cada fetch.
pub async fn remotes(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
) -> Result<Json<Vec<Remote>>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let remotes = tokio::task::spawn_blocking(move || Git2Repo::open(&path)?.remotes())
        .await
        .map_err(|err| {
            tracing::error!(%err, "spawn_blocking falhou ao ler os remotes");
            RepoError::Join
        })??;

    Ok(Json(remotes))
}

/// `GET /api/v1/repos/{repo_id}/stashes` — a pilha de stash, do topo para o fundo (Passo 57).
pub async fn stashes(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
) -> Result<Json<Vec<StashEntry>>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let stashes = tokio::task::spawn_blocking(move || Git2Repo::open(&path)?.stashes())
        .await
        .map_err(|err| {
            tracing::error!(%err, "spawn_blocking falhou ao ler a pilha de stash");
            RepoError::Join
        })??;

    Ok(Json(stashes))
}

/// Lê o status completo: o `git status` shell-out (`porc_git::exec::status`, rápido porque
/// roda direto, não via `spawn_blocking`) mais o `git2::Repository::state()`
/// (`RepoRead::state`, bloqueante) — o `CLAUDE.md` já separa os dois mundos, e misturá-los
/// numa função só só complicaria sem precisar. Compartilhada por `status`, `stage` e
/// `unstage`: as duas mutações devolvem o status já atualizado, para a UI não precisar de um
/// segundo round-trip depois do otimista (Passo 48).
async fn read_status(path: std::path::PathBuf) -> Result<WorktreeStatus, RepoError> {
    let output = porc_git::exec::status::run(&path).await?;
    let mut report = porc_git::parse::status_v2::parse(&output.stdout);

    report.state = tokio::task::spawn_blocking(move || Git2Repo::open(&path)?.state())
        .await
        .map_err(|err| {
            tracing::error!(%err, "spawn_blocking falhou ao ler o estado do repositório");
            RepoError::Join
        })??;

    Ok(report)
}

/// `GET /api/v1/repos/{repo_id}/status` — status do working tree, agrupado em staged /
/// unstaged / untracked, mais o estado de merge/rebase em andamento (Passo 47).
pub async fn status(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
) -> Result<Json<WorktreeStatus>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    Ok(Json(read_status(path).await?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StagePathsRequest {
    /// Sempre explícita: nunca "tudo" implícito por lista vazia. Selecionar tudo é resolvido
    /// no cliente, que já tem os caminhos do último `status`.
    pub paths: Vec<String>,
}

/// `POST /api/v1/repos/{repo_id}/stage` — `git add -- <paths>` (Passo 48). Cobre modificado,
/// novo e deletado no mesmo comando; devolve o status já atualizado.
pub async fn stage(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Json(body): Json<StagePathsRequest>,
) -> Result<Json<WorktreeStatus>, RepoError> {
    if body.paths.is_empty() {
        return Err(RepoError::NoPaths);
    }

    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;
    porc_git::exec::stage::add(&path, &body.paths).await?;

    Ok(Json(read_status(path).await?))
}

/// `POST /api/v1/repos/{repo_id}/unstage` — `git reset -- <paths>` (Passo 48). Funciona mesmo
/// em `HEAD` unborn; devolve o status já atualizado.
pub async fn unstage(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Json(body): Json<StagePathsRequest>,
) -> Result<Json<WorktreeStatus>, RepoError> {
    if body.paths.is_empty() {
        return Err(RepoError::NoPaths);
    }

    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;
    porc_git::exec::stage::reset(&path, &body.paths).await?;

    Ok(Json(read_status(path).await?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeDiffParams {
    /// Caminho como o `status` o entregou — o lado novo, sempre.
    pub path: String,
    /// `unstaged` (índice ↔ worktree) ou `staged` (`HEAD` ↔ índice). Sem padrão: qual dos dois
    /// lados se está olhando é a informação central desta rota, e adivinhar seria mostrar o
    /// diff errado sem avisar.
    pub side: DiffSide,
}

/// `GET /api/v1/repos/{repo_id}/diff?path=…&side=…` — os hunks de **um** arquivo do trabalho
/// local (Passo 49). Mesma forma de resposta do diff de commit, para o visualizador do Passo 41
/// servir aos dois.
pub async fn worktree_diff(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Query(params): Query<WorktreeDiffParams>,
) -> Result<Json<FileDiff>, RepoError> {
    let fs_path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let diff = tokio::task::spawn_blocking(move || {
        Git2Repo::open(&fs_path)?.worktree_diff(params.side, &params.path)
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao ler o diff do trabalho local");
        RepoError::Join
    })??;

    Ok(Json(diff))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApplyHunksRequest {
    pub path: String,
    /// **De qual lado os trechos estão agora** — e, por consequência, para onde eles vão. Do
    /// lado `unstaged` a operação é stagear; do lado `staged`, desfazer. Um campo só decide as
    /// duas coisas porque elas nunca divergem: ninguém stagea o que já está no índice.
    pub side: DiffSide,
    /// Os trechos escolhidos: hunk inteiro, ou linhas dentro dele.
    pub hunks: Vec<HunkPick>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HunkPick {
    /// Índice do hunk, na mesma numeração do `GET /diff` do mesmo arquivo e do mesmo lado.
    pub hunk: usize,
    /// Ausente é o hunk inteiro (Passo 50). Presente são as **linhas de mudança** escolhidas
    /// (Passo 51), numeradas só entre as de mudança do hunk — contexto não entra na conta,
    /// porque contexto não muda de lado.
    #[serde(default)]
    pub lines: Option<Vec<usize>>,
}

/// `POST /api/v1/repos/{repo_id}/apply` — move trechos de um lado para o outro (Passo 50).
///
/// O caminho é o do `BLOCO-E.md`: recortar o patch (`porc_git::patch`) e entregá-lo ao
/// `git apply --cached`, que é quem sabe aplicar. Devolve o status já atualizado, como
/// `stage`/`unstage`.
pub async fn apply_hunks(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Json(body): Json<ApplyHunksRequest>,
) -> Result<Json<WorktreeStatus>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let fs_path = path.clone();
    let ApplyHunksRequest {
        path: file,
        side,
        hunks,
    } = body;

    let selection: Vec<_> = hunks
        .into_iter()
        .map(|pick| porc_git::patch::HunkSelection {
            hunk: pick.hunk,
            lines: pick.lines,
        })
        .collect();

    // O patch é lido do git2 (bloqueante) e recortado na mesma volta: entre ler e aplicar não
    // pode entrar nada, senão a numeração de hunk que o cliente mandou já não é a mesma.
    let selected = tokio::task::spawn_blocking(move || {
        let raw = Git2Repo::open(&fs_path)?.worktree_patch(side, &file)?;
        Ok::<_, RepoError>(porc_git::patch::parse(&raw)?.select(&selection)?)
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao recortar o patch");
        RepoError::Join
    })??;

    // Do lado unstaged, aplicar ao índice é stagear; do lado staged, aplicar ao contrário é
    // desfazer. Nos dois casos `--cached`: o arquivo em disco não é tocado.
    porc_git::exec::apply::cached(&path, &selected, side == DiffSide::Staged).await?;

    Ok(Json(read_status(path).await?))
}

/// `POST /api/v1/repos/{repo_id}/discard` — joga fora mudanças do working tree (Passo 55).
///
/// **A operação que destrói trabalho sem reflog para socorrer.** A confirmação é da interface,
/// que nomeia o que será perdido; aqui o que se garante é que só se perde o que foi nomeado:
/// lista explícita de caminhos (nunca "tudo"), e o **servidor** decide o comando a partir do
/// `status` — arquivo rastreado volta ao índice, não rastreado é apagado. Deixar o cliente
/// escolher entre "restaurar" e "apagar" seria deixá-lo pedir a remoção de um arquivo
/// rastreado por engano.
pub async fn discard(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Json(body): Json<StagePathsRequest>,
) -> Result<Json<WorktreeStatus>, RepoError> {
    if body.paths.is_empty() {
        return Err(RepoError::NoPaths);
    }

    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;
    let status = read_status(path.clone()).await?;

    let pedidos: std::collections::HashSet<&str> = body.paths.iter().map(String::as_str).collect();

    let untracked: Vec<String> = status
        .untracked
        .iter()
        .filter(|entry| pedidos.contains(entry.path.as_str()))
        .map(|entry| entry.path.clone())
        .collect();

    // O que não está entre os não rastreados é rastreado: o `checkout` cuida de modificado,
    // deletado e typechange do mesmo jeito.
    let rastreados: Vec<String> = body
        .paths
        .iter()
        .filter(|path| !untracked.contains(path))
        .cloned()
        .collect();

    if !rastreados.is_empty() {
        porc_git::exec::discard::checkout(&path, &rastreados).await?;
    }
    if !untracked.is_empty() {
        porc_git::exec::discard::remove_untracked(&path, &untracked)?;
    }

    tracing::info!(
        repo = %path.display(),
        rastreados = rastreados.len(),
        apagados = untracked.len(),
        "mudanças descartadas"
    );

    Ok(Json(read_status(path).await?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscardHunksRequest {
    pub path: String,
    /// Os trechos a jogar fora, na numeração do `GET /diff?side=unstaged` do mesmo arquivo —
    /// só desse lado: o que está no índice se desfaz com `unstage`, sem perder nada.
    pub hunks: Vec<HunkPick>,
}

/// `POST /api/v1/repos/{repo_id}/discard/hunks` — desfaz trechos **no arquivo em disco**.
///
/// É o `apply` do Passo 50 sem o `--cached` e ao contrário: ali a origem continua intacta no
/// worktree, aqui é o worktree que muda — e o que ele perde não está em lugar nenhum.
pub async fn discard_hunks(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Json(body): Json<DiscardHunksRequest>,
) -> Result<Json<WorktreeStatus>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let fs_path = path.clone();
    let file = body.path;
    let selection: Vec<_> = body
        .hunks
        .into_iter()
        .map(|pick| porc_git::patch::HunkSelection {
            hunk: pick.hunk,
            lines: pick.lines,
        })
        .collect();

    let selected = tokio::task::spawn_blocking(move || {
        // Sempre o lado unstaged: descartar é jogar fora o que **não** está no índice.
        let raw = Git2Repo::open(&fs_path)?.worktree_patch(DiffSide::Unstaged, &file)?;
        Ok::<_, RepoError>(porc_git::patch::parse(&raw)?.select(&selection)?)
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao recortar o patch de descarte");
        RepoError::Join
    })??;

    porc_git::exec::discard::revert_worktree(&path, &selected).await?;

    Ok(Json(read_status(path).await?))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitRequest {
    /// Assunto e corpo já juntos, como o usuário escreveu. Quem limpa é o `--cleanup=strip`.
    pub message: String,
    /// Reescreve o commit anterior em vez de criar um novo.
    #[serde(default)]
    pub amend: bool,
    /// Acrescenta o trailer `Signed-off-by:`.
    #[serde(default)]
    pub signoff: bool,
    /// Ausente respeita o `commit.gpgSign` do usuário. Presente força — só para este commit,
    /// por flag, nunca escrevendo na config dele.
    #[serde(default)]
    pub gpg_sign: Option<bool>,
    /// Oid do commit que este corrige. Presente, a mensagem é montada pelo git (`fixup! …`) e
    /// o campo `message` é ignorado.
    #[serde(default)]
    pub fixup: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDone {
    /// Oid completo do commit recém-criado — é por ele que a UI salta no log.
    pub oid: String,
    /// O status logo depois, que quase sempre é bem mais vazio do que antes.
    pub status: WorktreeStatus,
}

/// `POST /api/v1/repos/{repo_id}/commit` — fecha o commit com o que está no índice (Passo 52).
pub async fn commit_create(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Json(body): Json<CommitRequest>,
) -> Result<Json<CommitDone>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let oid = porc_git::exec::commit::commit(
        &path,
        &CommitOptions {
            message: body.message,
            amend: body.amend,
            signoff: body.signoff,
            gpg_sign: body.gpg_sign,
            fixup: body.fixup,
        },
    )
    .await?;

    tracing::info!(repo = %path.display(), %oid, "commit criado");

    Ok(Json(CommitDone {
        oid,
        status: read_status(path).await?,
    }))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitTemplate {
    /// `null` quando o usuário não configurou `commit.template` — o caso comum.
    pub template: Option<String>,
}

/// `GET /api/v1/repos/{repo_id}/commit/template` — o `commit.template` do usuário, se houver.
///
/// Rota própria e não um campo de outra: é uma leitura de config que a caixa de mensagem faz
/// uma vez ao abrir, e não tem por que viajar junto do `status`, que é pedido a toda hora.
pub async fn commit_template(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
) -> Result<Json<CommitTemplate>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    Ok(Json(CommitTemplate {
        template: porc_git::exec::commit::template(&path).await,
    }))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareParams {
    /// Lado esquerdo: hash, nome de branch ou tag, `HEAD~2` — o que o terminal entende.
    pub from: String,
    pub to: String,
}

/// `GET /api/v1/repos/{repo_id}/compare?from=&to=` — a diferença acumulada entre dois pontos
/// quaisquer do histórico (Passo 56).
pub async fn compare(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Query(params): Query<CompareParams>,
) -> Result<Json<porc_git::model::RangeDiff>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let diff = tokio::task::spawn_blocking(move || {
        Git2Repo::open(&path)?.range_diff(&params.from, &params.to)
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao comparar");
        RepoError::Join
    })??;

    Ok(Json(diff))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareDiffParams {
    pub from: String,
    pub to: String,
    /// Caminho **atual** do arquivo — o mesmo `path` que o `compare` já devolveu em cada
    /// `FileChange`.
    pub path: String,
}

/// `GET /api/v1/repos/{repo_id}/compare/diff?from=&to=&path=` — os hunks de **um** arquivo
/// dentro da comparação, sob demanda. Mesma economia do diff de commit.
pub async fn compare_diff(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Query(params): Query<CompareDiffParams>,
) -> Result<Json<FileDiff>, RepoError> {
    let fs_path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let diff = tokio::task::spawn_blocking(move || {
        Git2Repo::open(&fs_path)?.range_file_diff(&params.from, &params.to, &params.path)
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao ler o diff da comparação");
        RepoError::Join
    })??;

    Ok(Json(diff))
}

/// Teto do autocomplete de caminho. Uma pasta normal não tem mais que isto de entradas
/// diretas; se tiver, o resto só apareceria conforme a pessoa continuasse digitando.
const PATH_AUTOCOMPLETE_LIMIT: usize = 100;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PathsParams {
    /// Ausente é a raiz da árvore. Mesmo formato que a UI manda de volta: `pasta/parcial`.
    #[serde(default)]
    pub prefix: String,
}

/// `GET /api/v1/repos/{repo_id}/paths?prefix=…` — autocomplete do filtro por caminho
/// (Passo 45). Lê a árvore de `HEAD` via git2, não o histórico — é o motivo de não precisar
/// de índice nem de streaming, diferente do `path-filter` (`POST /api/v1/jobs/path-filter`).
pub async fn paths(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Query(params): Query<PathsParams>,
) -> Result<Json<Vec<String>>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let entries = tokio::task::spawn_blocking(move || {
        Git2Repo::open(&path)?.list_paths(&params.prefix, PATH_AUTOCOMPLETE_LIMIT)
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao listar caminhos");
        RepoError::Join
    })??;

    Ok(Json(entries))
}

/// Teto do que uma busca devolve de uma vez. Não é sobre performance (o FTS5 é rápido mesmo em
/// 100k linhas) — é sobre a lista de resultados continuar sendo "os que interessam", não um
/// segundo log inteiro por baixo de outro nome.
const SEARCH_LIMIT: usize = 500;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchParams {
    /// Vazia é "sem filtro" — a rota devolve lista vazia sem consultar o SQLite.
    pub q: String,
}

/// `GET /api/v1/repos/{repo_id}/search?q=…` — busca por mensagem ou autor no índice FTS5
/// (Passo 42). Não usa `git2`: é consulta pura ao `porc-index`, então nem abre o repositório.
pub async fn search(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SearchHit>>, RepoError> {
    // Mesma checagem de "repositório aberto neste boot" que toda rota de `repo_id` faz, mesmo
    // não indo ao git2: um id desconhecido não devia buscar em índice nenhum.
    if state.repos.path_of(&repo_id).is_none() {
        return Err(RepoError::Unknown);
    }

    let index = state.index.clone();
    let hits = tokio::task::spawn_blocking(move || {
        index.search_commits(&repo_id, &params.q, SEARCH_LIMIT)
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao buscar");
        RepoError::Join
    })??;

    Ok(Json(hits))
}

/// `GET /api/v1/repos/{repo_id}/commits/{oid}` — mensagem completa, assinaturas e diffstat de
/// um commit. É o que preenche o painel de detalhe ao selecionar uma linha do log.
pub async fn commit(
    State(state): State<AppState>,
    Path((repo_id, oid)): Path<(String, String)>,
) -> Result<Json<CommitDetail>, RepoError> {
    let path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let detail = tokio::task::spawn_blocking(move || Git2Repo::open(&path)?.commit_detail(&oid))
        .await
        .map_err(|err| {
            tracing::error!(%err, "spawn_blocking falhou ao ler o commit");
            RepoError::Join
        })??;

    Ok(Json(detail))
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffParams {
    /// Caminho **atual** do arquivo (o lado novo de um rename) — o mesmo `path` que
    /// `commit_detail` já devolveu em cada `FileChange`.
    pub path: String,
}

/// `GET /api/v1/repos/{repo_id}/commits/{oid}/diff?path=…` — os hunks de **um** arquivo, sob
/// demanda. Um commit pode tocar centenas de arquivos; mandar todo mundo de uma vez na rota de
/// detalhe seria o oposto do que uma tela de revisão precisa no primeiro clique.
pub async fn commit_diff(
    State(state): State<AppState>,
    Path((repo_id, oid)): Path<(String, String)>,
    Query(params): Query<DiffParams>,
) -> Result<Json<FileDiff>, RepoError> {
    let fs_path = state.repos.path_of(&repo_id).ok_or(RepoError::Unknown)?;

    let diff = tokio::task::spawn_blocking(move || {
        Git2Repo::open(&fs_path)?.commit_diff(&oid, &params.path)
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao ler o diff");
        RepoError::Join
    })??;

    Ok(Json(diff))
}

/// Repositórios abertos **neste boot**. Não é a lista de recentes (Passo 26), que persiste.
pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Repo>>, RepoError> {
    let open = state.repos.list();

    let repos = tokio::task::spawn_blocking(move || {
        open.into_iter()
            // Um repositório que sumiu do disco entre o registro e agora não derruba a lista
            // inteira: some dela, e a próxima abertura dará o erro certo.
            .filter_map(|(repo_id, path)| {
                let info = Git2Repo::open(&path).and_then(|repo| repo.info()).ok()?;
                Some(Repo { repo_id, info })
            })
            .collect::<Vec<_>>()
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao listar repositórios");
        RepoError::Join
    })?;

    Ok(Json(repos))
}
