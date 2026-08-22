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
    exec::init::{InitError, InitOptions},
    exec::ExecError,
    model::{CommitDetail, FileDiff, Head, LogPage, RefMarker, RepoInfo, WorktreeStatus},
    read::{Git2Repo, GitError, LogQuery, RepoRead},
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
            RepoError::Git(GitError::FileNotInCommit) => StatusCode::NOT_FOUND,
            // `AlreadyARepository` é 409: o pedido está bem formado, o mundo é que já está do
            // jeito que ele queria criar.
            RepoError::Init(InitError::AlreadyARepository(_)) => StatusCode::CONFLICT,
            RepoError::Init(InitError::Exec(_)) => StatusCode::INTERNAL_SERVER_ERROR,
            RepoError::Init(_) => StatusCode::BAD_REQUEST,
            RepoError::NoPaths => StatusCode::BAD_REQUEST,
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
