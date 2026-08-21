//! Flags de linha de comando.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use clap::Parser;

/// `name` é o que sai em `--version` (o produto), `bin_name` é o que sai no `Usage`
/// (o executável). São mesmo dois nomes diferentes.
#[derive(Debug, Parser)]
#[command(
    name = "porcelain",
    bin_name = "porc",
    version,
    about = "cliente git local-first: servidor em 127.0.0.1 e interface no navegador"
)]
pub struct Cli {
    /// Porta preferida. Se estiver ocupada, cai para uma porta livre qualquer.
    #[arg(long, value_name = "N")]
    pub port: Option<u16>,

    /// Não abrir o navegador no boot; só imprimir a URL.
    #[arg(long)]
    pub no_browser: bool,

    /// Repositório a abrir ao subir.
    #[arg(long, value_name = "CAMINHO")]
    pub repo: Option<PathBuf>,

    /// Habilita o terminal embutido. Desligado por padrão: é a única superfície onde uma
    /// falha nas camadas de autenticação vira execução de código.
    #[arg(long)]
    pub enable_terminal: bool,
}

impl Cli {
    /// Resolve `--repo` para um caminho canônico existente.
    ///
    /// Canonicalizar já aqui é o que impede que um `..` ou um symlink chegue ao resto do
    /// programa como se fosse outro lugar.
    pub fn repo(&self) -> Result<Option<PathBuf>> {
        let Some(repo) = self.repo.as_deref() else {
            return Ok(None);
        };

        Ok(Some(canonical_dir(repo)?))
    }
}

fn canonical_dir(path: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("não encontrei o caminho {}", path.display()))?;

    if !canonical.is_dir() {
        bail!("{} não é um diretório", canonical.display());
    }

    Ok(canonical)
}
