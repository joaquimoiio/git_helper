//! `POST /api/v1/jobs/<tipo>`, `GET /api/v1/jobs/{id}`, `DELETE /api/v1/jobs/{id}`.
//!
//! Criar um job responde **202** com o `jobId` e volta na hora — não espera o trabalho. O que
//! acontece depois chega pelo WebSocket, e quem perdeu o socket pergunta ao `GET`.
//!
//! O tipo é um segmento estático (`/jobs/test`, `/jobs/clone`) e não um parâmetro: cada tipo
//! tem um corpo diferente, e um `{kind}` genérico só empurraria o `match` para dentro do
//! handler, com o corpo chegando como JSON solto.

use std::time::Duration;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use porc_git::exec::clone::{CloneError, CloneOptions, Prepared, StderrTail};
use serde::{Deserialize, Serialize};

use crate::{
    jobs::{Cleanup, JobHandle, JobSnapshot, JobsError, Progress},
    routes::repos::RepoError,
    AppState,
};

/// Quanto o job de teste espera entre um número e o seguinte. Devagar o bastante para dar para
/// ver o progresso subir e cancelar no meio — é para isso que ele existe.
const TICK: Duration = Duration::from_millis(400);
const COUNT_TO: u32 = 10;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Accepted {
    pub job_id: String,
}

