//! `GET /api/v1/recents` e `DELETE /api/v1/recents/{repo_id}`.
//!
//! Recurso próprio, e não `/repos/recent`: recentes são caminhos lembrados em disco, e
//! `/repos` são repositórios abertos neste boot. Misturar os dois num prefixo só faria uma
//! rota estática disputar espaço com `/repos/{repo_id}` sem ganho nenhum de clareza.
//!
//! Não existe rota para *abrir* um recente: a UI manda o caminho para `POST /api/v1/repos`,
//! que revalida o confinamento. Um atalho que abrisse por id pularia essa checagem — e o banco
//! é um arquivo que o usuário pode editar.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use porc_index::{recents::Recent, IndexError};
use serde::Serialize;

use crate::AppState;

/// Teto da lista. Recentes servem para reabrir o que se estava fazendo, não para ser um
/// histórico — passando de algumas dezenas, a lista deixa de ajudar a escolher.
const LIMIT: usize = 20;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RecentEntry {
    #[serde(flatten)]
    recent: Recent,
    /// `false` quando a pasta sumiu do disco ou deixou de ser repositório. A entrada continua
    /// na lista, desabilitada: sumir sozinha esconderia do usuário que ele moveu a pasta.
    available: bool,
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub struct RecentsError(#[from] IndexError);

impl IntoResponse for RecentsError {
    fn into_response(self) -> Response {
        tracing::warn!(source = ?std::error::Error::source(&self.0), "falha no índice");

        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<RecentEntry>>, RecentsError> {
    let index = state.index.clone();

    // SQLite e os `stat` do `is_repo` são bloqueantes.
    let entries = tokio::task::spawn_blocking(move || {
        let recents = index.recents(LIMIT)?;

        Ok::<_, IndexError>(
            recents
                .into_iter()
                .map(|recent| RecentEntry {
                    available: porc_git::discover::is_repo(std::path::Path::new(&recent.path)),
                    recent,
                })
                .collect::<Vec<_>>(),
        )
    })
    .await
    // Um `spawn_blocking` que não termina só acontece se a task entrou em pânico; a lista de
    // recentes vazia é uma degradação aceitável, e o pânico já foi logado pelo runtime.
    .unwrap_or_else(|err| {
        tracing::error!(%err, "spawn_blocking falhou ao listar recentes");
        Ok(Vec::new())
    })?;

    Ok(Json(entries))
}

pub async fn forget(
    State(state): State<AppState>,
    Path(repo_id): Path<String>,
) -> Result<StatusCode, RecentsError> {
    let index = state.index.clone();

    tokio::task::spawn_blocking(move || index.forget_recent(&repo_id))
        .await
        .unwrap_or_else(|err| {
            tracing::error!(%err, "spawn_blocking falhou ao esquecer recente");
            Ok(())
        })?;

    Ok(StatusCode::NO_CONTENT)
}
