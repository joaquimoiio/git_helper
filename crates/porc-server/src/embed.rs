//! Frontend embutido no binário.
//!
//! `web/dist` inteiro — HTML, JS, CSS e as duas fontes — vira bytes dentro do executável.
//! É o que faz `cargo build --release` produzir um arquivo só, que funciona movido para
//! outra pasta, em outra máquina, sem internet.

use axum::{
    body::Body,
    http::{
        header::{CACHE_CONTROL, CONTENT_TYPE, ETAG, IF_NONE_MATCH},
        HeaderMap, StatusCode, Uri,
    },
    response::{IntoResponse, Response},
};
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
// Relativo ao `Cargo.toml` deste crate, que é como o derive resolve o caminho.
#[folder = "../../web/dist"]
struct Assets;

const INDEX: &str = "index.html";

/// Prefixo que o Vite usa para tudo que leva hash no nome.
const HASHED: &str = "assets/";

/// Um ano e `immutable`, para os arquivos cujo nome carrega o hash do conteúdo: mudou o
/// conteúdo, muda o nome, então não existe versão velha para o cache servir por engano.
const CACHE_IMMUTABLE: &str = "public, max-age=31536000, immutable";

/// O `index.html` não leva hash — é o ponto de entrada. `no-cache` não significa "não
/// guarde", e sim "revalide antes de usar", o que com ETag custa um 304 de nada.
const CACHE_REVALIDATE: &str = "no-cache";

pub async fn serve(headers: HeaderMap, uri: Uri) -> Response {
    let requested = uri.path().trim_start_matches('/');
    let requested = if requested.is_empty() {
        INDEX
    } else {
        requested
    };

    let path = match Assets::get(requested) {
        Some(_) => requested,
        // Arquivo com hash que não existe é erro de verdade — devolver o index aqui faria
        // o navegador tentar executar HTML como JavaScript e o erro sairia irreconhecível.
        None if requested.starts_with(HASHED) => {
            return (StatusCode::NOT_FOUND, "arquivo não encontrado").into_response()
        }
        // Qualquer outro caminho é rota da SPA: quem resolve é o roteador do frontend.
        None => INDEX,
    };

    let Some(file) = Assets::get(path) else {
        tracing::error!("`{INDEX}` não está no binário — o build do frontend não rodou");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            "este binário foi compilado sem o frontend",
        )
            .into_response();
    };

    let etag = etag(&file.metadata.sha256_hash());
    let cache = if path.starts_with(HASHED) {
        CACHE_IMMUTABLE
    } else {
        CACHE_REVALIDATE
    };

    if headers
        .get(IF_NONE_MATCH)
        .and_then(|value| value.to_str().ok())
        .is_some_and(|value| value == etag)
    {
        return (
            StatusCode::NOT_MODIFIED,
            [(CACHE_CONTROL, cache), (ETAG, etag.as_str())],
        )
            .into_response();
    }

    let mime = mime_guess::from_path(path).first_or_octet_stream();

    (
        [
            (CONTENT_TYPE, mime.as_ref()),
            (CACHE_CONTROL, cache),
            (ETAG, etag.as_str()),
        ],
        Body::from(file.data.into_owned()),
    )
        .into_response()
}

/// ETag forte a partir do hash que o `rust-embed` já calculou. 8 bytes bastam: o único
/// papel aqui é distinguir versões do mesmo arquivo, não resistir a colisão maliciosa.
fn etag(hash: &[u8; 32]) -> String {
    use std::fmt::Write;

    let mut out = String::with_capacity(18);
    out.push('"');
    for byte in &hash[..8] {
        write!(out, "{byte:02x}").expect("String nunca falha em write!");
    }
    out.push('"');
    out
}
