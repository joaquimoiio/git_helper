//! Lockfile de instância única.
//!
//! Não é um lock de exclusão mútua de verdade (não tenta `flock`): é um bilhete deixado em
//! disco dizendo "há um `porc` na porta N". Quem confirma a instância é a resposta de
//! `/health`, não o arquivo — arquivo mente, processo vivo não.

use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

const LOCK_FILE: &str = "porc.lock";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Lock {
    pub pid: u32,
    pub port: u16,
    /// Mesmo valor que `/health` devolve. Se não bater, quem está na porta é outro boot.
    pub instance_id: String,
}

/// `~/.local/share/porcelain/porc.lock` no Linux, `~/Library/Application Support/porcelain/`
/// no macOS, `%APPDATA%\porcelain\` no Windows.
pub fn path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "porcelain")
        .context("não consegui descobrir o diretório de dados do usuário")?;

    Ok(dirs.data_dir().join(LOCK_FILE))
}

/// Lê o lockfile. Ausente ou corrompido devolve `None`: em ambos os casos a saída é a
/// mesma — seguir o boot e sobrescrever.
pub fn read(path: &Path) -> Option<Lock> {
    let raw = match fs::read(path) {
        Ok(raw) => raw,
        Err(err) if err.kind() == ErrorKind::NotFound => return None,
        Err(err) => {
            tracing::debug!(%err, path = %path.display(), "não consegui ler o lockfile");
            return None;
        }
    };

    match serde_json::from_slice(&raw) {
        Ok(lock) => Some(lock),
        Err(err) => {
            tracing::warn!(%err, "lockfile ilegível, será sobrescrito");
            None
        }
    }
}

pub fn write(path: &Path, lock: &Lock) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("não consegui criar {}", parent.display()))?;
    }

    let raw = serde_json::to_vec_pretty(lock).context("não consegui serializar o lockfile")?;

    fs::write(path, raw).with_context(|| format!("não consegui escrever {}", path.display()))
}

/// Remove o lockfile. Já não existir é sucesso — é o estado que queríamos.
pub fn remove(path: &Path) {
    if let Err(err) = fs::remove_file(path) {
        if err.kind() != ErrorKind::NotFound {
            tracing::warn!(%err, path = %path.display(), "não consegui remover o lockfile");
        }
    }
}
