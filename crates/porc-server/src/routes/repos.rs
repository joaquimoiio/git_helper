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
    model::{LogPage, RepoInfo},
    read::{Git2Repo, GitError, LogQuery, RepoRead},
};
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
    #[error("a leitura do repositório não terminou")]
    Join,
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
            // pedido malformado — não falha da leitura.
            RepoError::Git(GitError::InvalidCursor) => StatusCode::BAD_REQUEST,
            // `AlreadyARepository` é 409: o pedido está bem formado, o mundo é que já está do
            // jeito que ele queria criar.
            RepoError::Init(InitError::AlreadyARepository(_)) => StatusCode::CONFLICT,
            RepoError::Init(InitError::Exec(_)) => StatusCode::INTERNAL_SERVER_ERROR,
            RepoError::Init(_) => StatusCode::BAD_REQUEST,
            RepoError::Git(_) | RepoError::Join => StatusCode::INTERNAL_SERVER_ERROR,
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

/// Abre o repositório e devolve quem ele é. Idempotente: abrir duas vezes devolve o mesmo id.
pub async fn open(
    State(state): State<AppState>,
    Json(body): Json<OpenRequest>,
) -> Result<Json<Repo>, RepoError> {
    let settings = state.settings.clone();
    let registry = state.repos.clone();
    let index = state.index.clone();

    // Todo o corpo é bloqueante: `canonicalize`, os `stat` do `is_repo`, o libgit2 e o SQLite.
    let repo = tokio::task::spawn_blocking(move || {
        let path = crate::routes::fs::resolve(&settings.root, Some(&body.path))?;
        register_opened(&registry, &index, &path)
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao abrir repositório");
        RepoError::Join
    })??;

    tracing::info!(repo = %repo.info.path, branch = %repo.info.branch, "repositório aberto");

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

    let repo = tokio::task::spawn_blocking(move || {
        // Reconferido depois de criado: o caminho passou por `create_dir` e `canonicalize`, e
        // confinamento que se checa só na entrada é confinamento que se perde numa refatoração.
        let path = crate::routes::fs::resolve(&settings.root, Some(&created.to_string_lossy()))?;

        register_opened(&registry, &index, &path)
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao abrir o repositório criado");
        RepoError::Join
    })??;

    tracing::info!(repo = %repo.info.path, branch = %repo.info.branch, "repositório criado");

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
