//! Token de sessão e o handshake que o troca por cookie.
//!
//! O token vale só para este boot: 256 bits de `getrandom`, guardados em memória e nunca
//! escritos em disco. Matar o processo invalida tudo que foi emitido.

use std::sync::Arc;

use axum::{
    extract::State,
    http::{header::SET_COOKIE, HeaderValue, StatusCode},
    response::{AppendHeaders, IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use subtle::ConstantTimeEq;

use crate::AppState;

/// Cookie de sessão. `HttpOnly`: JS nunca precisa lê-lo, então não pode.
pub const SESSION_COOKIE: &str = "porc_sess";

/// Cookie de CSRF. **Sem** `HttpOnly`, de propósito: o frontend precisa lê-lo para
/// devolvê-lo no header `X-CSRF-Token`. Por isso é um segredo distinto do de sessão —
/// vazá-lo não dá sessão a ninguém.
pub const CSRF_COOKIE: &str = "porc_csrf";

/// 32 bytes = 256 bits, em hex.
const TOKEN_BYTES: usize = 32;

#[derive(Debug, thiserror::Error)]
#[error("não consegui gerar o token de sessão: {0}")]
pub struct TokenError(#[source] getrandom::Error);

/// Segredo do boot, em hex. `Arc` porque o `AppState` do axum é clonado por request.
///
/// Serve tanto para a sessão quanto para o CSRF — são dois valores independentes do mesmo
/// tipo, nunca o mesmo valor em dois lugares.
#[derive(Clone)]
pub struct Secret(Arc<str>);

impl Secret {
    pub fn generate() -> Result<Self, TokenError> {
        let mut bytes = [0u8; TOKEN_BYTES];
        getrandom::fill(&mut bytes).map_err(TokenError)?;

        let mut hex = String::with_capacity(TOKEN_BYTES * 2);
        for byte in bytes {
            use std::fmt::Write;
            // `write!` em String não falha; o unwrap é sobre um Infallible de fato.
            write!(hex, "{byte:02x}").expect("String nunca falha em write!");
        }

        Ok(Self(hex.into()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Comparação em tempo constante. O comprimento do token é público (64 hex), então
    /// sair cedo por tamanho não vaza nada de útil.
    pub fn matches(&self, candidate: &str) -> bool {
        let expected = self.0.as_bytes();
        let candidate = candidate.as_bytes();

        if expected.len() != candidate.len() {
            return false;
        }

        expected.ct_eq(candidate).into()
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRequest {
    pub token: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionResponse {
    pub ok: bool,
}

/// `POST /api/v1/session` — troca o token de boot pelos cookies de sessão e de CSRF.
///
/// Sem `Secure`: a origem é `http://127.0.0.1`, onde `Secure` impediria o próprio cookie
/// de ser gravado. `SameSite=Strict` é o que barra o request cross-site.
pub async fn create(State(state): State<AppState>, Json(body): Json<SessionRequest>) -> Response {
    if !state.session.matches(&body.token) {
        tracing::warn!("handshake recusado: token inválido");
        return (StatusCode::UNAUTHORIZED, "token inválido").into_response();
    }

    let session = format!(
        "{SESSION_COOKIE}={}; HttpOnly; SameSite=Strict; Path=/",
        state.session.as_str()
    );
    let csrf = format!(
        "{CSRF_COOKIE}={}; SameSite=Strict; Path=/",
        state.csrf.as_str()
    );

    // `AppendHeaders` e não um array cru: um array de tuplas faz `insert`, e o segundo
    // `Set-Cookie` sobrescreveria o primeiro em vez de somar.
    let cookies = match (
        HeaderValue::from_str(&session),
        HeaderValue::from_str(&csrf),
    ) {
        (Ok(session), Ok(csrf)) => AppendHeaders([(SET_COOKIE, session), (SET_COOKIE, csrf)]),
        _ => {
            tracing::error!("cookie de sessão inválido");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    (StatusCode::OK, cookies, Json(SessionResponse { ok: true })).into_response()
}

/// Lê um cookie do header `Cookie` cru. Não usamos uma crate de cookie porque só há dois
/// (`porc_sess`, `porc_csrf`), ambos emitidos por nós e sem atributos exóticos.
pub fn cookie_value<'h>(header: &'h str, name: &str) -> Option<&'h str> {
    header.split(';').find_map(|pair| {
        let (key, value) = pair.split_once('=')?;
        (key.trim() == name).then(|| value.trim())
    })
}
