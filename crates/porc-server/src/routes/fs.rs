//! `GET /api/v1/fs/list` — navegador de pastas do lado do servidor.
//!
//! O navegador não consegue escolher um diretório do disco (não existe API para isso que não
//! seja um `<input type=file>` inútil aqui), então o seletor de pastas do porcelain é uma
//! rota. É por ela que o usuário chega a um repositório, e é por isso que ela é a superfície
//! mais exposta do app: recebe um caminho vindo do cliente.
//!
//! O confinamento é feito em três camadas, nesta ordem:
//!   1. componente `..` no pedido → 403 antes de tocar o disco;
//!   2. `fs::canonicalize`, que resolve symlink e `.` — se o alvo não existe, 404;
//!   3. prefixo contra a raiz canônica → 403.
//!
//! Cada entrada listada também é canonicalizada e conferida: um symlink dentro da raiz
//! apontando para fora simplesmente não aparece, em vez de aparecer e dar 403 ao ser clicado.

use std::{
    io::ErrorKind,
    path::{Component, Path, PathBuf},
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};

use crate::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    /// Ausente significa "a raiz". Relativo é resolvido a partir dela.
    path: Option<String>,
    /// Ocultos ficam escondidos por padrão — numa home o ruído de `.cache`, `.local` e
    /// companhia enterra as pastas que o usuário procura.
    #[serde(default)]
    hidden: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    name: String,
    path: String,
    is_repo: bool,
    hidden: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Listing {
    /// Sempre o caminho canônico, nunca o que o cliente mandou.
    path: String,
    /// `None` na raiz: é onde o "subir um nível" da UI para.
    parent: Option<String>,
    /// A raiz do confinamento, para a UI saber desenhar o breadcrumb sem adivinhar.
    root: String,
    entries: Vec<Entry>,
}

#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("caminho fora da área permitida")]
    Confined,
    #[error("caminho não encontrado")]
    NotFound,
    #[error("não é um diretório")]
    NotADirectory,
    #[error("não consegui ler o diretório: {0}")]
    Io(#[source] std::io::Error),
}

impl IntoResponse for FsError {
    fn into_response(self) -> Response {
        let status = match self {
            FsError::Confined => StatusCode::FORBIDDEN,
            FsError::NotFound => StatusCode::NOT_FOUND,
            FsError::NotADirectory => StatusCode::BAD_REQUEST,
            FsError::Io(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        // Mensagem nossa, nunca o `io::Error` cru: "Permission denied (os error 13)" não é
        // texto para um humano ler numa interface.
        if let FsError::Io(err) = &self {
            tracing::warn!(%err, "falha lendo diretório");
        }

        (status, self.to_string()).into_response()
    }
}

/// Resolve o caminho pedido para uma forma canônica dentro da raiz.
///
/// `pub(crate)` porque `POST /api/v1/repos` — o outro lugar que recebe caminho do cliente —
/// passa por aqui. Duas implementações de confinamento seria uma a mais do que o número de
/// implementações que dá para manter corretas.
pub(crate) fn resolve(root: &Path, requested: Option<&str>) -> Result<PathBuf, FsError> {
    let Some(requested) = requested.map(str::trim).filter(|path| !path.is_empty()) else {
        return Ok(root.to_path_buf());
    };

    let requested = Path::new(requested);

    // `..` é recusado antes de qualquer syscall. A canonicalização abaixo já barraria a fuga,
    // mas rejeitar cedo mantém o log honesto: `..` num seletor de pastas nunca é acidente.
    if requested
        .components()
        .any(|component| matches!(component, Component::ParentDir))
    {
        return Err(FsError::Confined);
    }

    let joined = if requested.is_absolute() {
        requested.to_path_buf()
    } else {
        root.join(requested)
    };

    let canonical = joined.canonicalize().map_err(|err| match err.kind() {
        ErrorKind::NotFound => FsError::NotFound,
        ErrorKind::PermissionDenied => FsError::Confined,
        _ => FsError::Io(err),
    })?;

    // O teste que vale: depois de resolver symlink, ainda estamos debaixo da raiz?
    if !canonical.starts_with(root) {
        return Err(FsError::Confined);
    }

    if !canonical.is_dir() {
        return Err(FsError::NotADirectory);
    }

    Ok(canonical)
}

/// Lê o diretório. Bloqueante de ponta a ponta (`read_dir` + um `canonicalize` e alguns
/// `stat` por entrada), então roda fora do event loop.
fn list_dir(root: &Path, dir: &Path, show_hidden: bool) -> Result<Vec<Entry>, FsError> {
    let mut entries = Vec::new();

    for entry in std::fs::read_dir(dir).map_err(FsError::Io)? {
        // Uma entrada ilegível não invalida o diretório inteiro — some da lista e segue.
        let Ok(entry) = entry else { continue };

        let name = entry.file_name().to_string_lossy().into_owned();
        let hidden = name.starts_with('.');
        if hidden && !show_hidden {
            continue;
        }

        // `canonicalize` segue o symlink: é o mesmo caminho que a próxima chamada vai receber
        // de volta em `?path=`, e é o que decide se a entrada escapa da raiz.
        let Ok(path) = entry.path().canonicalize() else {
            continue;
        };
        if !path.is_dir() || !path.starts_with(root) {
            continue;
        }

        entries.push(Entry {
            is_repo: porc_git::discover::is_repo(&path),
            path: path.to_string_lossy().into_owned(),
            name,
            hidden,
        });
    }

    // Alfabética sem diferenciar caixa: é a ordem que um seletor de pastas precisa ter para
    // ser previsível. Repositório não vem antes — mover a linha que o usuário procura só
    // porque ela é um repo torna a lista imprevisível.
    entries.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.name.cmp(&b.name))
    });

    Ok(entries)
}

