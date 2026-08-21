//! Guard de `Host`/`Origin` — a defesa real contra DNS rebinding.
//!
//! Bind em `127.0.0.1` não protege: um site aberto noutra aba resolve o próprio domínio
//! para `127.0.0.1` e o browser da vítima entrega o request aqui, com os cookies dela. O
//! que denuncia o ataque é o header `Host` carregar o domínio do atacante em vez do nosso
//! literal. Por isso este guard vem antes de qualquer coisa olhar cookie, e vale também no
//! upgrade do WebSocket — WS não sofre CORS e escapa de `SameSite`.

use axum::{
    extract::{Request, State},
    http::{
        header::{HOST, ORIGIN},
        StatusCode,
    },
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::AppState;

/// Os únicos nomes pelos quais o servidor aceita ser chamado.
const ALLOWED_HOSTNAMES: [&str; 2] = ["127.0.0.1", "localhost"];

pub async fn guard(State(state): State<AppState>, request: Request, next: Next) -> Response {
    let headers = request.headers();

    let host_ok = headers
        .get(HOST)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|host| is_allowed_authority(host, state.port));

    if !host_ok {
        tracing::warn!(
            host = ?headers.get(HOST),
            path = %request.uri().path(),
            "request recusado: Host fora do esperado"
        );
        return (StatusCode::FORBIDDEN, "host não permitido").into_response();
    }

    // `Origin` é opcional (curl e navegação direta não mandam), mas se veio tem que bater.
    // `Origin: null` — sandbox, redirect cross-origin, `data:` — nunca bate.
    if let Some(origin) = headers.get(ORIGIN) {
        let origin_ok = origin
            .to_str()
            .ok()
            .is_some_and(|origin| is_allowed_origin(origin, state.port));

        if !origin_ok {
            tracing::warn!(?origin, "request recusado: Origin fora do esperado");
            return (StatusCode::FORBIDDEN, "origin não permitida").into_response();
        }
    }

    next.run(request).await
}

/// `host[:porta]` — exige o nome literal **e** a porta em que estamos servindo.
fn is_allowed_authority(authority: &str, port: u16) -> bool {
    let Some((name, authority_port)) = authority.rsplit_once(':') else {
        // Sem porta explícita significaria porta 80, que nunca é a nossa.
        return false;
    };

    ALLOWED_HOSTNAMES.contains(&name) && authority_port == port.to_string()
}

/// Só `http://`: a origem do app é servida em claro no loopback, e um `https://` com o
/// mesmo host seria outra origem — vinda de fora.
fn is_allowed_origin(origin: &str, port: u16) -> bool {
    origin
        .strip_prefix("http://")
        .is_some_and(|authority| is_allowed_authority(authority, port))
}

#[cfg(test)]
mod tests {
    use super::*;

    const PORT: u16 = 7867;

    #[test]
    fn aceita_os_dois_nomes_de_loopback() {
        assert!(is_allowed_authority("127.0.0.1:7867", PORT));
        assert!(is_allowed_authority("localhost:7867", PORT));
        assert!(is_allowed_origin("http://127.0.0.1:7867", PORT));
        assert!(is_allowed_origin("http://localhost:7867", PORT));
    }

    #[test]
    fn recusa_dns_rebinding_e_variantes() {
        // O caso que motiva o middleware inteiro.
        assert!(!is_allowed_authority("evil.com", PORT));
        assert!(!is_allowed_authority("evil.com:7867", PORT));
        // Sufixo que "termina em" localhost não é localhost.
        assert!(!is_allowed_authority("notlocalhost:7867", PORT));
        assert!(!is_allowed_authority("localhost.evil.com:7867", PORT));
        // Porta errada é outro servidor.
        assert!(!is_allowed_authority("127.0.0.1:9999", PORT));
        // Sem porta nenhuma.
        assert!(!is_allowed_authority("127.0.0.1", PORT));
    }

    #[test]
    fn recusa_origin_null_e_esquema_errado() {
        assert!(!is_allowed_origin("null", PORT));
        assert!(!is_allowed_origin("https://127.0.0.1:7867", PORT));
        assert!(!is_allowed_origin("file://", PORT));
    }
}
