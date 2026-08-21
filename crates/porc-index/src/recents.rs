//! Repositórios abertos recentemente.
//!
//! Guarda **caminho**, não `repo_id`: o id é derivado do caminho, então o caminho é o dado
//! primário e o id é reconstituível. Guardar só o id tornaria a lista ilegível no dia em que o
//! algoritmo do hash mudasse.

use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{params, OptionalExtension};
use serde::Serialize;

use crate::{Index, IndexError};

pub(crate) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS recents (
    repo_id   TEXT PRIMARY KEY,
    path      TEXT NOT NULL UNIQUE,
    name      TEXT NOT NULL,
    opened_at INTEGER NOT NULL
);
CREATE INDEX IF NOT EXISTS recents_opened_at ON recents (opened_at DESC);
";

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Recent {
    pub repo_id: String,
    pub path: String,
    pub name: String,
    /// Epoch em **milissegundos** — segundos empatariam entre dois repositórios abertos no
    /// mesmo segundo, e a ordem da lista é a única informação que ela carrega. Formatar é
    /// trabalho da UI, que sabe o fuso e o idioma do usuário.
    pub opened_at: i64,
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        // Relógio antes de 1970 é absurdo o bastante para valer um zero em vez de um erro que
        // impediria o repositório de ser registrado.
        .map(|since| since.as_millis() as i64)
        .unwrap_or(0)
}

impl Index {
    /// Registra a abertura de um repositório, ou atualiza a data se ele já estava na lista.
    pub fn touch_recent(&self, repo_id: &str, path: &Path, name: &str) -> Result<(), IndexError> {
        self.touch_recent_at(repo_id, path, name, now())
    }

    /// Mesma coisa com o instante dado. Existe para o teste ser determinístico: com o relógio
    /// real, três aberturas seguidas caem no mesmo milissegundo e a ordem vira sorteio.
    pub fn touch_recent_at(
        &self,
        repo_id: &str,
        path: &Path,
        name: &str,
        at: i64,
    ) -> Result<(), IndexError> {
        let path = path.to_string_lossy().into_owned();

        self.with(|conn| {
            conn.execute(
                "INSERT INTO recents (repo_id, path, name, opened_at)
                 VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(repo_id) DO UPDATE SET
                     path = excluded.path,
                     name = excluded.name,
                     opened_at = excluded.opened_at",
                params![repo_id, path, name, at],
            )
        })
        .map(|_| ())
        .map_err(IndexError::Write)
    }

    /// Os mais recentes primeiro.
    pub fn recents(&self, limit: usize) -> Result<Vec<Recent>, IndexError> {
        self.with(|conn| {
            let mut statement = conn.prepare(
                "SELECT repo_id, path, name, opened_at
                 FROM recents
                 ORDER BY opened_at DESC, name ASC
                 LIMIT ?1",
            )?;

            let rows = statement.query_map(params![limit as i64], |row| {
                Ok(Recent {
                    repo_id: row.get(0)?,
                    path: row.get(1)?,
                    name: row.get(2)?,
                    opened_at: row.get(3)?,
                })
            })?;

            rows.collect()
        })
        .map_err(IndexError::Read)
    }

    /// Tira um repositório da lista. Já não estar lá é sucesso — é o estado que queríamos.
    pub fn forget_recent(&self, repo_id: &str) -> Result<(), IndexError> {
        self.with(|conn| conn.execute("DELETE FROM recents WHERE repo_id = ?1", params![repo_id]))
            .map(|_| ())
            .map_err(IndexError::Write)
    }

    /// Caminho de um recente. É o que permite reabrir por id sem o cliente mandar caminho.
    pub fn recent_path(&self, repo_id: &str) -> Result<Option<String>, IndexError> {
        self.with(|conn| {
            conn.query_row(
                "SELECT path FROM recents WHERE repo_id = ?1",
                params![repo_id],
                |row| row.get(0),
            )
            .optional()
        })
        .map_err(IndexError::Read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reabrir_atualiza_a_data_em_vez_de_duplicar() {
        let index = Index::in_memory();

        index
            .touch_recent_at("aaa", Path::new("/tmp/a"), "a", 1_000)
            .unwrap();
        index
            .touch_recent_at("bbb", Path::new("/tmp/b"), "b", 2_000)
            .unwrap();
        index
            .touch_recent_at("aaa", Path::new("/tmp/a"), "a", 3_000)
            .unwrap();

        let recents = index.recents(10).unwrap();
        assert_eq!(recents.len(), 2);
        // `a` foi tocado por último, então tem que estar na frente — mesmo tendo entrado antes.
        assert_eq!(recents[0].repo_id, "aaa");
    }

    #[test]
    fn recent_path_e_forget() {
        let index = Index::in_memory();
        index.touch_recent("aaa", Path::new("/tmp/a"), "a").unwrap();

        assert_eq!(index.recent_path("aaa").unwrap().as_deref(), Some("/tmp/a"));

        index.forget_recent("aaa").unwrap();
        assert_eq!(index.recent_path("aaa").unwrap(), None);
        // Esquecer duas vezes não é erro.
        index.forget_recent("aaa").unwrap();
    }

    #[test]
    fn limite_corta_a_lista() {
        let index = Index::in_memory();
        for n in 0..5 {
            index
                .touch_recent(&format!("id{n}"), Path::new(&format!("/tmp/{n}")), "r")
                .unwrap();
        }

        assert_eq!(index.recents(3).unwrap().len(), 3);
    }
}
