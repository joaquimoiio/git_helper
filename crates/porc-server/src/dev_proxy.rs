//! Proxy de desenvolvimento para o Vite.
//!
//! Sem isto o ciclo de frontend seria insuportável: cada mudança de CSS exigiria
//! recompilar Rust. Com isto, o navegador fala **só** com a porta do `porc` — as camadas
//! de `Host`/`Origin`, sessão e CSRF continuam valendo para tudo, e a UI em dev vive na
//! mesma origem que em release. É a única forma de o handshake do token ser exercitado do
//! mesmo jeito nos dois modos.
//!
//! Compilado apenas em build de debug (`#[cfg(debug_assertions)]` no `lib.rs`): não há
//! caminho pelo qual um binário de release fale com a porta 5173.

use std::sync::OnceLock;

use axum::{
    body::Body,
    extract::Request,
    http::{header::HOST, uri::Uri, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::TokioExecutor,
};

/// Onde o `npm run dev` atende. Literal e sem configuração: o `vite.config.ts` fixa a
/// porta com `strictPort`, então ou é aqui ou o Vite não subiu.
const VITE: &str = "127.0.0.1:5173";

fn client() -> &'static Client<HttpConnector, Body> {
    static CLIENT: OnceLock<Client<HttpConnector, Body>> = OnceLock::new();

    CLIENT.get_or_init(|| {
        let mut connector = HttpConnector::new();
        // Vite às vezes está no meio de um restart. Falhar rápido e mostrar o recado é
        // melhor do que a aba ficar girando.
        connector.set_connect_timeout(Some(std::time::Duration::from_millis(500)));
        connector.set_nodelay(true);

        Client::builder(TokioExecutor::new()).build(connector)
    })
}

/// `fallback` do router: tudo que não é rota do servidor é arquivo do frontend.
///
/// O HMR **não** passa por aqui. O cliente do Vite é configurado no `vite.config.ts` para
/// abrir o WebSocket direto na 5173: proxiar upgrade de WebSocket só para o HMR seria
/// código de produção mantido para uso exclusivo de dev.
pub async fn proxy(mut request: Request) -> Response {
    let path_and_query = request
        .uri()
        .path_and_query()
        .map(|value| value.as_str())
        .unwrap_or("/");

    let Ok(uri) = format!("http://{VITE}{path_and_query}").parse::<Uri>() else {
        return (StatusCode::BAD_REQUEST, "caminho inválido").into_response();
    };
    *request.uri_mut() = uri;

    // O `Host` que chegou é o do `porc`. Reescrever evita que o Vite decida rejeitar um
    // host que não conhece, e mantém os links que ele gera apontando para ele mesmo.
    request
        .headers_mut()
        .insert(HOST, HeaderValue::from_static(VITE));

    match client().request(request).await {
        Ok(response) => response.map(Body::new),
        Err(err) => {
            tracing::warn!(%err, "vite não respondeu");
            (
                StatusCode::BAD_GATEWAY,
                "o servidor de desenvolvimento do Vite não está no ar — rode `npm run dev` em web/",
            )
                .into_response()
        }
    }
}
