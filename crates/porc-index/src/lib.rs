//! Índice em SQLite (`rusqlite` bundled + FTS5): schema, commits, busca, recentes e
//! config. Descartável por definição — apagar o `.db` e reconstruir é saída válida.
//!
//! O banco é **descartável de verdade**, e isso governa o desenho: nada aqui é fonte da
//! verdade. A verdade está no repositório do usuário; isto é cache e conveniência. Por isso
//! `Index::open` nunca devolve erro fatal para o boot — se o arquivo não abrir, o índice cai
//! para memória e o app sobe assim mesmo, sem recentes entre boots.

pub mod commits;
pub mod recents;

use std::{
    path::{Path, PathBuf},
    sync::Mutex,
};

use rusqlite::Connection;

/// Versão do schema, guardada no `PRAGMA user_version`.
///
/// Migração é sempre opcional: se um dia migrar der trabalho demais, apagar e recriar é uma
/// saída legítima justamente porque nada aqui é fonte da verdade.
const SCHEMA_VERSION: i64 = 3;

const DB_FILE: &str = "porcelain.db";

#[derive(Debug, thiserror::Error)]
pub enum IndexError {
    #[error("não consegui abrir o índice")]
    Open(#[source] rusqlite::Error),
    #[error("não consegui criar o diretório do índice")]
    Dir(#[source] std::io::Error),
    #[error("não consegui ler o índice")]
    Read(#[source] rusqlite::Error),
    #[error("não consegui escrever no índice")]
    Write(#[source] rusqlite::Error),
}

/// Handle do índice.
///
/// `Connection` do rusqlite não é `Sync`, então mora atrás de um `Mutex`. Toda chamada é
/// bloqueante e tem que estar dentro de um `spawn_blocking` — a regra é a mesma do libgit2.
pub struct Index {
    conn: Mutex<Connection>,
}

impl Index {
    /// `~/.local/share/porcelain/porcelain.db` (equivalentes no macOS e no Windows).
    pub fn default_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "porcelain")
            .map(|dirs| dirs.data_dir().join(DB_FILE))
    }

    /// Abre o índice em disco, ou em memória se o disco não colaborar.
    ///
    /// Nunca falha: um índice que não abre não pode impedir o usuário de usar o git dele. O
    /// que se perde é a memória entre boots, e isso vira um `warn` no log.
    pub fn open_or_memory(path: Option<&Path>) -> Self {
        match path.map(Self::open) {
            Some(Ok(index)) => return index,
            Some(Err(err)) => tracing::warn!(%err, "índice em disco indisponível, usando memória"),
            None => tracing::warn!("sem diretório de dados do usuário, índice só em memória"),
        }

        Self::in_memory()
    }

    pub fn open(path: &Path) -> Result<Self, IndexError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(IndexError::Dir)?;
        }

        let conn = Connection::open(path).map_err(IndexError::Open)?;
        Self::from_connection(conn)
    }

    pub fn in_memory() -> Self {
        let conn = Connection::open_in_memory().expect("SQLite em memória sempre abre");

        Self::from_connection(conn).expect("schema em banco novo sempre aplica")
    }

    fn from_connection(conn: Connection) -> Result<Self, IndexError> {
        // WAL: leitura não bloqueia escrita. O log vai ler enquanto a indexação escreve, e sem
        // isto os dois se esperariam. `synchronous = NORMAL` porque perder as últimas escritas
        // num crash custa uma reindexação, não um dado do usuário.
        conn.pragma_update(None, "journal_mode", "WAL")
            .map_err(IndexError::Open)?;
        conn.pragma_update(None, "synchronous", "NORMAL")
            .map_err(IndexError::Open)?;
        conn.pragma_update(None, "foreign_keys", "ON")
            .map_err(IndexError::Open)?;

        migrate(&conn)?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    /// Acesso serializado à conexão.
    ///
    /// `expect` num mutex envenenado: se uma escrita entrou em pânico no meio, o estado do
    /// banco não é confiável, e o certo é derrubar em vez de seguir com dado torto.
    fn with<T>(
        &self,
        f: impl FnOnce(&Connection) -> Result<T, rusqlite::Error>,
    ) -> Result<T, rusqlite::Error> {
        let conn = self.conn.lock().expect("índice envenenado");
        f(&conn)
    }

    /// Como `with`, mas dentro de uma transação: ou tudo aplica, ou nada aplica. É o que a
    /// reindexação usa para substituir os commits de um repositório sem deixar o índice pela
    /// metade se o processo morrer no meio (`commits::replace`).
    fn with_transaction<T>(
        &self,
        f: impl FnOnce(&rusqlite::Transaction) -> Result<T, rusqlite::Error>,
    ) -> Result<T, rusqlite::Error> {
        let mut conn = self.conn.lock().expect("índice envenenado");
        let tx = conn.transaction()?;
        let result = f(&tx)?;
        tx.commit()?;
        Ok(result)
    }
}

fn migrate(conn: &Connection) -> Result<(), IndexError> {
    let version: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(IndexError::Open)?;

    if version >= SCHEMA_VERSION {
        return Ok(());
    }

    conn.execute_batch(recents::SCHEMA)
        .map_err(IndexError::Open)?;
    conn.execute_batch(commits::SCHEMA)
        .map_err(IndexError::Open)?;

    conn.pragma_update(None, "user_version", SCHEMA_VERSION)
        .map_err(IndexError::Open)?;

    tracing::debug!(
        from = version,
        to = SCHEMA_VERSION,
        "schema do índice aplicado"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fts5_esta_disponivel() {
        // A busca por mensagem/autor do Bloco H depende disto. Não há feature de rusqlite para
        // ligar FTS5: ou o SQLite do `bundled` foi compilado com ele, ou não foi. Melhor
        // descobrir agora, num teste de dois segundos, do que a doze passos daqui.
        let index = Index::in_memory();

        index
            .with(|conn| conn.execute_batch("CREATE VIRTUAL TABLE probe USING fts5(corpo)"))
            .expect("FTS5 tem que estar compilado no SQLite bundled");
    }

    #[test]
    fn migrate_e_idempotente() {
        let index = Index::in_memory();

        // Segunda passada não pode recriar nada nem falhar: é o caminho de todo boot depois do
        // primeiro.
        index
            .with(|conn| {
                migrate(conn).map_err(|err| match err {
                    IndexError::Open(err) => err,
                    other => panic!("erro inesperado: {other}"),
                })
            })
            .unwrap();
    }
}