impl IntoResponse for JobsError {
    fn into_response(self) -> Response {
        let status = match self {
            JobsError::Unknown => StatusCode::NOT_FOUND,
            JobsError::TooMany => StatusCode::TOO_MANY_REQUESTS,
            JobsError::Id => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}

/// `POST /api/v1/jobs/test` — conta até dez.
///
/// Não é enfeite: é o job que prova o canal inteiro (registry, WebSocket, reconexão,
/// cancelamento) sem depender de rede nem de repositório. Quando o clone quebrar, é ele que diz
/// se o problema é o clone ou a infra.
pub async fn create_test(State(state): State<AppState>) -> Result<Response, JobsError> {
    let handle = state.jobs.create("test")?;
    let job_id = handle.job_id.clone();

    tokio::spawn(run_test(handle));

    Ok((StatusCode::ACCEPTED, Json(Accepted { job_id })).into_response())
}

async fn run_test(handle: JobHandle) {
    handle.log(format!("contando até {COUNT_TO}"));

    for step in 1..=COUNT_TO {
        // O cancelamento compete com a espera: um job que só olhasse o token entre as etapas
        // demoraria uma etapa inteira para responder ao botão.
        tokio::select! {
            _ = tokio::time::sleep(TICK) => {}
            _ = handle.cancel.cancelled() => {
                handle.log("cancelado");
                return handle.cancelled();
            }
        }

        handle.progress(Progress {
            phase: "contando".to_owned(),
            fraction: Some(step as f32 / COUNT_TO as f32),
            detail: Some(format!("{step}/{COUNT_TO}")),
        });
        handle.log(format!("{step}"));
    }

    handle.done(serde_json::json!({ "counted": COUNT_TO }));
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CloneRequest {
    pub url: String,
    /// Pasta **existente** onde o clone cai. Mesmo contrato de caminho do `fs/list`.
    pub path: String,
    /// Nome da pasta a criar. Ausente, sai da URL.
    pub folder: Option<String>,
    pub branch: Option<String>,
    pub depth: Option<u32>,
    #[serde(default)]
    pub recurse_submodules: bool,
    /// Nome do remote. Ausente é `origin`.
    pub remote: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum CloneStartError {
    #[error(transparent)]
    Repo(#[from] RepoError),
    #[error(transparent)]
    Clone(#[from] CloneError),
    #[error(transparent)]
    Jobs(#[from] JobsError),
}

impl IntoResponse for CloneStartError {
    fn into_response(self) -> Response {
        match self {
            CloneStartError::Repo(err) => err.into_response(),
            CloneStartError::Jobs(err) => err.into_response(),
            // Tudo que o `prepare` recusa é pedido malformado: URL vazia, pasta cheia, nome de
            // branch impossível. Nada disso chegou a tocar a rede.
            CloneStartError::Clone(CloneError::NotEmpty(_)) => {
                (StatusCode::CONFLICT, self.to_string()).into_response()
            }
            CloneStartError::Clone(_) => {
                (StatusCode::BAD_REQUEST, self.to_string()).into_response()
            }
        }
    }
}

/// `POST /api/v1/jobs/clone` — começa o clone e volta na hora com o `jobId`.
///
/// A validação inteira acontece **antes** do 202: quem recebeu o id sabe que a URL é aceitável,
/// que a pasta de destino serve e que a limpeza já está registrada. O que pode dar errado depois
/// é rede, credencial e o repositório do outro lado — e isso vira evento, não status HTTP.
pub async fn create_clone(
    State(state): State<AppState>,
    Json(body): Json<CloneRequest>,
) -> Result<Response, CloneStartError> {
    let settings = state.settings.clone();
    let requested = body.path.clone();

    // `resolve` e as checagens de `prepare` batem no disco: fora do event loop.
    let prepared = tokio::task::spawn_blocking(move || {
        let parent = crate::routes::fs::resolve(&settings.root, Some(&requested))
            .map_err(RepoError::from)?;

        porc_git::exec::clone::prepare(&CloneOptions {
            url: body.url,
            parent,
            folder: body.folder,
            branch: body.branch,
            depth: body.depth,
            recurse_submodules: body.recurse_submodules,
            remote: body.remote,
        })
        .map_err(CloneStartError::from)
    })
    .await
    .map_err(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao preparar o clone");
        CloneStartError::Repo(RepoError::Join)
    })??;

    let handle = state.jobs.create("clone")?;
    let job_id = handle.job_id.clone();

    // **Antes** de começar, sempre — e só se a pasta for nossa. Registrar depois deixaria uma
    // janela em que o git já criou arquivos e ninguém sabe que pode apagá-los.
    if prepared.creates_target {
        handle.add_cleanup(Cleanup::RemoveCreatedDir(prepared.target().to_path_buf()));
    }

    tokio::spawn(run_clone(handle, prepared, state));

    Ok((StatusCode::ACCEPTED, Json(Accepted { job_id })).into_response())
}

/// Abre o canal de senha do job, se a plataforma tiver um.
///
/// A sessão tem que continuar viva enquanto o `git` roda — quando ela cai, o socket some. Por
/// isso ela é devolvida e guardada na task, não descartada aqui.
#[cfg(unix)]
fn start_askpass(
    handle: &JobHandle,
    state: &AppState,
) -> Option<(crate::askpass::Session, porc_git::exec::Askpass)> {
    let helper = crate::askpass::Session::helper()
        .inspect_err(|err| tracing::warn!(%err, "não descobri o caminho do próprio binário"))
        .ok()?;

    let session = crate::askpass::start(
        handle.job_id.clone(),
        state.jobs.clone(),
        state.prompts.clone(),
        handle.cancel.clone(),
    )
    .inspect_err(|err| tracing::warn!(%err, "sem canal de senha para este job"))
    .ok()?;

    let askpass = porc_git::exec::Askpass {
        helper,
        socket: session.socket().to_path_buf(),
    };

    Some((session, askpass))
}

async fn run_clone(handle: JobHandle, prepared: Prepared, state: AppState) {
    use porc_git::parse::progress::ProgressEvent;

    handle.log(format!("clonando para {}", prepared.target().display()));

    // No Windows não há socket unix em tokio, e o git usa o Credential Manager. Lá o clone com
    // chave protegida por passphrase ainda não passa pela interface.
    #[cfg(unix)]
    let session = start_askpass(&handle, &state);
    #[cfg(unix)]
    let askpass = session.as_ref().map(|(_, askpass)| askpass);
    #[cfg(not(unix))]
    let askpass: Option<&porc_git::exec::Askpass> = None;

    let mut tail = StderrTail::default();

    let outcome = porc_git::exec::clone::run(&prepared, askpass, handle.cancel.clone(), |event| {
        tail.push(&event);

        match &event {
            // O que não é barra de progresso é mensagem: vai para o log do job, que é o "ver
            // detalhes" da UI.
            ProgressEvent::Other(line) => handle.log(line.clone()),
            _ => handle.progress(Progress {
                phase: event.phase().to_owned(),
                fraction: event.fraction(),
                detail: event.detail(),
            }),
        }
    })
    .await;

    let path = match outcome {
        Ok(path) => path,
        Err(CloneError::Cancelled) => {
            handle.log("cancelado");
            // A limpeza registrada roda aqui dentro: é o `cancelled` que apaga a pasta parcial.
            return handle.cancelled();
        }
        Err(err) => {
            let stderr = tail.text();
            tracing::warn!(%err, %stderr, "clone falhou");

            // O que o usuário lê é a frase; o stderr cru já está no log do job, que é o "ver
            // detalhes" da interface. As duas coisas existem, em camadas diferentes.
            let diagnosis = porc_git::exec::error::diagnose(&stderr);

            return handle.fail_with(diagnosis.message, diagnosis.action);
        }
    };

    // Clonou: abrir é o que o usuário queria de verdade. Abrir aqui evita a UI ter que descobrir
    // sozinha o caminho canônico do que acabou de nascer.
    let registry = state.repos.clone();
    let index = state.index.clone();

    let opened = tokio::task::spawn_blocking(move || {
        crate::routes::repos::register_opened(&registry, &index, &path)
    })
    .await;

    match opened {
        Ok(Ok(repo)) => {
            handle.log(format!("pronto: {}", repo.info.path));
            handle.done(serde_json::to_value(repo).unwrap_or(serde_json::Value::Null));
        }
        Ok(Err(err)) => {
            // Caso raro e específico: o clone terminou, mas o que ficou no disco não abre. A
            // pasta **não** é apagada — o usuário tem um repositório para investigar.
            handle.fail(format!(
                "o clone terminou, mas o repositório não abriu: {err}"
            ));
        }
        Err(err) => {
            tracing::error!(%err, "spawn_blocking falhou ao abrir o repositório clonado");
            handle.fail("o clone terminou, mas não consegui abrir o repositório");
        }
    }
}

/// O último estado conhecido. É por aqui que uma aba recarregada recupera o progresso.
pub async fn get(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<Json<JobSnapshot>, JobsError> {
    state
        .jobs
        .snapshot(&job_id)
        .map(Json)
        .ok_or(JobsError::Unknown)
}

pub async fn list(State(state): State<AppState>) -> Json<Vec<JobSnapshot>> {
    Json(state.jobs.list())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AskpassAnswer {
    pub prompt_id: String,
    /// A passphrase. Entra aqui, vai para o socket e some — não é logada, não é guardada e não
    /// aparece em nenhuma resposta.
    pub secret: String,
}

/// `POST /api/v1/jobs/{job_id}/askpass` — responde ao pedido de senha.
///
/// O `job_id` está na rota por simetria e para o log fazer sentido; quem de fato encontra o
/// pedido é o `promptId`, que é único no processo.
#[cfg(unix)]
pub async fn answer_askpass(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
    Json(body): Json<AskpassAnswer>,
) -> Result<StatusCode, crate::askpass::AskpassError> {
    tracing::info!(job_id, prompt_id = %body.prompt_id, "senha respondida pela interface");

    state.prompts.answer(&body.prompt_id, body.secret)?;

    Ok(StatusCode::NO_CONTENT)
}

#[cfg(unix)]
impl IntoResponse for crate::askpass::AskpassError {
    fn into_response(self) -> Response {
        let status = match self {
            crate::askpass::AskpassError::Unknown => StatusCode::NOT_FOUND,
            crate::askpass::AskpassError::Socket(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, self.to_string()).into_response()
    }
}

/// Pede o cancelamento. Responde 202, não 200: o job ainda vai levar um instante para parar e
/// limpar, e é o evento `job.done` que conta essa história.
pub async fn cancel(
    State(state): State<AppState>,
    Path(job_id): Path<String>,
) -> Result<StatusCode, JobsError> {
    state.jobs.cancel(&job_id)?;

    Ok(StatusCode::ACCEPTED)
}
