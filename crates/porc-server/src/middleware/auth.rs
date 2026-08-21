//! Exige um cookie de sessão válido em tudo que não seja rota de boot.

use axum::{
    extract::{Request, State},
    http::{header::COOKIE, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{
    session::{self, SESSION_COOKIE},
    AppState,
};

/// A fronteira é `/api/`: dados exigem sessão, a casca do app não.
///
/// A casca **precisa** ser pública, e não por conveniência: o handshake acontece dentro
/// dela. O HTML, o JS, o CSS e as fontes carregam antes de existir cookie algum — exigir
/// sessão neles seria um ciclo (o token só é trocado pelo script que a sessão barraria).
/// E não há o que proteger ali: os arquivos do frontend são os mesmos para qualquer um que
/// tenha o binário. Tudo que a UI **mostra** vem de `/api/`, que é o que este guard cobre.
///
/// `/health` fica de fora do prefixo de propósito: é o que a segunda instância consulta
/// para saber se a primeira está viva, antes de ter qualquer credencial.
/// `/api/v1/session` é o próprio handshake, então não pode exigir o que ele cria.
fn is_public(path: &str) -> bool {
    !path.starts_with("/api/") || path == "/api/v1/session"
}

pub async fn require_session(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    if is_public(request.uri().path()) {
        return next.run(request).await;
    }

    let authenticated = request
        .headers()
        .get(COOKIE)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| session::cookie_value(header, SESSION_COOKIE))
        .is_some_and(|value| state.session.matches(value));

    if !authenticated {
        tracing::debug!(path = %request.uri().path(), "request sem sessão válida");
        return (StatusCode::UNAUTHORIZED, "sessão inválida").into_response();
    }

    next.run(request).await
}