pub async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Listing>, FsError> {
    let settings = state.settings.clone();

    let listing = tokio::task::spawn_blocking(move || {
        let root = &settings.root;
        let dir = resolve(root, query.path.as_deref())?;
        let entries = list_dir(root, &dir, query.hidden)?;

        Ok::<_, FsError>(Listing {
            // Só existe "acima" enquanto estivermos abaixo da raiz.
            parent: (dir != *root)
                .then(|| dir.parent())
                .flatten()
                .map(|parent| parent.to_string_lossy().into_owned()),
            path: dir.to_string_lossy().into_owned(),
            root: root.to_string_lossy().into_owned(),
            entries,
        })
    })
    .await
    .map_err(|err| FsError::Io(std::io::Error::other(err)))??;

    Ok(Json(listing))
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scan {
    root: String,
    depth: usize,
    /// `true` quando a varredura bateu no teto e parou — a lista está incompleta e a UI precisa
    /// dizer isso, em vez de deixar o usuário procurando um repositório que existe.
    truncated: bool,
    repos: Vec<ScanEntry>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScanEntry {
    name: String,
    path: String,
    /// Caminho relativo à raiz — é o que distingue dois repositórios de mesmo nome na lista.
    relative: String,
    depth: usize,
}

/// `GET /api/v1/fs/scan` — repositórios debaixo da raiz configurada.
///
/// Não recebe parâmetro nenhum de propósito: raiz e profundidade vêm do `config.toml`. Deixar o
/// cliente escolher onde e quão fundo varrer seria dar a ele um `find` na máquina inteira.
pub async fn scan(State(state): State<AppState>) -> Result<Json<Scan>, FsError> {
    let settings = state.settings.clone();

    let scan = tokio::task::spawn_blocking(move || {
        let found = porc_git::discover::scan(
            &settings.root,
            settings.scan_depth,
            // Um a mais que o teto: é assim que se sabe que havia mais.
            settings.scan_limit + 1,
        );

        let truncated = found.len() > settings.scan_limit;
        let root = settings.root.to_string_lossy().into_owned();

        Scan {
            repos: found
                .into_iter()
                .take(settings.scan_limit)
                .map(|repo| ScanEntry {
                    relative: repo
                        .path
                        .strip_prefix(&settings.root)
                        .unwrap_or(&repo.path)
                        .to_string_lossy()
                        .into_owned(),
                    path: repo.path.to_string_lossy().into_owned(),
                    name: repo.name,
                    depth: repo.depth,
                })
                .collect(),
            depth: settings.scan_depth,
            truncated,
            root,
        }
    })
    .await
    .map_err(|err| FsError::Io(std::io::Error::other(err)))?;

    Ok(Json(scan))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        // O repositório do próprio projeto serve de raiz: existe, é canônico e tem
        // subdiretórios conhecidos.
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .canonicalize()
            .unwrap()
    }

    #[test]
    fn sem_path_devolve_a_raiz() {
        let root = root();
        assert_eq!(resolve(&root, None).unwrap(), root);
        assert_eq!(resolve(&root, Some("  ")).unwrap(), root);
    }

    #[test]
    fn relativo_resolve_a_partir_da_raiz() {
        let root = root();
        assert_eq!(resolve(&root, Some("crates")).unwrap(), root.join("crates"));
    }

    #[test]
    fn recusa_fuga_por_dot_dot() {
        let root = root();
        assert!(matches!(
            resolve(&root, Some("../../etc")),
            Err(FsError::Confined)
        ));
        // Também no meio do caminho, não só no começo.
        assert!(matches!(
            resolve(&root, Some("crates/../../etc")),
            Err(FsError::Confined)
        ));
    }

    #[test]
    fn recusa_absoluto_fora_da_raiz() {
        let root = root();
        assert!(matches!(
            resolve(&root, Some("/etc")),
            Err(FsError::Confined)
        ));
    }

    #[test]
    fn arquivo_nao_e_diretorio() {
        let root = root();
        assert!(matches!(
            resolve(&root, Some("Cargo.toml")),
            Err(FsError::NotADirectory)
        ));
    }

    #[test]
    fn ocultos_so_aparecem_quando_pedidos() {
        let root = root();

        let visiveis = list_dir(&root, &root, false).unwrap();
        assert!(visiveis.iter().all(|entry| !entry.hidden));
        assert!(visiveis.iter().any(|entry| entry.name == "crates"));

        let todos = list_dir(&root, &root, true).unwrap();
        assert!(todos.iter().any(|entry| entry.name == ".git"));
    }
}
