//! CSRF por double-submit: o segredo tem que chegar ao mesmo tempo pelo cookie
//! `porc_csrf` e pelo header `X-CSRF-Token`.
//!
//! Um site atacante consegue fazer o browser da vítima **mandar** o cookie, mas não
//! consegue ler o valor dele para montar o header — a leitura é barrada pela
//! same-origin policy. É essa assimetria que o double-submit explora.

use axum::{
    extract::{Request, State},
    http::{header::COOKIE, Method, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{
    session::{self, CSRF_COOKIE},
    AppState,
};

const CSRF_HEADER: &str = "x-csrf-token";

/// O handshake é um POST que **cria** o cookie de CSRF, então não pode exigi-lo. Ele já é
/// protegido pelo token de boot e pelo guard de origem.
fn is_exempt(path: &str) -> bool {
    path == "/api/v1/session"
}

pub async fn require_double_submit(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Response {
    // GET e HEAD não mudam estado; exigir header neles quebraria navegação normal.
    if matches!(*request.method(), Method::GET | Method::HEAD) || is_exempt(request.uri().path()) {
        return next.run(request).await;
    }

    let headers = request.headers();

    let from_cookie = headers
        .get(COOKIE)
        .and_then(|header| header.to_str().ok())
        .and_then(|header| session::cookie_value(header, CSRF_COOKIE));

    let from_header = headers
        .get(CSRF_HEADER)
        .and_then(|header| header.to_str().ok());

    // Os dois são conferidos contra o segredo do servidor, não um contra o outro: assim
    // um par forjado idêntico também não passa.
    let ok = from_cookie.is_some_and(|value| state.csrf.matches(value))
        && from_header.is_some_and(|value| state.csrf.matches(value));

    if !ok {
        tracing::warn!(
            method = %request.method(),
            path = %request.uri().path(),
            "request recusado: CSRF ausente ou inválido"
        );
        return (StatusCode::FORBIDDEN, "csrf inválido").into_response();
    }

    next.run(request).await
}
